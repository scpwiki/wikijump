/*
 * parsing/rule/impls/block/blocks/raw.rs
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

// NOTE: "accepts_newlines" needs to be false here to avoid end trimming from get_body_text
pub const BLOCK_RAW: BlockRule = BlockRule {
    name: "block-raw",
    accepts_names: &["raw"],
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
    trace!("Parsing raw block (name '{name}', in-head {in_head}, score {flag_score})");
    assert!(!flag_star, "Raw doesn't allow star flag");
    assert!(!flag_score, "Raw doesn't allow score flag");

    assert_block_name(&BLOCK_RAW, name);

    let mut content = parser.get_body_text(&BLOCK_RAW)?;

    // Empty block
    if content.eq("\n") {
        content = "";
    }
    // Trim the first and last \n if it's a multi-line block.
    // We already checked for the single newline case above.
    else if content.starts_with('\n') && content.ends_with('\n') {
        content = content.trim_start_matches('\n').trim_end_matches('\n');
    }

    let element = Element::Raw(cow!(content));
    ok!(element)
}
