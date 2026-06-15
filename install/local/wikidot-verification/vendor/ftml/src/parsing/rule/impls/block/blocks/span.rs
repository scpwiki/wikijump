/*
 * parsing/rule/impls/block/blocks/span.rs
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

pub const BLOCK_SPAN: BlockRule = BlockRule {
    name: "block-span",
    accepts_names: &["span"],
    accepts_star: false,
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
    debug!("Parsing span block (name '{name}', in-head {in_head})");
    assert!(!flag_star, "Span doesn't allow star flag");
    assert_block_name(&BLOCK_SPAN, name);

    let arguments = parser.get_head_map(&BLOCK_SPAN, in_head)?;

    // "span" means we wrap interpret as-is
    // "span_" means we strip out any newlines or paragraph breaks
    let strip_line_breaks = flag_score;

    // Get body content, without paragraphs
    let (mut elements, errors, paragraph_safe) =
        parser.get_body_elements(&BLOCK_SPAN, false)?.into();

    if strip_line_breaks {
        strip_newlines(&mut elements);
    }

    let element = Element::Container(Container::new(
        ContainerType::Span,
        elements,
        arguments.to_attribute_map(parser.settings()),
    ));

    ok!(paragraph_safe; element, errors)
}
