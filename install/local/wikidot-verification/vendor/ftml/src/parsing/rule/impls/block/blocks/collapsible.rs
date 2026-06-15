/*
 * parsing/rule/impls/block/blocks/collapsible.rs
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
use crate::parsing::{ParseError, ParseErrorKind};

pub const BLOCK_COLLAPSIBLE: BlockRule = BlockRule {
    name: "block-collapsible",
    accepts_names: &["collapsible"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: true,
    parse_fn,
};

fn parse_fn<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Parsing collapsible block (in-head {in_head})");
    assert!(!flag_star, "Collapsible doesn't allow star flag");
    assert!(!flag_score, "Collapsible doesn't allow score flag");
    assert_block_name(&BLOCK_COLLAPSIBLE, name);

    let mut arguments = parser.get_head_map(&BLOCK_COLLAPSIBLE, in_head)?;

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

    // Get body content, with paragraphs.
    // Discard paragraph_safe, since collapsibles never are.
    let (elements, errors, _) =
        parser.get_body_elements(&BLOCK_COLLAPSIBLE, true)?.into();

    // Build element and return
    let element = Element::Collapsible {
        elements,
        attributes: arguments.to_attribute_map(parser.settings()),
        start_open,
        show_text,
        hide_text,
        show_top,
        show_bottom,
    };

    ok!(element, errors)
}

fn parse_hide_location(s: &str, parser: &Parser) -> Result<(bool, bool), ParseError> {
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

    warn!("Unknown hideLocation argument '{s}'");
    Err(parser.make_err(ParseErrorKind::BlockMalformedArguments))
}
