/*
 * parse/rule/impls/italics.rs
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

pub const RULE_ITALICS: Rule = Rule {
    name: "italics",
    try_consume_fn,
};

fn try_consume_fn<'r, 't>(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> Consumption<'r, 't> {
    debug!(log, "Trying to create italics container");

    try_container(
        log,
        (extracted, remaining, full_text),
        (RULE_ITALICS, ContainerType::Italics),
        (Token::Italics, Token::Italics),
        &[Token::ParagraphBreak],
        &[
            (Token::Italics, Token::Whitespace),
            (Token::Whitespace, Token::Italics),
        ],
    )
}
