/*
 * parsing/rule/impls/block/blocks/module/modules/mod.rs
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

mod prelude {
    pub use super::super::{BLOCK_MODULE, ModuleParseOutput, ModuleRule, prelude::*};
    pub use crate::tree::Module;

    #[inline]
    pub fn assert_module_name(module_rule: &ModuleRule, actual_name: &str) {
        assert_generic_name(module_rule.accepts_names, actual_name, "module")
    }
}

mod backlinks;
mod categories;
mod css;
mod join;
mod page_tree;
mod rate;

pub use self::backlinks::MODULE_BACKLINKS;
pub use self::categories::MODULE_CATEGORIES;
pub use self::css::MODULE_CSS;
pub use self::join::MODULE_JOIN;
pub use self::page_tree::MODULE_PAGE_TREE;
pub use self::rate::MODULE_RATE;
