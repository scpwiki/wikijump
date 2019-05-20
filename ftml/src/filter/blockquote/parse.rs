/*
 * filter/blockquote/parse.rs
 *
 * ftml - Convert Wikidot code to HTML
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

use crate::Result;
use either::Either;
use regex::Regex;
use std::mem;

lazy_static! {
    static ref QUOTE_LINE: Regex = {
        Regex::new(r"^(?P<depth>>+)(?: *)(?P<contents>.*)$").unwrap()
    };
}

#[derive(Debug)]
struct QuoteLine<'a> {
    depth: usize,
    contents: &'a str,
}

#[derive(Debug)]
struct OtherLine<'a> {
    contents: &'a str,
}

pub fn substitute(text: &mut String) -> Result<()> {
    let mut lines = Vec::new();

    for raw_line in text.lines() {
        let line = match QUOTE_LINE.captures(raw_line) {
            Some(mtch) => {
                let depth = mtch
                    .name("depth")
                    .expect("No group 'depth' found in capture")
                    .as_str()
                    .len();

                let contents = mtch
                    .name("contents")
                    .expect("No group 'contents' found in capture")
                    .as_str();

                Either::Left(QuoteLine { depth, contents })
            }
            None => Either::Right(OtherLine { contents: raw_line }),
        };

        lines.push(line);
    }

    // Build filtered source file
    let mut buffer = String::new();
    let mut prev_depth = 0;
    for (i, line) in lines.iter().enumerate() {
        match line {
            // Quote line
            Either::Left(line) => {
                // Add open or close tags as needed
                if line.depth > prev_depth {
                    let diff = line.depth - prev_depth;
                    for _ in 0..diff {
                        buffer.push_str("[[quote]]\n");
                    }
                } else if prev_depth > line.depth {
                    let diff = prev_depth - line.depth;
                    for _ in 0..diff {
                        buffer.push_str("[[/quote]]\n");
                    }
                }

                // Add contents
                buffer.push_str(line.contents);
                prev_depth = line.depth;
            }
            // Other line
            Either::Right(line) => {
                // Add any extra closing tags
                for _ in 0..prev_depth {
                    buffer.push_str("[[/quote]]\n");
                }

                // Add contents
                buffer.push_str(line.contents);
                prev_depth = 0;
            }
        }

        // Only add newlines in the middle
        // If there are end tags yet to be added, it's still the middle
        if i < lines.len() - 1 || prev_depth != 0 {
            buffer.push('\n');
        }
    }

    // Finally, add closing tags
    for _ in 0..prev_depth {
        buffer.push_str("[[/quote]]\n");
    }

    // Trim leading and trailing newlines
    while buffer.starts_with('\n') {
        buffer.remove(0);
    }

    while buffer.ends_with('\n') {
        buffer.pop();
    }

    // Replace string
    mem::swap(&mut buffer, text);
    mem::drop(buffer);

    Ok(())
}

#[test]
fn test_regexes() {
    let _ = &*QUOTE_LINE;
}
