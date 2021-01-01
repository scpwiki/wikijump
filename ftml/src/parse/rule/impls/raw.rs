/*
 * parse/rule/impls/raw.rs
 *
 * ftml - Library to parse Wikidot text
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

use super::prelude::*;

macro_rules! raw {
    ($value:expr) => {
        Element::Raw(cow!($value))
    };
}

pub const RULE_RAW: Rule = Rule {
    name: "raw",
    try_consume_fn,
};

fn try_consume_fn<'p, 'l, 'r, 't>(
    log: &'l slog::Logger,
    parser: &'p mut Parser<'l, 'r, 't>,
) -> ParseResult<'r, 't, Element<'t>> {
    debug!(log, "Consuming tokens until end of raw");

    // Are we in a @@..@@ type raw, or a @<..>@ type?
    let ending_token = match parser.current().token {
        Token::Raw => Token::Raw,
        Token::LeftRaw => Token::RightRaw,
        _ => panic!("Current token is not a starting raw"),
    };

    // Check for four special cases:
    // * Raw Raw  "@" -> Element::Raw("@")
    // * Raw Raw !Raw -> Element::Raw("")
    // * Raw Raw  Raw -> Element::Raw("@@")
    // * Raw ??   Raw -> Element::Raw(slice)
    if ending_token == Token::Raw {
        trace!(log, "First token is '@@', checking for special cases");

        // Get next two tokens. If they don't exist, exit early
        let next_1 = parser.look_ahead_error(0)?;
        let next_2 = parser.look_ahead_error(1)?;

        // Determine which case they fall under
        match (next_1.token, next_2.token) {
            // "@@@@@@" -> Element::Raw("@@")
            (Token::Raw, Token::Raw) => {
                debug!(log, "Found meta-raw (\"@@@@@@\"), returning");
                parser.step_n(2);
                return ok!(raw!("@@"), parser.remaining());
            }

            // "@@@@@" -> Element::Raw("@")
            // This case is strange since the lexer returns Raw Raw Other (@@ @@ @)
            // So we capture this and return the intended output
            (Token::Raw, Token::Other) => {
                if next_2.slice == "@" {
                    debug!(log, "Found single-raw (\"@@@@@\"), returning");
                    parser.step_n(2);
                    return ok!(raw!("@"), parser.remaining());
                } else {
                    debug!(log, "Found empty raw (\"@@@@\"), followed by other text");
                    parser.step_n(1);
                    return ok!(raw!(""), parser.remaining());
                }
            }

            // "@@@@" -> Element::Raw("")
            // Only consumes two tokens.
            (Token::Raw, _) => {
                debug!(log, "Found empty raw (\"@@@@\"), returning");
                parser.step_n(1);
                return ok!(raw!(""), parser.remaining());
            }

            // "@@ \n @@" -> Abort
            (Token::LineBreak, Token::Raw) | (Token::ParagraphBreak, Token::Raw) => {
                debug!(log, "Found interrupted raw, aborting");
                return Err(parser.make_error(ParseErrorKind::RuleFailed));
            }

            // "@@ [something] @@" -> Element::Raw(token)
            (_, Token::Raw) => {
                debug!(log, "Found single-element raw, returning");
                parser.step_n(2);
                return ok!(raw!(next_1.slice), parser.remaining());
            }

            // Other, proceed with rule logic
            (_, _) => (),
        }
    }

    // Handle the other cases, which are:
    // * "@@ [tokens] @@"
    // * "@< [tokens] >@"
    //
    // Collect the first and last token to build a slice of its contents.
    // The last will be updated with each step in the iterator.

    let (start, mut end) = {
        let extracted = parser.step()?;

        (extracted, extracted)
    };

    loop {
        let ExtractedToken { token, span, slice } = parser.current();

        debug!(
            log,
            "Received token inside raw";
            "token" => token,
            "slice" => slice,
            "span-start" => span.start,
            "span-end" => span.end,
        );

        // Check token
        match token {
            // Possibly hit end of raw. If not, continue.
            Token::RightRaw | Token::Raw => {
                // If block is inside match rule for clarity
                if *token == ending_token {
                    trace!(log, "Reached end of raw, returning");

                    let slice = if start == end {
                        /* Empty raw */
                        ""
                    } else {
                        /* Gather slice from spans */
                        parser.full_text().slice(log, start, end)
                    };

                    let element = Element::Raw(cow!(slice));
                    return ok!(element, parser.remaining());
                }

                trace!(log, "Wasn't end of raw, continuing");
            }

            // Hit a newline, abort
            Token::LineBreak | Token::ParagraphBreak => {
                trace!(log, "Reached newline, aborting");

                return Err(parser.make_error(ParseErrorKind::RuleFailed));
            }

            // Hit the end of the input, abort
            Token::InputEnd => {
                trace!(log, "Reached end of input, aborting");

                return Err(parser.make_error(ParseErrorKind::EndOfInput));
            }

            // No special handling, append to slices like normal
            _ => (),
        }

        trace!(log, "Appending present token to raw");

        // Update last token and step.
        end = parser.step()?;
    }
}
