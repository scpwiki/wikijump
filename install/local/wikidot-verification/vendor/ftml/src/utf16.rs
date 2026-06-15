/*
 * utf16.rs
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

use std::collections::HashMap;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct Utf16IndexMap<'t> {
    /// A mapping of UTF-8 byte indices to UTF-16 indices, with the character.
    ///
    /// Schema: utf8_index -> utf16_index
    map: HashMap<usize, usize>,

    /// Borrow marker for the underlying string.
    ///
    /// This prevents this object from being valid if the underlying
    /// UTF-8 string is destructed.
    ///
    /// However since we don't actually need the string's contents,
    /// we use `PhantomData` here.
    marker: PhantomData<&'t str>,
}

impl<'t> Utf16IndexMap<'t> {
    /// Produces a mapping of UTF-8 byte index to UTF-16 index.
    ///
    /// This enables objects to be converted from UTF-8 into UTF-16 using character indices
    /// for strings rather than byte indices. This is useful for environments
    /// which do use UTF-16 strings, such as Javascript (via WebASM).
    pub fn new(text: &'t str) -> Self {
        let mut map = HashMap::new();
        let mut utf16_index = 0;
        let mut last_utf8_index = None;

        // Add index for the start of each character
        for (utf8_index, ch) in text.char_indices() {
            map.insert(utf8_index, utf16_index);
            utf16_index += ch.len_utf16();
            last_utf8_index = Some(utf8_index + ch.len_utf8());
        }

        // Add last index, needed for the final token span.
        if let Some(utf8_index) = last_utf8_index {
            map.insert(utf8_index, utf16_index);
        }

        Utf16IndexMap {
            map,
            marker: PhantomData,
        }
    }

    /// Converts a UTF-8 byte index into a UTF-16 one.
    ///
    /// # Panics
    /// Panics if the index is out of range for the string,
    /// or the index is not on a UTF-8 byte boundary.
    #[inline]
    pub fn get_index(&self, utf8_index: usize) -> usize {
        self.map[&utf8_index]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn utf16_indices() {
        macro_rules! test {
            ($text:expr, $spans:expr) => {{
                let map = Utf16IndexMap::new($text);
                let spans: &[(usize, usize)] = &$spans;

                let start_indices: Vec<usize> = spans.iter().map(|span| span.0).collect();
                let end_indices: Vec<usize> = spans.iter().map(|span| span.1).collect();

                let start_iterator = $text.char_indices().zip(start_indices).enumerate();
                let end_iterator = $text.char_indices().zip(end_indices).enumerate();

                for (char_index, ((utf8_index, _), expected_utf16_index)) in
                    start_iterator
                {
                    let actual_utf16_index = map.get_index(utf8_index);

                    assert_eq!(
                        expected_utf16_index,
                        actual_utf16_index,
                        "Actual UTF-16 start index doesn't match expected (char #{})",
                        char_index + 1,
                    );
                }

                for (char_index, ((utf8_index, ch), expected_utf16_index)) in end_iterator
                {
                    let actual_utf16_index = map.get_index(utf8_index + ch.len_utf8());

                    assert_eq!(
                        expected_utf16_index,
                        actual_utf16_index,
                        "Actual UTF-16 end index doesn't match expected (char #{})",
                        char_index + 1,
                    );
                }
            }};
        }

        test!("", []);
        test!("abc", [(0, 1), (1, 2), (2, 3)]);
        test!("aßc", [(0, 1), (1, 2), (2, 3)]);
        test!("aℝc", [(0, 1), (1, 2), (2, 3)]);
        test!("a🦀c", [(0, 1), (1, 3), (3, 4)]);
        test!("x💣yßz", [(0, 1), (1, 3), (3, 4), (4, 5), (5, 6)]);
    }

    fn check(text: &str) {
        let map = Utf16IndexMap::new(text);
        let utf16_bytes: Vec<u16> = text.encode_utf16().collect();

        for (utf8_start, ch) in text.char_indices() {
            // Get UTF-8 slice
            let utf8_stop = utf8_start + ch.len_utf8();
            let utf8_slice = &text[utf8_start..utf8_stop];

            // Get equivalent UTF-16 slice
            let utf16_start = map.get_index(utf8_start);
            let utf16_stop = map.get_index(utf8_stop);
            let utf16_slice = &utf16_bytes[utf16_start..utf16_stop];

            // Check that converting from UTF-16 -> UTF-8 yields the same data
            let utf16_conv_str =
                String::from_utf16(utf16_slice).expect("UTF-16 slice wasn't valid");

            assert_eq!(
                utf8_slice, utf16_conv_str,
                "Converted UTF-16 -> UTF-8 slice didn't match",
            );

            // Check that converting from UTF-8 -> yields the same data
            let utf8_conv_bytes: Vec<u16> = utf8_slice.encode_utf16().collect();

            assert_eq!(
                utf16_slice, utf8_conv_bytes,
                "Converted UTF-8 -> UTF-16 slice didn't match",
            );
        }
    }

    #[test]
    fn utf16_slices() {
        check("");
        check("a");

        check("abc");
        check("aßc");
        check("aℝc");
        check("a🦀c");

        check("b");
        check("ß");
        check("ℝ");
        check("🦀");

        check("1b");
        check("1ß");
        check("1ℝ");
        check("1🦀");

        check("b1");
        check("ß1");
        check("ℝ1");
        check("🦀1");

        check("bb");
        check("ßß");
        check("ℝℝ");
        check("🦀🦀");

        check("2bb");
        check("2ßß");
        check("2ℝℝ");
        check("2🦀🦀");

        check("bb2");
        check("ßß2");
        check("ℝℝ2");
        check("🦀🦀2");

        check("bßℝ🦀");
        check("🦀ℝßb");
        check("b_ß_ℝ_🦀");
        check("b__ß__ℝ__🦀");

        check("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
        check("ßßßßßßßßßßßßßßßßßßßßßßßßßßßßßßßßßßßßßßßßßß");
        check("ℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝℝ");
        check("🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(4096))]

        #[test]
        fn utf16_prop(s in ".*") {
            check(&s);
        }
    }
}
