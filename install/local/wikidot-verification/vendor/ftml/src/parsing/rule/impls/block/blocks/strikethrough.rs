/*
 * parsing/rule/impls/block/blocks/strikethrough.rs
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

pub const BLOCK_STRIKETHROUGH: BlockRule = BlockRule {
    name: "block-strikethrough",
    accepts_names: &["s", "strikethrough"],
    accepts_star: false,
    accepts_score: false,
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
    debug!("Parsing strikethrough block (name '{name}', in-head {in_head})");
    assert!(!flag_star, "Strikethrough doesn't allow star flag");
    assert!(!flag_score, "Strikethrough doesn't allow score flag");
    assert_block_name(&BLOCK_STRIKETHROUGH, name);

    let arguments = parser.get_head_map(&BLOCK_STRIKETHROUGH, in_head)?;

    // Get body content, without paragraphs
    let (elements, errors, paragraph_safe) = parser
        .get_body_elements(&BLOCK_STRIKETHROUGH, false)?
        .into();

    let element = Element::Container(Container::new(
        ContainerType::Strikethrough,
        elements,
        arguments.to_attribute_map(parser.settings()),
    ));

    ok!(paragraph_safe; element, errors)
}
