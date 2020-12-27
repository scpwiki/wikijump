/*
 * parse/string.rs
 *
 * ftml - Library to parse Wikidot text
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

use std::borrow::Cow;

/// Parses a double-quoted string.
///
/// Takes inputs starting and ending with `"`
/// and containing characters, or any of these
/// escapes:
/// * `\\`
/// * `\"`
/// * `\'`
/// * `\r`
/// * `\n`
/// * `\t`
///
/// # Panics
/// Assumes that the string is in the proper form.
/// If it is not, this function may panic.
pub fn parse_string(input: &str) -> Cow<str> {
    let text = slice_middle(input);
    let mut output = Cow::Borrowed(text);

    // Parse state
    // Tracks if the previous character was '\\'
    let mut wants_escape = false;

    // Iterate and replace
    for (idx, ch) in text.chars().enumerate() {
        if wants_escape {
            let replacement = match ch {
                '\"' => "\"",
                '\'' => "\'",
                'r' => "\r",
                'n' => "\n",
                't' => "\t",
                _ => panic!("Unknown escape character: {}", ch),
            };

            // Replace escape sequence
            output.to_mut().replace_range(idx-1..idx, replacement);

            // Reset
            wants_escape = false;
        } else if ch == '\\' {
            wants_escape = true;
        }
    }

    output
}

/// Slices the first and last characters off of the string.
/// Assumes there are codepoint boundaries there.
fn slice_middle(input: &str) -> &str {
    let len = input.len();
    let last = len - 1;

    &input[1..last]
}

#[test]
fn test_parse_string() {
    macro_rules! test {
        ($input:expr, $expected:expr, $variant:tt) => {{
            let actual = parse_string($input);

            assert_eq!(&actual, $expected, "Actual string (left) doesn't match expected (right)");

            assert!(
                matches!(actual, Cow::$variant),
                "Outputted string of the incorrect variant",
            );
        }};
    }

    // TODO
}

#[test]
fn test_slice_middle() {
    macro_rules! test {
        ($input:expr, $expected:expr) => {{
            let actual = slice_middle($input);

            assert_eq!(actual, $expected, "Actual (left) doesn't match expected (right)");
        }};
    }

    test!("\"\"", "");
    test!("\"!\"", "!");
    test!("\"abc\"", "abc");
    test!("\"apple banana cherry\"", "apple banana cherry");
}
