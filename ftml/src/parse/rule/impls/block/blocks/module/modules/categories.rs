/*
 * parse/rule/impls/block/blocks/module/modules/categories.rs
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

pub const MODULE_CATEGORIES: ModuleRule = ModuleRule {
    name: "module-categories",
    accepts_names: &["Categories"],
    parse_fn,
};

fn parse_fn<'r, 't>(
    log: &slog::Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    mut arguments: Arguments<'t>,
) -> ParseResult<'r, 't, Module<'t>> {
    debug!(log, "Parsing categories module");

    assert!(
        name.eq_ignore_ascii_case("Categories"),
        "Module doesn't have a valid name",
    );

    let include_hidden = match arguments.get("includeHidden") {
        Some(value) => parse_boolean(value)
            .map_err(|_| parser.make_warn(ParseWarningKind::BlockMalformedArguments))?,
        None => false,
    };

    ok!(Module::Categories { include_hidden })
}
