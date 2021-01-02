/*
 * parse/rule/impls/block/impls/code.rs
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

pub const BLOCK_CODE: BlockRule = BlockRule {
    name: "block-code",
    accepts_names: &["code"],
    accepts_special: false,
    parse_fn,
};

fn parse_fn<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &'p mut BlockParser<'p, 'r, 't>,
    name: &'t str,
    special: bool,
) -> ParseResult<'r, 't, Element<'t>> {
    assert_eq!(special, false, "Code doesn't allow special variant");
    assert!(
        name.eq_ignore_ascii_case("code"),
        "Code doesn't have a valid name",
    );

    let mut arguments = parser.get_argument_map()?;
    let language = arguments.get("type");
    parser.get_line_break()?;

    let first = parser.current();
    let mut end;

    // Keep iterating until we find "[[/code]]"
    loop {
        if !parser.borrow().evaluate(ParseCondition::token_pair(
            Token::LineBreak,
            Token::LeftBlockEnd,
        )) {
            parser.step()?;
            continue;
        }

        end = parser.current();
        parser.step()?;

        // Check if it's an end block
        // Ignore errors, might be just more code
        let name = match parser.get_end_block() {
            Ok(name) => name,
            Err(_) => continue,
        };

        // Check if it's a closing block
        if name.eq_ignore_ascii_case("code") {
            break;
        }
    }

    let code = parser.full_text().slice(log, first, end);
    let element = Element::Code {
        contents: cow!(code),
        language,
    };

    ok!(element)
}
