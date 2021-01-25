/*
 * parsing/rule/impls/block/blocks/collapsible.rs
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
use crate::parsing::{ParseWarning, ParseWarningKind};

pub const BLOCK_COLLAPSIBLE: BlockRule = BlockRule {
    name: "block-collapsible",
    accepts_names: &["collapsible"],
    accepts_special: false,
    accepts_newlines: true,
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
        "Parsing collapsible block";
        "in-head" => in_head,
    );

    assert_eq!(special, false, "Collapsible doesn't allow special variant");
    assert_block_name(&BLOCK_COLLAPSIBLE, name);

    let mut arguments = parser.get_head_map(&BLOCK_COLLAPSIBLE, in_head)?;

    // Get styling arguments
    let id = arguments.get("id");
    let class = arguments.get("class");
    let style = arguments.get("style");

    // Get display arguments
    let show_text = arguments.get("show");
    let hide_text = arguments.get("hide");

    // Get folding arguments
    //
    // We invert this first argument since "folded=no" means "start_open=yes"
    let start_open = !arguments.get_bool(parser, "folded")?.unwrap_or(true);
    let (show_top, show_bottom) = match arguments.get("hideLocation") {
        Some(value) => parse_hide_location(&value, parser)?,
        None => (true, false),
    };

    // Get body content, with paragraphs
    let (elements, exceptions) =
        parser.get_body_elements(&BLOCK_COLLAPSIBLE, true)?.into();

    // Build element and return
    let element = Element::Collapsible {
        elements,
        start_open,
        show_text,
        hide_text,
        show_top,
        show_bottom,
        id,
        class,
        style,
    };

    ok!(element, exceptions)
}

fn parse_hide_location(s: &str, parser: &Parser) -> Result<(bool, bool), ParseWarning> {
    const NAMES: [(&str, (bool, bool)); 5] = [
        ("top", (true, false)),
        ("bottom", (false, true)),
        ("both", (true, true)),
        ("neither", (false, false)),
        ("none", (false, false)),
    ];

    let s = s.trim();
    for &(name, value) in &NAMES {
        if name.eq_ignore_ascii_case(s) {
            return Ok(value);
        }
    }

    debug!(&parser.log(), "Unknown hideLocation argument"; "value" => s);

    Err(parser.make_warn(ParseWarningKind::BlockMalformedArguments))
}
