/*
 * parsing/rule/impls/block/blocks/iframe.rs
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

pub const BLOCK_IFRAME: BlockRule = BlockRule {
    name: "block-iframe",
    accepts_names: &["iframe"],
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
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(log, "Parsing iframe block"; "in-head" => in_head);

    assert_eq!(special, false, "iframe doesn't allow special variant");
    assert_block_name(&BLOCK_IFRAME, name);

    let (url, arguments) = parser.get_head_name_map(&BLOCK_IFRAME, in_head)?;
    let element = Element::Iframe {
        url: cow!(url),
        attributes: arguments.to_hash_map(),
    };

    ok!(element)
}
