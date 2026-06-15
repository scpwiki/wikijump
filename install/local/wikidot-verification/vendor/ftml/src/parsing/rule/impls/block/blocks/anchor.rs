/*
 * parsing/rule/impls/block/blocks/anchor.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2026 Wikijump Team
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
use crate::parsing::strip_newlines;
use crate::tree::AnchorTarget;

pub const BLOCK_ANCHOR: BlockRule = BlockRule {
    name: "block-anchor",
    accepts_names: &["a", "anchor"],
    accepts_star: true,
    accepts_score: true,
    accepts_newlines: false,
    parse_fn,
};

fn parse_fn<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Parsing anchor block (name '{name}', in-head {in_head}, star {flag_star})");
    assert_block_name(&BLOCK_ANCHOR, name);

    let arguments = parser.get_head_map(&BLOCK_ANCHOR, in_head)?;
    let attributes = arguments.to_attribute_map(parser.settings());

    // "a" means we wrap interpret as-is
    // "a_" means we strip out any newlines or paragraph breaks
    let strip_line_breaks = flag_score;

    // Get anchor target depending on special
    let target = if flag_star {
        Some(AnchorTarget::NewTab)
    } else {
        None
    };

    // Get body content, without paragraphs
    let (mut elements, errors, paragraph_safe) =
        parser.get_body_elements(&BLOCK_ANCHOR, false)?.into();

    if strip_line_breaks {
        strip_newlines(&mut elements);
    }

    let element = Element::Anchor {
        elements,
        attributes,
        target,
    };

    ok!(paragraph_safe; element, errors)
}
