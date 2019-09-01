/*
 * filter/include/parse.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith
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

use super::Includer;
use crate::{Error, Result};
use pest::Parser;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Range;

#[derive(Debug, Clone, Parser)]
#[grammar = "filter/include.pest"]
struct IncludeParser;

#[derive(Debug, Clone)]
struct IncludeRef {
    range: Range<usize>,
    resource: Cow<'static, str>,
}

// Helper function
pub fn substitute(text: &mut String, includer: &dyn Includer) -> Result<()> {
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
        // TODO limit recursion
        let resource = includer.get_resource(name, &args)?;

        includes.push(IncludeRef { range, resource });
    }

    // Go through in reverse order to not mess up indices
    //
    // This is playing a bit fast and loose with references since
    // we don't actually have a borrow of the string slice.

    includes.reverse();
    for include in includes {
        let IncludeRef { range, resource } = include;

        text.replace_range(range, resource.as_ref());
    }

    // TODO

    Ok(())
}
