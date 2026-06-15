/*
 * preproc/whitespace.rs
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

//! This performs the various miscellaneous substitutions that Wikidot does
//! in preparation for its parsing and handling processes. These are:
//! * Replacing DOS and legacy Mac newlines
//! * Trimming whitespace lines
//! * Concatenating lines that end with backslashes
//! * Convert tabs to four spaces
//! * Convert null characters to regular spaces
//! * Compress groups of 3+ newlines into 2 newlines

use super::Replacer;
use regex::{Regex, RegexBuilder};
use std::sync::LazyLock;

static LEADING_NONSTANDARD_WHITESPACE: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new("^[\u{00a0}\u{2007}]+")
        .multi_line(true)
        .build()
        .unwrap()
});
static WHITESPACE_ONLY_LINE: LazyLock<Replacer> =
    LazyLock::new(|| Replacer::RegexReplace {
        regex: RegexBuilder::new(r"^\s+$")
            .multi_line(true)
            .build()
            .unwrap(),
        replacement: "",
    });
static LEADING_NEWLINES: LazyLock<Replacer> = LazyLock::new(|| Replacer::RegexReplace {
    regex: Regex::new(r"^\n+").unwrap(),
    replacement: "",
});
static TRAILING_NEWLINES: LazyLock<Replacer> = LazyLock::new(|| Replacer::RegexReplace {
    regex: Regex::new(r"\n+$").unwrap(),
    replacement: "",
});
static DOS_MAC_NEWLINES: LazyLock<Replacer> = LazyLock::new(|| Replacer::RegexReplace {
    regex: Regex::new(r"\r\n?").unwrap(),
    replacement: "\n",
});
static CONCAT_LINES: LazyLock<Replacer> = LazyLock::new(|| Replacer::RegexReplace {
    regex: Regex::new(r"\\\n").unwrap(),
    replacement: "",
});
static TABS: LazyLock<Replacer> = LazyLock::new(|| Replacer::RegexReplace {
    regex: Regex::new("\t").unwrap(),
    replacement: "    ",
});
static NULL_SPACE: LazyLock<Replacer> = LazyLock::new(|| Replacer::RegexReplace {
    regex: Regex::new("\0").unwrap(),
    replacement: " ",
});

/// Performs all whitespace substitutions in-place in the given text.
pub fn substitute(text: &mut String) {
    let mut buffer = String::new();

    macro_rules! replace {
        ($replacer:expr) => {
            $replacer.replace(text, &mut buffer)
        };
    }

    // Replace DOS and Mac newlines
    replace!(DOS_MAC_NEWLINES);

    // Replace leading non-standard spaces with regular spaces
    // Leave other non-standard spaces as-is (such as nbsp in
    // the middle of paragraphs)
    replace_leading_spaces(text);

    // Strip lines with only whitespace
    replace!(WHITESPACE_ONLY_LINE);

    // Join concatenated lines (ending with '\')
    replace!(CONCAT_LINES);

    // Tabs to spaces
    replace!(TABS);

    // Null characters to spaces
    replace!(NULL_SPACE);

    // Remove leading and trailing newlines
    replace!(LEADING_NEWLINES);
    replace!(TRAILING_NEWLINES);
}

/// In-place replaces the leading non-standard spaces (such as nbsp) on each line with standard spaces
fn replace_leading_spaces(text: &mut String) {
    trace!("Replacing leading non-standard spaces with regular spaces");

    let mut offset = 0;

    while let Some(capture) = LEADING_NONSTANDARD_WHITESPACE.captures_at(text, offset) {
        let mtch = capture
            .get(0)
            .expect("Regular expression lacks a full match");

        let count = mtch.as_str().chars().count();
        let spaces = " ".repeat(count);

        offset = mtch.start() + count;

        text.replace_range(mtch.range(), &spaces);
    }
}

#[cfg(test)]
const TEST_CASES: [(&str, &str); 7] = [
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
    ("\u{00a0}\u{00a0}\u{2007} apple", "    apple"),
];

#[test]
fn regexes() {
    let _ = &*LEADING_NONSTANDARD_WHITESPACE;
    let _ = &*WHITESPACE_ONLY_LINE;
    let _ = &*LEADING_NEWLINES;
    let _ = &*TRAILING_NEWLINES;
    let _ = &*DOS_MAC_NEWLINES;
    let _ = &*CONCAT_LINES;
    let _ = &*TABS;
    let _ = &*NULL_SPACE;
}

#[test]
fn test_substitute() {
    use super::test::test_substitution;

    test_substitution("miscellaneous", substitute, &TEST_CASES);
}
