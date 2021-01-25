/*
 * parsing/rule/impls/block/blocks/css.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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

pub const BLOCK_CSS: BlockRule = BlockRule {
    name: "block-css",
    accepts_names: &["css"],
    accepts_special: false,
    accepts_newlines: true,
    parse_fn,
};

fn parse_fn<'r, 't>(
    log: &slog::Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    special: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Element<'t>> {
    debug!(log, "Parsing CSS block"; "in-head" => in_head);

    assert_eq!(special, false, "Code doesn't allow special variant");
    assert_block_name(&BLOCK_CSS, name);

    parser.get_head_none(&BLOCK_CSS, in_head)?;

    let css = parser.get_body_text(&BLOCK_CSS)?;
    let exceptions = vec![ParseException::Style(cow!(css))];
    ok!(Element::Null, exceptions)
}
