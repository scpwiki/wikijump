/*
 * parsing/rule/impls/link_single.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2026 Wikijump Team
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
use crate::tree::{AnchorTarget, LinkLabel, LinkLocation, LinkType};
use crate::url::is_url;

pub const RULE_LINK_SINGLE: Rule = Rule {
    name: "link-single",
    position: LineRequirement::Any,
    try_consume_fn: link,
};

pub const RULE_LINK_SINGLE_NEW_TAB: Rule = Rule {
    name: "link-single-new-tab",
    position: LineRequirement::Any,
    try_consume_fn: link_new_tab,
};

fn link<'r, 't>(parser: &mut Parser<'r, 't>) -> ParseResult<'r, 't, Elements<'t>> {
    trace!("Trying to create a single-bracket link (regular)");
    assert_step(parser, Token::LeftBracket)?;
    try_consume_link(parser, RULE_LINK_SINGLE, None)
}

fn link_new_tab<'r, 't>(
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    trace!("Trying to create a single-bracket link (new tab)");
    assert_step(parser, Token::LeftBracketStar)?;
    try_consume_link(parser, RULE_LINK_SINGLE_NEW_TAB, Some(AnchorTarget::NewTab))
}

/// Build a single-bracket link with the given target.
fn try_consume_link<'r, 't>(
    parser: &mut Parser<'r, 't>,
    rule: Rule,
    target: Option<AnchorTarget>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(
        "Trying to create a single-bracket link (target {})",
        match target {
            Some(target) => target.name(),
            None => "<none>",
        },
    );

    // Gather path for link
    let url = collect_text(
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

    // Return error if the resultant URL is not valid.
    if !url_valid(url) {
        return Err(parser.make_err(ParseErrorKind::InvalidUrl));
    }

    trace!("Retrieved URL '{url}' for link, now fetching label");

    // Gather label for link
    let label = collect_text(
        parser,
        rule,
        &[ParseCondition::current(Token::RightBracket)],
        &[
            ParseCondition::current(Token::ParagraphBreak),
            ParseCondition::current(Token::LineBreak),
        ],
        None,
    )?;

    trace!("Retrieved label for link, now build element (label '{label}')");

    // Trim label
    let label = label.trim();

    // Build link element
    let element = Element::Link {
        ltype: LinkType::Direct,
        link: LinkLocation::Url(cow!(url)),
        label: LinkLabel::Text(cow!(label)),
        target,
    };

    // Return result
    ok!(element)
}

fn url_valid(url: &str) -> bool {
    // If url is an empty string
    if url.is_empty() {
        return false;
    }

    // If it's a relative link
    if url.starts_with('/') {
        return true;
    }

    // If it's a URL
    if is_url(url) {
        return true;
    }

    false
}
