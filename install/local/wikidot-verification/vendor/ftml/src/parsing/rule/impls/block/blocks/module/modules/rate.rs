/*
 * parsing/rule/impls/block/blocks/module/modules/rate.rs
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

pub const MODULE_RATE: ModuleRule = ModuleRule {
    name: "module-rate",
    accepts_names: &["Rate"],
    parse_fn,
};

fn parse_fn<'r, 't>(
    _parser: &mut Parser<'r, 't>,
    name: &'t str,
    _arguments: Arguments<'t>,
) -> ParseResult<'r, 't, ModuleParseOutput<'t>> {
    debug!("Parsing categories module");
    assert_module_name(&MODULE_RATE, name);
    ok!(false; Module::Rate)
}
