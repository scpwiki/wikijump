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

use super::token::ExtractedToken;
use crate::tree::Element;
use std::fmt::{self, Debug};

#[derive(Copy, Clone)]
pub struct Rule {
    name: &'static str,
    try_consume_fn: TryConsumeFn,
}

impl Rule {
    #[inline]
    pub fn try_consume<'a>(
        &self,
        log: &slog::Logger,
        extract: &ExtractedToken<'a>,
        next: &[ExtractedToken<'a>],
    ) -> Option<RuleResult<'a>> {
        info!(log, "Trying to consume for parse rule '{}'", self.name);

        (self.try_consume_fn)(log, extract, next)
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

#[derive(Debug, Clone)]
pub struct RuleResult<'a> {
    pub offset: usize,
    pub element: Element<'a>,
}

pub type TryConsumeFn = for<'a> fn(
    log: &slog::Logger,
    extract: &ExtractedToken<'a>,
    next: &[ExtractedToken<'a>],
) -> Option<RuleResult<'a>>;
