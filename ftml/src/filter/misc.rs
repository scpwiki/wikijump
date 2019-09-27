/*
 * filter/misc.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith
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

//! This performs the various miscellaneous substitutions that Wikidot does
//! in preparation for its parsing and handling processes. These are:
//! * Remove Wikidot comments
//! * Replacing DOS and legacy Mac newlines
//! * Trimming whitespace lines
//! * Concatenating lines that end with backslashes
//! * Convert tabs to four spaces
//! * Compress groups of 3+ newlines into 2 newlines
//!
//! Note on the first item:
//! It was originally implemented in the parser, however it was moved here
//! to prevent typography from converting the `--` in `[!--` and `--]` into
//! em dashes.

use crate::Result;
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref COMMENT: Regex = {
        RegexBuilder::new(r"\[!--.*--\]")
            .dot_matches_new_line(true)
            .build()
            .unwrap()
    };
    static ref WHITESPACE: Regex = {
        RegexBuilder::new(r"^\s+$")
            .multi_line(true)
            .build()
            .unwrap()
    };
    static ref COMPRESS_NEWLINES: Regex = {
        RegexBuilder::new(r"(?:\n\s*){3,}")
            .multi_line(true)
            .build()
            .unwrap()
    };
}

pub fn substitute(text: &mut String) -> Result<()> {
    regex_replace(text, &*COMMENT, "");
    str_replace(text, "\r\n", "\n");
    str_replace(text, "\r", "\n");
    regex_replace(text, &*WHITESPACE, "");
    str_replace(text, "\\\n", "");
    str_replace(text, "\t", "    ");
    regex_replace(text, &*COMPRESS_NEWLINES, "\n\n");

    Ok(())
}

fn str_replace(text: &mut String, pattern: &str, replacement: &str) {
    while let Some(idx) = text.find(pattern) {
        let range = idx..idx + pattern.len();
        text.replace_range(range, replacement);
    }
}

fn regex_replace(text: &mut String, regex: &Regex, replacement: &str) {
    while let Some(mtch) = regex.find(text) {
        let range = mtch.start()..mtch.end();
        text.replace_range(range, replacement);
    }
}

#[cfg(test)]
const TEST_CASES: [(&str, &str); 6] = [
    (
        "\tapple\n\tbanana\tcherry\n",
        "    apple\n    banana    cherry\n",
    ),
    (
        "newlines:\r\n* apple\r* banana\r\ncherry\n\r* durian",
        "newlines:\n* apple\n* banana\ncherry\n\n* durian",
    ),
    (
        "apple\nbanana\n\ncherry\n\n\npineapple\n\n\n\nstrawberry\n\n\n\n\nblueberry\n\n\n\n\n\n",
        "apple\nbanana\n\ncherry\n\npineapple\n\nstrawberry\n\nblueberry\n",
    ),
    (
        "apple\rbanana\r\rcherry\r\r\rpineapple\r\r\r\rstrawberry\r\r\r\r\rblueberry\r\r\r\r\r\r",
        "apple\nbanana\n\ncherry\n\npineapple\n\nstrawberry\n\nblueberry\n",
    ),
    (
        "concat:\napple banana \\\nCherry\\\nPineapple \\ grape\nblueberry\n",
        "concat:\napple banana CherryPineapple \\ grape\nblueberry\n",
    ),
    ("<\n        \n      \n  \n      \n>", "<\n\n>"),
];

#[test]
fn test_regexes() {
    let _ = &*WHITESPACE;
    let _ = &*COMPRESS_NEWLINES;
}

#[test]
fn test_substitute() {
    use super::test::test_substitution;

    test_substitution("miscellaneous", substitute, &TEST_CASES);
}
