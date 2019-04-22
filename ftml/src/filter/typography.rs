/*
 * parse/typography.rs
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

//! Perform Wikidot's typographical modifications.
//! For full information, see the original source file:
//! https://github.com/Nu-SCPTheme/wikidot/blob/master/lib/Text_Wiki/Text/Wiki/Parse/Default/Typography.php
//!
//! The transformations performed here are listed:
//! * TODO

use either::Either;
use regex::Regex;

lazy_static! {
    // ‘ - LEFT SINGLE QUOTATION MARK
    // ’ - RIGHT SINGLE QUOTATION MARK
    static ref SINGLE_QUOTES: Replacer = Replacer {
        regex: Regex::new(r"`(.*?)'").unwrap(),
        replacement: Either::Left(("\u{2018}", "\u{2019}")),
    };

    // “ - LEFT DOUBLE QUOTATION MARK
    // ” - RIGHT DOUBLE QUOTATION MARK
    static ref DOUBLE_QUOTES: Replacer = Replacer {
        regex: Regex::new(r"``(.*?)''").unwrap(),
        replacement: Either::Left(("\u{201c}", "\u{201d}")),
    };

    // „ - DOUBLE LOW-9 QUOTATION MARK
    // ” - RIGHT DOUBLE QUOTATION MARK
    static ref LOW_DOUBLE_QUOTES: Replacer = Replacer {
        regex: Regex::new(r",,(.*?)''").unwrap(),
        replacement: Either::Left(("\u{201e}", "\u{201d}")),
    };

    // « - LEFT-POINTING DOUBLE ANGLE QUOTATION MARK
    static ref LEFT_DOUBLE_ANGLE: Replacer = Replacer {
        regex: Regex::new(r"<<").unwrap(),
        replacement: Either::Right("\u{0ab}"),
    };

    // » - RIGHT-POINTING DOUBLE ANGLE QUOTATION MARK
    static ref RIGHT_DOUBLE_ANGLE: Replacer = Replacer {
        regex: Regex::new(r">>").unwrap(),
        replacement: Either::Right("\u{0bb}"),
    };

    // … - HORIZONTAL ELLIPSIS
    static ref ELLIPSIS: Replacer = Replacer {
        regex: Regex::new(r"(?:\.\.\.|\. \. \.)").unwrap(),
        replacement: Either::Right("\u{2026}"),
    };

    // — - EM DASH
    static ref EM_DASH: Replacer = Replacer {
        regex: Regex::new(r"-{2,3}").unwrap(),
        replacement: Either::Right("\u{2014}"),
    };
}

#[derive(Debug)]
pub struct Replacer {
    regex: Regex,
    replacement: Either<(&'static str, &'static str), &'static str>,
}

impl Replacer {
    fn replace(&self, text: &mut String, buffer: &mut String) {
        while let Some(capture) = self.regex.captures(text) {
            let mtch = capture.get(0).expect("Regular expression lacks a full match");
            let range = mtch.start()..mtch.end();

            match self.replacement {
                Either::Left((begin, end)) => {
                    let mtch = capture.get(1).expect("Regular expression lacks a content group");

                    buffer.clear();
                    buffer.push_str(begin);
                    buffer.push_str(mtch.as_str());
                    buffer.push_str(end);

                    text.replace_range(range, &buffer);
                },
                Either::Right(value) => text.replace_range(range, value),
            }
        }
    }
}

pub fn substitute(text: &mut String) {
    let mut buffer = String::new();

    macro_rules! replace {
        ($replacer:expr) => ( $replacer.replace(text, &mut buffer) )
    }

    // Quotes
    replace!(DOUBLE_QUOTES);
    replace!(LOW_DOUBLE_QUOTES);
    replace!(SINGLE_QUOTES);

    // French quotes
    replace!(LEFT_DOUBLE_ANGLE);
    replace!(RIGHT_DOUBLE_ANGLE);

    // Miscellaneous
    replace!(ELLIPSIS);
    replace!(EM_DASH);
}

#[test]
fn test_regexes() {
    let _ = &*SINGLE_QUOTES;
    let _ = &*DOUBLE_QUOTES;
    let _ = &*LOW_DOUBLE_QUOTES;
    let _ = &*LEFT_DOUBLE_ANGLE;
    let _ = &*RIGHT_DOUBLE_ANGLE;
    let _ = &*ELLIPSIS;
    let _ = &*EM_DASH;
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

    substitute!("John laughed. ``You'll never defeat me!''\n``That's where you're wrong.''");
    println!("{}", &string);
}
