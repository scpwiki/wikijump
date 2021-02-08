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
    pub use crate::parsing::collect::*;
    pub use crate::parsing::condition::ParseCondition;
    pub use crate::parsing::parser::Parser;
    pub use crate::parsing::prelude::*;
    pub use crate::parsing::{ParseWarning, Token};
    pub use crate::tree::{
        Container, ContainerType, Element, StyledContainer, StyledContainerType,
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

mod anchor;
mod checkbox;
mod code;
mod collapsible;
mod css;
mod del;
mod div;
mod hidden;
mod html;
mod iframe;
mod include;
mod ins;
mod invisible;
mod lines;
mod mark;
mod module;
mod radio;
mod span;

pub use self::anchor::BLOCK_ANCHOR;
pub use self::checkbox::BLOCK_CHECKBOX;
pub use self::code::BLOCK_CODE;
pub use self::collapsible::BLOCK_COLLAPSIBLE;
pub use self::css::BLOCK_CSS;
pub use self::del::BLOCK_DEL;
pub use self::div::BLOCK_DIV;
pub use self::hidden::BLOCK_HIDDEN;
pub use self::html::BLOCK_HTML;
pub use self::iframe::BLOCK_IFRAME;
pub use self::include::BLOCK_INCLUDE;
pub use self::ins::BLOCK_INS;
pub use self::invisible::BLOCK_INVISIBLE;
pub use self::lines::BLOCK_LINES;
pub use self::mark::BLOCK_MARK;
pub use self::module::BLOCK_MODULE;
pub use self::radio::BLOCK_RADIO;
pub use self::span::BLOCK_SPAN;
