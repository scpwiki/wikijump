/*
 * parsing/rule/impls/block/blocks/size.rs
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
use std::borrow::Cow;

pub const BLOCK_SIZE: BlockRule = BlockRule {
    name: "block-size",
    accepts_names: &["size"],
    accepts_special: false,
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
        "Parsing size block";
        "in-head" => in_head,
        "name" => name,
    );

    assert_eq!(special, false, "Size doesn't allow special variant");
    assert_block_name(&BLOCK_SIZE, name);

    let size =
        parser.get_head_value(&BLOCK_SIZE, in_head, |parser, value| match value {
            Some(size) => Ok(format!("font-size: {};", size)),
            None => Err(parser.make_warn(ParseWarningKind::BlockMissingArguments)),
        })?;

    // Get body content, without paragraphs
    let (elements, exceptions) = parser.get_body_elements(&BLOCK_SIZE, false)?.into();

    let element = Element::StyledContainer(StyledContainer::new(
        StyledContainerType::Size,
        elements,
        hashmap! {
            Cow::Borrowed("style") => Cow::Owned(size),
        },
    ));

    ok!(element, exceptions)
}
