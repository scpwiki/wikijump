/*
 * preprocess/include/mod.rs
 *
 * ftml - Library to parse Wikidot code
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

#[cfg(test)]
mod test;

use crate::{Error, Handle, Result};
use pest::Parser;
use std::collections::HashMap;
use std::ops::Range;

const MAX_DEPTH: usize = 10;

#[derive(Debug, Clone, Parser)]
#[grammar = "preprocess/include.pest"]
struct IncludeParser;

#[derive(Debug, Clone)]
struct IncludeRef {
    range: Range<usize>,
    name: String,
    page: Option<String>,
}

fn substitute_n(text: &mut String, handle: &dyn Handle, depth: usize) -> Result<()> {
    let pairs = match IncludeParser::parse(Rule::page, text) {
        Ok(mut pairs) => get_inner_pairs!(pairs),
        Err(err) => {
            return Err(Error::Msg(format!(
                "Include transform parsing error: {}",
                err
            )));
        }
    };

    let mut includes = Vec::new();
    let mut args = HashMap::new();

    // Iterate through [[include]]s
    for pair in pairs {
        if pair.as_rule() != Rule::include {
            continue;
        }

        let range = {
            let span = pair.as_span();
            span.start()..span.end()
        };

        let mut pairs = pair.into_inner();
        let name = {
            let pair = pairs.next().expect("Include pairs iterator was empty");

            debug_assert_eq!(pair.as_rule(), Rule::resource);

            pair.as_str()
        };

        // Parse arguments
        args.clear();
        for pair in pairs {
            debug_assert_eq!(pair.as_rule(), Rule::argument);

            let mut pairs = pair.into_inner();

            let key = pairs
                .next()
                .expect("Argument pairs iterator was empty")
                .as_str()
                .trim();

            let value = pairs
                .next()
                .expect("Argument pairs iterator had only one element")
                .as_str()
                .trim();

            args.insert(key, value);
        }

        // Fetch included resource
        let page = handle.include_page(name, &args)?;
        let name = str!(name);

        includes.push(IncludeRef { range, name, page });
    }

    // Go through in reverse order to not mess up indices.
    //
    // This is playing a bit fast and loose with references since
    // we don't actually have a borrow of the string slice.

    let included = !includes.is_empty();

    includes.reverse();
    for include in includes {
        let IncludeRef { range, name, page } = include;

        let final_page = if depth >= MAX_DEPTH {
            handle.include_max_depth_error(MAX_DEPTH)
        } else {
            page.unwrap_or_else(|| handle.include_missing_error(&name))
        };

        text.replace_range(range, &final_page);
    }

    if included {
        substitute_n(text, handle, depth + 1)?;
    }

    // TODO

    Ok(())
}

#[inline]
pub fn substitute(text: &mut String, handle: &dyn Handle) -> Result<()> {
    substitute_n(text, handle, 0)
}
