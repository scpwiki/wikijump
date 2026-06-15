/*
 * preproc/typography.rs
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

//! Perform Wikidot's typographical modifications.
//! For full information, see the original source file:
//! <https://github.com/gabrys/wikidot/blob/master/lib/Text_Wiki/Text/Wiki/Parse/Default/Typography.php>
//!
//! The transformations performed here are listed:
//! * `` .. '' to fancy double quotes
//! * ` .. ' to fancy single quotes
//! * ,, .. '' to fancy lowered double quotes
//! * ... to an ellipsis
//!
//! Em dash conversion was originally implemented here, however
//! it was moved to the parser to prevent typography from converting
//! the `--` in `[!--` and `--]` into em dashes.

use super::Replacer;
use regex::Regex;
use std::sync::LazyLock;

// ‘ - LEFT SINGLE QUOTATION MARK
// ’ - RIGHT SINGLE QUOTATION MARK
static SINGLE_QUOTES: LazyLock<Replacer> = LazyLock::new(|| Replacer::RegexSurround {
    regex: Regex::new(r"`(.*?)'").unwrap(),
    begin: "\u{2018}",
    end: "\u{2019}",
});

// “ - LEFT DOUBLE QUOTATION MARK
// ” - RIGHT DOUBLE QUOTATION MARK
static DOUBLE_QUOTES: LazyLock<Replacer> = LazyLock::new(|| Replacer::RegexSurround {
    regex: Regex::new(r"``(.*?)''").unwrap(),
    begin: "\u{201c}",
    end: "\u{201d}",
});

// „ - DOUBLE LOW-9 QUOTATION MARK
static LOW_DOUBLE_QUOTES: LazyLock<Replacer> =
    LazyLock::new(|| Replacer::RegexSurround {
        regex: Regex::new(r",,(.*?)''").unwrap(),
        begin: "\u{201e}",
        end: "\u{201d}",
    });

// … - HORIZONTAL ELLIPSIS
static HORIZONTAL_ELLIPSIS: LazyLock<Replacer> =
    LazyLock::new(|| Replacer::RegexReplace {
        regex: Regex::new(r"(?:^|[^\.])(?<repl>(\.\.|\. \. )\.)(?:[^\.]|$)").unwrap(),
        replacement: "\u{2026}",
    });

/// Performs all typographic substitutions in-place in the given text
pub fn substitute(text: &mut String) {
    let mut buffer = String::new();
    debug!("Performing typography substitutions");

    macro_rules! replace {
        ($replacer:expr) => {
            $replacer.replace(text, &mut buffer)
        };
    }

    // Quotes
    replace!(DOUBLE_QUOTES);
    replace!(LOW_DOUBLE_QUOTES);
    replace!(SINGLE_QUOTES);

    // Miscellaneous
    replace!(HORIZONTAL_ELLIPSIS);
}

#[cfg(test)]
const TEST_CASES: [(&str, &str); 21] = [
    (
        "John laughed. ``You'll never defeat me!''\n``That's where you're wrong...''",
        "John laughed. “You'll never defeat me!”\n“That's where you're wrong…”",
    ),
    (
        ",,あんたは馬鹿です！''\n``Ehh?''\n,,本当！''\n[[footnoteblock]]",
        "„あんたは馬鹿です！”\n“Ehh?”\n„本当！”\n[[footnoteblock]]",
    ),
    (
        "**ENTITY MAKES DRAMATIC MOTION** . . . ",
        "**ENTITY MAKES DRAMATIC MOTION** … ",
    ),
    ("Whales... they are cool", "Whales… they are cool"),
    ("Whales ... they are cool", "Whales … they are cool"),
    ("Whales. . . they are cool", "Whales… they are cool"),
    ("Whales . . . they are cool", "Whales … they are cool"),
    ("...why would you think that?", "…why would you think that?"),
    (
        "... why would you think that?",
        "… why would you think that?",
    ),
    (
        ". . .why would you think that?",
        "…why would you think that?",
    ),
    (
        ". . . why would you think that?",
        "… why would you think that?",
    ),
    ("how could you...", "how could you…"),
    ("how could you ...", "how could you …"),
    ("how could you. . .", "how could you…"),
    ("how could you . . .", "how could you …"),
    // Spaced with extra dot after 3rd
    (". . .. ....", ". . .. ...."),
    // Multiple spaced dots in a row
    ("... . . . . . .", "… … …"),
    // Too many dots
    (".... ..", ".... .."),
    ("..........", ".........."),
    // Groups of three dots
    ("... ... ...", "… … …"),
    // Groups of three, mixed spaced and continuous
    ("... . . . ...", "… … …"),
];

#[test]
fn regexes() {
    let _ = &*SINGLE_QUOTES;
    let _ = &*DOUBLE_QUOTES;
    let _ = &*LOW_DOUBLE_QUOTES;
    let _ = &*HORIZONTAL_ELLIPSIS;
}

#[test]
fn test_substitute() {
    use super::test::test_substitution;

    test_substitution("typography", substitute, &TEST_CASES);
}
