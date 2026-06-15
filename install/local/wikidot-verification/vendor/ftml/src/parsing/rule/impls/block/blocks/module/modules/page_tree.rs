/*
 * parsing/rule/impls/block/blocks/module/modules/page_tree.rs
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

pub const MODULE_PAGE_TREE: ModuleRule = ModuleRule {
    name: "module-page-tree",
    accepts_names: &["PageTree"],
    parse_fn,
};

fn parse_fn<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    mut arguments: Arguments<'t>,
) -> ParseResult<'r, 't, ModuleParseOutput<'t>> {
    debug!("Parsing PageTree module");
    assert_module_name(&MODULE_PAGE_TREE, name);

    let root = arguments.get("root");
    let depth = arguments.get_value(parser, "depth")?;
    let show_root = arguments.get_bool(parser, "showRoot")?.unwrap_or(false);

    ok!(false; Module::PageTree {
        root,
        show_root,
        depth
    })
}
