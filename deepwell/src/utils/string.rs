/*
 * utils/string.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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

use regex::Regex;

/// Replaces all instances of the given fixed string in the buffer, in-place.
pub fn replace_in_place(string: &mut String, pattern: &str, replacement: &str) {
    while let Some(index) = string.find(pattern) {
        let end = index + replacement.len();
        string.replace_range(index..end, replacement);
    }
}

/// Replaces all matches for the given regex in the buffer, in-place.
pub fn regex_replace_in_place(string: &mut String, pattern: &Regex, replacement: &str) {
    while let Some(mtch) = pattern.find(string) {
        let range = mtch.start()..mtch.end();
        string.replace_range(range, replacement);
    }
}

/// Removes the given prefix in the buffer, if it exists, in-place.
pub fn trim_start_matches_in_place(string: &mut String, pattern: &str) {
    if string.starts_with(pattern) {
        string.drain(..pattern.len());
    }
}

/// Removes the given suffix in the buffer, if it exists, in-place.
#[allow(dead_code)]
pub fn trim_end_matches_in_place(string: &mut String, pattern: &str) {
    if string.starts_with(pattern) {
        string.drain(pattern.len() - 1..);
    }
}
