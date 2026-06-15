/*
 * id_prefix.rs
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

//! Utility to prefix HTML IDs for isolation.
//!
//! This adds `u-` to all IDs in the string (if non-empty)
//! to ensure non-collision with generated elements.
//!
//! However it is intelligent, and doesn't add the `u-` if
//! it's already prefixed with that.

pub fn isolate_ids(id_string: &str) -> String {
    const PREFIX: &str = "u-";
    let mut isolated_ids = String::new();

    for class in id_string.split_whitespace() {
        // Add space separator between each class
        if !isolated_ids.is_empty() {
            isolated_ids.push(' ');
        }

        // Prefix if not already present
        if !class.starts_with(PREFIX) {
            isolated_ids.push_str(PREFIX);
        }

        isolated_ids.push_str(class);
    }

    isolated_ids
}

#[test]
fn test_isolate_ids() {
    macro_rules! test {
        ($input:expr, $expected:expr) => {
            assert_eq!(
                isolate_ids($input),
                $expected,
                "Actual isolated ID string doesn't match expected",
            );
        };
    }

    test!("", "");
    test!("  ", "");
    test!("apple", "u-apple");
    test!("apple banana", "u-apple u-banana");
    test!("apple  banana", "u-apple u-banana");
    test!(" apple  banana", "u-apple u-banana");
    test!(" apple   banana ", "u-apple u-banana");
    test!("apple banana cherry", "u-apple u-banana u-cherry");
    test!("apple  banana cherry", "u-apple u-banana u-cherry");
    test!("  apple  banana\tcherry", "u-apple u-banana u-cherry");
    test!("u-apple banana cherry", "u-apple u-banana u-cherry");
    test!("u-apple u-banana cherry", "u-apple u-banana u-cherry");
    test!("u-apple u-banana u-cherry", "u-apple u-banana u-cherry");
    test!("apple u-banana cherry", "u-apple u-banana u-cherry");
    test!("u-u-apple", "u-u-apple");
}
