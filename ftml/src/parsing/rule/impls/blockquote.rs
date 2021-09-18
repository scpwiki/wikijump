/*
 * parsing/rule/impls/blockquote.rs
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
use crate::parsing::paragraph::ParagraphStack;
use crate::parsing::{process_depths, DepthItem, DepthList};
use crate::tree::{AttributeMap, Container, ContainerType};

const MAX_BLOCKQUOTE_DEPTH: usize = 30;

pub const RULE_BLOCKQUOTE: Rule = Rule {
    name: "blockquote",
    position: LineRequirement::StartOfLine,
    try_consume_fn,
};

fn try_consume_fn<'p, 'r, 't>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, PartialElements<'t>> {
    info!(log, "Parsing nested native blockquotes");

    // Context variables
    let mut depths = Vec::new();
    let mut exceptions = Vec::new();

    // Produce a depth list with elements
    loop {
        let current = parser.current();
        let depth = match current.token {
            // 1 or more ">"s in one token. Return ASCII length.
            Token::Quote => current.slice.len(),

            // Invalid token, bail
            _ => {
                warn!(
                    log,
                    "Didn't find blockquote token, ending list iteration";
                    "token" => current.token,
                    "slice" => current.slice,
                    "span" => SpanWrap::from(&current.span),
                );

                break;
            }
        };
        parser.step()?;
        parser.get_optional_space()?; // allow whitespace after ">"

        // Check that the depth isn't obscenely deep, to avoid DOS attacks via stack overflow.
        if depth > MAX_BLOCKQUOTE_DEPTH {
            info!(
                log,
                "Native blockquote has a depth greater than the maximum! Failing";
                "depth" => depth,
                "max-depth" => MAX_BLOCKQUOTE_DEPTH,
            );

            return Err(parser.make_warn(ParseWarningKind::BlockquoteDepthExceeded));
        }

        // Parse elements until we hit the end of the line
        let mut paragraph_safe = true;
        let mut elements = collect_consume(
            log,
            parser,
            RULE_BLOCKQUOTE,
            &[
                ParseCondition::current(Token::LineBreak),
                ParseCondition::current(Token::ParagraphBreak),
                ParseCondition::current(Token::InputEnd),
            ],
            &[],
            None,
        )?
        .chain(&mut exceptions, &mut paragraph_safe);

        // Add a line break for the end of the line
        elements.push(Element::LineBreak);

        // Append blockquote line
        //
        // Depth lists expect zero-based list depths, but tokens are one-based.
        // So, we subtract one.
        //
        // This will not overflow because Token::Quote requires at least one ">".
        depths.push((depth - 1, (), (elements, paragraph_safe)))
    }

    // This blockquote has no rows, so the rule fails
    if depths.is_empty() {
        return Err(parser.make_warn(ParseWarningKind::RuleFailed));
    }

    let depth_lists = process_depths((), depths);
    let elements: Vec<Element> = depth_lists
        .into_iter()
        .map(|(_, depth_list)| build_blockquote_element(log, depth_list))
        .collect();

    ok!(false; elements, exceptions)
}

fn build_blockquote_element<'t>(
    log: &Logger,
    list: DepthList<(), (Vec<Element<'t>>, bool)>,
) -> Element<'t> {
    let mut stack = ParagraphStack::new(log);

    // Convert depth list into a list of elements
    for item in list {
        match item {
            DepthItem::Item((elements, paragraph_safe)) => {
                for element in elements {
                    stack.push_element(element, paragraph_safe);
                }
            }
            DepthItem::List(_, list) => {
                let blockquote = build_blockquote_element(log, list);
                stack.pop_line_break();
                stack.push_element(blockquote, false);
            }
        }
    }

    stack.pop_line_break();

    Element::Container(Container::new(
        ContainerType::Blockquote,
        stack.into_elements(),
        AttributeMap::new(),
    ))
}
