/*
 * parse/rule/impls/block/blocks/css.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Ammon Smith
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
    parse_fn,
};

fn parse_fn<'p, 'r, 't>(
    log: &slog::Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    special: bool,
    in_block: bool,
) -> ParseResult<'r, 't, Element<'t>> {
    debug!(log, "Parsing CSS block"; "in-block" => in_block);

    assert_eq!(special, false, "Code doesn't allow special variant");
    assert!(
        name.eq_ignore_ascii_case("css"),
        "Code doesn't have a valid name",
    );

    if in_block {
        parser.get_argument_none()?;
    }

    let css = parser.get_body_text(&["css"], true)?;
    let exceptions = vec![ParseException::Style(cow!(css))];
    ok!(Element::Null, exceptions)
}
