/*
 * parse/rule/impls/tag/mod.rs
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

//! Meta-rule for all tag constructs.
//!
//! This matches `[[` or `[[*` and runs the tag parsing
//! against the upcoming tokens in accordance to how the
//! various tags define themselves.

use std::fmt::{self, Debug};

mod mapping;
mod rule;

pub mod impls;

pub use self::rule::{RULE_TAG, RULE_TAG_SPECIAL};

/// Define a rule for how to parse a tag.
#[derive(Clone)]
pub struct TagRule {
    /// The name of the tag. Must be kebab-case.
    name: &'static str,

    /// Which names you can use this tag with. Case-insensitive.
    /// Will panic if empty.
    accepts_names: &'static [&'static str],

    /// Whether this tag accepts `*` as a modifier.
    ///
    /// For instance, user can be invoked as both
    /// `[[user aismallard]]` and `[[*user aismallard]]`.
    accepts_special: bool,
}

impl TagRule {
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl Debug for TagRule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TagRule")
            .field("name", &self.name)
            .field("accepts_names", &self.accepts_names)
            .field("accepts_special", &self.accepts_special)
            .finish()
    }
}

impl slog::Value for TagRule {
    fn serialize(
        &self,
        _: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        serializer.emit_str(key, self.name())
    }
}

/// The function type used for processing a rule
pub type TryParseTagFn = fn();
