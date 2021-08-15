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
use crate::tree::{AnchorTarget, LinkLabel, LinkLocation};
use std::borrow::Cow;

pub const RULE_LINK_TRIPLE: Rule = Rule {
    name: "link-triple",
    try_consume_fn: link,
};

pub const RULE_LINK_TRIPLE_NEW_TAB: Rule = Rule {
    name: "link-triple-new-tab",
    try_consume_fn: link_new_tab,
};

fn link<'p, 'r, 't>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    trace!(log, "Trying to create a triple-bracket link (regular)");

    check_step(parser, Token::LeftLink)?;

    try_consume_link(log, parser, RULE_LINK_TRIPLE, None)
}

fn link_new_tab<'p, 'r, 't>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    trace!(log, "Trying to create a triple-bracket link (new tab)");

    check_step(parser, Token::LeftLinkStar)?;

    try_consume_link(
        log,
        parser,
        RULE_LINK_TRIPLE_NEW_TAB,
        Some(AnchorTarget::NewTab),
    )
}

/// Build a triple-bracket link with the given target.
fn try_consume_link<'p, 'r, 't>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
    rule: Rule,
    target: Option<AnchorTarget>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(
        log,
        "Trying to create a triple-bracket link";
        "target" => target.map(|t| t.name()),
    );

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
    log: &Logger,
    _parser: &'p mut Parser<'r, 't>,
    url: &'t str,
    target: Option<AnchorTarget>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(
        log,
        "Building link with same URL and label";
        "url" => url,
    );

    // Remove category, if present
    let label = strip_category(url).map(Cow::Borrowed);

    // Build and return element
    let element = Element::Link {
        url: LinkLocation::parse(cow!(url)),
        label: LinkLabel::Url(label),
        target,
    };

    ok!(element)
}

/// Helper to build link with separate URL and label.
/// e.g. `[[[page|label]]]`, or `[[[page|]]]`
fn build_separate<'p, 'r, 't>(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
    rule: Rule,
    url: &'t str,
    target: Option<AnchorTarget>,
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
        url: LinkLocation::parse(cow!(url)),
        label,
        target,
    };

    // Return result
    ok!(element)
}

/// Strip off the category for use in URL triple-bracket links.
///
/// The label for a URL link is its URL, but without its category.
/// For instance, `theme: Sigma-9` becomes just `Sigma-9`.
///
/// It returns `Some(_)` if a slice was performed, and `None` if
/// the string would have been returned as-is.
fn strip_category(url: &str) -> Option<&str> {
    match url.find(':') {
        // Link with site, e.g. :scp-wiki:component:image-block.
        // Remove leading colon and use the regular strip case (below).
        Some(0) => strip_category(&url[1..]),

        // Link with category but no site, e.g. theme:sigma-9.
        Some(idx) => Some(url[idx + 1..].trim_start()),

        // No stripping necessary
        None => None,
    }
}

#[test]
fn test_strip_category() {
    macro_rules! check {
        ($input:expr, $expected:expr $(,)?) => {{
            let actual = strip_category($input);

            assert_eq!(
                actual, $expected,
                "Actual stripped URL label doesn't match expected",
            );
        }};
    }

    check!("", None);
    check!("scp-001", None);
    check!("Guide Hub", None);
    check!("theme:just-girly-things", Some("just-girly-things"));
    check!("theme: just-girly-things", Some("just-girly-things"));
    check!("theme: Just Girly Things", Some("Just Girly Things"));
    check!("component:fancy-sidebar", Some("fancy-sidebar"));
    check!("component:Fancy Sidebar", Some("Fancy Sidebar"));
    check!("component: Fancy Sidebar", Some("Fancy Sidebar"));
    check!(
        "multiple:categories:here:test",
        Some("categories:here:test"),
    );
    check!(
        "multiple: categories: here: test",
        Some("categories: here: test"),
    );
    check!(":scp-wiki:scp-001", Some("scp-001"));
    check!(":scp-wiki : SCP-001", Some("SCP-001"));
    check!(": snippets : redirect", Some("redirect"));
    check!(":", None);
    check!("::::::", None);
}
