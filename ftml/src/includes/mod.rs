/*
 * includes/mod.rs
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

//! This module implements "messy includes", or Wikidot native includes.
//!
//! It is an annoying but necessary hack that parses the psueodblock
//! `[[include-messy]]` and directly replaces that part with the
//! foreign page's wikitext.

#[cfg(test)]
mod test;

mod include_ref;
mod includer;
mod parse;

pub use self::include_ref::{IncludeRef, IncludeVariables};
pub use self::includer::{DebugIncluder, FetchedPage, Includer, NullIncluder};

use self::parse::parse_include_block;
use crate::data::PageRef;
use crate::log::prelude::*;
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref INCLUDE_REGEX: Regex = {
        RegexBuilder::new(r"^\[\[\s*include-messy\s+")
            .case_insensitive(true)
            .multi_line(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap()
    };
}

pub fn include<'t, I, E, F>(
    log: &Logger,
    input: &'t str,
    mut includer: I,
    invalid_return: F,
) -> Result<(String, Vec<PageRef<'t>>), E>
where
    I: Includer<'t, Error = E>,
    F: FnOnce() -> E,
{
    let log = &log.new(slog_o!(
        "filename" => slog_filename!(),
        "lineno" => slog_lineno!(),
        "function" => "include",
        "text" => str!(input),
    ));

    info!(
        log,
        "Finding and replacing all instances of include blocks in text"
    );

    let mut ranges = Vec::new();
    let mut includes = Vec::new();

    // Get include references
    for mtch in INCLUDE_REGEX.find_iter(input) {
        let start = mtch.start();

        debug!(
            log,
            "Found include regex match";
            "start" => start,
            "slice" => mtch.as_str(),
        );

        match parse_include_block(log, &input[start..], start) {
            Ok((include, end)) => {
                ranges.push(start..end);
                includes.push(include);
            }
            Err(_) => warn!(log, "Unable to parse include regex match"),
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
        let (page_ref, _) = include.into();

        info!(
            log,
            "Replacing range for included page";
            "span" => SpanWrap::from(&range),
            "site" => page_ref.site(),
            "page" => page_ref.page(),
        );

        // Ensure the returned page reference matches
        if page_ref != fetched.page_ref {
            return Err(invalid_return());
        }

        // Get replaced content, or error message
        let message;
        let replace_with = match fetched.content {
            Some(ref content) => content,
            None => {
                message = includer.no_such_include(&page_ref)?;
                &message
            }
        };

        // Append page to final list
        pages.push(page_ref);

        // Perform the substitution
        output.replace_range(range, replace_with);
    }

    // Since we iterate in reverse order, the pages are reversed.
    pages.reverse();

    // Return
    Ok((output, pages))
}
