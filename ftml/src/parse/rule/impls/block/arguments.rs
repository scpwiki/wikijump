/*
 * parse/rule/impls/block/arguments.rs
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

use std::borrow::Cow;
use std::collections::HashMap;

/// Specifying the manner that this block accepts arguments.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BlockArgumentKind {
    /// This block accepts any number of key, value pair arguments.
    ///
    /// Examples: `[[div]]`, `[[image]]`
    KeyValue,

    /// This block accepts the enter space after the block name as the argument value.
    ///
    /// Examples: `[[user]]`
    SingleValue,

    /// This block accepts no arguments.
    ///
    /// Examples: `[[footnote]]`
    None,
}

/// The values received when
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockArguments<'t> {
    KeyValue(HashMap<&'t str, Cow<'t, str>>),
    SingleValue(&'t str),
    None,
}

impl<'t> BlockArguments<'t> {
    pub fn unwrap_map(self) -> HashMap<&'t str, Cow<'t, str>> {
        match self {
            BlockArguments::KeyValue(map) => map,
            _ => panic!(
                "BlockArguments wasn't the variant KeyValue(_) (was {:?})",
                self,
            ),
        }
    }

    pub fn unwrap_value(self) -> &'t str {
        match self {
            BlockArguments::SingleValue(value) => value,
            _ => panic!(
                "BlockArguments wasn't the variant SingleValue(_) (was {:?})",
                self,
            ),
        }
    }
}
