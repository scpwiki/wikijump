/*
 * include/test.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Ammon Smith
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

use super::{include, NullIncluder, PageRef};

#[test]
fn includes() {
    let log = crate::build_logger();

    macro_rules! test {
        ($text:expr, $expected:expr) => {{
            let mut text = str!($text);
            let result = include(&log, &mut text, NullIncluder);
            let (output, actual) = result.expect("Fetching pages failed");
            let expected = $expected;

            println!("Input: {:?}", $text);
            println!("Output: {:?}", output);
            println!("Pages (actual): {:?}", actual);
            println!("Pages (expected): {:?}", expected);
            println!();

            assert_eq!(
                &actual, &expected,
                "Actual pages to include doesn't match expected"
            );
        }};
    }

    // Valid cases
    test!("", vec![]);
    test!("[[include page]]", vec![PageRef::page_only("page")]);
    test!("[[include page a=1]]", vec![]);
    test!("[[include page a=1|]]", vec![]);
    test!("[[include page a=1 |]]", vec![]);
    test!("[[include page |a=1]]", vec![]);
    test!("[[include page | a=1]]", vec![]);
    test!("[[include page |a=1|]]", vec![]);
    test!("[[include page | a=1|]]", vec![]);
    test!("[[include page |a=1 |]]", vec![]);
    test!("[[include page | a=1 |]]", vec![]);
    test!("[[include page a=1 | b=2]]", vec![]);

    // TODO
    test!(
        "abc\n[[include page]]\ndef\n[[include page2\narg=1]]\nghi",
        vec![]
    );

    // Invalid cases
    test!("other text", vec![]);
    test!("[[include", vec![]);
    test!("include]]", vec![]);
}
