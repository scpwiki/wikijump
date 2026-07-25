/*
 * utils/string.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
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

use regex::Regex;

// General replacement

// TODO: When https://doc.rust-lang.org/stable/std/str/pattern/trait.Pattern.html is stabilized,
//       use it in the definition of replace_in_place() if possible.

/// Replaces all instances of the given fixed string in the buffer, in-place.
///
/// # Panics
/// Panics if `pattern` is an empty string.
pub fn replace_in_place(string: &mut String, pattern: &str, replacement: &str) {
    assert!(
        !pattern.is_empty(),
        "Cannot call replace_in_place() with an empty string"
    );

    // Resume each iteration of search after the last replacement.
    // Avoids issues with infinite loops if the replacement contains the pattern.
    let mut start_index = 0;
    while let Some(substr_index) = &string[start_index..].find(pattern) {
        let index = start_index + substr_index;
        let end = index + pattern.len();
        string.replace_range(index..end, replacement);
        start_index = end + replacement.len() - 1;
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
    if string.ends_with(pattern) {
        string.drain(string.len() - pattern.len()..);
    }
}

// Specific replacement
#[inline]
pub fn trim_spaces_in_place(string: &mut String) {
    regex_replace_in_place(string, regex!(r"(^\s+)|(\s+$)"), "");
}

/// This helper function removes U+2068 and U+2069 control characters from a string.
///
/// Fluent adds these U+2068 and U+2069 characters to assist text layout engines
/// when dealing with LTR / RTL text. However, this causes parsing issues
/// for us in wikitext or HTML, since these characters can gum up string
/// concatenation cases like URL construction.
///
/// So as a special case here, we provide a helper function to strip out any of
/// these characters. We don't remove these characters from all locale strings
/// since they are a desired localization property of Fluent.
///
/// See the [Fluent Docs](https://fluent-compiler.readthedocs.io/en/latest/usage.html#:~:text=You%20will%20notice%20the%20extra%20characters%20\u2068%20and%20\u2069%20in%20the%20output.)
#[inline]
pub fn strip_fluent_control_chars(string: &mut String) {
    regex_replace_in_place(string, regex!("[\u{2068}\u{2069}]"), "");
}

// Tests

#[test]
fn test_replace_in_place() {
    macro_rules! test {
        ($input:expr => $output:expr, $pattern:expr => $replacement:expr $(,)?) => {{
            let mut string = str!($input);
            replace_in_place(&mut string, $pattern, $replacement);
            assert_eq!(string, $output, "Replaced contents did not match expected");
        }};
    }

    test!("foo/bar" => "foo + bar", "/" => " + ");
    test!("apple banana cherry" => "pple bnn cherry", "a" => "");
    test!("apple banana cherry" => "appluxi banana chuxirry", "e" => "uxi");
    test!("class pass hassle dash" => "cly py hyle dash", "ass" => "y");
    // should terminate despite replacement value
    test!("apple banana cherry" => "applexi banana chexirry", "e" => "exi");
    test!("e ee eee" => "eye eyeeye eyeeyeeye", "e" => "eye");
}

#[test]
#[should_panic]
fn test_replace_in_place_empty() {
    let mut string = str!("apple banana");
    replace_in_place(&mut string, "", "cherry");
}

#[test]
fn test_regex_replace_in_place() {
    macro_rules! test {
        ($input:expr => $output:expr, $regex:expr => $replacement:expr $(,)?) => {{
            let mut string = str!($input);
            regex_replace_in_place(&mut string, regex!($regex), $replacement);
            assert_eq!(string, $output, "Replaced contents did not match expected");
        }};
    }

    test!("apple banana cherry" => "axle banana chexy", r"p{2}|r{2}|n{2}" => "x");
    test!("apple banana cherry" => "_ b_ cherry", r"a\w+" => "_");
    test!(
        "After 12.5 years, he could only achieve a high score of -5000 in 2 games" => "After $NUMBER years, he could only achieve a high score of $NUMBER in $NUMBER games",
        r"-?[0-9]+(\.[0-9])?" => "$NUMBER",
    );
    test!("猫の手も借りたい" => "猫手借", "[\u{3042}-\u{3094}]+" => "");
}

#[test]
fn test_trim_start_matches_in_place() {
    macro_rules! test {
        ($input:expr => $output:expr, $pattern:expr $(,)?) => {{
            let mut string = str!($input);
            trim_start_matches_in_place(&mut string, $pattern);
            assert_eq!(string, $output, "Trimmed contents did not match expected");
        }};
    }

    test!("" => "", "_");
    test!("_foo_" => "foo_", "_");
    test!(">>> foo" => "foo", ">>> ");
    test!("[foo]" => "[foo]", ">>> ");
    test!("悪い🥭!" => "🥭!", "悪い");
}

#[test]
fn test_trim_end_matches_in_place() {
    macro_rules! test {
        ($input:expr => $output:expr, $pattern:expr $(,)?) => {{
            let mut string = str!($input);
            trim_end_matches_in_place(&mut string, $pattern);
            assert_eq!(string, $output, "Trimmed contents did not match expected");
        }};
    }

    test!("" => "", "_");
    test!("_foo_" => "_foo", "_");
    test!(">>> foo" => ">>> foo", ">>> ");
    test!("foo <<<" => "foo", " <<<");
    test!("🥭 腐った" => "🥭 ", "腐った");
}

#[test]
fn test_trim_spaces_in_place() {
    macro_rules! test {
        ($input:expr => $output:expr $(,)?) => {{
            let mut string = str!($input);
            trim_spaces_in_place(&mut string);
            assert_eq!(string, $output, "Trimmed contents did not match expected");
        }};

        // Unmodified case, where no substition occurs
        ($input:expr $(,)?) => {
            test!($input => $input)
        };
    }

    test!("");
    test!("foo");
    test!(" foo" => "foo");
    test!("foo " => "foo");
    test!("\t apple\n\n" => "apple");
    test!("banana         " => "banana");
    test!("\r\t  cherry" => "cherry");
    test!(" 🥭  " => "🥭");
}

#[test]
fn test_strip_fluent_control_chars() {
    macro_rules! test {
        ($input:expr => $output:expr $(,)?) => {{
            let mut string = str!($input);
            strip_fluent_control_chars(&mut string);
            assert_eq!(string, $output, "Trimmed contents did not match expected");
        }};

        // Unmodified case, where no substition occurs
        ($input:expr $(,)?) => {
            test!($input => $input)
        };
    }

    test!("");
    test!("Hello, world!");
    test!("Berry: 🍓");
    test!("Good morning \u{2068}John\u{2069}" => "Good morning John");
    test!("\u{2068}alpha\u{2069} beta \u{2068}\u{2069}gamma" => "alpha beta gamma");
}
