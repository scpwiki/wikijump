/*
 * parsing/rule/impls/block/blocks/ins.rs
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

pub const BLOCK_INS: BlockRule = BlockRule {
    name: "block-ins",
    accepts_names: &["ins", "insertion"],
    accepts_special: false,
    parse_fn,
};

fn parse_fn<'r, 't>(
    log: &slog::Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    special: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Element<'t>> {
    debug!(
        log,
        "Parsing insertion block";
        "in-head" => in_head,
        "name" => name,
    );

    assert_eq!(special, false, "Ins doesn't allow special variant");
    assert_block_name(&BLOCK_INS, name);

    let mut arguments = parser.get_head_map(in_head)?;

    // Get styling arguments
    let id = arguments.get("id");
    let class = arguments.get("class");
    let style = arguments.get("style");

    // Get body content, without paragraphs
    let (elements, exceptions) = parser.get_body_elements(&BLOCK_INS, false)?.into();

    // Build and return element
    let element = Element::StyledContainer(StyledContainer::new(
        StyledContainerType::Insertion,
        elements,
        id,
        class,
        style,
    ));

    ok!(element, exceptions)
}
