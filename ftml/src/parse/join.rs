/*
 * parse/join.rs
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

use crate::ExtractedToken;

/// Helper utility to join adjacent string slices into a single string.
///
/// Previous parsing produces large numbers of text elements next to each other,
/// which we can effectively join by taking the first and last indices in their
/// spans and slicing from the original string instead.
///
/// The passed closure will determine if iteration should continue.
/// If `false` is received, then no further tokens are joined and
/// the result is returned.
pub fn join_strings<'t, F>(
    log: &slog::Logger,
    tokens: &[ExtractedToken<'t>],
    full_text: &'t str,
    mut conditional: F,
) -> &'t str
where
    F: FnMut(&ExtractedToken) -> bool,
{
    debug!(log, "Joining strings among tokens"; "tokens-len" => tokens.len());

    // We need the first element to get the starting slice index.
    // If it's empty, we just return an empty string.
    let start = match tokens.first() {
        Some(extracted) => extracted.span.start,
        None => return "",
    };

    let mut end = start;
    for extracted in tokens {
        // The closure tells us to cease iteration
        if !conditional(extracted) {
            break;
        }

        end = extracted.span.end;
    }

    // Now build the final string slice
    &full_text[start..end]
}
