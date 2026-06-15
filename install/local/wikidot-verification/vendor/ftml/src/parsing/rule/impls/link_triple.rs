/*
 * parsing/rule/impls/link_triple.rs
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

pub const RULE_LINK_TRIPLE: Rule = Rule {
    name: "link-triple",
    position: LineRequirement::Any,
    try_consume_fn: link,
};

pub const RULE_LINK_TRIPLE_NEW_TAB: Rule = Rule {
    name: "link-triple-new-tab",
    position: LineRequirement::Any,
    try_consume_fn: link_new_tab,
};

fn link<'r, 't>(parser: &mut Parser<'r, 't>) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Trying to create a triple-bracket link (regular)");
    assert_step(parser, Token::LeftLink)?;
    try_consume_link(parser, RULE_LINK_TRIPLE, None)
}

fn link_new_tab<'r, 't>(
    parser: &mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Trying to create a triple-bracket link (new tab)");
    assert_step(parser, Token::LeftLinkStar)?;
    try_consume_link(parser, RULE_LINK_TRIPLE_NEW_TAB, Some(AnchorTarget::NewTab))
}

/// Build a triple-bracket link with the given target.
fn try_consume_link<'r, 't>(
    parser: &mut Parser<'r, 't>,
    rule: Rule,
    target: Option<AnchorTarget>,
) -> ParseResult<'r, 't, Elements<'t>> {
    trace!("Trying to create a triple-bracket link");

    // Gather path for link
    let (url, last) = collect_text_keep(
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

    trace!("Retrieved url for link, now build element (url: '{url}')");

    // Trim text
    let url = url.trim();

    // If url is an empty string, parsing should fail, there's nothing here
    if url.is_empty() {
        return Err(parser.make_err(ParseErrorKind::RuleFailed));
    }

    // Determine what token we ended on, i.e. which [[[ variant it is.
    match last.token {
        // [[[name]]] type links
        Token::RightLink => build_same(parser, url, target),

        // [[[url|label]]] type links
        Token::Pipe => build_separate(parser, rule, url, target),

        // Token was already checked in collect_text(), impossible case
        _ => unreachable!(),
    }
}

/// Helper to build link with the same URL and label.
/// e.g. `[[[name]]]`
fn build_same<'r, 't>(
    parser: &mut Parser<'r, 't>,
    url: &'t str,
    target: Option<AnchorTarget>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Building link with same URL and label (url '{url}')");

    // Remove category, if present.
    // If None, then the label is the original URL.
    let label = match strip_category(url) {
        Some(stripped) => cow!(stripped),
        None => cow!(url),
    };

    // Parse out link location
    let (link, ltype) =
        match LinkLocation::parse_with_interwiki(cow!(url), parser.settings()) {
            Some(result) => result,
            None => return Err(parser.make_err(ParseErrorKind::RuleFailed)),
        };

    // Build and return element
    let element = Element::Link {
        ltype,
        link,
        label: LinkLabel::Slug(label),
        target,
    };

    ok!(element)
}

/// Helper to build link with separate URL and label.
/// e.g. `[[[page|label]]]`, or `[[[page|]]]`
fn build_separate<'r, 't>(
    parser: &mut Parser<'r, 't>,
    rule: Rule,
    url: &'t str,
    target: Option<AnchorTarget>,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Building link with separate URL and label (url '{url}')");

    // Gather label for link
    let label = collect_text(
        parser,
        rule,
        &[ParseCondition::current(Token::RightLink)],
        &[
            ParseCondition::current(Token::ParagraphBreak),
            ParseCondition::current(Token::LineBreak),
        ],
        None,
    )?;

    trace!("Retrieved label for link, now building element (label '{label}')");

    // Trim label
    let label = label.trim();

    // If label is empty, then it takes on the page's title
    // Otherwise, use the label
    let label = if label.is_empty() {
        LinkLabel::Page
    } else {
        LinkLabel::Text(cow!(label))
    };

    // Parse out link location
    let (link, ltype) =
        match LinkLocation::parse_with_interwiki(cow!(url), parser.settings()) {
            Some(result) => result,
            None => return Err(parser.make_err(ParseErrorKind::RuleFailed)),
        };

    // Build link element
    let element = Element::Link {
        ltype,
        link,
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
        Some(0) => {
            let url = &url[1..];

            // If there is no colon, it's malformed, return None.
            // Else, return a stripped version
            url.find(':').map(|idx| {
                let url = url[idx + 1..].trim_start();

                // Skip past the site portion, then use the regular strip case.
                //
                // We unwrap_or() here because, at minimum, we return the substring
                // not containing the site.
                strip_category(url).unwrap_or(url)
            })
        }

        // Link with category but no site, e.g. theme:sigma-9.
        Some(idx) => Some(url[idx + 1..].trim_start()),

        // No stripping necessary
        None => None,
    }
}

#[test]
fn test_strip_category() {
    macro_rules! test {
        ($input:expr, $expected:expr $(,)?) => {{
            let actual = strip_category($input);

            assert_eq!(
                actual, $expected,
                "Actual stripped URL label doesn't match expected",
            );
        }};
    }

    test!("", None);
    test!("scp-001", None);
    test!("Guide Hub", None);
    test!("theme:just-girly-things", Some("just-girly-things"));
    test!("theme: just-girly-things", Some("just-girly-things"));
    test!("theme: Just Girly Things", Some("Just Girly Things"));
    test!("component:fancy-sidebar", Some("fancy-sidebar"));
    test!("component:Fancy Sidebar", Some("Fancy Sidebar"));
    test!("component: Fancy Sidebar", Some("Fancy Sidebar"));
    test!(
        "multiple:categories:here:test",
        Some("categories:here:test"),
    );
    test!(
        "multiple: categories: here: test",
        Some("categories: here: test"),
    );
    test!(":scp-wiki:scp-001", Some("scp-001"));
    test!(":scp-wiki : SCP-001", Some("SCP-001"));
    test!(":scp-wiki:system:recent-changes", Some("recent-changes"));
    test!(
        ":scp-wiki : system : Recent Changes",
        Some("Recent Changes"),
    );
    test!(": snippets : redirect", Some("redirect"));
    test!(":", None);
}
