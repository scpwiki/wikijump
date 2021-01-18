/*
 * include/mod.rs
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

#[cfg(test)]
mod test;

mod includer;
mod object;
mod parse;

pub use self::includer::{DebugIncluder, FetchedPages, Includer, NullIncluder};
pub use self::object::{IncludeRef, IncludeVariables, PageRef};

use self::parse::parse_include_block;
use crate::span_wrap::SpanWrap;
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref INCLUDE_REGEX: Regex = {
        RegexBuilder::new(r"^\[\[\s*include\s+")
            .case_insensitive(true)
            .multi_line(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap()
    };
}

pub fn include<'t, I, E>(
    log: &slog::Logger,
    input: &'t str,
    mut includer: I,
) -> Result<(String, Vec<PageRef<'t>>), E>
where
    I: Includer<'t, Error = E>,
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
        debug!(
            log,
            "Found include regex match";
            "span" => SpanWrap::from(mtch.range()),
            "slice" => mtch.as_str(),
        );

        match parse_include_block(log, &input[mtch.range()], mtch.range()) {
            None => debug!(log, "Unable to parse include regex match"),
            Some(include) => {
                ranges.push(mtch.range());
                includes.push(include);
            }
        }
    }

    // Retrieve included pages
    let fetched_pages = includer.include_pages(&includes)?;

    // Substitute inclusions
    //
    // We must iterate backwards for all the indices to be valid

    let ranges_iter = ranges.into_iter();
    let includes_iter = includes.into_iter();

    // Borrowing from the original text and doing in-place insertions
    // will not work here. We are trying to both return the page names
    // (slices from the input string), and replace it with new content.
    let mut output = String::from(input);
    let mut pages = Vec::new();

    for (range, include) in ranges_iter.zip(includes_iter).rev() {
        let (page_ref, _) = include.into();

        debug!(
            log,
            "Replacing range for included page";
            "span" => SpanWrap::from(&range),
            "site" => page_ref.site(),
            "page" => page_ref.page(),
        );

        // Get replaced content, or error message
        let message;
        let replace_with = match fetched_pages.get(&page_ref) {
            Some(content) => content,
            None => {
                message = includer.no_such_include(&page_ref);
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
