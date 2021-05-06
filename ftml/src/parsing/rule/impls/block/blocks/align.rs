/*
 * parsing/rule/impls/block/blocks/align.rs
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
use crate::tree::{Alignment, AttributeMap};

macro_rules! make_align_block {
    ($block_const:ident, $block_name:expr, $symbol:expr, $align:ident) => {
        use super::align::parse_alignment_block;
        use super::prelude::*;
        use crate::tree::Alignment;

        pub const $block_const: BlockRule = BlockRule {
            name: $block_name,
            accepts_names: &[$symbol],
            accepts_star: false,
            accepts_score: false,
            accepts_newlines: true,
            parse_fn,
        };

        fn parse_fn<'r, 't>(
            log: &Logger,
            parser: &mut Parser<'r, 't>,
            name: &'t str,
            special: bool,
            modifier: bool,
            in_head: bool,
        ) -> ParseResult<'r, 't, Elements<'t>> {
            parse_alignment_block(
                (&$block_const, Alignment::$align),
                log,
                parser,
                name,
                special,
                modifier,
                in_head,
            )
        }
    };
}

pub fn parse_alignment_block<'r, 't>(
    (block_rule, alignment): (&BlockRule, Alignment),
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    special: bool,
    modifier: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(
        log,
        "Parsing alignment block";
        "block-rule" => block_rule.name,
        "alignment" => alignment.name(),
        "in-head" => in_head,
        "name" => name,
    );

    assert!(!special, "Alignment block doesn't allow special variant");
    assert!(!modifier, "Alignment block doesn't allow modifier variant");
    assert_block_name(block_rule, name);

    parser.get_head_none(block_rule, in_head)?;

    // Get body content, with paragraphs
    let (elements, exceptions, _) = parser.get_body_elements(block_rule, true)?.into();

    // Build element
    let element = Element::Container(Container::new(
        ContainerType::Align(alignment),
        elements,
        AttributeMap::new(),
    ));

    ok!(element, exceptions)
}
