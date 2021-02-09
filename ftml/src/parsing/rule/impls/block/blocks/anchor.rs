/*
 * parsing/rule/impls/block/blocks/anchor.rs
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
use crate::tree::AnchorTarget;

pub const BLOCK_ANCHOR: BlockRule = BlockRule {
    name: "block-anchor",
    accepts_names: &["a", "anchor"],
    accepts_special: true,
    accepts_newlines: false,
    parse_fn,
};

fn parse_fn<'r, 't>(
    log: &slog::Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    special: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(
        log,
        "Parsing anchor block";
        "in-head" => in_head,
        "name" => name,
        "special" => special,
    );

    assert_block_name(&BLOCK_ANCHOR, name);

    let arguments = parser.get_head_map(&BLOCK_ANCHOR, in_head)?;
    let attributes = arguments.to_hash_map();

    // Get anchor target depending on special
    let target = if special {
        AnchorTarget::NewTab
    } else {
        AnchorTarget::Same
    };

    // Get body content, without paragraphs
    let (elements, exceptions) = parser.get_body_elements(&BLOCK_ANCHOR, false)?.into();

    let element = Element::Anchor {
        elements,
        attributes,
        target,
    };

    ok!(element, exceptions)
}
