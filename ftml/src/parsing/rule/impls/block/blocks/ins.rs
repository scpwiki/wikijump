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
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: false,
    parse_fn,
};

fn parse_fn<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, PartialElements<'t>> {
    info!(
        log,
        "Parsing insertion block";
        "in-head" => in_head,
        "name" => name,
    );

    assert!(!flag_star, "Ins doesn't allow star flag");
    assert!(!flag_score, "Ins doesn't allow score flag");
    assert_block_name(&BLOCK_INS, name);

    let arguments = parser.get_head_map(&BLOCK_INS, in_head)?;

    // Get body content, without paragraphs
    let (elements, exceptions, paragraph_safe) =
        parser.get_body_elements(&BLOCK_INS, false)?.into();

    // Build and return element
    let element = Element::Container(Container::new(
        ContainerType::Insertion,
        elements,
        arguments.to_attribute_map(),
    ));

    ok!(paragraph_safe; element, exceptions)
}
