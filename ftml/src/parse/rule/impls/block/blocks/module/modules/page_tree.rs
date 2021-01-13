/*
 * parse/rule/impls/block/blocks/module/modules/page_tree.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Ammon Smith
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
use crate::parse::parse_boolean;

pub const MODULE_PAGE_TREE: ModuleRule = ModuleRule {
    name: "module-page-tree",
    accepts_names: &["PageTree"],
    parse_fn,
};

fn parse_fn<'r, 't>(
    log: &slog::Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    mut arguments: Arguments<'t>,
) -> ParseResult<'r, 't, Module<'t>> {
    debug!(log, "Parsing PageTree module");
    assert_module_name(&MODULE_PAGE_TREE, name);

    let root = arguments.get("root");

    let show_root = match arguments.get("includeHidden") {
        Some(value) => parse_boolean(value)
            .map_err(|_| parser.make_warn(ParseWarningKind::BlockMalformedArguments))?,
        None => false,
    };

    let depth = match arguments.get("depth") {
        Some(value) => {
            let depth = value.as_ref().parse().map_err(|_| {
                parser.make_warn(ParseWarningKind::BlockMalformedArguments)
            })?;

            Some(depth)
        }
        None => None,
    };

    ok!(Module::PageTree {
        root,
        show_root,
        depth
    })
}
