/*
 * services/render/wikidot_expression.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use super::literal_regions::LiteralRegionIndex;
use regex::Regex;
use std::ops::Range;
use std::sync::LazyLock;

const MAX_EXPRESSION_BYTES: usize = 256;
const MAX_OPERATIONS: usize = 512;
const MAX_PARENTHESES: usize = 32;
const MAX_IFEXPR_PASSES: usize = 32;

static WIKIDOT_EXPR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[#expr\s+(?P<expression>[^\]]*?)\s*\]\]").unwrap()
});
static WIKIDOT_IFEXPR_OPEN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)\[\[#ifexpr\s+").unwrap());

/// Resolve the context-free Wikidot expression functions left after runtime
/// variables have been substituted. Invalid expressions remain visible and
/// therefore fail closed. Division or modulo by zero becomes numeric zero;
/// this matches the deterministic missing-vote policy used by corpus
/// ListPages rows whose frozen source has no vote-count field.
pub(super) fn resolve_parser_functions(value: &str) -> String {
    let with_conditionals = resolve_ifexpr_functions(value);

    let literal_regions = LiteralRegionIndex::new(&with_conditionals);
    WIKIDOT_EXPR_REGEX
        .replace_all(&with_conditionals, |captures: &regex::Captures<'_>| {
            let original = captures.get(0).map_or("", |matched| matched.as_str());
            let start = captures.get(0).map_or(0, |matched| matched.start());
            if literal_regions.contains(start) {
                return original.to_owned();
            }
            match evaluate(&captures["expression"]) {
                Ok(result) => format_value(result),
                Err(ExpressionError::Invalid) => original.to_owned(),
            }
        })
        .into_owned()
}

#[derive(Debug)]
struct IfExprParts {
    end: usize,
    expression: Range<usize>,
    when_true: Range<usize>,
    when_false: Option<Range<usize>>,
}

fn resolve_ifexpr_functions(value: &str) -> String {
    let mut resolved = value.to_owned();
    for _ in 0..MAX_IFEXPR_PASSES {
        let source = resolved.clone();
        let literal_regions = LiteralRegionIndex::new(&source);
        let mut replacements: Vec<(Range<usize>, String)> = Vec::new();
        let mut search_start = 0usize;

        while let Some(open) = WIKIDOT_IFEXPR_OPEN_REGEX.find(&source[search_start..]) {
            let function_start = search_start + open.start();
            let expression_start = search_start + open.end();
            let Some(parts) = find_ifexpr_parts(&source, expression_start) else {
                search_start = expression_start;
                continue;
            };
            if literal_regions.contains(function_start) {
                search_start = parts.end;
                continue;
            }

            let Ok(result) = evaluate(source[parts.expression.clone()].trim()) else {
                // An invalid outer function must not hide a valid nested one.
                search_start = expression_start;
                continue;
            };
            let selected = if truthy(result) {
                &source[parts.when_true]
            } else {
                parts.when_false.map_or("", |range| &source[range])
            };
            replacements.push((function_start..parts.end, selected.trim().to_owned()));
            search_start = parts.end;
        }

        if replacements.is_empty() {
            return resolved;
        }
        for (range, replacement) in replacements.into_iter().rev() {
            resolved.replace_range(range, &replacement);
        }
    }
    resolved
}

fn find_ifexpr_parts(source: &str, expression_start: usize) -> Option<IfExprParts> {
    let bytes = source.as_bytes();
    let mut cursor = expression_start;
    let mut depth = 1usize;
    let mut separators = [None, None];

    while cursor + 1 < bytes.len() {
        if bytes[cursor..].starts_with(b"[[") {
            depth += 1;
            cursor += 2;
            continue;
        }
        if bytes[cursor..].starts_with(b"]]") {
            if depth == 1 {
                let first = separators[0]?;
                let true_end = separators[1].unwrap_or(cursor);
                return Some(IfExprParts {
                    end: cursor + 2,
                    expression: expression_start..first,
                    when_true: first + 1..true_end,
                    when_false: separators[1].map(|second| second + 1..cursor),
                });
            }
            depth -= 1;
            cursor += 2;
            continue;
        }
        if depth == 1 && bytes[cursor] == b'|' {
            if bytes[cursor + 1] == b'|' {
                cursor += 2;
                continue;
            }
            if separators[0].is_none() {
                separators[0] = Some(cursor);
            } else if separators[1].is_none() {
                separators[1] = Some(cursor);
            }
        }
        cursor += 1;
    }
    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExpressionError {
    Invalid,
}

fn evaluate(expression: &str) -> Result<f64, ExpressionError> {
    if expression.len() > MAX_EXPRESSION_BYTES || !expression.is_ascii() {
        return Err(ExpressionError::Invalid);
    }

    let mut parser = ExpressionParser {
        input: expression.as_bytes(),
        offset: 0,
        operations: 0,
        parentheses: 0,
    };
    let result = parser.parse_or()?;
    parser.skip_space();
    if parser.offset != parser.input.len() || !result.is_finite() {
        return Err(ExpressionError::Invalid);
    }
    Ok(result)
}

#[derive(Debug)]
struct ExpressionParser<'a> {
    input: &'a [u8],
    offset: usize,
    operations: usize,
    parentheses: usize,
}

impl ExpressionParser<'_> {
    fn parse_or(&mut self) -> Result<f64, ExpressionError> {
        let mut value = self.parse_and()?;
        while self.consume("||") {
            self.operation()?;
            let right = self.parse_and()?;
            value = f64::from(truthy(value) || truthy(right));
        }
        Ok(value)
    }

    fn parse_and(&mut self) -> Result<f64, ExpressionError> {
        let mut value = self.parse_comparison()?;
        while self.consume("&&") {
            self.operation()?;
            let right = self.parse_comparison()?;
            value = f64::from(truthy(value) && truthy(right));
        }
        Ok(value)
    }

    fn parse_comparison(&mut self) -> Result<f64, ExpressionError> {
        let left = self.parse_additive()?;
        let operator = [">=", "<=", "==", "!=", "=", ">", "<"]
            .into_iter()
            .find(|operator| self.consume(operator));
        let Some(operator) = operator else {
            return Ok(left);
        };
        self.operation()?;
        let right = self.parse_additive()?;
        let result = match operator {
            ">=" => left >= right,
            "<=" => left <= right,
            "=" | "==" => nearly_equal(left, right),
            "!=" => !nearly_equal(left, right),
            ">" => left > right,
            "<" => left < right,
            _ => unreachable!("comparison operator comes from fixed list"),
        };
        Ok(f64::from(result))
    }

    fn parse_additive(&mut self) -> Result<f64, ExpressionError> {
        let mut value = self.parse_multiplicative()?;
        loop {
            if self.consume("+") {
                self.operation()?;
                value += self.parse_multiplicative()?;
            } else if self.consume("-") {
                self.operation()?;
                value -= self.parse_multiplicative()?;
            } else {
                return Ok(value);
            }
        }
    }

    fn parse_multiplicative(&mut self) -> Result<f64, ExpressionError> {
        let mut value = self.parse_unary()?;
        loop {
            if self.consume("*") {
                self.operation()?;
                value *= self.parse_unary()?;
            } else if self.consume("/") {
                self.operation()?;
                let divisor = self.parse_unary()?;
                if divisor == 0.0 {
                    value = 0.0;
                } else {
                    value /= divisor;
                }
            } else if self.consume("%") {
                self.operation()?;
                let divisor = self.parse_unary()?;
                if divisor == 0.0 {
                    value = 0.0;
                } else {
                    value %= divisor;
                }
            } else {
                return Ok(value);
            }
        }
    }

    fn parse_unary(&mut self) -> Result<f64, ExpressionError> {
        if self.consume("+") {
            self.operation()?;
            self.parse_unary()
        } else if self.consume("-") {
            self.operation()?;
            Ok(-self.parse_unary()?)
        } else if self.consume("!") {
            self.operation()?;
            Ok(f64::from(!truthy(self.parse_unary()?)))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<f64, ExpressionError> {
        if self.consume("(") {
            self.parentheses += 1;
            if self.parentheses > MAX_PARENTHESES {
                return Err(ExpressionError::Invalid);
            }
            let value = self.parse_or()?;
            if !self.consume(")") {
                return Err(ExpressionError::Invalid);
            }
            self.parentheses -= 1;
            return Ok(value);
        }

        self.skip_space();
        if self
            .input
            .get(self.offset)
            .is_some_and(u8::is_ascii_alphabetic)
        {
            return self.parse_function();
        }
        self.parse_number()
    }

    fn parse_function(&mut self) -> Result<f64, ExpressionError> {
        self.skip_space();
        let start = self.offset;
        while self
            .input
            .get(self.offset)
            .is_some_and(u8::is_ascii_alphabetic)
        {
            self.offset += 1;
        }
        let name = &self.input[start..self.offset];
        if name.eq_ignore_ascii_case(b"true") {
            return Ok(1.0);
        }
        if name.eq_ignore_ascii_case(b"false") {
            return Ok(0.0);
        }
        if !self.consume("(") {
            return Err(ExpressionError::Invalid);
        }

        self.parentheses += 1;
        if self.parentheses > MAX_PARENTHESES {
            return Err(ExpressionError::Invalid);
        }
        let mut arguments = vec![self.parse_or()?];
        while self.consume(",") {
            arguments.push(self.parse_or()?);
        }
        if !self.consume(")") {
            return Err(ExpressionError::Invalid);
        }
        self.parentheses -= 1;
        self.operation()?;

        match name {
            name if name.eq_ignore_ascii_case(b"abs") && arguments.len() == 1 => {
                Ok(arguments[0].abs())
            }
            name if name.eq_ignore_ascii_case(b"min") => arguments
                .into_iter()
                .reduce(f64::min)
                .ok_or(ExpressionError::Invalid),
            name if name.eq_ignore_ascii_case(b"max") => arguments
                .into_iter()
                .reduce(f64::max)
                .ok_or(ExpressionError::Invalid),
            _ => Err(ExpressionError::Invalid),
        }
    }

    fn parse_number(&mut self) -> Result<f64, ExpressionError> {
        self.skip_space();
        let start = self.offset;
        let mut decimal = false;
        while let Some(byte) = self.input.get(self.offset) {
            if byte.is_ascii_digit() {
                self.offset += 1;
            } else if *byte == b'.' && !decimal {
                decimal = true;
                self.offset += 1;
            } else {
                break;
            }
        }
        if start == self.offset || self.input[start..self.offset] == *b"." {
            return Err(ExpressionError::Invalid);
        }
        std::str::from_utf8(&self.input[start..self.offset])
            .ok()
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|value| value.is_finite())
            .ok_or(ExpressionError::Invalid)
    }

    fn consume(&mut self, expected: &str) -> bool {
        self.skip_space();
        if self.input[self.offset..].starts_with(expected.as_bytes()) {
            self.offset += expected.len();
            true
        } else {
            false
        }
    }

    fn skip_space(&mut self) {
        while self
            .input
            .get(self.offset)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.offset += 1;
        }
    }

    fn operation(&mut self) -> Result<(), ExpressionError> {
        self.operations += 1;
        if self.operations > MAX_OPERATIONS {
            Err(ExpressionError::Invalid)
        } else {
            Ok(())
        }
    }
}

fn truthy(value: f64) -> bool {
    value != 0.0
}

fn nearly_equal(left: f64, right: f64) -> bool {
    (left - right).abs() <= f64::EPSILON
}

fn format_value(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    let mut output = format!("{value:.11}");
    while output.ends_with('0') {
        output.pop();
    }
    if output.ends_with('.') {
        output.pop();
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_documented_wikidot_expression_examples() {
        assert_eq!(resolve_parser_functions("[[#expr abs(-100)]]"), "100");
        assert_eq!(
            resolve_parser_functions("[[#expr min(4, 1, -4, 6, -10)]]"),
            "-10"
        );
        assert_eq!(
            resolve_parser_functions("[[#expr max(4, 1, -4, 6, -10)]]"),
            "6"
        );
        assert_eq!(
            resolve_parser_functions("[[#expr 2*4/12-4+66%2]]"),
            "-3.33333333333"
        );
        assert_eq!(
            resolve_parser_functions("[[#ifexpr 2*4/12-4+66%2 < -3.5 | less | greater]]"),
            "greater"
        );
        assert_eq!(resolve_parser_functions("[[#expr 2*(2-1)]]"), "2");
    }

    #[test]
    fn resolves_ratio_bar_shapes_and_zero_vote_division() {
        let source = concat!(
            "[[#ifexpr 0 == 0 | table-row | none]] ",
            "[[#expr (49+0)/2]] ",
            "[[#expr (0-49)/2/0*(-180)]]",
        );

        assert_eq!(resolve_parser_functions(source), "table-row 24.5 0");
        assert_eq!(
            resolve_parser_functions(
                "[[#ifexpr 0 >= 15 && (0-49)/2/0 < 0.2 | leaked | hidden]]"
            ),
            "hidden"
        );
        assert_eq!(resolve_parser_functions("[[#expr 1/0+1]]"), "1");
    }

    #[test]
    fn accepts_wikidot_ifexpr_boolean_or_and_optional_false_branch() {
        assert_eq!(
            resolve_parser_functions("[[#ifexpr 0 || 1 | yes | no]]"),
            "yes"
        );
        assert_eq!(
            resolve_parser_functions("[[#ifexpr true == (TRUE) | yes ]]"),
            "yes"
        );
        assert_eq!(resolve_parser_functions("x[[#ifexpr false | y ]]z"), "xz");
        assert_eq!(resolve_parser_functions("[[#ifexpr 1 = 1 | yes ]]"), "yes");
    }

    #[test]
    fn resolves_ifexpr_branches_with_balanced_wikidot_markup() {
        assert_eq!(
            resolve_parser_functions(
                "[[#ifexpr 1 && 1 | [[span data-value=\"a|b\"]]shown[[/span]] | hidden]]"
            ),
            "[[span data-value=\"a|b\"]]shown[[/span]]",
        );
        assert_eq!(
            resolve_parser_functions(
                "[[#ifexpr 0 | hidden | [[span]]shown | still shown[[/span]]]]"
            ),
            "[[span]]shown | still shown[[/span]]",
        );
    }

    #[test]
    fn resolves_nested_ifexpr_functions_in_bounded_passes() {
        let source = concat!(
            "[[#ifexpr 1 | ",
            "[[#ifexpr 0 | hidden | [[span]]shown[[/span]] ]]",
            " | outer-hidden]]",
        );
        assert_eq!(resolve_parser_functions(source), "[[span]]shown[[/span]]",);

        let mut deeply_nested = "leaf".to_owned();
        for _ in 0..(MAX_IFEXPR_PASSES + 8) {
            deeply_nested = format!("[[#ifexpr 1 | {deeply_nested} | hidden]]");
        }
        let resolved = resolve_parser_functions(&deeply_nested);
        assert_eq!(resolved.matches("[[#ifexpr").count(), 8);
        assert!(resolved.contains("leaf"));
    }

    #[test]
    fn preserves_finite_values_smaller_than_machine_epsilon() {
        assert_eq!(
            resolve_parser_functions("[[#expr 0.0000000000000001]]"),
            "0"
        );
        assert_eq!(
            resolve_parser_functions("[[#ifexpr 0.0000000000000001 | true | false]]"),
            "true"
        );
        assert_eq!(
            resolve_parser_functions("[[#expr 1/0.0000000000000001]]"),
            "10000000000000000"
        );
    }

    #[test]
    fn malformed_or_overlong_expressions_fail_closed() {
        let malformed = "[[#expr unknown(1)]]";
        assert_eq!(resolve_parser_functions(malformed), malformed);

        let overlong = format!("[[#expr {}]]", "1+".repeat(129));
        assert_eq!(resolve_parser_functions(&overlong), overlong);
    }
}
