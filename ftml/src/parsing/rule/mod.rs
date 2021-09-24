/*
 * parsing/rule/mod.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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

use super::prelude::*;
use crate::parsing::Parser;
use std::fmt::{self, Debug};

mod mapping;

pub mod impls;

pub use self::mapping::{get_rules_for_token, RULE_MAP};

/// Defines a rule that can possibly match tokens and return an `Element`.
#[derive(Copy, Clone)]
pub struct Rule {
    /// The name for this rule, in kebab-case.
    ///
    /// It must be globally unique.
    name: &'static str,

    /// What requirements this rule needs regarding its position in a line.
    position: LineRequirement,

    /// The consumption attempt function for this rule.
    try_consume_fn: TryConsumeFn,
}

impl Rule {
    #[inline]
    pub fn name(self) -> &'static str {
        self.name
    }

    #[inline]
    pub fn try_consume<'p, 'r, 't>(
        self,
        log: &Logger,
        parser: &'p mut Parser<'r, 't>,
    ) -> ParseResult<'r, 't, Elements<'t>> {
        info!(log, "Trying to consume for parse rule"; "name" => self.name);

        // Check that the line position matches what the rule wants.
        match self.position {
            LineRequirement::Any => (),
            LineRequirement::StartOfLine => {
                if !parser.start_of_line() {
                    return Err(parser.make_warn(ParseWarningKind::NotStartOfLine));
                }
            }
        }

        // Fork parser and try running the rule.
        let mut sub_parser = parser.clone_with_rule(self);
        let result = (self.try_consume_fn)(log, &mut sub_parser);

        // We only keep the parser state if it succeeded.
        if let Ok(ref output) = result {
            output.check_partials(parser)?;

            // But first, ensure there aren't any partial elements in the result.
            parser.update(&sub_parser);
        }

        result
    }
}

impl Debug for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Rule")
            .field("name", &self.name)
            .field("try_consume_fn", &(self.try_consume_fn as *const ()))
            .finish()
    }
}

#[cfg(feature = "log")]
impl slog::Value for Rule {
    fn serialize(
        &self,
        _: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        serializer.emit_str(key, self.name())
    }
}

/// The enum describing what requirements a rule has regarding lines.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum LineRequirement {
    /// This rule does not care where it is in a line.
    Any,

    /// This rule may only activate when it is at the start of a line.
    ///
    /// This includes situations which are not technically line breaks,
    /// such as start of input and paragraph breaks.
    StartOfLine,
}

/// The function type for actually trying to consume tokens
pub type TryConsumeFn = for<'p, 'r, 't> fn(
    log: &Logger,
    parser: &'p mut Parser<'r, 't>,
) -> ParseResult<'r, 't, Elements<'t>>;
