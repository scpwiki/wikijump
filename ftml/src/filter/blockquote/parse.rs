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

use crate::{Error, Result};
use either::Either;
use pest::Parser;
use std::mem;

#[derive(Debug, Clone, Parser)]
#[grammar = "filter/blockquote.pest"]
struct BlockQuoteParser;

pub fn substitute(text: &mut String) -> Result<()> {
    #[derive(Debug)]
    struct QuoteLine<'a> {
        depth: usize,
        contents: &'a str,
    }

    #[derive(Debug)]
    struct OtherLine<'a> {
        contents: &'a str,
    }

    if text.is_empty() {
        return Ok(());
    }

    let pairs = match BlockQuoteParser::parse(Rule::page, text) {
        Ok(mut pairs) => get_inner_pairs!(pairs),
        Err(err) => {
            return Err(Error::Msg(format!(
                "Blockquote transform parsing error: {}",
                err
            )))
        }
    };

    // Run parser and generate lines
    let mut lines = Vec::new();
    for pair in pairs {
        let line = match pair.as_rule() {
            Rule::quote_line => {
                let depth = {
                    let pair = get_nth_pair!(pair, 0);
                    debug_assert_eq!(pair.as_rule(), Rule::quote_depth);
                    pair.as_str().len()
                };

                let contents = {
                    let pair = get_nth_pair!(pair, 1);
                    debug_assert_eq!(pair.as_rule(), Rule::line_contents);
                    pair.as_str()
                };

                Either::Left(QuoteLine { depth, contents })
            }
            Rule::other_line => {
                let contents = {
                    let pair = get_first_pair!(pair);
                    debug_assert_eq!(pair.as_rule(), Rule::line_contents);
                    pair.as_str()
                };

                Either::Right(OtherLine { contents })
            }
            Rule::EOI => break,
            _ => panic!("Invalid rule for blockquote-parser: {:?}", pair.as_rule()),
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

    // Replace string
    mem::swap(&mut buffer, text);
    mem::drop(buffer);

    Ok(())
}
