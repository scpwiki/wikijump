/*
 * parse/rule/impls/mod.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Ammon Smith
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
    pub use crate::parse::check_step::check_step;
    pub use crate::parse::collect::*;
    pub use crate::parse::condition::ParseCondition;
    pub use crate::parse::consume::consume;
    pub use crate::parse::exception::{ParseException, ParseWarning, ParseWarningKind};
    pub use crate::parse::parser::Parser;
    pub use crate::parse::result::{ParseResult, ParseSuccess};
    pub use crate::parse::rule::Rule;
    pub use crate::parse::token::{ExtractedToken, Token};
    pub use crate::text::FullText;
    pub use crate::tree::{Container, ContainerType, Element};
}

mod block;
mod color;
mod comment;
mod dash;
mod email;
mod emphasis;
mod fallback;
mod horizontal_rule;
mod line_break;
mod link_anchor;
mod link_single;
mod link_triple;
mod monospace;
mod null;
mod page;
mod raw;
mod strikethrough;
mod strong;
mod subscript;
mod superscript;
mod text;
mod todo;
mod underline;
mod url;

pub use self::block::{RULE_BLOCK, RULE_BLOCK_SKIP, RULE_BLOCK_SPECIAL};
pub use self::color::RULE_COLOR;
pub use self::comment::RULE_COMMENT;
pub use self::dash::RULE_DASH;
pub use self::email::RULE_EMAIL;
pub use self::emphasis::RULE_EMPHASIS;
pub use self::fallback::RULE_FALLBACK;
pub use self::horizontal_rule::RULE_HORIZONTAL_RULE;
pub use self::line_break::{RULE_LINE_BREAK, RULE_LINE_BREAK_PARAGRAPH};
pub use self::link_anchor::RULE_LINK_ANCHOR;
pub use self::link_single::{RULE_LINK_SINGLE, RULE_LINK_SINGLE_NEW_TAB};
pub use self::link_triple::{RULE_LINK_TRIPLE, RULE_LINK_TRIPLE_NEW_TAB};
pub use self::monospace::RULE_MONOSPACE;
pub use self::null::RULE_NULL;
pub use self::page::RULE_PAGE;
pub use self::raw::RULE_RAW;
pub use self::strikethrough::RULE_STRIKETHROUGH;
pub use self::strong::RULE_STRONG;
pub use self::subscript::RULE_SUBSCRIPT;
pub use self::superscript::RULE_SUPERSCRIPT;
pub use self::text::RULE_TEXT;
pub use self::todo::RULE_TODO;
pub use self::underline::RULE_UNDERLINE;
pub use self::url::RULE_URL;
