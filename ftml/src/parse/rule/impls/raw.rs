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
    full_text: FullText<'t>,
) -> Consumption<'t, 'r> {
    debug!(log, "Consuming tokens until end of raw");

    // Are we in a @@..@@ type raw, or a @<..>@ type?
    let ending_token = match extracted.token {
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
        if remaining.len() < 2 {
            debug!(
                log,
                "Insufficient tokens remaining for raw parsing, aborting"
            );

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

            // "@@@@@" -> Element::Raw("@")
            // This case is strange since the lexer returns Raw Raw Other (@@ @@ @)
            // So we capture this and return the intended output
            (Token::Raw, Token::Other) => {
                if next_extracted_2.slice == "@" {
                    debug!(log, "Found single-raw (\"@@@@@\"), returning");
                    return Consumption::ok(Element::Raw("@"), &remaining[2..]);
                } else {
                    debug!(log, "Found empty raw (\"@@@@\"), followed by other text");
                    return Consumption::ok(Element::Raw(""), &remaining[1..]);
                }
            }

            // "@@@@" -> Element::Raw("")
            // Only consumes two tokens.
            (Token::Raw, _) => {
                debug!(log, "Found empty raw (\"@@@@\"), returning");

                return Consumption::ok(Element::Raw(""), &remaining[1..]);
            }

            // "@@ \n @@" -> Abort
            (Token::LineBreak, Token::Raw) | (Token::ParagraphBreak, Token::Raw) => {
                debug!(log, "Found interrupted raw, aborting");

                return Consumption::err(ParseError::new(
                    ParseErrorKind::RuleFailed,
                    RULE_RAW,
                    next_extracted_1,
                ));
            }

            // "@@ <something> @@" -> Element::Raw(token)
            (_, Token::Raw) => {
                debug!(log, "Found single-element raw, returning");

                return Consumption::ok(Element::Raw(next_extracted_1.slice), &remaining[2..]);
            }

            // Other, proceed with rule logic
            (_, _) => (),
        }
    }

    // Handle the other cases, which are:
    // * "@@ <tokens> @@"
    // * "@< <tokens> >@"
    //
    // Collect the first and last token to build a slice of its contents.
    // The last will be updated with each step in the iterator.

    let (start, mut end) = {
        let extracted = remaining
            .first()
            .expect("There should be at least one token left after the special cases");

        (extracted, extracted)
    };

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
                        full_text.slice(log, start, end)
                    };

                    let element = Element::Raw(slice);
                    return Consumption::ok(element, new_remaining);
                }

                trace!(log, "Wasn't end of raw, continuing");
            }

            // Hit a newline, abort
            Token::LineBreak | Token::ParagraphBreak => {
                trace!(log, "Reached newline, aborting");

                return Consumption::err(ParseError::new(
                    ParseErrorKind::RuleFailed,
                    RULE_RAW,
                    new_extracted,
                ));
            }

            // Hit the end of the input, abort
            Token::InputEnd => {
                trace!(log, "Reached end of input, aborting");

                return Consumption::err(ParseError::new(
                    ParseErrorKind::EndOfInput,
                    RULE_RAW,
                    new_extracted,
                ));
            }

            // No special handling, append to slices like normal
            _ => (),
        }

        trace!(log, "Appending present token to raw");

        // Update last token and slice.
        end = extracted;
        remaining = new_remaining;
    }

    panic!("Reached end of input without encountering a Token::InputEnd");
}
