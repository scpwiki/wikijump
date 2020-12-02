/*
 * parse/rule/impls/raw.rs
 *
 * ftml - Library to parse Wikidot code
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

pub const RULE_RAW: Rule = Rule {
    name: "raw",
    try_consume_fn,
};

fn try_consume_fn<'t, 'r>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    mut remaining: &'r [ExtractedToken<'t>],
) -> Consumption<'t, 'r> {
    debug!(log, "Consuming tokens until end of raw");

    // Are we in a @@..@@ type raw, or a @<..>@ type?
    let ending_token = match extracted.token {
        Token::Raw => Token::Raw,
        Token::LeftRaw => Token::RightRaw,
        _ => panic!("Current token is not a starting raw"),
    };

    // Check for two special cases:
    // * Raw Raw !Raw -> Element::Raw("")
    // * Raw Raw  Raw -> Element::Raw("@@")
    if ending_token == Token::Raw {
        trace!(log, "First token is '@@', checking for special cases");

        // Get next two tokens. If they don't exist, exit early
        if remaining.len() < 2 {
            debug!(log, "Insufficient tokens remaining for raw parsing, aborting");

            return Consumption::err(ParseError::new(
                ParseErrorKind::EndOfInput,
                RULE_RAW,
                extracted,
            ));
        }

        let next_extracted_1 = &remaining[0];
        let next_extracted_2 = &remaining[1];

        // Determine which case they fall under
        match (next_extracted_1.token, next_extracted_2.token) {
            // "@@@@@@" -> Element::Raw("@@")
            (Token::Raw, Token::Raw) => {
                debug!(log, "Found meta-raw (\"@@@@@@\"), returning");

                return Consumption::ok(Element::Raw("@@"), &remaining[2..]);
            }

            // "@@@@" -> Element::Raw("")
            // Only consumes two tokens.
            (Token::Raw, _) => {
                debug!(log, "Found empty raw (\"@@@@\"), returning");

                return Consumption::ok(Element::Raw(""), &remaining[1..]);
            }

            // "@@ <invalid> @@" -> Abort
            (Token::LineBreak, Token::Raw) | (Token::ParagraphBreak, Token::Raw) => {
                debug!(log, "Found interrupted raw, aborting");

                return Consumption::err(ParseError::new(
                    ParseErrorKind::RuleFailed,
                    RULE_RAW,
                    next_extracted_1,
                ))
            }

            // "@@ <something> @@" -> Element::Raw(token)
            (_, Token::Raw) => {
                debug!(log, "Found single-element raw, returning");

                return Consumption::ok(Element::Raw(next_extracted_1.slice), &remaining[2..])
            }

            // Other, proceed with rule logic
            (_, _) => (),
        }
    }

    //TODO:
    //four cases needed here:
    // Raw Raw !Raw -> Element::Raw("")
    // Raw Raw Raw -> Element::Raw("@@")
    //
    // Raw ... Raw -> Element::Raw(" ... ")
    // LeftRaw ... RightRaw -> Element::Raw(" ... ")

    while let Some((new_extracted, new_remaining)) = remaining.split_first() {
        let ExtractedToken { token, span, slice } = new_extracted;
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
            // Hit the end of the comment, return
            Token::RightComment => {
                trace!(log, "Reached end of comment, finishing comment.");

                return Consumption::ok(Element::Null, new_remaining);
            }

            // Hit the end of the input, abort
            Token::InputEnd => {
                trace!(log, "Reached end of input, aborting comment.");

                return Consumption::err(ParseError::new(
                    ParseErrorKind::RuleFailed,
                    RULE_RAW,
                    new_extracted,
                ));
            }

            // Consume any other comment
            _ => {
                trace!(log, "Token inside comment received. Discarding.");

                // Update pointer
                remaining = new_remaining;
            }
        }
    }

    panic!("Reached end of input without encountering a Token::InputEnd");
}
