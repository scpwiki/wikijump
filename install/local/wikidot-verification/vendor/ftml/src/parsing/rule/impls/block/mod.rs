/*
 * parsing/rule/impls/block/mod.rs
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

//! Meta-rule for all block constructs.
//!
//! This matches `[[` or `[[*` and runs the block parsing
//! against the upcoming tokens in accordance to how the
//! various blocks define themselves.

use crate::parsing::Parser;
use crate::parsing::result::ParseResult;
use crate::parsing::rule::{LineRequirement, Rule};
use crate::tree::Elements;
use std::fmt::{self, Debug};

mod arguments;
mod mapping;
mod parser;
mod rule;

pub mod blocks;

pub use self::arguments::Arguments;
pub use self::rule::{RULE_BLOCK, RULE_BLOCK_SKIP_NEWLINE, RULE_BLOCK_STAR};

/// Define a rule for how to parse a block.
#[derive(Clone)]
pub struct BlockRule {
    /// The code name of the block.
    ///
    /// As this is an internal structure, we can assert the following things:
    /// * It is in kebab-case.
    /// * It is globally unique.
    /// * It is prefixed with `block-`.
    name: &'static str,

    /// Which names you can use this block with. Case-insensitive.
    /// Will panic if empty.
    accepts_names: &'static [&'static str],

    /// Whether this block accepts the star flag (`*`).
    ///
    /// For instance, user can be invoked as both
    /// `[[user aismallard]]` and `[[*user aismallard]]`.
    accepts_star: bool,

    /// Whether this block accepts the score flag (`_`).
    ///
    /// For instance, div can be invoked as both
    /// `[[div]]` and `[[div_]]`.
    accepts_score: bool,

    /// Whether this block optionally allows its head and tail to be separated by newlines.
    /// These newlines will be consumed and not be interpreted as line breaks.
    ///
    /// For instance, `[[div]]`, which can be declared on separate lines, or inline, without
    /// those newlines becoming part of the resultant element:
    ///
    /// ```text
    /// [[div]]
    /// My fancy div!
    /// [[/div]]
    /// ```
    ///
    /// ```text
    /// [[div]]My fancy inline div![[/div]]
    /// ```
    accepts_newlines: bool,

    /// Function which implements the processing for this rule.
    parse_fn: BlockParseFn,
}

impl BlockRule {
    /// Produces a pseudo parse `Rule` associated with this `BlockRule`.
    ///
    /// It should not be invoked, it is for error construction.
    #[cold]
    pub fn rule(&self) -> Rule {
        // Stubbed try_consume_fn implementation for the Rule.
        fn try_consume_fn<'r, 't>(
            _: &mut Parser<'r, 't>,
        ) -> ParseResult<'r, 't, Elements<'t>> {
            panic!("Pseudo rule for this block should not be executed directly!");
        }

        Rule {
            name: self.name,
            position: LineRequirement::Any,
            try_consume_fn,
        }
    }
}

impl Debug for BlockRule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BlockRule")
            .field("name", &self.name)
            .field("accepts_names", &self.accepts_names)
            .field("accepts_star", &self.accepts_star)
            .field("accepts_score", &self.accepts_score)
            .field("accepts_newlines", &self.accepts_newlines)
            .field("parse_fn", &(self.parse_fn as *const ()))
            .finish()
    }
}

/// Function pointer type to implement block parsing.
///
/// The arguments are, in order:
/// * `parser` -- `Parser` instance
/// * `name` -- The name of the block
/// * `flag_star` -- Whether this block is has the star flag (`*`).
/// * `flag_score` -- Whether this block has the score flag (`_`).
/// * `in_head` -- Whether we're still in the block head, or if it's finished
pub type BlockParseFn = for<'r, 't> fn(
    &mut Parser<'r, 't>,
    &'t str,
    bool,
    bool,
    bool,
) -> ParseResult<'r, 't, Elements<'t>>;
