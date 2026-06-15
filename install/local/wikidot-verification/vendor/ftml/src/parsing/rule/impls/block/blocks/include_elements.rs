/*
 * parsing/rule/impls/block/blocks/include_elements.rs
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
use crate::data::PageRef;
use crate::parsing::UnstructuredParseResult;

// TODO: maybe scrap this? we want to move to components anyways

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
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Found invalid include-elements block");
    parser.check_page_syntax()?;
    assert!(!flag_star, "Include (elements) doesn't allow star flag");
    assert!(!flag_score, "Include (elements) doesn't allow score flag");
    assert_block_name(&BLOCK_INCLUDE_ELEMENTS, name);

    // Parse block
    let (page_name, variables) =
        parser.get_head_name_map(&BLOCK_INCLUDE_ELEMENTS, in_head)?;

    let page_ref = match PageRef::parse(page_name) {
        Ok(page_ref) => page_ref,
        Err(_) => return Err(parser.make_err(ParseErrorKind::BlockMalformedArguments)),
    };

    // Get page to be included
    let UnstructuredParseResult {
        result,
        mut html_blocks,
        mut code_blocks,
        mut table_of_contents_depths,
        mut footnotes,
        has_footnote_block,
        mut bibliographies,
    } = include_page(parser, &page_ref)?;

    if has_footnote_block {
        parser.set_footnote_block();
    }

    // Extract elements and errors
    let ParseSuccess {
        item: elements,
        errors,
        paragraph_safe,
        ..
    } = result?;

    // Update parser state, build, and return
    parser.append_shared_items(
        &mut html_blocks,
        &mut code_blocks,
        &mut table_of_contents_depths,
        &mut footnotes,
        &mut bibliographies,
    );

    let variables = variables.to_hash_map();
    let element = Element::Include {
        paragraph_safe,
        variables,
        location: page_ref,
        elements,
    };

    ok!(element, errors)
}

fn include_page<'r, 't>(
    parser: &Parser<'r, 't>,
    _page: &PageRef,
) -> Result<UnstructuredParseResult<'r, 't>, ParseError> {
    // TODO stubbed

    if false {
        return Err(parser.make_err(ParseErrorKind::NoSuchPage));
    }

    Ok(UnstructuredParseResult {
        result: Ok(ParseSuccess::new(
            vec![text!("<INCLUDED PAGE (ELEMENTS)>")],
            vec![],
            false,
        )),
        html_blocks: vec![],
        code_blocks: vec![],
        table_of_contents_depths: vec![],
        footnotes: vec![],
        has_footnote_block: false,
        bibliographies: Default::default(),
    })
}
