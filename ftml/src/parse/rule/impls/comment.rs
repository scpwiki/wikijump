/*
 * parse/rule/impls/comment.rs
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

pub const RULE_COMMENT: Rule = Rule {
    name: "comment",
    try_consume_fn,
};

fn try_consume_fn<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    mut remaining: &'r [ExtractedToken<'t>],
    _full_text: FullText<'t>,
) -> Consumption<'r, 't> {
    debug!(log, "Consuming tokens until end of comment");

    assert_eq!(
        extracted.token,
        Token::LeftComment,
        "Current token isn't a LeftComment",
    );

    while let Some((new_extracted, new_remaining)) = remaining.split_first() {
        let ExtractedToken { token, span, slice } = new_extracted;
        debug!(
            log,
            "Received token inside comment";
            "token" => token,
            "slice" => slice,
            "span-start" => span.start,
            "span-end" => span.end,
        );

        // Check token
        match token {
            // Hit the end of the comment, return
            Token::RightComment => {
                trace!(log, "Reached end of comment, returning");

                return Consumption::ok(Element::Null, new_remaining);
            }

            // Hit the end of the input, abort
            Token::InputEnd => {
                trace!(log, "Reached end of input, aborting");

                return Consumption::err(ParseError::new(
                    ParseErrorKind::EndOfInput,
                    RULE_COMMENT,
                    new_extracted,
                ));
            }

            // Consume any other token
            _ => {
                trace!(log, "Token inside comment received. Discarding.");

                // Update pointer
                remaining = new_remaining;
            }
        }
    }

    panic!("Reached end of input without encountering a Token::InputEnd");
}
