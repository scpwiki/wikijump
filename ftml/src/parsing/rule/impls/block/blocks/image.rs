/*
 * parsing/rule/impls/block/blocks/image.rs
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
use crate::tree::{FloatAlignment, ImageSource};

pub const BLOCK_IMAGE: BlockRule = BlockRule {
    name: "block-image",
    accepts_names: &["image", "=image", "<image", ">image", "f<image", "f>image"],
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
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(
        log,
        "Parsing image block";
        "in-head" => in_head,
        "name" => name,
    );

    assert!(!flag_star, "Image doesn't allow star flag");
    assert!(!flag_score, "Image doesn't allow score flag");
    assert_block_name(&BLOCK_IMAGE, name);

    let (source, mut arguments) = parser.get_head_name_map(&BLOCK_IMAGE, in_head)?;
    let link = arguments.get("link");
    let alignment = FloatAlignment::parse(name);

    // Parse the image source based on format
    let source = match ImageSource::parse(source) {
        Some(source) => source,
        None => return Err(parser.make_warn(ParseWarningKind::BlockMalformedArguments)),
    };

    // Build image
    let element = Element::Image {
        source,
        link,
        alignment,
        attributes: arguments.to_attribute_map(),
    };

    ok!(element)
}
