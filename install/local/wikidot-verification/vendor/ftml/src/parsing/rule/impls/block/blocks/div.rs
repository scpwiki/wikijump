/*
 * parsing/rule/impls/block/blocks/div.rs
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

pub const BLOCK_DIV: BlockRule = BlockRule {
    name: "block-div",
    accepts_names: &["div"],
    accepts_star: false,
    accepts_score: true,
    accepts_newlines: true,
    parse_fn,
};

fn parse_fn<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Parsing div block (name '{name}', in-head {in_head}, score {flag_score})");
    assert!(!flag_star, "Div doesn't allow star flag");
    assert_block_name(&BLOCK_DIV, name);

    let arguments = parser.get_head_map(&BLOCK_DIV, in_head)?;

    // "div" means we wrap in paragraphs, like normal
    // "div_" means we don't wrap it
    let wrap_paragraphs = !flag_score;

    // Get body content, based on whether we want paragraphs or not.
    // Discard paragraph_safe, since divs never are.
    let (elements, errors, _) = parser
        .get_body_elements(&BLOCK_DIV, wrap_paragraphs)?
        .into();

    // Build element and return
    let element = Element::Container(Container::new(
        ContainerType::Div,
        elements,
        arguments.to_attribute_map(parser.settings()),
    ));

    ok!(element, errors)
}
