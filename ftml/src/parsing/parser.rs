/*
 * parsing/parser.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use super::condition::ParseCondition;
use super::prelude::*;
use super::rule::Rule;
use super::RULE_PAGE;
use crate::log::prelude::*;
use crate::tokenizer::Tokenization;
use std::ptr;

const MAX_RECURSION_DEPTH: usize = 100;

#[derive(Debug, Clone)]
pub struct Parser<'r, 't> {
    // Logger instance
    log: Logger,

    // Parse state
    current: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,

    // Rule state
    rule: Rule,
    depth: usize,

    // Flags
    in_list: bool,
}

impl<'r, 't> Parser<'r, 't> {
    /// Constructor. Should only be created by `parse()`.
    ///
    /// All other instances should be `.clone()` or `.clone_with_rule()`d from
    /// the main instance used during parsing.
    pub(crate) fn new(log: &Logger, tokenization: &'r Tokenization<'t>) -> Self {
        let log = Logger::clone(log);
        let full_text = tokenization.full_text();
        let (current, remaining) = tokenization
            .tokens()
            .split_first()
            .expect("Parsed tokens list was empty (expected at least one element)");

        Parser {
            log,
            current,
            remaining,
            full_text,
            rule: RULE_PAGE,
            depth: 0,
            in_list: false,
        }
    }

    // Getters
    #[inline]
    pub fn log(&self) -> Logger {
        Logger::clone(&self.log)
    }

    #[inline]
    pub fn full_text(&self) -> FullText<'t> {
        self.full_text
    }

    #[inline]
    pub fn rule(&self) -> Rule {
        self.rule
    }

    #[inline]
    pub fn in_list(&self) -> bool {
        self.in_list
    }

    // Setters
    #[inline]
    pub fn set_rule(&mut self, rule: Rule) {
        self.rule = rule;
    }

    pub fn clone_with_rule(&self, rule: Rule) -> Self {
        let mut clone = self.clone();
        clone.set_rule(rule);
        clone
    }

    pub fn depth_increment(&mut self) -> Result<(), ParseWarning> {
        debug!(self.log, "Incrementing recursion depth"; "depth" => self.depth);

        self.depth += 1;

        if self.depth > MAX_RECURSION_DEPTH {
            return Err(self.make_warn(ParseWarningKind::RecursionDepthExceeded));
        }

        Ok(())
    }

    #[inline]
    pub fn depth_decrement(&mut self) {
        debug!(self.log, "Decrementing recursion depth"; "depth" => self.depth);

        self.depth -= 1;
    }

    #[inline]
    pub fn set_list_flag(&mut self, value: bool) {
        self.in_list = value;
    }

    // State evaluation
    pub fn evaluate(&self, condition: ParseCondition) -> bool {
        debug!(
            &self.log,
            "Evaluating parser condition";
            "current-token" => self.current.token,
            "current-slice" => self.current.slice,
            "current-span" => SpanWrap::from(&self.current.span),
        );

        match condition {
            ParseCondition::CurrentToken(token) => self.current.token == token,
            ParseCondition::PeekToken(token) => match self.look_ahead(0) {
                Some(actual) => actual.token == token,
                None => false,
            },
            ParseCondition::TokenPair(current, next) => {
                if self.current().token != current {
                    trace!(
                        &self.log,
                        "Current token in pair doesn't match, failing";
                        "expected" => current,
                        "actual" => self.current().token,
                    );

                    return false;
                }

                match self.look_ahead(0) {
                    Some(actual) => {
                        if actual.token != next {
                            trace!(
                                &self.log,
                                "Second token in pair doesn't match, failing";
                                "expected" => next,
                                "actual" => actual.token,
                            );

                            return false;
                        }
                    }
                    None => {
                        trace!(
                            &self.log,
                            "Second token in pair doesn't exist, failing";
                            "expected" => next,
                        );

                        return false;
                    }
                }

                true
            }
        }
    }

    #[inline]
    pub fn evaluate_any(&self, conditions: &[ParseCondition]) -> bool {
        trace!(
            &self.log,
            "Evaluating to see if any parser condition is true";
            "conditions-len" => conditions.len(),
        );

        conditions.iter().any(|&condition| self.evaluate(condition))
    }

    #[inline]
    pub fn evaluate_fn<F>(&self, f: F) -> bool
    where
        F: FnOnce(&mut Parser<'r, 't>) -> Result<bool, ParseWarning>,
    {
        debug!(&self.log, "Evaluating closure for parser condition");

        f(&mut self.clone()).unwrap_or(false)
    }

    pub fn save_evaluate_fn<F>(&mut self, f: F) -> Option<&'r ExtractedToken<'t>>
    where
        F: FnOnce(&mut Parser<'r, 't>) -> Result<bool, ParseWarning>,
    {
        debug!(
            &self.log,
            "Evaluating closure for parser condition, saving progress on success",
        );

        let mut parser = self.clone();
        if f(&mut parser).unwrap_or(false) {
            let last = self.current;
            self.update(&parser);
            Some(last)
        } else {
            None
        }
    }

    // Token pointer state and manipulation
    #[inline]
    pub fn current(&self) -> &'r ExtractedToken<'t> {
        self.current
    }

    #[inline]
    pub fn remaining(&self) -> &'r [ExtractedToken<'t>] {
        self.remaining
    }

    #[inline]
    pub fn update(&mut self, parser: &Parser<'r, 't>) {
        self.current = parser.current;
        self.remaining = parser.remaining;
    }

    #[inline]
    pub fn same_pointer(&self, old_remaining: &'r [ExtractedToken<'t>]) -> bool {
        ptr::eq(self.remaining, old_remaining)
    }

    /// Move the token pointer forward one step.
    #[inline]
    pub fn step(&mut self) -> Result<&'r ExtractedToken<'t>, ParseWarning> {
        debug!(self.log, "Stepping to the next token");

        match self.remaining.split_first() {
            Some((current, remaining)) => {
                self.current = current;
                self.remaining = remaining;
                Ok(current)
            }
            None => Err(self.make_warn(ParseWarningKind::EndOfInput)),
        }
    }

    /// Move the token pointer forward `count` steps.
    #[inline]
    pub fn step_n(&mut self, count: usize) -> Result<(), ParseWarning> {
        trace!(self.log, "Stepping n times"; "count" => count);

        for _ in 0..count {
            self.step()?;
        }

        Ok(())
    }

    /// Look for the token `offset + 1` beyond the current one.
    ///
    /// For instance, submitting `0` will yield the first item of `parser.remaining()`.
    #[inline]
    pub fn look_ahead(&self, offset: usize) -> Option<&'r ExtractedToken<'t>> {
        debug!(self.log, "Looking ahead to a token"; "offset" => offset);

        self.remaining.get(offset)
    }

    /// Like `look_ahead`, except returns a warning if the token isn't found.
    #[inline]
    pub fn look_ahead_warn(
        &self,
        offset: usize,
    ) -> Result<&'r ExtractedToken<'t>, ParseWarning> {
        self.look_ahead(offset)
            .ok_or_else(|| self.make_warn(ParseWarningKind::EndOfInput))
    }

    // Utilities
    #[cold]
    #[inline]
    pub fn make_warn(&self, kind: ParseWarningKind) -> ParseWarning {
        ParseWarning::new(kind, self.rule, self.current)
    }
}
