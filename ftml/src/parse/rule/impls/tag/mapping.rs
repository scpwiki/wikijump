/*
 * parse/rule/impls/tag/mapping.rs
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

use super::{impls::*, TagRule};
use std::collections::HashMap;
use unicase::UniCase;

/// Listing of all `TagRule`s in no particular order.
pub const TAG_RULES: [TagRule; 0] = [];

/// Type definition for the `TAG_RULE_MAP` constant.
pub type TagRuleMap = HashMap<UniCase<&'static str>, &'static TagRule>;

lazy_static! {
    /// Mapping of tag names with their rule information.
    pub static ref TAG_RULE_MAP: TagRuleMap = {
        let mut map = HashMap::new();

        for tag in &TAG_RULES {
            for name in tag.accepts_names {
                let name = UniCase::ascii(*name);
                let previous = map.insert(name, tag);

                assert!(
                    previous.is_none(),
                    "Overwrote previous tag rule during rule population!",
                );
            }
        }

        map
    };
}

#[inline]
pub fn tag_with_name(name: &str) -> Option<&'static TagRule> {
    let name = UniCase::ascii(name);

    TAG_RULE_MAP.get(&name).map(|rule| *rule)
}
