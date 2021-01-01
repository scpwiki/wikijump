/*
 * parse/rule/impls/block/impls/div.rs
 *
 * ftml - Library to parse Wikidot text
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

// TODO while refactoring Parser
#![allow(unused_variables)]

use super::prelude::*;

pub const BLOCK_DIV: BlockRule = BlockRule {
    name: "block-div",
    accepts_names: &["div", "div_"],
    accepts_special: false,
    parse_fn,
};

fn parse_fn<'l, 'p, 'r, 't>(
    log: &'l slog::Logger,
    parser: &'p mut BlockParser<'l, 'p, 'r, 't>,
    name: &'t str,
    special: bool,
) -> ParseResult<'r, 't, Element<'t>> {
    assert_eq!(special, false, "Code doesn't allow special variant");

    let mut arguments = parser.get_argument_map()?;
    parser.get_line_break()?;

    // "div" means we wrap in paragraphs, like normal
    // "div_" means we don't wrap it
    let wrap_paragraphs = !name.ends_with('_');

    // Get styling arguments
    let id = arguments.get("id");
    let class = arguments.get("class");
    let style = arguments.get("style");

    todo!()

    //// Gather elements for div contents
    //let (elements, exceptions) = {
    //    if wrap_paragraphs {
    //        let consumption = try_paragraphs(
    //            log,
    //            parser.state(),
    //            BLOCK_DIV.rule(),
    //            &[Token::LeftBlockEnd], // TODO this is insufficient ugh
    //            &[],
    //        );

    //        todo!()
    //    } else {
    //        todo!()
    //    }
    //};

    //// Build element and return
    //let element = Element::Div {
    //    elements,
    //    id,
    //    class,
    //    style,
    //};

    //ok!(element, parser.remaining(), exceptions)
}
