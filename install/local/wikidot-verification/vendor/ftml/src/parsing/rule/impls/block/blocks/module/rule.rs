/*
 * parsing/rule/impls/block/blocks/module/rule.rs
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

use super::mapping::get_module_rule_with_name;
use super::prelude::*;

pub const BLOCK_MODULE: BlockRule = BlockRule {
    name: "block-module",
    accepts_names: &["module", "module654"],
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
    debug!("Parsing module block (in-head {in_head})");
    parser.check_page_syntax()?;
    assert!(!flag_star, "Module doesn't allow star flag");
    assert!(!flag_score, "Module doesn't allow score flag");
    assert_block_name(&BLOCK_MODULE, name);

    // Get module name and arguments
    let (subname, arguments) = parser.get_head_name_map(&BLOCK_MODULE, in_head)?;

    // Get the module rule for this name
    let module_rule = match get_module_rule_with_name(subname) {
        Some(rule) => rule,
        None => return Err(parser.make_err(ParseErrorKind::NoSuchModule)),
    };

    // Prepare to run the module's parsing function
    parser.set_module(module_rule);

    // Run the parse function until the end.
    // This starts after the head and its newline.
    //
    // If the module accepts a body, it should consume it,
    // then the tail. Otherwise it shouldn't move the token pointer.
    let (elements, errors, paragraph_safe) =
        (module_rule.parse_fn)(parser, subname, arguments)?.into();

    ok!(paragraph_safe; elements, errors)
}
