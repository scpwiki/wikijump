/*
 * parsing/rule/impls/block/blocks/media.rs
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
use crate::tree::{Alignment, FileSource, FloatAlignment};

pub const BLOCK_AUDIO: BlockRule = BlockRule {
    name: "block-audio",
    accepts_names: &["audio"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: false,
    parse_fn: parse_audio,
};

pub const BLOCK_VIDEO: BlockRule = BlockRule {
    name: "block-video",
    accepts_names: &["video"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: false,
    parse_fn: parse_video,
};

fn parse_audio<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    parse_media_block(
        parser,
        &BLOCK_AUDIO,
        name,
        flag_star,
        flag_score,
        in_head,
        |source, alignment, attributes| Element::Audio {
            source,
            alignment,
            attributes,
        },
    )
}

fn parse_video<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    parse_media_block(
        parser,
        &BLOCK_VIDEO,
        name,
        flag_star,
        flag_score,
        in_head,
        |source, alignment, attributes| Element::Video {
            source,
            alignment,
            attributes,
        },
    )
}

fn parse_media_block<'r, 't, F>(
    parser: &mut Parser<'r, 't>,
    block_rule: &BlockRule,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
    build_element: F,
) -> ParseResult<'r, 't, Elements<'t>>
where
    F: FnOnce(
        FileSource<'t>,
        Option<FloatAlignment>,
        crate::tree::AttributeMap<'t>,
    ) -> Element<'t>,
{
    debug!(
        "Parsing media block (rule {}, name {name}, in-head {in_head})",
        block_rule.name
    );
    assert!(!flag_star, "Media blocks don't allow star flag");
    assert!(!flag_score, "Media blocks don't allow score flag");
    assert_block_name(block_rule, name);

    let (source, mut arguments) = parser.get_head_name_map(block_rule, in_head)?;
    let alignment = parse_media_alignment(parser, &mut arguments)?;

    let source = match FileSource::parse(source) {
        Some(source) => source,
        None => return Err(parser.make_err(ParseErrorKind::BlockMalformedArguments)),
    };

    if arguments.get("src").is_some() {
        return Err(parser.make_err(ParseErrorKind::BlockMalformedArguments));
    }

    // TODO: html render settings to allow this?
    let _autoplay = arguments.get("autoplay");

    ok!(build_element(
        source,
        alignment,
        arguments.to_attribute_map(parser.settings()),
    ))
}

fn parse_media_alignment<'r, 't>(
    parser: &mut Parser<'r, 't>,
    arguments: &mut Arguments<'t>,
) -> Result<Option<FloatAlignment>, ParseError> {
    let Some(value) = arguments.get("align") else {
        return Ok(None);
    };

    let align = match value.as_ref() {
        "left" => Alignment::Left,
        "right" => Alignment::Right,
        "center" => Alignment::Center,
        _ => return Err(parser.make_err(ParseErrorKind::BlockMalformedArguments)),
    };

    Ok(Some(FloatAlignment {
        align,
        float: false,
    }))
}
