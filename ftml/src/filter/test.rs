/*
 * filter/test.rs
 *
 * ftml - Convert Wikidot code to HTML
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

use super::prefilter;
use crate::handle::TestHandle;
use crate::Result;

pub fn test_substitution<F>(filter_name: &str, mut substitute: F, tests: &[(&str, &str)])
where
    F: FnMut(&mut String) -> Result<()>,
{
    let mut string = String::new();

    for (input, expected) in tests {
        string.clear();
        string.push_str(input);

        println!(
            "Testing {} substitution:\nInput:    {:?}\nExpected: {:?}\n",
            filter_name, input, expected,
        );

        if let Err(err) = substitute(&mut string) {
            panic!(
                "Failed to perform {} substitution test string:\n{}\nExpected:\n{}\n-----\nProduced error: {}",
                filter_name, input, expected, err
            );
        }

        assert_eq!(
            &string, expected,
            "\nOutput of {} substitution test didn't match",
            filter_name,
        );
    }
}

const PREFILTER_TEST_CASES: [(&str, &str); 13] = [
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
        "<< [[[SCP-999]]] | SCP-1000 | [[[SCP-1001]]] >>",
        "« [[[SCP-999]]] | SCP-1000 | [[[SCP-1001]]] »",
    ),
    (
        " . . . <<I'm not sure about this,>>",
        " … «I'm not sure about this,»",
    ),
    (
        "[[include info:start]]\nApple\nBanana\n[[include info:end]]\n",
        "<PAGE 'info:start' #0>\nApple\nBanana\n<PAGE 'info:end' #0>",
    ),
    (
        "Apple\n[[include component:image-block\n    name = somefile.png |\n    caption=The Thing|\n    width= 200px\n]]\nBanana",
        "Apple\n<PAGE 'component:image-block' #3>\nBanana",
    ),
];

#[test]
fn test_prefilter() {
    test_substitution(
        "prefilter",
        |s| prefilter(s, &TestHandle),
        &PREFILTER_TEST_CASES,
    );
}
