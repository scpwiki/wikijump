/*
 * parsing/rule/impls/link_triple.rs
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

//! Rules for triple-bracket links.
//!
//! This method of designating links is for local pages.
//! The syntax here uses a pipe to separate the destination from the label.
//! However, this method also works for regular URLs, for some reason.
//!
//! Wikidot, in its infinite wisdom, has two means for designating links.
//! This method allows any URL, either opening in a new tab or not.
//! Its syntax is `[[[page-name | Label text]`.

use super::prelude::*;
use crate::enums::{AnchorTarget, LinkLabel};

pub const RULE_LINK_TRIPLE: Rule = Rule {
    name: "link-triple",
    try_consume_fn: link,
};

pub const RULE_LINK_TRIPLE_NEW_TAB: Rule = Rule {
    name: "link-triple-new-tab",
    try_consume_fn: link_new_tab,
};

fn link<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    trace!(log, "Trying to create a triple-bracket link (regular)");

    check_step(parser, Token::LeftLink)?;

    try_consume_link(log, parser, RULE_LINK_TRIPLE, AnchorTarget::Same)
}

fn link_new_tab<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    trace!(log, "Trying to create a triple-bracket link (new tab)");

    check_step(parser, Token::LeftLinkSpecial)?;

    try_consume_link(log, parser, RULE_LINK_TRIPLE_NEW_TAB, AnchorTarget::NewTab)
}

/// Build a triple-bracket link with the given target.
fn try_consume_link<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
    rule: Rule,
    target: AnchorTarget,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(log, "Trying to create a triple-bracket link"; "target" => target.name());

    // Gather path for link
    let (url, last) = collect_text_keep(
        log,
        parser,
        rule,
        &[
            ParseCondition::current(Token::Pipe),
            ParseCondition::current(Token::RightLink),
        ],
        &[
            ParseCondition::current(Token::ParagraphBreak),
            ParseCondition::current(Token::LineBreak),
        ],
        None,
    )?;

    debug!(
        log,
        "Retrieved url for link, now build element";
        "url" => url,
    );

    // Trim text
    let url = url.trim();

    // If url is an empty string, parsing should fail, there's nothing here
    if url.is_empty() {
        return Err(parser.make_warn(ParseWarningKind::RuleFailed));
    }

    // Determine what token we ended on, i.e. which [[[ variant it is.
    match last.token {
        // [[[name]]] type links
        Token::RightLink => build_same(log, parser, url, target),

        // [[[url|label]]] type links
        Token::Pipe => build_separate(log, parser, rule, url, target),

        // Token was already checked in collect_text(), impossible case
        _ => unreachable!(),
    }
}

/// Helper to build link with the same URL and label.
/// e.g. `[[[name]]]`
fn build_same<'p, 'r, 't>(
    log: &slog::Logger,
    _parser: &'p mut Parser<'r, 't>,
    url: &'t str,
    target: AnchorTarget,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(
        log,
        "Building link with same URL and label";
        "url" => url,
    );

    let element = Element::Link {
        url: cow!(url),
        label: LinkLabel::Url,
        target,
    };

    ok!(element)
}

/// Helper to build link with separate URL and label.
/// e.g. `[[[page|label]]]`, or `[[[page|]]]`
fn build_separate<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut Parser<'r, 't>,
    rule: Rule,
    url: &'t str,
    target: AnchorTarget,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(
        log,
        "Building link with separate URL and label";
        "url" => url,
    );

    // Gather label for link
    let label = collect_text(
        log,
        parser,
        rule,
        &[ParseCondition::current(Token::RightLink)],
        &[
            ParseCondition::current(Token::ParagraphBreak),
            ParseCondition::current(Token::LineBreak),
        ],
        None,
    )?;

    debug!(
        log,
        "Retrieved label for link, now building element";
        "label" => label,
    );

    // Trim label
    let label = label.trim();

    // If label is empty, then it takes on the page's title
    // Otherwise, use the label
    let label = if label.is_empty() {
        LinkLabel::Page
    } else {
        LinkLabel::Text(cow!(label))
    };

    // Build link element
    let element = Element::Link {
        url: cow!(url),
        label,
        target,
    };

    // Return result
    ok!(element)
}
