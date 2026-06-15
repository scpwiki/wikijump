/*
 * parsing/rule/impls/block/blocks/code.rs
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
use crate::tree::CodeBlock;
use wikidot_normalize::normalize;

pub const BLOCK_CODE: BlockRule = BlockRule {
    name: "block-code",
    accepts_names: &["code"],
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
    debug!("Parsing code block (in-head {in_head})");
    assert!(!flag_star, "Code doesn't allow star flag");
    assert!(!flag_score, "Code doesn't allow score flag");
    assert_block_name(&BLOCK_CODE, name);

    let mut arguments = parser.get_head_map(&BLOCK_CODE, in_head)?;

    let mut language = arguments.get("type");
    if let Some(ref mut language) = language {
        language.to_mut().make_ascii_lowercase();
    }

    let mut name = arguments.get("name");
    if let Some(ref mut name) = name {
        normalize(name.to_mut());
    }

    let code = parser.get_body_text(&BLOCK_CODE)?;
    let code_block = CodeBlock {
        contents: cow!(code),
        language,
        name,
    };

    // We need to clone here since the same code block is
    // conveyed in two places, and some of the fields may
    // be Cow::Owned.
    let element = Element::Code(code_block.clone());
    parser.push_code_block(code_block);
    ok!(element)
}
