/*
 * preproc/whitespace.rs
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

//! This performs the various miscellaneous substitutions that Wikidot does
//! in preparation for its parsing and handling processes. These are:
//! * Replacing DOS and legacy Mac newlines
//! * Trimming whitespace lines
//! * Concatenating lines that end with backslashes
//! * Convert tabs to four spaces
//! * Convert null characters to regular spaces
//! * Compress groups of 3+ newlines into 2 newlines

use crate::log::prelude::*;
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref WHITESPACE: Regex = {
        RegexBuilder::new(r"^\s+$")
            .multi_line(true)
            .build()
            .unwrap()
    };
    static ref LEADING_NEWLINES: Regex = Regex::new(r"^\n+").unwrap();
    static ref TRAILING_NEWLINES: Regex = Regex::new(r"\n+$").unwrap();
}

pub fn substitute(log: &Logger, text: &mut String) {
    // Replace DOS and Mac newlines
    str_replace(log, text, "\r\n", "\n");
    str_replace(log, text, "\r", "\n");

    // Strip lines with only whitespace
    regex_replace(log, text, &*WHITESPACE, "");

    // Join concatenated lines (ending with '\')
    str_replace(log, text, "\\\n", "");

    // Tabs to spaces
    str_replace(log, text, "\t", "    ");

    // Null characters to spaces
    str_replace(log, text, "\0", " ");

    // Remove leading and trailing newlines,
    // save one at the end
    regex_replace(log, text, &*LEADING_NEWLINES, "");
    regex_replace(log, text, &*TRAILING_NEWLINES, "");
}

fn str_replace(log: &Logger, text: &mut String, pattern: &str, replacement: &str) {
    debug!(
        log,
        "Replacing miscellaneous static string";
        "type" => "string",
        "text" => &*text,
        "pattern" => pattern,
        "replacement" => replacement,
    );

    while let Some(idx) = text.find(pattern) {
        let range = idx..idx + pattern.len();
        text.replace_range(range, replacement);
    }
}

fn regex_replace(log: &Logger, text: &mut String, regex: &Regex, replacement: &str) {
    debug!(
        log,
        "Replacing miscellaneous regular expression";
        "type" => "regex",
        "text" => &*text,
        "pattern" => regex.as_str(),
        "replacement" => replacement,
    );

    while let Some(mtch) = regex.find(text) {
        let range = mtch.start()..mtch.end();
        text.replace_range(range, replacement);
    }
}

#[cfg(test)]
const TEST_CASES: [(&str, &str); 6] = [
    (
        "\tapple\n\tbanana\tcherry\n",
        "    apple\n    banana    cherry",
    ),
    (
        "newlines:\r\n* apple\r* banana\r\ncherry\n\r* durian",
        "newlines:\n* apple\n* banana\ncherry\n\n* durian",
    ),
    (
        "apple\nbanana\n\ncherry\n\n\npineapple\n\n\n\nstrawberry\n\n\n\n\nblueberry\n\n\n\n\n\n",
        "apple\nbanana\n\ncherry\n\npineapple\n\nstrawberry\n\nblueberry",
    ),
    (
        "apple\rbanana\r\rcherry\r\r\rpineapple\r\r\r\rstrawberry\r\r\r\r\rblueberry\r\r\r\r\r\r",
        "apple\nbanana\n\ncherry\n\npineapple\n\nstrawberry\n\nblueberry",
    ),
    (
        "concat:\napple banana \\\nCherry\\\nPineapple \\ grape\nblueberry\n",
        "concat:\napple banana CherryPineapple \\ grape\nblueberry",
    ),
    ("<\n        \n      \n  \n      \n>", "<\n\n>"),
];

#[test]
fn regexes() {
    let _ = &*WHITESPACE;
    let _ = &*LEADING_NEWLINES;
    let _ = &*TRAILING_NEWLINES;
}

#[test]
fn test_substitute() {
    use super::test::test_substitution;

    test_substitution("miscellaneous", substitute, &TEST_CASES);
}
