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

// Private preludes for modules
macro_rules! make_rule {
    ($name:expr, $try_consume:expr) => {
        Rule {
            name: $name,
            try_consume_fn: $try_consume,
        }
    };
}

mod prelude {
    pub use crate::parse::rule::{Rule, RuleResult, TryConsumeFn};
    pub use crate::parse::token::{ExtractedToken, Token};
    pub use crate::tree::Element;
}

// Implementations, re-exported
mod em_dash;
mod email;
mod hr;
mod newline;
mod null;
mod paragraph;
mod strikethrough;
mod text;
mod url;

pub use self::em_dash::RULE_EM_DASH;
pub use self::email::RULE_EMAIL;
pub use self::hr::RULE_HORIZONTAL_RULE;
pub use self::newline::RULE_LINE_BREAK;
pub use self::null::RULE_NULL;
pub use self::paragraph::RULE_PARAGRAPH_BREAK;
pub use self::strikethrough::RULE_STRIKETHROUGH;
pub use self::text::RULE_TEXT;
pub use self::url::RULE_URL;
