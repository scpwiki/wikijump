/*
 * preproc/test.rs
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

use super::preprocess;
use crate::log::prelude::*;
use proptest::prelude::*;

pub fn test_substitution<F>(filter_name: &str, mut substitute: F, tests: &[(&str, &str)])
where
    F: FnMut(&Logger, &mut String),
{
    let mut string = String::new();
    let log = crate::build_logger();

    for (input, expected) in tests {
        string.clear();
        string.push_str(input);

        info!(log, "Testing {} substitution", filter_name; "input" => input, "expected" => expected);

        substitute(&log, &mut string);

        assert_eq!(
            &string, expected,
            "Output of {} substitution test didn't match",
            filter_name,
        );
    }
}

const PREFILTER_TEST_CASES: [(&str, &str); 10] = [
    ("", ""),
    ("tab\ttest", "tab    test"),
    (
        "fn main() {\n\tprintln!();\n\tlet _ = ();\n}",
        "fn main() {\n    println!();\n    let _ = ();\n}",
    ),
    ("newlines:\r\nA\rB\nC\nD\n\rE", "newlines:\nA\nB\nC\nD\n\nE"),
    (
        "compress:\nA\n\nB\n\n\nC\n\n\n\nD\n\n\n\n\nE\n\n\n\n\n\n",
        "compress:\nA\n\nB\n\nC\n\nD\n\nE",
    ),
    (
        "concat:\nApple Banana \\\nCherry\\\nPineapple \\ Grape\nBlueberry\n",
        "concat:\nApple Banana CherryPineapple \\ Grape\nBlueberry",
    ),
    ("[\n  \n    \n       \n  \n      \n \n   \n]", "[\n\n]"),
    (
        "SCP-4455-Ω said, ``It was a dark and stormy night. I looked down on my arch-nemesis, the Streamliner.''",

        "SCP-4455-Ω said, “It was a dark and stormy night. I looked down on my arch-nemesis, the Streamliner.”",
    ),
    (
        ",,あんたはばかです！''\n``Ehh?''\n,,ほんと！''",
        "„あんたはばかです！”\n“Ehh?”\n„ほんと！”",
    ),
    (
        " . . . I'm not sure about this,",
        " … I'm not sure about this,",
    ),
];

#[test]
fn prefilter() {
    test_substitution(
        "prefilter",
        |log, text| preprocess(log, text),
        &PREFILTER_TEST_CASES,
    );
}

proptest! {
    #[test]
    fn prefilter_prop(mut s in ".*") {
        let log = crate::build_logger();

        crate::preprocess(&log, &mut s);

        const INVALID_SUBSTRINGS: [&str; 7] = [
            "...",
            ". . .",
            "\r\n",
            "\r",
            "\\\n",
            "\t",
            "\0",
        ];

        for substring in &INVALID_SUBSTRINGS {
            assert!(!s.contains(substring));
        }
    }
}
