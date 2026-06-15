/*
 * parsing/rule/impls/block/blocks/radio.rs
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

pub const BLOCK_RADIO: BlockRule = BlockRule {
    name: "block-radio",
    accepts_names: &["radio", "radio-button"],
    accepts_star: true,
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
    debug!(
        "Parsing radio button block (name '{name}', in-head {in_head}, star {flag_star})",
    );
    assert!(!flag_score, "Radio buttons don't allow score flag");
    assert_block_name(&BLOCK_RADIO, name);

    let (name, arguments) = parser.get_head_name_map(&BLOCK_RADIO, in_head)?;
    parser.get_optional_space()?;

    let element = Element::RadioButton {
        name: cow!(name),
        checked: flag_star,
        attributes: arguments.to_attribute_map(parser.settings()),
    };

    ok!(element)
}
