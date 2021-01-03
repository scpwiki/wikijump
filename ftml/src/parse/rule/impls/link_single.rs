/*
 * parse/rule/impls/link_single.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Ammon Smith
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

//! Rules for single-bracket links.
//!
//! Wikidot, in its infinite wisdom, has two means for designating links.
//! This method allows any URL, either opening in a new tab or not.
//! Its syntax is `[https://example.com/ Label text]`.

use super::prelude::*;
use crate::enums::{AnchorTarget, LinkLabel};

pub const RULE_LINK_SINGLE: Rule = Rule {
    name: "link-single",
    try_consume_fn: link,
};

pub const RULE_LINK_SINGLE_NEW_TAB: Rule = Rule {
    name: "link-single-new-tab",
    try_consume_fn: link_new_tab,
};

fn link<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Element<'t>> {
    trace!(log, "Trying to create a single-bracket link (regular)");

    check_step(parser, Token::LeftBracket)?;

    try_consume_link(log, parser, RULE_LINK_SINGLE, AnchorTarget::Same)
}

fn link_new_tab<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Element<'t>> {
    trace!(log, "Trying to create a single-bracket link (new tab)");

    check_step(parser, Token::LeftBracketSpecial)?;

    try_consume_link(log, parser, RULE_LINK_SINGLE_NEW_TAB, AnchorTarget::NewTab)
}

/// Build a single-bracket link with the given anchor.
fn try_consume_link<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
    rule: Rule,
    anchor: AnchorTarget,
) -> ParseResult<'r, 't, Element<'t>> {
    debug!(log, "Trying to create a single-bracket link"; "anchor" => anchor.name());

    // Gather path for link
    let url = collect_merge(
        log,
        parser,
        rule,
        &[ParseCondition::current(Token::Whitespace)],
        &[
            ParseCondition::current(Token::RightBracket),
            ParseCondition::current(Token::ParagraphBreak),
            ParseCondition::current(Token::LineBreak),
        ],
        None,
    )?;

    if !url_valid(url) {
        return Err(parser.make_error(ParseErrorKind::InvalidUrl));
    }

    debug!(
        log,
        "Retrieved URL for link, now fetching label";
        "url" => url,
    );

    // Gather label for link
    let label = collect_merge(
        log,
        parser,
        rule,
        &[ParseCondition::current(Token::RightBracket)],
        &[
            ParseCondition::current(Token::ParagraphBreak),
            ParseCondition::current(Token::LineBreak),
        ],
        None,
    )?;

    debug!(
        log,
        "Retrieved label for link, now build element";
        "label" => label,
    );

    // Trim label
    let label = label.trim();

    // Build link element
    let element = Element::Link {
        url: cow!(url),
        label: LinkLabel::Text(cow!(label)),
        anchor,
    };

    // Return result
    ok!(element)
}

fn url_valid(url: &str) -> bool {
    const PROTOCOLS: &[&str] = &["http://", "https://", "ftp://"];

    // If url is an empty string
    if url.is_empty() {
        return false;
    }

    // If it's a relative link
    if url.starts_with('/') {
        return true;
    }

    // If it's a URL
    for protocol in PROTOCOLS {
        if url.starts_with(protocol) {
            return true;
        }
    }

    false
}
