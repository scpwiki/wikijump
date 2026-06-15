/*
 * includes/parse.rs
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

//! This module provides functions to parse strings into [`IncludeRef`]s

mod parser {
    // Since pest generates some code that clippy doesn't like
    #![allow(clippy::empty_docs)]

    #[derive(Parser, Debug)]
    #[grammar = "includes/grammar.pest"]
    pub struct IncludeParser;
}

use self::parser::*;
use super::IncludeRef;
use crate::data::{PageRef, PageRefParseError};
use pest::Parser;
use pest::iterators::Pairs;
use std::borrow::Cow;
use std::collections::HashMap;

/// Parses a single include block in the text.
///
/// # Arguments
/// The "start" argument is the index at which the include block starts.
/// It does not necessarily relate to the index of the include within the text str.
///
/// # Return values
/// Returns a tuple of an [`IncludeRef`] that represents the included text and a usize that
/// represents the end index of the include block, such that start..end covers the full include
/// block (before the include goes through).
pub fn parse_include_block<'t>(
    text: &'t str,
    start: usize,
) -> Result<(IncludeRef<'t>, usize), IncludeParseError> {
    match IncludeParser::parse(Rule::include, &text[start..]) {
        Ok(mut pairs) => {
            // Extract inner pairs
            // These actually make up the include block's tokens
            let first = pairs.next().expect("No pairs returned on successful parse");
            let span = first.as_span();

            debug!("Parsed include block");

            // Convert into an IncludeRef
            let include = process_pairs(first.into_inner())?;

            // Adjust offset and return
            Ok((include, start + span.end()))
        }
        Err(error) => {
            warn!("Include block was invalid: {error}");
            Err(IncludeParseError)
        }
    }
}

/// Creates an [`IncludeRef`] out of pest [`Pairs`].
fn process_pairs(mut pairs: Pairs<Rule>) -> Result<IncludeRef, IncludeParseError> {
    let page_raw = pairs.next().ok_or(IncludeParseError)?.as_str();
    let page_ref = PageRef::parse(page_raw)?;

    trace!("Got page for include {page_ref:?}");
    let mut arguments = HashMap::new();
    let mut var_reference = String::new();

    for pair in pairs {
        debug_assert_eq!(pair.as_rule(), Rule::argument);

        let (key, value) = {
            let mut argument_pairs = pair.into_inner();

            let key = argument_pairs
                .next()
                .expect("Argument pairs terminated early")
                .as_str();

            let value = argument_pairs
                .next()
                .expect("Argument pairs terminated early")
                .as_str();

            (key, value)
        };

        trace!("Adding argument for include (key '{key}', value '{value}')");

        // In Wikidot, the first argument takes precedence.
        //
        // However, with nested includes, you can set a fallback
        // by making the first argument its corresponding value.
        //
        // For instance, if we're in `component:test`:
        // ```
        // [[include component:test-backend
        //     width={$width} |
        //     width=300px
        // ]]
        // ```

        var_reference.clear();
        str_write!(var_reference, "{{${key}}}");

        if !arguments.contains_key(key) && value != var_reference {
            let key = Cow::Borrowed(key);
            let value = Cow::Borrowed(value);

            arguments.insert(key, value);
        }
    }

    Ok(IncludeRef::new(page_ref, arguments))
}

#[derive(Debug, PartialEq, Eq)]
pub struct IncludeParseError;

impl From<PageRefParseError> for IncludeParseError {
    #[inline]
    fn from(_: PageRefParseError) -> Self {
        IncludeParseError
    }
}
