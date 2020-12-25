/*
 * parse/rule/impls/block/mapping.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2020 Ammon Smith
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

use super::{impls::*, BlockRule};
use std::collections::HashMap;
use unicase::UniCase;

/// Listing of all `BlockRule`s in no particular order.
pub const BLOCK_RULES: [BlockRule; 0] = [];

/// Type definition for the `TAG_RULE_MAP` constant.
pub type BlockRuleMap = HashMap<UniCase<&'static str>, &'static BlockRule>;

lazy_static! {
    /// Mapping of block names with their rule information.
    pub static ref BLOCK_RULE_MAP: BlockRuleMap = build_block_rule_map(&BLOCK_RULES);
}

#[inline]
pub fn block_with_name(name: &str) -> Option<&'static BlockRule> {
    let name = UniCase::ascii(name);

    BLOCK_RULE_MAP.get(&name).map(|rule| *rule)
}

pub(crate) fn build_block_rule_map(block_rules: &'static [BlockRule]) -> BlockRuleMap {
    let mut map = HashMap::new();

    for block_rule in block_rules {
        for name in block_rule.accepts_names {
            let name = UniCase::ascii(*name);
            let previous = map.insert(name, block_rule);

            assert!(
                previous.is_none(),
                "Overwrote previous block rule during rule population!",
            );
        }
    }

    map
}
