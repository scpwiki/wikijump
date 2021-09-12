/*
 * parsing/rule/impls/block/blocks/include_elements.rs
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

use super::prelude::*;
use crate::data::PageRef;
use crate::tree::SyntaxTree;

/// Block rule for include (elements).
///
/// This takes the resultant `SyntaxTree` from another page and
/// inserts them into this page being built.
pub const BLOCK_INCLUDE_ELEMENTS: BlockRule = BlockRule {
    name: "block-include-elements",
    accepts_names: &["include-elements"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: true,
    parse_fn,
};

fn parse_fn<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(log, "Found invalid include-elements block");

    assert!(!flag_star, "Include (elements) doesn't allow star flag");
    assert!(!flag_score, "Include (elements) doesn't allow score flag");
    assert_block_name(&BLOCK_INCLUDE_ELEMENTS, name);

    // Parse block
    let (page_name, arguments) =
        parser.get_head_name_map(&BLOCK_INCLUDE_ELEMENTS, in_head)?;

    let page_ref = match PageRef::parse(page_name) {
        Ok(page_ref) => page_ref,
        Err(_) => return Err(parser.make_warn(ParseWarningKind::BlockMalformedArguments)),
    };

    // Get page to be included
    let SyntaxTree {
        elements,
        styles,
        mut table_of_contents,
        mut footnotes,
    } = include_page(parser, page_ref)?;

    // Add gathered items and return
    parser.append_toc_and_footnotes(&mut Vec::new(), &mut footnotes);

    let exceptions = styles
        .into_iter() //
        .map(ParseException::Style)
        .collect();

    ok!(elements, exceptions)
}

fn include_page(
    parser: &Parser,
    _page: PageRef,
) -> Result<SyntaxTree<'static>, ParseWarning> {
    // TODO stubbed

    if false {
        return Err(parser.make_warn(ParseWarningKind::NoSuchPage));
    }

    Ok(SyntaxTree {
        elements: vec![],
        styles: vec![],
        table_of_contents: vec![],
        footnotes: vec![],
    })
}
