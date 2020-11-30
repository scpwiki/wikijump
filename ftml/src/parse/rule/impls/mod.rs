/*
 * parse/rule/impls/mod.rs
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

mod prelude {
    pub use crate::parse::consume::consume;
    pub use crate::parse::error::{ParseError, ParseErrorKind};
    pub use crate::parse::rule::{Consumption, ConsumptionResult, Rule, TryConsumeFn};
    pub use crate::parse::token::{ExtractedToken, Token};
    pub use crate::tree::{Container, ContainerType, Element};
}

// TODO add remaining rules

mod bold;
mod email;
mod fallback;
mod null;
mod text;
mod url;

pub use self::bold::RULE_BOLD;
pub use self::email::RULE_EMAIL;
pub use self::fallback::RULE_FALLBACK;
pub use self::null::RULE_NULL;
pub use self::text::RULE_TEXT;
pub use self::url::RULE_URL;
