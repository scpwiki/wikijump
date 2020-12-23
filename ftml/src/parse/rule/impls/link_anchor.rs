/*
 * parse/rule/impls/link_anchor.rs
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

//! Rules for anchor soft-links.
//!
//! A variant on single-bracket links which targets an anchor on the current page,
//! or is a fake link.

use super::prelude::*;
use crate::enums::{AnchorTarget, LinkLabel};
use std::borrow::Cow;
use wikidot_normalize::normalize;

pub const RULE_LINK_ANCHOR: Rule = Rule {
    name: "link-anchor",
    try_consume_fn,
};

fn try_consume_fn<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> Consumption<'r, 't> {
    debug!(log, "Trying to create a single-bracket anchor link");

    // Gather path for link
    let consumption = try_merge(
        log,
        (extracted, remaining, full_text),
        RULE_LINK_ANCHOR,
        &[Token::Whitespace],
        &[Token::RightBracket, Token::ParagraphBreak, Token::LineBreak],
        &[],
    );

    // Return if failure, and get last token for try_merge()
    let (url, extracted, remaining, mut all_exceptions) =
        try_consume_last!(remaining, consumption);

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
    let consumption = try_merge(
        log,
        (extracted, remaining, full_text),
        RULE_LINK_ANCHOR,
        &[Token::RightBracket],
        &[Token::ParagraphBreak, Token::LineBreak],
        &[],
    );

    // Append errors, or return if failure
    let (label, remaining, mut exceptions) = try_consume!(consumption);

    debug!(
        log,
        "Retrieved label for link, now build element";
        "label" => label,
    );

    // Trimming label
    let label = label.trim();

    // Add on new exceptions
    all_exceptions.append(&mut exceptions);

    // Build link element
    let element = Element::Link {
        url,
        label: LinkLabel::Text(cow!(label)),
        anchor: AnchorTarget::Same,
    };

    // Return result
    Consumption::warn(element, remaining, all_exceptions)
}
