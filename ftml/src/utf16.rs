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

    /// A mapping of character indices to UTF-8 byte indices, with the character.
    ///
    /// Schema: utf8_byte_index -> (char_index, char)
    map: HashMap<usize, (usize, char)>,
}

impl<'t> Utf16IndexMap<'t> {
    /// Produces a mapping of UTF-8 byte index to UTF-16 index.
    ///
    /// This enables objects to be converted into using character indices
    /// for strings rather than byte indices. This is useful for environments
    /// which do use UTF-16 strings, such as Javascript (via WebASM).
    pub fn new(text: &'t str) -> Self {
        let mut map = HashMap::new();

        for (char_index, (byte_index, ch)) in text.char_indices().enumerate() {
            map.insert(byte_index, (char_index, ch));
        }

        Utf16IndexMap { text, map }
    }
}
