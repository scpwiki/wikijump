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
use crate::parsing::{process_depths, DepthItem, DepthList};
use crate::span_wrap::SpanWrap;
use crate::tree::{AttributeMap, StyledContainer, StyledContainerType};

const MAX_BLOCKQUOTE_DEPTH: usize = 30;

pub const RULE_BLOCKQUOTE: Rule = Rule {
    name: "blockquote",
    try_consume_fn,
};

fn try_consume_fn<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(log, "Parsing nested native blockquotes");

    assert!(
        parser.current().token == Token::InputStart
            || parser.current().token == Token::LineBreak,
        "Starting token for list is not start of input or newline",
    );
    parser.step()?;

    // Produce a depth list with elements
    let mut depths = Vec::new();
    let mut exceptions = Vec::new();

    loop {
        let current = parser.current();
        let depth = match current.token {
            // 1 or more ">"s in one token. Return ASCII length.
            Token::Quote => current.slice.len(),

            // Exactly ">>", for potential "»" replacement.
            Token::RightDoubleAngle => 2,

            // Invalid token, bail
            _ => {
                debug!(
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
        let mut elements = collect_consume(
            log,
            parser,
            RULE_BLOCKQUOTE,
            &[
                ParseCondition::current(Token::LineBreak),
                ParseCondition::current(Token::InputEnd),
            ],
            &[ParseCondition::current(Token::ParagraphBreak)],
            None,
        )?
        .chain(&mut exceptions);

        // Add a line break for the end of the line
        elements.push(Element::LineBreak);

        // Append blockquote line
        //
        // Depth lists expect zero-based list depths, but tokens are one-based.
        // So, we subtract one.
        //
        // This will not overflow becaus Token::Quote requires at least one ">".
        depths.push((depth - 1, (), elements))
    }

    // This blockquote has no rows, so the rule fails
    if depths.is_empty() {
        return Err(parser.make_warn(ParseWarningKind::RuleFailed));
    }

    let depth_lists = process_depths((), depths);
    let elements: Vec<Element> = depth_lists
        .into_iter()
        .map(|(_, depth_list)| build_blockquote_element(depth_list))
        .collect();

    ok!(elements, exceptions)
}

fn build_blockquote_element(list: DepthList<(), Vec<Element>>) -> Element {
    let mut all_elements = Vec::new();

    // Convert depth list into a list of elements
    for item in list {
        match item {
            DepthItem::Item(mut elements) => all_elements.append(&mut elements),
            DepthItem::List(_, list) => {
                let element = build_blockquote_element(list);
                all_elements.push(element);
            }
        }
    }

    // Remove the trailing linew break, these should
    // only be between lines in the blockquote.
    if let Some(Element::LineBreak) = all_elements.last() {
        all_elements.pop();
    }

    // Wrap blockquote internals in a paragraph, like [[blockquote]] does.
    let paragraph = Container::new(ContainerType::Paragraph, all_elements);

    // Place paragraph in the blockquote container
    Element::StyledContainer(StyledContainer::new(
        StyledContainerType::Blockquote,
        vec![Element::Container(paragraph)],
        AttributeMap::new(),
    ))
}
