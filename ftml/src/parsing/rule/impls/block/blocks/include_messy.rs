/*
 * parsing/rule/impls/block/blocks/include_messy.rs
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

/// Psuedo block rule for include (messy).
///
/// Because includes are performed first, before preprocessing,
/// tokenizing, or any other steps, no `[[include-messy]]` blocks
/// should actually be present in the wikitext.
///
/// If they are, this indicates that an error occurred parsing
/// them. As such, we return a particular warning instead of
/// interpreting the block.
pub const BLOCK_INCLUDE_MESSY: BlockRule = BlockRule {
    name: "block-include-messy",
    accepts_names: &["include-messy"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: true,
    parse_fn,
};

fn parse_fn<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    _in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    info!(log, "Found invalid include-messy block");

    parser.check_page_syntax()?;
    assert!(!flag_star, "Include (messy) doesn't allow star flag");
    assert!(!flag_score, "Include (messy) doesn't allow score flag");
    assert_block_name(&BLOCK_INCLUDE_MESSY, name);

    // Includes are handled specially, so we should never actually be
    // parsing a block here. So, we return a warning.

    Err(parser.make_warn(ParseWarningKind::InvalidInclude))
}
