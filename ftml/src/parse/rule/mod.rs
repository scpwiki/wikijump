/*
 * parse/rule/mod.rs
 *
 * ftml - Library to parse Wikidot code
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

use super::ParseError;
use crate::parse::token::ExtractedToken;
use crate::tree::Element;
use self::container::try_container;
use std::fmt::{self, Debug};

mod container;
mod mapping;

pub mod impls;

pub use self::mapping::{rules_for_token, RULE_MAP};

/// Defines a rule that can possibly match tokens and return an `Element`.
#[derive(Copy, Clone)]
pub struct Rule {
    name: &'static str,
    try_consume_fn: TryConsumeFn,
}

impl Rule {
    #[inline]
    pub fn name(self) -> &'static str {
        self.name
    }

    #[inline]
    pub fn try_consume<'t, 'r>(
        self,
        log: &slog::Logger,
        extract: &'r ExtractedToken<'t>,
        remaining: &'r [ExtractedToken<'t>],
    ) -> Consumption<'r, 't> {
        info!(log, "Trying to consume for parse rule '{}'", self.name);

        (self.try_consume_fn)(log, extract, remaining)
    }
}

impl Debug for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Rule")
            .field("name", &self.name)
            .field("try_consume_fn", &"<fn pointer>")
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

/// Result of attempting to consume tokens in a parse rule.
#[derive(Debug, Clone)]
pub struct Consumption<'t, 'r> {
    pub result: ConsumptionResult<'t, 'r>,
    pub error: Option<ParseError>,
}

impl<'t, 'r> Consumption<'t, 'r> {
    #[inline]
    pub fn ok(element: Element<'t>, remaining: &'r [ExtractedToken<'t>]) -> Self {
        Consumption {
            result: ConsumptionResult::Success { element, remaining },
            error: None,
        }
    }

    #[inline]
    pub fn warn(
        element: Element<'t>,
        remaining: &'r [ExtractedToken<'t>],
        error: ParseError,
    ) -> Self {
        Consumption {
            result: ConsumptionResult::Success { element, remaining },
            error: Some(error),
        }
    }

    #[inline]
    pub fn err(error: ParseError) -> Self {
        Consumption {
            result: ConsumptionResult::Failure,
            error: Some(error),
        }
    }

    #[inline]
    pub fn is_success(&self) -> bool {
        match self.result {
            ConsumptionResult::Success { .. } => true,
            ConsumptionResult::Failure => false,
        }
    }

    #[inline]
    pub fn is_error(&self) -> bool {
        !self.is_success()
    }
}

#[derive(Debug, Clone)]
pub enum ConsumptionResult<'t, 'r>
where
    'r: 't,
{
    Success {
        element: Element<'t>,
        remaining: &'r [ExtractedToken<'t>],
    },
    Failure,
}

/// The function type for actually trying to consume tokens
pub type TryConsumeFn = for<'t, 'r> fn(
    log: &slog::Logger,
    extract: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
) -> Consumption<'t, 'r>;
