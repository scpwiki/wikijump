/*
 * id_prefix.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2022 Wikijump Team
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

pub fn isolate_ids(id_string: &str) -> String {
    let mut isolated_ids = String::new();

    for class in id_string.split_whitespace() {
        if !isolated_ids.is_empty() {
            isolated_ids.push(' ');
        }

        str_write!(isolated_ids, "u-{}", class);
    }

    isolated_ids
}

#[test]
fn test_isolate_ids() {
    macro_rules! check {
        ($input:expr, $expected:expr) => {
            assert_eq!(
                isolate_ids($input),
                $expected,
                "Actual isolated ID string doesn't match expected",
            );
        };
    }

    check!("", "");
    check!("  ", "");
    check!("apple", "u-apple");
    check!("apple banana", "u-apple u-banana");
    check!("apple  banana", "u-apple u-banana");
    check!(" apple  banana", "u-apple u-banana");
    check!(" apple   banana ", "u-apple u-banana");
    check!("apple banana cherry", "u-apple u-banana u-cherry");
    check!("apple  banana cherry", "u-apple u-banana u-cherry");
    check!("  apple  banana\tcherry", "u-apple u-banana u-cherry");
}
