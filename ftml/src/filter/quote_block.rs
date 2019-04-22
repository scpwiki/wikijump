/*
 * parse/quote_block.rs
 *
 * wikidot-html - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith for Project Foundation
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

//! Basically, quote blocks in Wikidot are a huge pain to parse given
//! the way the grammar is set up. There were several ideas to handle
//! them in the parser alone, but they have proven to be unworkable
//! or unwieldy.
//!
//! Additionally, because of their loose pass-through-and-substitute
//! "parsing" method, they also support tags with inconsistent quote
//! block levels, something the library currently does not support.
//!
//! Instead, we will use a regular expression and substitution to
//! replace Wikidot quote-block prefixes with this library's
//! `[[quote]]` blocks, which we can handle easily.

use regex::Regex;

lazy_static! {
    static ref BLOCK_QUOTE: Regex = Regex::new(r"(?:>+ *[^\n]*(?:\n|$))+").unwrap();
    static ref BLOCK_QUOTE_LINE: Regex = {
        Regex::new(r"^(?P<depth>>+) *(?P<contents>[^\n]*)$").unwrap()
    };
}

pub fn substitute(text: &mut String) {
    let mut buffer = String::new();
    let mut last_index = 0;

    while let Some(mtch) = BLOCK_QUOTE.find_at(text.as_ref(), last_index) {
        // Build up the replacement buffer
        let mut prev_depth = 0;
        for line in mtch.as_str().lines() {
            let capture = BLOCK_QUOTE_LINE.captures(line)
                .expect("Regular expression BLOCK_QUOTE_LINE didn't match");
            let depth = capture["depth"].len();
            let contents = &capture["contents"];

            // Open or close tag(s) as needed
            if depth > prev_depth {
                let diff = depth - prev_depth;
                for _ in 0..diff {
                    buffer.push_str("[[quote]]\n");
                }
            } else if prev_depth > depth {
                let diff = prev_depth - depth;
                for _ in 0..diff {
                    buffer.push_str("[[/quote]]\n");
                }
            }

            // Add line content
            buffer.push_str(contents);
            buffer.push('\n');
            prev_depth = depth;
        }

        // Add any extra closing tags
        for _ in 0..prev_depth {
            buffer.push_str("[[/quote]]\n");
        }

        // Do the substitution
        let range = mtch.start()..mtch.end();
        last_index = mtch.start() + buffer.len() - 1;
        text.replace_range(range, &buffer);
        buffer.clear();
    }
}

#[test]
fn test_regexes() {
    let _ = &*BLOCK_QUOTE;
    let _ = &*BLOCK_QUOTE_LINE;
}

#[test]
fn test_substitute() {
    let mut string = String::new();

    macro_rules! substitute {
        ($str:expr) => {{
            string.clear();
            string.push_str($str);
            substitute(&mut string);
        }}
    }

    substitute!("test\n> abc\n> def\n> ghi\n>> apple\n>> banana\n>>> durian\n>> fruit list\nend");
    assert_eq!(&string, "test\n[[quote]]\nabc\ndef\nghi\n[[quote]]\napple\nbanana\n[[quote]]\ndurian\n[[/quote]]\nfruit list\n[[/quote]]\n[[/quote]]\nend");

    substitute!(">>>> deep quote block\n>>>> contents");
    assert_eq!(&string, "[[quote]]\n[[quote]]\n[[quote]]\n[[quote]]\ndeep quote block\ncontents\n[[/quote]]\n[[/quote]]\n[[/quote]]\n[[/quote]]\n");

    substitute!(">no space test\n> it's weird wikidot requires it\n>  extra space");
    assert_eq!(&string, "[[quote]]\nno space test\nit's weird wikidot requires it\nextra space\n[[/quote]]\n");

    substitute!("> multiple quotes test\n\n> another block\n>> omega\n");
    assert_eq!(&string, "[[quote]]\nmultiple quotes test\n[[/quote]]\n\n[[quote]]\nanother block\n[[quote]]\nomega\n[[/quote]]\n[[/quote]]\n");

    substitute!("this string doesn't have any quotes in it");
    assert_eq!(&string, "this string doesn't have any quotes in it");

    substitute!("> apple\n> > fake quote\n> >> even faker\n");
    assert_eq!(&string, "[[quote]]\napple\n> fake quote\n>> even faker\n[[/quote]]\n");

    substitute!("[[div]]\napple\n> banana\n[[/div]]\n> durian\n");
    assert_eq!(&string, "[[div]]\napple\n[[quote]]\nbanana\n[[/quote]]\n[[/div]]\n[[quote]]\ndurian\n[[/quote]]\n");
}
