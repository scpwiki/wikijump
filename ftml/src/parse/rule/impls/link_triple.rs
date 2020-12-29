/*
 * parse/rule/impls/link_triple.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2020 Ammon Smith
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

fn link<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> ParseResult<'r, 't, Element<'t>> {
    trace!(log, "Trying to create a triple-bracket link (regular)");

    try_consume_link(
        log,
        extracted,
        remaining,
        full_text,
        RULE_LINK_TRIPLE,
        AnchorTarget::Same,
    )
}

fn link_new_tab<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> ParseResult<'r, 't, Element<'t>> {
    trace!(log, "Trying to create a triple-bracket link (new tab)");

    try_consume_link(
        log,
        extracted,
        remaining,
        full_text,
        RULE_LINK_TRIPLE_NEW_TAB,
        AnchorTarget::NewTab,
    )
}

/// Build a triple-bracket link with the given anchor.
fn try_consume_link<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
    rule: Rule,
    anchor: AnchorTarget,
) -> ParseResult<'r, 't, Element<'t>> {
    debug!(log, "Trying to create a triple-bracket link"; "anchor" => anchor.name());

    // Gather path for link
    let consumption = try_merge(
        log,
        (extracted, remaining, full_text),
        rule,
        &[Token::Pipe, Token::RightLink],
        &[Token::ParagraphBreak, Token::LineBreak],
        &[],
    );

    // Return if failure, get ready for second part
    let (url, extracted, remaining, exceptions) =
        try_consume_last!(remaining, consumption);

    // Trim text
    let url = url.trim();

    // If url is an empty string, parsing should fail, there's nothing here
    if url.is_empty() {
        return Err(ParseError::new(ParseErrorKind::RuleFailed, rule, extracted));
    }

    // Determine what token we ended on, i.e. which [[[ variant it is.
    match extracted.token {
        // [[[name]]] type links
        Token::RightLink => build_same(log, remaining, exceptions, url, anchor),

        // [[[url|label]]] type links
        Token::Pipe => build_separate(
            log,
            (extracted, remaining, full_text),
            exceptions,
            rule,
            url,
            anchor,
        ),

        // Token was already checked in try_merge(), impossible case
        _ => unreachable!(),
    }
}

/// Helper to build link with the same URL and label.
/// e.g. `[[[name]]]`
fn build_same<'r, 't>(
    log: &slog::Logger,
    remaining: &'r [ExtractedToken<'t>],
    errors: Vec<ParseException<'t>>,
    url: &'t str,
    anchor: AnchorTarget,
) -> ParseResult<'r, 't, Element<'t>> {
    debug!(
        log,
        "Building link with same URL and label";
        "url" => url,
    );

    let element = Element::Link {
        url: cow!(url),
        label: LinkLabel::Url,
        anchor,
    };

    ok!(element, remaining, errors)
}

/// Helper to build link with separate URL and label.
/// e.g. `[[[page|label]]]`, or `[[[page|]]]`
fn build_separate<'r, 't>(
    log: &slog::Logger,
    (extracted, remaining, full_text): (
        &'r ExtractedToken<'t>,
        &'r [ExtractedToken<'t>],
        FullText<'t>,
    ),
    mut all_exc: Vec<ParseException<'t>>,
    rule: Rule,
    url: &'t str,
    anchor: AnchorTarget,
) -> ParseResult<'r, 't, Element<'t>> {
    debug!(
        log,
        "Building link with separate URL and label";
        "url" => url,
    );

    // Gather label for link
    let result = try_merge(
        log,
        (extracted, remaining, full_text),
        rule,
        &[Token::RightLink],
        &[Token::ParagraphBreak, Token::LineBreak],
        &[],
    );

    // Append errors, or return if failure
    let (label, remaining, mut exceptions) = result?.into();

    debug!(
        log,
        "Retrieved label for link, now build element";
        "label" => label,
    );

    // Trimming label
    let label = label.trim();

    // If label is empty, then it takes on the page's title
    // Otherwise, use the label
    let label = if label.is_empty() {
        LinkLabel::Page
    } else {
        LinkLabel::Text(cow!(label))
    };

    // Add on new exceptions
    all_exc.append(&mut exceptions);

    // Build link element
    let element = Element::Link {
        url: cow!(url),
        label,
        anchor,
    };

    // Return result
    ok!(element, remaining, all_exc)
}
