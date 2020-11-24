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

use crate::Handle;
use std::collections::HashMap;
use std::ops::Range;

const MAX_DEPTH: usize = 10;

#[derive(Debug, Clone)]
struct IncludeRef {
    range: Range<usize>,
    name: String,
    page: Result<Option<String>, String>,
}

fn substitute_depth(log: &slog::Logger, text: &mut String, handle: &dyn Handle, depth: usize) {
    info!(log, "Substituting include blocks"; "text" => &*text, "depth" => depth);

    let tokens: Vec<()> = vec![]; // stub

    let mut includes = Vec::new();
    let mut args = HashMap::new();

    // Iterate through include-tokens
    for _token in tokens {
        info!(log, "TODO: iterate through include-tokens");
        // stub
        args.insert("name", "test");
        let range = 0..0;
        let name = "page-name";

        // Fetch included resource
        debug!(
            log,
            "Fetching included page '{}'", name;
            "text" => &*text,
            "argument-count" => args.len(),
            "depth" => depth,
        );

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
            warn!(
                log,
                "Page exceeds maximum page include depth";
                "name" => name,
                "depth" => depth,
                "max-depth" => MAX_DEPTH,
            );
            handle.include_max_depth_error(MAX_DEPTH)
        } else {
            debug!(
                log,
                "Replacing include with fetched page result";
                "page-exists" => page.as_ref().unwrap_or(&None).is_some(),
                "error" => page.is_err(),
            );

            match page {
                Ok(Some(content)) => content,
                Ok(None) => handle.include_missing_error(&name),
                Err(error) => error,
            }
        };

        text.replace_range(range, &final_page);
    }

    // Next level of substitution
    if has_includes {
        trace!(log, "Resultant content has includes, going to next level");
        substitute_depth(log, text, handle, depth + 1);
    }
}

#[inline]
pub fn substitute(log: &slog::Logger, text: &mut String, handle: &dyn Handle) {
    substitute_depth(log, text, handle, 0)
}
