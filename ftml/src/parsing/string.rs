/*
 * parsing/string.rs
 *
 * ftml - Library to parse Wikidot text
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
/// If in invalid escape is found, the input
/// is returned. So for `\$`, it will emit a
/// `\` followed by a `$`.
pub fn parse_string(input: &str) -> Cow<str> {
    // We could do an iteration thing, but tracking
    // the index across replacements is complicated.
    //
    // So we check if there are any possible escapes,
    // and if so, build a new string.
    //
    // This removes the double quotes on either end
    // and lets us only deal with the center.
    // If it's not a string (i.e. doesn't start/end with ")
    // then it just quits.

    let input = match slice_middle(input) {
        Some(input) => input,
        None => {
            warn!("Not a 'string', returning as-is: {:?}", input);
            return Cow::Borrowed(input);
        }
    };

    if !input.contains('\\') {
        trace!("No escapes, returning as-is: {:?}", input);
        return Cow::Borrowed(input);
    }

    let mut output = String::new();
    let mut wants_escape = false;

    for ch in input.chars() {
        if wants_escape {
            match escape_char(ch) {
                Some(replacement) => {
                    trace!("Replacing backslash escape: \\{ch}");
                    output.push(replacement);
                }
                None => {
                    warn!("Invalid backslash escape found, ignoring: \\{ch}");
                    output.push('\\');
                    output.push(ch);
                }
            }

            wants_escape = false;
        } else if ch == '\\' {
            wants_escape = true;
        } else {
            output.push(ch);
        }
    }

    Cow::Owned(output)
}

/// Remove the contents of a string if it is one.
///
/// Checks if the first and last characters are ASCII `"`,
/// and if so, slices the first and last characters off of them.
/// Does not make any assumptions about codepoints.
fn slice_middle(input: &str) -> Option<&str> {
    // Starts and ends with "
    //
    // Regarding the length check:
    // We can use byte length here, since ASCII " x2 is 2 bytes,
    // so any other irregular pattern must be *at least* that.
    //
    // If shorter, it cannot be valid.
    if input.len() < 2 || !input.starts_with('"') || !input.ends_with('"') {
        return None;
    }

    // Okay, we know the first and last chars are ASCII, it's safe to slice
    let last = input.len() - 1;
    Some(&input[1..last])
}

/// Helper function to convert escapes to the actual character.
fn escape_char(ch: char) -> Option<char> {
    let escaped = match ch {
        '\\' => '\\',
        '\"' => '\"',
        '\'' => '\'',
        'r' => '\r',
        'n' => '\n',
        't' => '\t',
        _ => return None,
    };

    Some(escaped)
}

#[test]
fn test_parse_string() {
    macro_rules! test {
        ($input:expr, $expected:expr, $variant:tt $(,)?) => {{
            let actual = parse_string($input);

            assert_eq!(
                &actual, $expected,
                "Actual string (left) doesn't match expected (right)"
            );

            assert!(
                matches!(actual, Cow::$variant(_)),
                "Outputted string of the incorrect variant",
            );
        }};
    }

    test!(r#""""#, "", Borrowed);
    test!(r#""!""#, "!", Borrowed);
    test!(r#""\"""#, "\"", Owned);
    test!(r#""\'""#, "\'", Owned);
    test!(r#""apple banana""#, "apple banana", Borrowed);
    test!(r#""abc \\""#, "abc \\", Owned);
    test!(r#""\n def""#, "\n def", Owned);
    test!(
        r#""abc \t (\\\t) \r (\\\r) def""#,
        "abc \t (\\\t) \r (\\\r) def",
        Owned,
    );
    test!(r#""abc \t \x \y \z \n""#, "abc \t \\x \\y \\z \n", Owned);
    test!("'abc'", "'abc'", Borrowed);
    test!("\"abc", "\"abc", Borrowed);
    test!("foo", "foo", Borrowed);
}

#[test]
fn test_slice_middle() {
    macro_rules! test {
        ($input:expr, $expected:expr $(,)?) => {{
            let actual = slice_middle($input).expect("Invalid string input");

            assert_eq!(
                actual, $expected,
                "Actual (left) doesn't match expected (right)",
            );
        }};

        ($input:expr $(,)?) => {{
            assert!(
                slice_middle($input).is_none(),
                "Invalid string was accepted",
            );
        }};
    }

    test!(r#""""#, "");
    test!(r#""!""#, "!");
    test!(r#""abc""#, "abc");
    test!(r#""apple banana cherry""#, "apple banana cherry");

    test!("");
    test!("\"");
    test!("\"'");
    test!("''");
    test!("[]");
}
