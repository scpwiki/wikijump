/*
 * parsing/rule/impls/block/blocks/later.rs
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

//! This module is an easter egg.
//!
//! It only outputs `later.` and nothing else.
//! It is a reference to Wikidot's broken `RecentThreads` module
//! (not to be confused with `MiniRecentThreads`) which only
//! outputted "later." and no other functionality.
//!
//! See <https://twitter.com/wikidotbugs/status/1328588862218702850>

use super::prelude::*;

pub const BLOCK_LATER: BlockRule = BlockRule {
    name: "block-later",
    accepts_names: &["later"],
    accepts_star: true,
    accepts_score: false,
    accepts_newlines: true,
    parse_fn,
};

fn parse_fn<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    _flag_star: bool,
    _flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Parsing later block (easter egg, in-head {in_head})");
    assert_block_name(&BLOCK_LATER, name);
    parser.get_head_none(&BLOCK_LATER, in_head)?;
    ok!(text!("later."))
}
