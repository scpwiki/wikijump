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
use crate::parse::rule::Rule;
use crate::parse::token::{ExtractedToken, Token};
use crate::parse::{ParseError, ParseErrorKind, UpcomingTokens};
use crate::text::FullText;

mod arguments;
mod body;
mod rule;

pub mod impls;

pub use self::rule::{RULE_BLOCK, RULE_BLOCK_SPECIAL};

#[derive(Debug)]
pub struct BlockParser<'l, 'r, 't> {
    log: &'l slog::Logger,
    special: bool,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
    rule: Rule,
}

impl<'l, 'r, 't> BlockParser<'l, 'r, 't> {
    #[inline]
    pub fn new(
        log: &'l slog::Logger,
        special: bool,
        extracted: &'r ExtractedToken<'t>,
        remaining: &'r [ExtractedToken<'t>],
        full_text: FullText<'t>,
    ) -> Self {
        let rule = if special {
            RULE_BLOCK_SPECIAL
        } else {
            RULE_BLOCK
        };

        BlockParser {
            log,
            special,
            extracted,
            remaining,
            full_text,
            rule,
        }
    }

    fn update_pointer(
        &mut self,
        pointer: Option<(&'r ExtractedToken<'t>, &'r [ExtractedToken<'t>])>,
    ) -> Result<(), ParseError> {
        match pointer {
            Some((extracted, remaining)) => {
                self.extracted = extracted;
                self.remaining = remaining;

                Ok(())
            }
            None => Err(ParseError::new(
                ParseErrorKind::EndOfInput,
                self.rule,
                self.extracted,
            )),
        }
    }

    pub fn step(&mut self) -> Result<(), ParseError> {
        trace!(self.log, "Stepping to the next token");

        self.update_pointer(self.remaining.split_first())
    }

    pub fn tokens_mut<F, T>(&mut self, f: F) -> Result<T, ParseError>
    where
        F: FnOnce(&mut UpcomingTokens<'r, 't>) -> T,
    {
        let BlockParser {
            extracted,
            remaining,
            ..
        } = self;

        let mut tokens = UpcomingTokens::Split {
            extracted,
            remaining,
        };
        let result = f(&mut tokens);
        self.update_pointer(tokens.split())?;

        Ok(result)
    }

    pub fn optional_space(&mut self) -> Result<(), ParseError> {
        trace!(
            self.log,
            "Looking for optional space";
            "token" => self.extracted.token,
        );

        if self.extracted.token == Token::Whitespace {
            self.step()?;
        }

        Ok(())
    }
}

/// Define a rule for how to parse a block.
#[derive(Clone)]
pub struct BlockRule {
    /// The code name of the block. Must be kebab-case and globally unique.
    name: &'static str,

    /// Which names you can use this block with. Case-insensitive.
    /// Will panic if empty.
    accepts_names: &'static [&'static str],

    /// Whether this block accepts `*` as a modifier.
    ///
    /// For instance, user can be invoked as both
    /// `[[user aismallard]]` and `[[*user aismallard]]`.
    accepts_special: bool,

    /// Function which implements the processing for this rule.
    parse_fn: BlockParseFn,
}

pub type BlockParseFn = for<'l, 'r, 't> fn(
    &'l slog::Logger,
    &mut BlockParser<'l, 'r, 't>,
) -> Consumption<'r, 't>;

#[derive(Clone)]
pub struct BlockRuleOld {
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
