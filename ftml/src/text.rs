/*
 * text.rs
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

use crate::log::prelude::*;
use crate::parsing::ExtractedToken;

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

    /// Returns the entire inner string. This should not be used.
    ///
    /// If you wish to slice between tokens, use the other methods instead.
    /// This is for very unusual cases where you need the entire input string
    /// as-is, with no tokenization.
    #[inline]
    #[doc(hidden)]
    pub(crate) fn inner(&self) -> &'t str {
        self.text
    }

    /// Slices from the given start to end token.
    ///
    /// This is performed inclusively, capturing both tokens on each side,
    /// and all the tokens which lie in the middle.
    ///
    /// # Panics
    /// If the ending token does not come after the first, or if
    /// the slices specified are out of range for the string (unlikely),
    /// this function will panic.
    pub fn slice(
        &self,
        log: &Logger,
        start_token: &ExtractedToken,
        end_token: &ExtractedToken,
    ) -> &'t str {
        let start = start_token.span.start;
        let end = end_token.span.end;

        self.slice_impl(log, "full", start, end)
    }

    /// Slices from the given start, but before the end token.
    ///
    /// This is performed exclusively, capturing the full starting token,
    /// but terminating at the start of the end token (capturing none of it).
    ///
    /// This function is provided specifically because it easier to bump
    /// the start token should you wish to exclude it, but doing so
    /// for the end token is not trivial while observing lifetime safety rules.
    ///
    /// # Panics
    /// If the ending token does not come after the first, or if
    /// the slices specified are out of range for the string (unlikely),
    /// this function will panic.
    pub fn slice_partial(
        &self,
        log: &Logger,
        start_token: &ExtractedToken,
        end_token: &ExtractedToken,
    ) -> &'t str {
        let start = start_token.span.start;
        let end = end_token.span.start;

        self.slice_impl(log, "partial", start, end)
    }

    fn slice_impl(
        &self,
        log: &Logger,
        _slice_kind: &'static str,
        start: usize,
        end: usize,
    ) -> &'t str {
        info!(
            log,
            "Extracting {} slice from full text",
            _slice_kind;
            "start" => start,
            "end" => end,
        );

        assert!(
            start <= end,
            "Starting index is later than the ending index: {} > {}",
            start,
            end,
        );

        &self.text[start..end]
    }
}

#[test]
fn slice() {
    use crate::parsing::Token;

    let log = crate::build_logger();
    let text = "Apple banana!";
    let full_text = FullText::new(text);

    macro_rules! range {
        ($span:expr) => {
            &ExtractedToken {
                token: Token::Other,
                slice: &text[$span],
                span: $span,
            }
        };
    }

    {
        let slice = full_text.slice(&log, range!(0..1), range!(4..5));

        assert_eq!(slice, "Apple", "Full slice didn't match expected");
    }

    {
        let slice = full_text.slice_partial(&log, range!(6..9), range!(12..13));

        assert_eq!(slice, "banana", "Partial slice didn't match expected");
    }
}

#[test]
#[should_panic]
fn slice_invalid() {
    use crate::parsing::Token;

    let log = crate::build_logger();
    let text = "Durian...";
    let full_text = FullText::new(text);

    macro_rules! range {
        ($span:expr) => {
            &ExtractedToken {
                token: Token::Other,
                slice: &text[$span],
                span: $span,
            }
        };
    }

    // "Durian"
    let _ = full_text.slice(&log, range!(6..7), range!(0..1));
}
