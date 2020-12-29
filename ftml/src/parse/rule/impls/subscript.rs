/*
 * parse/rule/impls/subscript.rs
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

pub const RULE_SUBSCRIPT: Rule = Rule {
    name: "subscript",
    try_consume_fn,
};

fn try_consume_fn<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> ParseResult<'r, 't, Element<'t>> {
    debug!(log, "Trying to create subscript container");

    try_container(
        log,
        (extracted, remaining, full_text),
        (RULE_SUBSCRIPT, ContainerType::Subscript),
        (Token::Subscript, Token::Subscript),
        &[Token::ParagraphBreak],
        &[
            (Token::Subscript, Token::Whitespace),
            (Token::Whitespace, Token::Subscript),
        ],
    )
}
