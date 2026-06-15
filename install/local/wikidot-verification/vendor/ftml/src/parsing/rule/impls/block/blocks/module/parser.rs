/*
 * parsing/rule/impls/block/blocks/module/parser.rs
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

use super::ModuleRule;
use crate::parsing::Parser;

impl<'r, 't> Parser<'r, 't>
where
    'r: 't,
{
    #[inline]
    pub fn set_module(&mut self, module_rule: &ModuleRule) {
        debug!("Running module rule {} for these tokens", module_rule.name);
        self.set_rule(module_rule.rule());
    }
}
