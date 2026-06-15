/*
 * includes/mod.rs
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

//! This module implements "messy includes", or legacy Wikidot includes.
//!
//! It is an annoying but necessary hack that parses the psuedoblock
//! `[[include]]` and directly replaces that part with the
//! foreign page's wikitext.

#[warn(missing_docs)]
#[cfg(test)]
mod test;

mod include_ref;
mod includer;
mod parse;

pub use self::include_ref::IncludeRef;
pub use self::includer::{DebugIncluder, FetchedPage, Includer, NullIncluder};

use self::parse::parse_include_block;
use crate::data::PageRef;
use crate::settings::WikitextSettings;
use crate::tree::VariableMap;
use regex::{Regex, RegexBuilder};
use std::sync::LazyLock;

static INCLUDE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"^\[\[\s*include\s+")
        .case_insensitive(true)
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap()
});
static VARIABLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\$(?P<name>[a-zA-Z0-9_\-]+)\}").unwrap());

/// Replaces the include blocks in a string with the content of the pages referenced by those
/// blocks.
pub fn include<'t, I, E, F>(
    input: &'t str,
    settings: &WikitextSettings,
    mut includer: I,
    invalid_return: F,
) -> Result<(String, Vec<PageRef>), E>
where
    I: Includer<'t, Error = E>,
    F: FnOnce() -> E,
{
    if !settings.enable_page_syntax {
        debug!("Includes are disabled for this input, skipping");

        let output = str!(input);
        let pages = vec![];
        return Ok((output, pages));
    }

    info!(
        "Inserting text for all include blocks in text ({} bytes)",
        input.len(),
    );

    let mut ranges = Vec::new();
    let mut includes = Vec::new();

    // Get include references
    for mtch in INCLUDE_REGEX.find_iter(input) {
        let start = mtch.start();

        trace!(
            "Found include regex match (start {}, slice '{}')",
            start,
            mtch.as_str(),
        );

        match parse_include_block(input, start) {
            Ok((include, end)) => {
                ranges.push(start..end);
                includes.push(include);
            }
            Err(_) => warn!("Unable to parse include regex match"),
        }
    }

    // Retrieve included pages
    let fetched_pages = includer.include_pages(&includes)?;

    // Ensure it matches up with the request
    if includes.len() != fetched_pages.len() {
        return Err(invalid_return());
    }

    // Substitute inclusions
    //
    // We must iterate backwards for all the indices to be valid

    let ranges_iter = ranges.into_iter();
    let includes_iter = includes.into_iter();
    let fetched_iter = fetched_pages.into_iter();
    let joined_iter = ranges_iter.zip(includes_iter).zip(fetched_iter).rev();

    // Borrowing from the original text and doing in-place insertions
    // will not work here. We are trying to both return the page names
    // (slices from the input string), and replace it with new content.
    let mut output = String::from(input);
    let mut pages = Vec::new();

    for ((range, include), fetched) in joined_iter {
        let (page_ref, variables) = include.into();

        debug!(
            "Replacing range for included page ({}..{})",
            range.start, range.end,
        );

        // Ensure the returned page reference matches
        if page_ref != fetched.page_ref {
            return Err(invalid_return());
        }

        // Get replaced content, or error message
        let replace_with = match fetched.content {
            // Take fetched content, replace variables
            Some(mut content) => {
                replace_variables(content.to_mut(), &variables);
                content
            }

            // Include not found, return premade template
            None => includer.no_such_include(&page_ref)?,
        };

        // Append page to final list
        pages.push(page_ref);

        // Perform the substitution
        output.replace_range(range, &replace_with);
    }

    // Since we iterate in reverse order, the pages are reversed.
    pages.reverse();

    // Return
    Ok((output, pages))
}

/// Replaces all specified variables in the content to be included.
///
/// Read <https://www.wikidot.com/doc-wiki-syntax:include> for more details.
fn replace_variables(content: &mut String, variables: &VariableMap) {
    let mut matches = Vec::new();

    // Find all variables
    for capture in VARIABLE_REGEX.captures_iter(content) {
        let mtch = capture.get(0).unwrap();
        let name = &capture["name"];

        if let Some(value) = variables.get(name) {
            matches.push((value, mtch.range()));
        }
    }

    // Replace the variables
    // Iterates backwards so indices stay valid
    matches.reverse();
    for (value, range) in matches {
        content.replace_range(range, value);
    }
}
