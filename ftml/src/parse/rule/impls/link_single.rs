/*
 * parse/rule/impls/link_single.rs
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

fn link<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> Consumption<'r, 't> {
    trace!(log, "Trying to create a single-bracket link (regular)");

    try_consume_link(
        log,
        extracted,
        remaining,
        full_text,
        RULE_LINK_SINGLE,
        AnchorTarget::Same,
    )
}

fn link_new_tab<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> Consumption<'r, 't> {
    trace!(log, "Trying to create a single-bracket link (new tab)");

    try_consume_link(
        log,
        extracted,
        remaining,
        full_text,
        RULE_LINK_SINGLE_NEW_TAB,
        AnchorTarget::NewTab,
    )
}

/// Build a single-bracket link with the given anchor.
fn try_consume_link<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
    rule: Rule,
    anchor: AnchorTarget,
) -> Consumption<'r, 't> {
    debug!(log, "Trying to create a single-bracket link"; "anchor" => anchor.name());

    // Gather path for link
    let consumption = try_merge(
        log,
        (extracted, remaining, full_text),
        rule,
        &[Token::Whitespace],
        &[Token::RightBracket, Token::ParagraphBreak, Token::LineBreak],
        &[],
    );

    // Return if failure, and get last token for try_merge()
    let (url, extracted, remaining, mut all_errors) =
        try_consume_last!(remaining, consumption);

    if !url_valid(url) {
        return Consumption::err(ParseError::new(
            ParseErrorKind::InvalidUrl,
            rule,
            extracted,
        ));
    }

    debug!(
        log,
        "Retrieved URL for link, now fetching label";
        "url" => url,
    );

    // Gather label for link
    let consumption = try_merge(
        log,
        (extracted, remaining, full_text),
        rule,
        &[Token::RightBracket],
        &[Token::ParagraphBreak, Token::LineBreak],
        &[],
    );

    // Append errors, or return if failure
    let (label, remaining, mut errors) = try_consume!(consumption);

    debug!(
        log,
        "Retrieved label for link, now build element";
        "label" => label,
    );

    // Trimming label
    let label = label.trim();

    // Add on new errors
    all_errors.append(&mut errors);

    // Build link element
    let element = Element::Link {
        url: cow!(url),
        label: LinkLabel::Text(cow!(label)),
        anchor,
    };

    // Return result
    Consumption::warn(element, remaining, all_errors)
}

fn url_valid(url: &str) -> bool {
    const PROTOCOLS: [&str; 3] = ["http://", "https://", "ftp://"];

    // If url is an empty string
    if url.is_empty() {
        return false;
    }

    // If it's a relative link
    if url.starts_with('/') {
        return true;
    }

    // If it's a URL
    for protocol in &PROTOCOLS {
        if url.starts_with(protocol) {
            return true;
        }
    }

    false
}
