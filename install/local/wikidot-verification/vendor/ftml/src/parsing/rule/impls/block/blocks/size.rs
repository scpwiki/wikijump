/*
 * parsing/rule/impls/block/blocks/size.rs
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
use crate::tree::AttributeMap;
use std::borrow::Cow;

pub const BLOCK_SIZE: BlockRule = BlockRule {
    name: "block-size",
    accepts_names: &["size"],
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
    debug!("Parsing size block (name '{name}', in-head {in_head})");
    assert!(!flag_star, "Size doesn't allow star flag");
    assert!(!flag_score, "Size doesn't allow score flag");
    assert_block_name(&BLOCK_SIZE, name);

    let size =
        parser.get_head_value(&BLOCK_SIZE, in_head, |parser, value| match value {
            Some(size) => Ok(format!("font-size: {size};")),
            None => Err(parser.make_err(ParseErrorKind::BlockMissingArguments)),
        })?;

    // Get body content, without paragraphs
    let (elements, errors, paragraph_safe) =
        parser.get_body_elements(&BLOCK_SIZE, false)?.into();

    let attributes = {
        let mut map = AttributeMap::new();
        map.insert("style", Cow::Owned(size));
        map
    };

    let element =
        Element::Container(Container::new(ContainerType::Size, elements, attributes));

    ok!(paragraph_safe; element, errors)
}
