/*
 * parsing/rule/impls/block/blocks/include_wikidot.rs
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

/// Psuedo block rule for legacy Wikidot include ("messy include").
///
/// Because includes are performed first, before preprocessing,
/// tokenizing, or any other steps, no `[[include]]` blocks
/// should actually be present in the wikitext.
///
/// If they are, this indicates that an error occurred parsing
/// them. As such, we return a particular error instead of
/// interpreting the block.
pub const BLOCK_INCLUDE_WIKIDOT: BlockRule = BlockRule {
    name: "block-include",
    accepts_names: &["include"],
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
    _in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Found invalid include block");
    parser.check_page_syntax()?;
    assert!(!flag_star, "Include (Wikidot) doesn't allow star flag");
    assert!(!flag_score, "Include (Wikidot) doesn't allow score flag");
    assert_block_name(&BLOCK_INCLUDE_WIKIDOT, name);

    // Includes are handled specially, so we should never actually be
    // parsing a block here. So, we return an error.
    Err(parser.make_err(ParseErrorKind::InvalidInclude))
}
