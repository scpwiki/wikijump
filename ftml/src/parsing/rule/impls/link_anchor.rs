/*
 * parsing/rule/impls/link_anchor.rs
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

//! Rules for anchor soft-links.
//!
//! A variant on single-bracket links which targets an anchor on the current page,
//! or is a fake link.

use super::prelude::*;
use crate::tree::{LinkLabel, LinkLocation};
use std::borrow::Cow;
use wikidot_normalize::normalize;

pub const RULE_LINK_ANCHOR: Rule = Rule {
    name: "link-anchor",
    position: LineRequirement::Any,
    try_consume_fn,
};

fn try_consume_fn<'p, 'r, 't>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, PartialElements<'t>> {
    info!(log, "Trying to create a single-bracket anchor link");

    check_step(parser, Token::LeftBracketAnchor)?;

    // Gather path for link
    let url = collect_text(
        log,
        parser,
        RULE_LINK_ANCHOR,
        &[ParseCondition::current(Token::Whitespace)],
        &[
            ParseCondition::current(Token::RightBracket),
            ParseCondition::current(Token::ParagraphBreak),
            ParseCondition::current(Token::LineBreak),
        ],
        None,
    )?;

    // Determine if this is an anchor link or fake link
    let url = if url.is_empty() {
        Cow::Borrowed("javascript:;")
    } else {
        // Make URL "#name", where 'name' is normalized.
        let mut url = str!(url);
        normalize(&mut url);
        url.insert(0, '#');

        Cow::Owned(url)
    };

    // Gather label for link
    let label = collect_text(
        log,
        parser,
        RULE_LINK_ANCHOR,
        &[ParseCondition::current(Token::RightBracket)],
        &[
            ParseCondition::current(Token::ParagraphBreak),
            ParseCondition::current(Token::LineBreak),
        ],
        None,
    )?;

    debug!(
        log,
        "Retrieved label for link, building element";
        "label" => label,
    );

    // Trim label
    let label = label.trim();

    // Build link element
    let element = Element::Link {
        link: LinkLocation::Url(url),
        label: LinkLabel::Text(cow!(label)),
        target: None,
    };

    // Return result
    ok!(element)
}
