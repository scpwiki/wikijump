/*
 * preproc/include/mod.rs
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

use crate::{Handle, RemoteResult};
use std::collections::HashMap;
use std::ops::Range;

const MAX_DEPTH: usize = 10;

#[derive(Debug, Clone)]
struct IncludeRef {
    range: Range<usize>,
    name: String,
    page: RemoteResult<Option<String>>,
}

fn substitute_depth(text: &mut String, handle: &dyn Handle, depth: usize) {
    let tokens: Vec<()> = vec![]; // stub

    let mut includes = Vec::new();
    let mut args = HashMap::new();

    // Iterate through include-tokens
    for _token in tokens {
        // stub
        args.insert("name", "test");
        let range = 0..0;
        let name = "page-name";

        // Fetch included resource
        let page = handle.include_page(name, &args);
        let name = str!(name);

        includes.push(IncludeRef { range, name, page });
        args.clear();
    }

    // Go through in reverse order to not mess up indices.
    //
    // This is playing a bit fast and loose with references since
    // we don't actually have a borrow of the string slice.

    let has_includes = !includes.is_empty();

    includes.reverse();
    for include in includes {
        let IncludeRef { range, name, page } = include;

        let final_page = if depth >= MAX_DEPTH {
            // Avoid infinite recursion
            handle.include_max_depth_error(MAX_DEPTH)
        } else {
            match page {
                Ok(Some(content)) => content,
                Ok(None) => handle.include_missing_error(&name),
                Err(error) => error.into(),
            }
        };

        text.replace_range(range, &final_page);
    }

    // Next level of substitution
    if has_includes {
        substitute_depth(text, handle, depth + 1);
    }
}

#[inline]
pub fn substitute(text: &mut String, handle: &dyn Handle) {
    substitute_depth(text, handle, 0)
}
