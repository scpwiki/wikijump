/*
 * parsing/rule/impls/block/blocks/module/mapping.rs
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

use super::{ModuleRule, modules::*};
use std::collections::HashMap;
use std::sync::LazyLock;
use unicase::UniCase;

pub const MODULE_RULES: [ModuleRule; 6] = [
    MODULE_BACKLINKS,
    MODULE_CATEGORIES,
    MODULE_CSS,
    MODULE_JOIN,
    MODULE_PAGE_TREE,
    MODULE_RATE,
];

pub type ModuleRuleMap = HashMap<UniCase<&'static str>, &'static ModuleRule>;

pub static MODULE_RULE_MAP: LazyLock<ModuleRuleMap> =
    LazyLock::new(|| build_module_rule_map(&MODULE_RULES));

#[inline]
pub fn get_module_rule_with_name(name: &str) -> Option<&'static ModuleRule> {
    let name = UniCase::ascii(name);

    MODULE_RULE_MAP.get(&name).copied()
}

fn build_module_rule_map(module_rules: &'static [ModuleRule]) -> ModuleRuleMap {
    let mut map = HashMap::new();

    for module_rule in module_rules {
        assert!(
            module_rule.name.starts_with("module-"),
            "Module name does not start with 'module-'.",
        );

        assert!(
            !module_rule.accepts_names.is_empty(),
            "Module has no accepted names",
        );

        for name in module_rule.accepts_names {
            let name = UniCase::ascii(*name);
            let previous = map.insert(name, module_rule);

            assert!(
                previous.is_none(),
                "Overwrote previous module rule during rule population!",
            );
        }
    }

    map
}

#[test]
fn module_rule_map() {
    let _ = &*MODULE_RULE_MAP;
}
