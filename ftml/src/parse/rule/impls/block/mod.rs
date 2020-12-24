/*
 * parse/rule/impls/block/mod.rs
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

//! Meta-rule for all block constructs.
//!
//! This matches `[[` or `[[*` and runs the block parsing
//! against the upcoming tokens in accordance to how the
//! various blocks define themselves.

use self::arguments::{BlockArguments, BlockArgumentsKind};
use self::body::{Body, BodyKind};
use crate::parse::consume::Consumption;
use crate::parse::token::ExtractedToken;
use crate::text::FullText;
use std::fmt::{self, Debug};

mod arguments;
mod body;
mod mapping;
mod rule;

pub mod impls;

pub use self::rule::{RULE_BLOCK, RULE_BLOCK_SPECIAL};

/// Define a rule for how to parse a block.
#[derive(Clone)]
pub struct BlockRule {
    /// The name of the block. Must be kebab-case.
    name: &'static str,

    /// Which names you can use this block with. Case-insensitive.
    /// Will panic if empty.
    accepts_names: &'static [&'static str],

    /// Whether this block accepts a second name.
    ///
    /// For instance, `[[module]]` requires the name of the module
    /// being used specified, where something like `[[code]]` is
    /// just "code".
    requires_sub_name: bool,

    /// Whether this block accepts `*` as a modifier.
    ///
    /// For instance, user can be invoked as both
    /// `[[user aismallard]]` and `[[*user aismallard]]`.
    accepts_special: bool,

    /// How this block accepts arguments.
    arguments: BlockArgumentsKind,

    /// How this block accepts a body.
    ///
    /// For instance `[[code]]` wants internals, whereas `[[module Rate]]`
    /// is standalone.
    body: BodyKind,

    /// The parse function for this block.
    ///
    /// This is the specified function to process the block's token stream
    /// and produce an element.
    parse_fn: TryParseBlockFn,
}

impl BlockRule {
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl Debug for BlockRule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BlockRule")
            .field("name", &self.name)
            .field("accepts_names", &self.accepts_names)
            .field("requires_sub_name", &self.requires_sub_name)
            .field("accepts_special", &self.accepts_special)
            .field("arguments", &self.arguments)
            .field("body", &self.body)
            .field("parse_fn", &"<fn pointer>")
            .finish()
    }
}

impl slog::Value for BlockRule {
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
pub type TryParseBlockFn = for<'r, 't> fn(
    log: &slog::Logger,
    name: &'t str,
    sub_name: Option<&'t str>,
    special: bool,
    arguments: BlockArguments<'t>,
    body: Option<&'r [ExtractedToken<'t>]>,
    full_text: FullText<'t>,
) -> Consumption<'r, 't>;
