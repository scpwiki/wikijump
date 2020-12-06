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
    pub use crate::parse::rule::collect::*;
    pub use crate::parse::rule::{
        Consumption, ConsumptionResult, GenericConsumption, GenericConsumptionResult, Rule,
        TryConsumeFn,
    };
    pub use crate::parse::token::{ExtractedToken, Token};
    pub use crate::text::FullText;
    pub use crate::tree::{Container, ContainerType, Element};
}

mod bold;
mod color;
mod comment;
mod email;
mod fallback;
mod italics;
mod line_break;
mod null;
mod raw;
mod text;
mod todo;
mod url;

pub use self::bold::RULE_BOLD;
pub use self::color::RULE_COLOR;
pub use self::comment::RULE_COMMENT;
pub use self::email::RULE_EMAIL;
pub use self::fallback::RULE_FALLBACK;
pub use self::italics::RULE_ITALICS;
pub use self::line_break::RULE_LINE_BREAK;
pub use self::null::RULE_NULL;
pub use self::raw::RULE_RAW;
pub use self::text::RULE_TEXT;
pub use self::todo::RULE_TODO;
pub use self::url::RULE_URL;
