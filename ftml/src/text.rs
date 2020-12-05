/*
 * text.rs
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

/// Wrapper for the input string that was tokenized.
///
/// This structure does not expose the internal string (preventing weird ad-hoc
/// or hack parsing), but permits joining adjacent `ExtractedToken` string slices
/// by selecting from the original text source.
#[derive(Debug, Copy, Clone)]
pub struct FullText<'t> {
    text: &'t str,
}

impl<'t> FullText<'t> {
    #[inline]
    pub fn new(text: &'t str) -> Self {
        FullText { text }
    }

    /// Slices from the given start to end token.
    ///
    /// # Panics
    /// If the ending token does not come after the first, or if
    /// the slices specified are out of range for the string (unlikely),
    /// this function will panic.
    pub fn slice(&self, start_token: &ExtractedToken, end_token: &ExtractedToken) -> &'t str {
        let start = start_token.span.start;
        let end = end_token.span.end;

        if start > end {
            panic!(
                "Starting index is later than the ending index: {} > {}",
                start, end
            );
        }

        &self.text[start..end]
    }
}
