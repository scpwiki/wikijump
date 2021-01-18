/*
 * include/mod.rs
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

mod includer;
mod object;

pub use self::includer::{Includer, NullIncluder};
pub use self::object::{IncludeRef, PageRef};

use crate::span_wrap::SpanWrap;
use pest::Parser;
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref INCLUDE_REGEX: Regex = {
        RegexBuilder::new(r"\[\[\s*include\s*.+\]\]")
            .case_insensitive(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap()
    };
}

#[derive(Parser, Debug)]
#[grammar = "include/grammar.pest"]
struct IncludeParser;

pub fn include<'t>(
    log: &slog::Logger,
    text: &'t mut String,
    includer: &mut dyn Includer<'t>,
) -> Vec<PageRef<'t>> {
    let log = &log.new(slog_o!(
        "filename" => slog_filename!(),
        "lineno" => slog_lineno!(),
        "function" => "include",
        "text" => str!(text),
    ));

    info!(
        log,
        "Finding and replacing all instances of include blocks in text"
    );

    let mut includes = Vec::new();

    for mtch in INCLUDE_REGEX.find_iter(text) {
        let start = mtch.start();
        let end = mtch.end();
        let slice = &text[start..end];

        match IncludeParser::parse(Rule::include, slice) {
            Ok(pairs) => {
                debug!(
                    log,
                    "Parsed include block";
                    "span" => SpanWrap::from(start..end),
                    "slice" => slice,
                );

                for pair in pairs {
                    // TODO
                    println!("rule: {:?}, slice: {:?}", pair.as_rule(), pair.as_str());
                }

                includes.push((start..end, ()));
            }
            Err(error) => {
                debug!(
                    log,
                    "Found invalid include block";
                    "error" => str!(error),
                    "span" => SpanWrap::from(start..end),
                    "slice" => slice,
                );
            }
        }
    }

    todo!()
}

#[test]
fn test_include() {
    let log = crate::build_logger();

    macro_rules! test {
        ($text:expr, $expected:expr) => {{
            let mut text = str!($text);
            let actual = include(&log, &mut text, &mut NullIncluder);
            let expected = $expected;

            println!("Input: {:?}", $text);
            println!("Pages (actual): {:?}", actual);
            println!("Pages (expected): {:?}", expected);
            println!();

            assert_eq!(
                &actual, &expected,
                "Actual pages to include doesn't match expected"
            );
        }};
    }

    test!("", vec![]);
    test!("[[include page]]", vec![PageRef::page_only("page")]);

    test!(
        "abc\n[[include page]]\ndef\n[[include page2\narg=1]]\nghi",
        vec![]
    );
}
