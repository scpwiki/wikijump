/*
 * parsing/rule/impls/block/blocks/monospace.rs
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

pub const BLOCK_MONOSPACE: BlockRule = BlockRule {
    name: "block-monospace",
    accepts_names: &["tt", "mono", "monospace"],
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
        "Parsing monospace block";
        "in-head" => in_head,
        "name" => name,
    );

    assert!(!flag_star, "Monospace doesn't allow star flag");
    assert!(!flag_score, "Monospace doesn't allow score flag");
    assert_block_name(&BLOCK_MONOSPACE, name);

    let arguments = parser.get_head_map(&BLOCK_MONOSPACE, in_head)?;

    // Get body content, without paragraphs
    let (elements, exceptions, paragraph_safe) =
        parser.get_body_elements(&BLOCK_MONOSPACE, false)?.into();

    let element = Element::Container(Container::new(
        ContainerType::Monospace,
        elements,
        arguments.to_attribute_map(),
    ));

    ok!(paragraph_safe; element, exceptions)
}
