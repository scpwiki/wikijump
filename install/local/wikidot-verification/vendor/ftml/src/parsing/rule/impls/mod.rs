/*
 * parsing/rule/impls/mod.rs
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

mod prelude {
    pub use crate::parsing::check_step::assert_step;
    pub use crate::parsing::collect::*;
    pub use crate::parsing::condition::ParseCondition;
    pub use crate::parsing::consume::consume;
    pub use crate::parsing::error::{ParseError, ParseErrorKind};
    pub use crate::parsing::parser::Parser;
    pub use crate::parsing::result::ParseResult;
    pub use crate::parsing::rule::{LineRequirement, Rule};
    pub use crate::parsing::token::{ExtractedToken, Token};
    pub use crate::tree::{AttributeMap, ContainerType, Element, Elements};
}

mod anchor;
mod bibcite;
mod block;
mod blockquote;
mod bold;
mod center;
mod clear_float;
mod color;
mod comment;
mod dash;
mod definition_list;
mod double_angle;
mod email;
mod fallback;
mod header;
mod horizontal_rule;
mod italics;
mod line_break;
mod link_anchor;
mod link_single;
mod link_triple;
mod list;
mod math;
mod monospace;
mod null;
mod page;
mod raw;
mod strikethrough;
mod subscript;
mod superscript;
mod table;
mod text;
mod underline;
mod underscore_line_break;
mod url;
mod variable;

pub use self::anchor::RULE_ANCHOR;
pub use self::bibcite::RULE_BIBCITE;
pub use self::block::{RULE_BLOCK, RULE_BLOCK_SKIP_NEWLINE, RULE_BLOCK_STAR};
pub use self::blockquote::RULE_BLOCKQUOTE;
pub use self::bold::RULE_BOLD;
pub use self::center::RULE_CENTER;
pub use self::clear_float::RULE_CLEAR_FLOAT;
pub use self::color::RULE_COLOR;
pub use self::comment::RULE_COMMENT;
pub use self::dash::RULE_DASH;
pub use self::definition_list::{
    RULE_DEFINITION_LIST, RULE_DEFINITION_LIST_SKIP_NEWLINE,
};
pub use self::double_angle::RULE_DOUBLE_ANGLE;
pub use self::email::RULE_EMAIL;
pub use self::fallback::RULE_FALLBACK;
pub use self::header::RULE_HEADER;
pub use self::horizontal_rule::RULE_HORIZONTAL_RULE;
pub use self::italics::RULE_ITALICS;
pub use self::line_break::{RULE_LINE_BREAK, RULE_LINE_BREAK_PARAGRAPH};
pub use self::link_anchor::RULE_LINK_ANCHOR;
pub use self::link_single::{RULE_LINK_SINGLE, RULE_LINK_SINGLE_NEW_TAB};
pub use self::link_triple::{RULE_LINK_TRIPLE, RULE_LINK_TRIPLE_NEW_TAB};
pub use self::list::RULE_LIST;
pub use self::math::RULE_MATH;
pub use self::monospace::RULE_MONOSPACE;
pub use self::null::RULE_NULL;
pub use self::page::RULE_PAGE;
pub use self::raw::RULE_RAW;
pub use self::strikethrough::{RULE_STRIKETHROUGH_DASH, RULE_STRIKETHROUGH_TILDE};
pub use self::subscript::RULE_SUBSCRIPT;
pub use self::superscript::RULE_SUPERSCRIPT;
pub use self::table::RULE_TABLE;
pub use self::text::RULE_TEXT;
pub use self::underline::RULE_UNDERLINE;
pub use self::underscore_line_break::RULE_UNDERSCORE_LINE_BREAK;
pub use self::url::RULE_URL;
pub use self::variable::RULE_VARIABLE;
