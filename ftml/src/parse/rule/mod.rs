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
use crate::text::FullText;
use crate::tree::Element;
use std::fmt::{self, Debug};

mod collect;
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
        full_text: FullText<'t>,
    ) -> Consumption<'r, 't> {
        info!(log, "Trying to consume for parse rule"; "name" => self.name);

        (self.try_consume_fn)(log, extract, remaining, full_text)
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

#[derive(Debug, Clone)]
pub struct GenericConsumption<'t, 'r, T> {
    pub result: GenericConsumptionResult<'t, 'r, T>,
    pub error: Option<ParseError>,
}

impl<'t, 'r, T> GenericConsumption<'t, 'r, T>
where
    T: 't,
{
    #[inline]
    pub fn ok(item: T, remaining: &'r [ExtractedToken<'t>]) -> Self {
        GenericConsumption {
            result: GenericConsumptionResult::Success { item, remaining },
            error: None,
        }
    }

    #[inline]
    pub fn warn(item: T, remaining: &'r [ExtractedToken<'t>], error: ParseError) -> Self {
        GenericConsumption {
            result: GenericConsumptionResult::Success { item, remaining },
            error: Some(error),
        }
    }

    #[inline]
    pub fn err(error: ParseError) -> Self {
        GenericConsumption {
            result: GenericConsumptionResult::Failure,
            error: Some(error),
        }
    }

    #[inline]
    pub fn is_success(&self) -> bool {
        match self.result {
            GenericConsumptionResult::Success { .. } => true,
            GenericConsumptionResult::Failure => false,
        }
    }

    #[inline]
    pub fn is_error(&self) -> bool {
        !self.is_success()
    }

    pub fn map<F, U>(self, f: F) -> GenericConsumption<'t, 'r, U>
    where
        F: FnOnce(T) -> U,
    {
        let GenericConsumption { result, error } = self;
        let result = match result {
            GenericConsumptionResult::Success { item, remaining } => {
                GenericConsumptionResult::Success {
                    item: f(item),
                    remaining,
                }
            }
            GenericConsumptionResult::Failure => GenericConsumptionResult::Failure,
        };

        GenericConsumption { result, error }
    }
}

#[derive(Debug, Clone)]
pub enum GenericConsumptionResult<'t, 'r, T>
where
    T: 't,
    'r: 't,
{
    Success {
        item: T,
        remaining: &'r [ExtractedToken<'t>],
    },
    Failure,
}

pub type Consumption<'t, 'r> = GenericConsumption<'t, 'r, Element<'t>>;
pub type ConsumptionResult<'t, 'r> = GenericConsumptionResult<'t, 'r, Element<'t>>;

/// The function type for actually trying to consume tokens
pub type TryConsumeFn = for<'t, 'r> fn(
    log: &slog::Logger,
    extracted: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,
) -> Consumption<'t, 'r>;
