/*
 * parse/rule/mod.rs
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

use super::prelude::*;
use crate::parse::Parser;
use std::fmt::{self, Debug};

mod collect;
mod mapping;

pub mod impls;

pub use self::mapping::{rules_for_token, RULE_MAP};

/// Defines a rule that can possibly match tokens and return an `Element`.
#[derive(Copy, Clone)]
pub struct Rule {
    /// The name for this rule, in kebab-case.
    ///
    /// It is globally unique.
    name: &'static str,

    /// The consumption attempt function for this rule.
    try_consume_fn: TryConsumeFn,
}

impl Rule {
    #[inline]
    pub fn name(self) -> &'static str {
        self.name
    }

    #[inline]
    pub fn try_consume<'p, 'l, 'r, 't>(
        self,
        log: &'l slog::Logger,
        parser: &'p mut Parser<'l, 'r, 't>,
    ) -> ParseResult<'r, 't, Element<'t>> {
        info!(log, "Trying to consume for parse rule"; "name" => self.name);

        (self.try_consume_fn)(log, parser)
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

/// The function type for actually trying to consume tokens
pub type TryConsumeFn = for<'p, 'l, 'r, 't> fn(
    log: &'l slog::Logger,
    parser: &'lp mut Parser<'l, 'r, 't>,
) -> ParseResult<'r, 't, Element<'t>>;
