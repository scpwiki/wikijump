/*
 * parsing/rule/impls/block/blocks/mod.rs
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

mod prelude {
    pub use super::super::{Arguments, BlockRule};
    pub use crate::log::prelude::*;
    pub use crate::parsing::collect::*;
    pub use crate::parsing::condition::ParseCondition;
    pub use crate::parsing::parser::Parser;
    pub use crate::parsing::prelude::*;
    pub use crate::parsing::{ParseWarning, Token};
    pub use crate::tree::{
        Container, ContainerType, Element, PartialElement, PartialElements,
    };

    #[cfg(debug)]
    pub fn assert_generic_name(
        expected_names: &[&str],
        actual_name: &str,
        name_type: &str,
    ) {
        for name in expected_names {
            if name.eq_ignore_ascii_case(actual_name) {
                return;
            }
        }

        panic!(
            "Actual {} name doesn't match any expected: {:?} (was {})",
            name_type, expected_names, actual_name,
        );
    }

    #[cfg(not(debug))]
    #[inline]
    pub fn assert_generic_name(_: &[&str], _: &str, _: &str) {}

    #[inline]
    pub fn assert_block_name(block_rule: &BlockRule, actual_name: &str) {
        assert_generic_name(block_rule.accepts_names, actual_name, "block")
    }
}

#[macro_use]
mod align;

mod align_center;
mod align_justify;
mod align_left;
mod align_right;
mod anchor;
mod blockquote;
mod bold;
mod char;
mod checkbox;
mod code;
mod collapsible;
mod css;
mod del;
mod div;
mod footnote;
mod hidden;
mod html;
mod ifcategory;
mod iframe;
mod iftags;
mod image;
mod include_elements;
mod include_messy;
mod ins;
mod invisible;
mod italics;
mod later;
mod lines;
mod list;
mod mark;
mod module;
mod monospace;
mod paragraph;
mod radio;
mod size;
mod span;
mod strikethrough;
mod subscript;
mod superscript;
mod table;
mod toc;
mod underline;
mod user;

pub use self::align_center::BLOCK_ALIGN_CENTER;
pub use self::align_justify::BLOCK_ALIGN_JUSTIFY;
pub use self::align_left::BLOCK_ALIGN_LEFT;
pub use self::align_right::BLOCK_ALIGN_RIGHT;
pub use self::anchor::BLOCK_ANCHOR;
pub use self::blockquote::BLOCK_BLOCKQUOTE;
pub use self::bold::BLOCK_BOLD;
pub use self::char::BLOCK_CHAR;
pub use self::checkbox::BLOCK_CHECKBOX;
pub use self::code::BLOCK_CODE;
pub use self::collapsible::BLOCK_COLLAPSIBLE;
pub use self::css::BLOCK_CSS;
pub use self::del::BLOCK_DEL;
pub use self::div::BLOCK_DIV;
pub use self::footnote::{BLOCK_FOOTNOTE, BLOCK_FOOTNOTE_BLOCK};
pub use self::hidden::BLOCK_HIDDEN;
pub use self::html::BLOCK_HTML;
pub use self::ifcategory::BLOCK_IFCATEGORY;
pub use self::iframe::BLOCK_IFRAME;
pub use self::iftags::BLOCK_IFTAGS;
pub use self::image::BLOCK_IMAGE;
pub use self::include_elements::BLOCK_INCLUDE_ELEMENTS;
pub use self::include_messy::BLOCK_INCLUDE_MESSY;
pub use self::ins::BLOCK_INS;
pub use self::invisible::BLOCK_INVISIBLE;
pub use self::italics::BLOCK_ITALICS;
pub use self::later::BLOCK_LATER;
pub use self::lines::BLOCK_LINES;
pub use self::list::*;
pub use self::mark::BLOCK_MARK;
pub use self::module::BLOCK_MODULE;
pub use self::monospace::BLOCK_MONOSPACE;
pub use self::paragraph::BLOCK_PARAGRAPH;
pub use self::radio::BLOCK_RADIO;
pub use self::size::BLOCK_SIZE;
pub use self::span::BLOCK_SPAN;
pub use self::strikethrough::BLOCK_STRIKETHROUGH;
pub use self::subscript::BLOCK_SUBSCRIPT;
pub use self::superscript::BLOCK_SUPERSCRIPT;
pub use self::table::{
    BLOCK_TABLE, BLOCK_TABLE_CELL_HEADER, BLOCK_TABLE_CELL_REGULAR, BLOCK_TABLE_ROW,
};
pub use self::toc::BLOCK_TABLE_OF_CONTENTS;
pub use self::underline::BLOCK_UNDERLINE;
pub use self::user::BLOCK_USER;
