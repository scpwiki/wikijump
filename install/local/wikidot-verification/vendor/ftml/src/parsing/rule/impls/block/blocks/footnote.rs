/*
 * parsing/rule/impls/block/blocks/footnote.rs
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
use std::ops::{Deref, DerefMut};

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
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Parsing footnote ref block (in-head {in_head})");

    // Check footnote flag
    //
    // This is true if we're a [[footnote]] inside a [[footnote]],
    // which is not allowed.
    if parser.in_footnote() {
        return Err(parser.make_err(ParseErrorKind::FootnotesNested));
    }

    // Set footnote ref flag
    let parser = &mut ParserWrap::new(parser);

    // Parse out block
    assert!(!flag_star, "Footnote reference doesn't allow star flag");
    assert!(!flag_score, "Footnote reference doesn't allow score flag");
    assert_block_name(&BLOCK_FOOTNOTE, name);

    parser.get_head_none(&BLOCK_FOOTNOTE, in_head)?;

    // Gather footnote contents with paragraphs.
    //
    // However, if there's only one, then we strip it
    // and make it inline.
    let (mut elements, errors, _) =
        parser.get_body_elements(&BLOCK_FOOTNOTE, true)?.into();

    if elements.len() == 1 {
        match elements.pop().unwrap() {
            // Unwrap the paragraph and get its contents.
            Element::Container(container)
                if container.ctype() == ContainerType::Paragraph =>
            {
                let mut new_elements: Vec<Element> = container.into();
                elements.append(&mut new_elements);
            }

            // Other element, keep as-is.
            element => elements.push(element),
        };
    }

    // Append footnote contents and return.
    parser.push_footnote(elements);

    ok!(Element::Footnote, errors)
}

fn parse_footnote_block<'r, 't>(
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    debug!("Parsing footnote list block (in-head {in_head})");
    assert!(!flag_star, "Footnote block doesn't allow star flag");
    assert!(!flag_score, "Footnote block doesn't allow score flag");
    assert_block_name(&BLOCK_FOOTNOTE_BLOCK, name);

    // Parse arguments
    let mut arguments = parser.get_head_map(&BLOCK_FOOTNOTE_BLOCK, in_head)?;

    let title = arguments.get("title");
    let hide = arguments.get_bool(parser, "hide")?.unwrap_or(false);

    if !arguments.is_empty() {
        warn!("Invalid argument keys found");
        return Err(parser.make_err(ParseErrorKind::BlockMalformedArguments));
    }

    // Tell parser that a footnote block was added
    parser.set_footnote_block();

    // Build and return
    ok!(Element::FootnoteBlock { title, hide })
}

/// Helper structure to set the `in_footnote` flag.
///
/// This is only for `[[footnote]]`, the flag is meant
/// to prevent nested `[[footnote]]`s.
#[derive(Debug)]
struct ParserWrap<'p, 'r, 't> {
    parser: &'p mut Parser<'r, 't>,
}

impl<'p, 'r, 't> ParserWrap<'p, 'r, 't> {
    #[inline]
    fn new(parser: &'p mut Parser<'r, 't>) -> Self {
        parser.set_footnote_flag(true);

        ParserWrap { parser }
    }
}

impl<'r, 't> Deref for ParserWrap<'_, 'r, 't> {
    type Target = Parser<'r, 't>;

    #[inline]
    fn deref(&self) -> &Parser<'r, 't> {
        self.parser
    }
}

impl<'r, 't> DerefMut for ParserWrap<'_, 'r, 't> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Parser<'r, 't> {
        self.parser
    }
}

impl Drop for ParserWrap<'_, '_, '_> {
    fn drop(&mut self) {
        self.parser.set_footnote_flag(false);
    }
}
