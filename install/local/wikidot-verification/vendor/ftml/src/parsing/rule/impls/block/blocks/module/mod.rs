/*
 * parsing/rule/impls/block/blocks/module/mod.rs
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

mod mapping;
mod modules;
mod output;
mod parser;
mod rule;

use super::prelude;
use crate::parsing::rule::impls::block::Arguments;
use crate::parsing::rule::{LineRequirement, Rule};
use crate::parsing::{ParseResult, Parser};
use crate::tree::Elements;
use std::fmt::{self, Debug};

pub use self::output::ModuleParseOutput;
pub use self::rule::BLOCK_MODULE;

/// Define a rule for how to parse a module.
#[derive(Clone)]
pub struct ModuleRule {
    /// The code name of the module.
    ///
    /// As this is an internal structure, we can assert the following things:
    /// * It is in kebab-case.
    /// * It is globally unique.
    /// * It is prefixed with `module-`.
    name: &'static str,

    /// Which names you can use this module with. Case-insensitive.
    /// Will panic if empty.
    accepts_names: &'static [&'static str],

    /// Function which implements the processing for this rule.
    parse_fn: ModuleParseFn,
}

impl ModuleRule {
    /// Produces a pseudo parse `Rule` associated with this `BlockRule`.
    ///
    /// It should not be invoked, it is for error construction.
    #[cold]
    pub fn rule(&self) -> Rule {
        // Stubbed try_consume_fn implementation for the Rule.
        fn try_consume_fn<'r, 't>(
            _: &mut Parser<'r, 't>,
        ) -> ParseResult<'r, 't, Elements<'t>> {
            panic!("Pseudo rule for this module should not be executed directly!");
        }

        Rule {
            name: self.name,
            position: LineRequirement::Any,
            try_consume_fn,
        }
    }
}

impl Debug for ModuleRule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ModuleRule")
            .field("name", &self.name)
            .field("accepts_names", &self.accepts_names)
            .field("parse_fn", &(self.parse_fn as *const ()))
            .finish()
    }
}

/// Function pointer type to implement module parsing.
///
/// The arguments are, in order:
/// * `parser` -- `Parser` instance
/// * `name` -- The name of this module
/// * `arguments` -- The arguments passed into the module
pub type ModuleParseFn = for<'r, 't> fn(
    &mut Parser<'r, 't>,
    &'t str,
    Arguments<'t>,
) -> ParseResult<'r, 't, ModuleParseOutput<'t>>;
