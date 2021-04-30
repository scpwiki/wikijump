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

        for (utf8_index, ch) in text.char_indices() {
            map.insert(utf8_index, utf16_index);
            utf16_index += ch.len_utf16();
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
}

#[test]
fn utf16_indices() {
    macro_rules! check {
        ($text:expr, $indices:expr) => {{
            let map = Utf16IndexMap::new($text);
            let indices: &[usize] = &$indices;
            let iterator = $text.char_indices().zip(indices).enumerate();

            for (char_index, ((utf8_index, _), expected_utf16_index)) in iterator {
                let actual_utf16_index = map.get_index(utf8_index);

                assert_eq!(
                    *expected_utf16_index,
                    actual_utf16_index,
                    "Actual UTF-16 index doesn't match expected (char {})",
                    char_index + 1,
                );
            }
        }};
    }

    check!("", []);
    check!("abc", [1, 2, 3]);
    check!("aßc", [1, 2, 4]);
    check!("ℝeo", [3, 4, 5]);
    check!("x💣yßz", [1, 5, 6, 8, 10]);
}
