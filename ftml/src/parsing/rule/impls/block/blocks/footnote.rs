/*
 * parsing/rule/impls/block/blocks/footnote.rs
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

pub const BLOCK_FOOTNOTE: BlockRule = BlockRule {
    name: "block-footnote",
    accepts_names: &["footnote"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: false,
    parse_fn: parse_footnote_ref,
};

pub const BLOCK_FOOTNOTE_BLOCK: BlockRule = BlockRule {
    name: "block-footnote-block",
    accepts_names: &["footnoteblock"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: true,
    parse_fn: parse_footnote_block,
};

fn parse_footnote_ref<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(
        log,
        "Parsing footnote ref block";
        "in-head" => in_head,
    );

    assert!(!flag_star, "Footnote reference doesn't allow star flag");
    assert!(!flag_score, "Footnote reference doesn't allow score flag");
    assert_block_name(&BLOCK_FOOTNOTE, name);

    parser.get_head_none(&BLOCK_FOOTNOTE, in_head)?;

    // Gather footnote contents with paragraphs.
    //
    // However, if there's only one, then we strip it
    // and make it inline.
    let (mut elements, exceptions, _) =
        parser.get_body_elements(&BLOCK_FOOTNOTE, true)?.into();

    if elements.len() == 1 {
        match elements.pop().unwrap() {
            // Unwrap the paragraph and get its contents.
            Element::Container(container) => {
                let mut new_elements: Vec<Element> = container.into();
                elements.append(&mut new_elements);
            }

            // Other element, keep as-is.
            element => elements.push(element),
        };
    }

    // Append footnote contents and return.
    parser.push_footnote(elements);

    ok!(Element::Footnote, exceptions)
}

fn parse_footnote_block<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!(
        log,
        "Parsing footnote list block";
        "in-head" => in_head,
    );

    assert!(!flag_star, "Footnote block doesn't allow star flag");
    assert!(!flag_score, "Footnote block doesn't allow score flag");
    assert_block_name(&BLOCK_FOOTNOTE_BLOCK, name);

    let mut arguments = parser.get_head_map(&BLOCK_FOOTNOTE_BLOCK, in_head)?;

    let title = arguments.get("title");
    let hide = arguments.get_bool(parser, "hide")?.unwrap_or(false);

    // Build and return
    ok!(Element::FootnoteBlock { title, hide })
}
