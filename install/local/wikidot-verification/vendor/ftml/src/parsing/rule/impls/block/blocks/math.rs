/*
 * parsing/rule/impls/block/blocks/math.rs
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

pub const BLOCK_MATH: BlockRule = BlockRule {
    name: "block-math",
    accepts_names: &["math"],
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
    debug!("Parsing math block (name '{name}', in-head {in_head})");
    assert!(!flag_star, "User doesn't allow star flag");
    assert!(!flag_score, "User doesn't allow score flag");
    assert_block_name(&BLOCK_MATH, name);

    let name = parser.get_head_value(&BLOCK_MATH, in_head, |_, value| {
        Ok(value.map(|s| cow!(s.trim())))
    })?;

    let latex_source = parser.get_body_text(&BLOCK_MATH)?.trim();
    if latex_source.is_empty() {
        return Err(parser.make_err(ParseErrorKind::RuleFailed));
    }

    let element = Element::Math {
        name,
        latex_source: cow!(latex_source),
    };

    ok!(element)
}
