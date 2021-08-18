/*
 * parsing/rule/impls/header.rs
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

use super::prelude::*;
use std::convert::TryInto;

pub const RULE_HEADER: Rule = Rule {
    name: "header",
    position: LineRequirement::StartOfLine,
    try_consume_fn,
};

fn try_consume_fn<'p, 'r, 't>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(log, "Trying to create header container");

    // Assert first tokens match rule
    check_step_multiple(
        parser,
        &[Token::InputStart, Token::LineBreak, Token::ParagraphBreak],
    )?;

    // Parse and builder header element
    parse_header(log, parser)
}

fn parse_header<'p, 'r, 't>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    // Helper to ensure the current token is expected
    macro_rules! step {
        ($token:expr) => {{
            let current = parser.current();
            if current.token != $token {
                return Err(parser.make_warn(ParseWarningKind::RuleFailed));
            }

            parser.step()?;
            current
        }};
    }

    // Get header depth
    let heading = step!(Token::Heading)
        .slice
        .try_into()
        .expect("Received invalid heading length token slice");

    // Step over whitespace
    step!(Token::Whitespace);

    let (elements, mut all_exceptions, _) = collect_container(
        log,
        parser,
        RULE_HEADER,
        ContainerType::Header(heading),
        &[
            ParseCondition::current(Token::InputEnd),
            ParseCondition::current(Token::LineBreak),
            ParseCondition::current(Token::ParagraphBreak),
        ],
        &[],
        None,
    )?
    .into();

    // If this heading wants a table of contents (TOC) entry, then add one
    if heading.has_toc {
        // collect_container() always produces one Element::Container.
        // We unwrap it so we can get the elements composing the name.
        let elements = match elements {
            Elements::Single(Element::Container(ref container)) => container.elements(),
            _ => panic!("Collected heading produced a non-single non-container element"),
        };

        // Create table of contents entry with the given level and name.
        parser.push_table_of_contents_entry(heading.level, elements);
    }

    // Recursively collect headings until we hit a warning.
    //
    // We do this because the container consumes the newline,
    // which we need to trigger the next header when using regular rules.
    let mut all_elements: Vec<_> = elements.into_iter().collect();

    if let Ok(success) = parse_header(log, parser) {
        let (elements, mut exceptions, _) = success.into();

        all_elements.extend(elements);
        all_exceptions.append(&mut exceptions);
    }

    // Build final Elements object
    ok!(false; all_elements, all_exceptions)
}
