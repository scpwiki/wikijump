/*
 * utf16.rs
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

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Utf16IndexMap<'t> {
    /// The underlying UTF-8 string that this map is acting on.
    text: &'t str,

    /// A mapping of UTF-8 byte indices to UTF-16 indices, with the character.
    ///
    /// Schema: utf8_index -> utf16_index
    map: HashMap<usize, usize>,
}

impl<'t> Utf16IndexMap<'t> {
    /// Produces a mapping of UTF-8 byte index to UTF-16 index.
    ///
    /// This enables objects to be converted into using character indices
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

        // Add last index
        if let Some(utf8_index) = last_utf8_index {
            map.insert(utf8_index, utf16_index);
        }

        Utf16IndexMap { text, map }
    }

    /// Converts a UTF-8 byte index into a UTF-16 one.
    ///
    /// # Panics
    /// Panics if the index is out of range for the string.
    #[inline]
    pub fn get_index(&self, utf8_index: usize) -> usize {
        self.map[&utf8_index]
    }

    /// Converts the end index of a UTF-8 character span into the equivalent UTF-16 one.
    ///
    /// Imagine the string "a💣b". In UTF-8 this looks like:
    /// ```raw
    /// [0x61, 0xf0, 0x9f, 0x92, 0xa3, 0x62]
    /// ```
    ///
    /// Whereas in UTF-16 it looks like:
    /// ```raw
    /// [0x61, 0xd83d, 0xdca3, 0x62]
    /// ```
    ///
    /// The UTF-8 span of the emoji is `1..4`, and the UTF-16 span is `1..2`.
    ///
    /// For this string, passing `4` results in `2` being returned.
    /// This is calculated by taking the index of the next character and getting
    /// the byte index right before it.
    ///
    /// # Panics
    /// Panics if the index is out of range for the string.
    #[inline]
    pub fn get_index_end(&self, utf8_index: usize) -> usize {
        self.get_index(utf8_index + 1) - 1
    }
}

#[test]
fn utf16_indices() {
    macro_rules! check {
        ($text:expr, $spans:expr) => {{
            let map = Utf16IndexMap::new($text);
            let spans: &[(usize, usize)] = &$spans;

            let start_indices: Vec<usize> = spans.iter().map(|span| span.0).collect();
            let end_indices: Vec<usize> = spans.iter().map(|span| span.1).collect();

            let start_iterator = $text.char_indices().zip(start_indices).enumerate();
            let end_iterator = $text.char_indices().zip(end_indices).enumerate();

            for (char_index, ((utf8_index, _ch), expected_utf16_index)) in start_iterator {
                let actual_utf16_index = map.get_index(utf8_index);

                assert_eq!(
                    expected_utf16_index,
                    actual_utf16_index,
                    "Actual UTF-16 start index doesn't match expected (char #{})",
                    char_index + 1,
                );
            }

            for (char_index, ((utf8_index, ch), expected_utf16_index)) in end_iterator {
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

    check!("", []);
    check!("abc", [(0, 1), (1, 2), (2, 3)]);
    check!("aßc", [(0, 1), (1, 2), (2, 3)]);
    check!("aℝc", [(0, 1), (1, 2), (2, 3)]);
    check!("a🦀c", [(0, 1), (1, 3), (3, 4)]);
    check!("x💣yßz", [(0, 1), (1, 3), (3, 4), (4, 5), (5, 6)]);
}
