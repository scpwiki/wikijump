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
use crate::parse::consume::GenericConsumption;
use crate::parse::token::ExtractedToken;
use crate::parse::UpcomingTokens;
use crate::text::FullText;

mod arguments;
mod body;
mod rule;

pub mod impls;

pub use self::rule::{RULE_BLOCK, RULE_BLOCK_SPECIAL};

#[derive(Debug)]
pub struct BlockParser<'r, 't> {
    tokens: UpcomingTokens<'r, 't>,
    full_text: FullText<'t>,
    special: bool,
}

impl<'r, 't> BlockParser<'r, 't> {
    #[inline]
    pub fn new(
        extracted: &'r ExtractedToken<'t>,
        remaining: &'r [ExtractedToken<'t>],
        full_text: FullText<'t>,
        special: bool,
    ) -> Self {
        let tokens = UpcomingTokens::Split { extracted, remaining };

        BlockParser { tokens, full_text, special }
    }
}

/// Define a rule for how to parse a block.
#[derive(Clone)]
pub struct BlockRule {
    /// The name of the block. Must be kebab-case and globally unique.
    name: &'static str,

    /// Which names you can use this block with. Case-insensitive.
    /// Will panic if empty.
    accepts_names: &'static [&'static str],

    /// Whether this block requires a sub name.
    ///
    /// For instance, `[[module]]` requires the name of the module
    /// being used specified, where something like `[[code]]` is
    /// just "code".
    ///
    /// This is a mapping of names to the block rules that implement
    /// that particular block.
    ///
    /// If this value is `Some(_)`, it cannot be empty.
    sub_names_mapping: Option<()>,

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
    parse_fn: (),
}
