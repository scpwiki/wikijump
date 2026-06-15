/*
 * parsing/rule/impls/block/blocks/tabs.rs
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

use super::prelude::*;
use crate::parsing::ParserWrap;
use crate::tree::{AcceptsPartial, PartialElement, Tab};

pub const BLOCK_TABVIEW: BlockRule = BlockRule {
    name: "block-tabview",
    accepts_names: &["tabview", "tabs"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: true,
    parse_fn: parse_tabview,
};

pub const BLOCK_TAB: BlockRule = BlockRule {
    name: "block-tab",
    accepts_names: &["tab"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: true,
    parse_fn: parse_tab,
};

fn parse_tabview<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    let parser = &mut ParserWrap::new(parser, AcceptsPartial::Tab);

    debug!("Parsing tabview block (name '{name}', in-head {in_head})");
    assert!(!flag_star, "Tabview doesn't allow star flag");
    assert!(!flag_score, "Tabview doesn't allow score flag");
    assert_block_name(&BLOCK_TABVIEW, name);

    parser.get_head_none(&BLOCK_TABVIEW, in_head)?;

    let (elements, errors, _) = parser.get_body_elements(&BLOCK_TABVIEW, false)?.into();

    // Build element and return
    let mut tabs = Vec::new();

    for element in elements {
        match element {
            // Append the next tab item.
            Element::Partial(PartialElement::Tab(tab)) => tabs.push(tab),

            // Ignore internal whitespace.
            element if element.is_whitespace() => (),

            // Return an error for anything else.
            _ => return Err(parser.make_err(ParseErrorKind::TabViewContainsNonTab)),
        }
    }

    // Ensure it's not empty
    if tabs.is_empty() {
        return Err(parser.make_err(ParseErrorKind::TabViewEmpty));
    }

    ok!(false; Element::TabView(tabs), errors)
}

fn parse_tab<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Parsing tab block (name '{name}', in-head {in_head})");
    assert!(!flag_star, "Tab doesn't allow star flag");
    assert!(!flag_score, "Tab doesn't allow score flag");
    assert_block_name(&BLOCK_TAB, name);

    let label =
        parser.get_head_value(&BLOCK_TAB, in_head, |parser, value| match value {
            Some(name) => Ok(name),
            None => Err(parser.make_err(ParseErrorKind::BlockMissingArguments)),
        })?;

    let (elements, errors, _) = parser.get_body_elements(&BLOCK_TAB, true)?.into();

    // Build element and return
    let element = Element::Partial(PartialElement::Tab(Tab {
        label: cow!(label),
        elements,
    }));

    ok!(false; element, errors)
}
