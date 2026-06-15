/*
 * parsing/mod.rs
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

#[macro_use]
mod macros;

mod boolean;
mod check_step;
mod collect;
mod condition;
mod consume;
mod depth;
mod element_condition;
mod error;
mod outcome;
mod paragraph;
mod parser;
mod parser_wrap;
mod result;
mod rule;
mod string;
mod strip;
mod token;

mod prelude {
    pub use crate::parsing::{
        ExtractedToken, ParseError, ParseErrorKind, ParseResult, ParseSuccess, Token,
    };
    pub use crate::settings::WikitextSettings;
    pub use crate::text::FullText;
    pub use crate::tree::{Element, Elements};
}

use self::depth::{DepthItem, DepthList, process_depths};
use self::element_condition::{ElementCondition, ElementConditionType};
use self::paragraph::{NO_CLOSE_CONDITION, gather_paragraphs};
use self::parser::Parser;
use self::parser_wrap::ParserWrap;
use self::rule::impls::RULE_PAGE;
use self::strip::{strip_newlines, strip_whitespace};
use crate::data::PageInfo;
use crate::next_index::{Incrementer, NextIndex};
use crate::settings::WikitextSettings;
use crate::tokenizer::Tokenization;
use crate::tree::{
    AttributeMap, BibliographyList, CodeBlock, Element, LinkLabel, LinkLocation,
    LinkType, ListItem, ListType, SyntaxTree,
};
use std::borrow::Cow;

pub use self::boolean::{NonBooleanValue, parse_boolean};
pub use self::error::{ParseError, ParseErrorKind};
pub use self::outcome::ParseOutcome;
pub use self::result::{ParseResult, ParseSuccess};
pub use self::token::{ExtractedToken, Token};

/// Parse through the given tokens and produce an AST.
///
/// This takes a list of [`ExtractedToken`] items produced by [tokenize](crate::tokenizer::tokenize()).
pub fn parse<'r, 't>(
    tokenization: &'r Tokenization<'t>,
    page_info: &'r PageInfo<'t>,
    settings: &'r WikitextSettings,
) -> ParseOutcome<SyntaxTree<'t>>
where
    'r: 't,
{
    // Run parsing, get raw results
    let UnstructuredParseResult {
        result,
        html_blocks,
        code_blocks,
        table_of_contents_depths,
        footnotes,
        has_footnote_block,
        bibliographies,
    } = parse_internal(page_info, settings, tokenization);

    // Mutable state
    let mut toc_indexer = settings.id_indexer();

    debug!("Finished paragraph gathering, matching on consumption");
    match result {
        Ok(ParseSuccess {
            item: elements,
            errors,
            ..
        }) => {
            debug!(
                "Finished parsing, producing final syntax tree ({} errors)",
                errors.len(),
            );

            // process_depths() wants a "list type", so we map in a () for each.
            let table_of_contents_depths = table_of_contents_depths
                .into_iter()
                .map(|(depth, contents)| (depth, (), contents));

            // Convert TOC depth lists
            let table_of_contents = process_depths((), table_of_contents_depths)
                .into_iter()
                .map(|(_, items)| build_toc_list_element(&mut toc_indexer, items))
                .collect::<Vec<_>>();

            // This flag notes that the renderer needs to insert its own footnote
            // block. This happens when there are footnotes to be rendered, but
            // there isn't a manually-placed footnote block somewhere in the elements.
            let needs_footnote_block = !footnotes.is_empty() && !has_footnote_block;

            SyntaxTree::from_element_result(
                elements,
                errors,
                (html_blocks, code_blocks),
                table_of_contents,
                (footnotes, needs_footnote_block),
                bibliographies,
                tokenization.full_text().len(),
            )
        }
        Err(error) => {
            // This path is only reachable if a very bad error occurs.
            //
            // If this happens, then just return the input source as the output
            // and the error.

            error!("Fatal error occurred at highest-level parsing: {error:#?}");
            let wikitext = tokenization.full_text().inner();
            let elements = vec![text!(wikitext)];
            let errors = vec![error];
            let table_of_contents = vec![];
            let footnotes = vec![];
            let needs_footnote_block = true;
            let bibliographies = BibliographyList::new();

            SyntaxTree::from_element_result(
                elements,
                errors,
                (html_blocks, code_blocks),
                table_of_contents,
                (footnotes, needs_footnote_block),
                bibliographies,
                tokenization.full_text().len(),
            )
        }
    }
}

/// Runs the parser, but returns the raw internal results prior to conversion.
pub fn parse_internal<'r, 't>(
    page_info: &'r PageInfo<'t>,
    settings: &'r WikitextSettings,
    tokenization: &'r Tokenization<'t>,
) -> UnstructuredParseResult<'r, 't>
where
    'r: 't,
{
    let mut parser = Parser::new(tokenization, page_info, settings);

    // At the top level, we gather elements into paragraphs
    info!("Running parser on {} tokens", tokenization.tokens().len());
    let result = gather_paragraphs(&mut parser, RULE_PAGE, NO_CLOSE_CONDITION);

    // Build and return
    let html_blocks = parser.remove_html_blocks();
    let code_blocks = parser.remove_code_blocks();
    let table_of_contents_depths = parser.remove_table_of_contents();
    let footnotes = parser.remove_footnotes();
    let has_footnote_block = parser.has_footnote_block();
    let bibliographies = parser.remove_bibliographies();

    UnstructuredParseResult {
        result,
        html_blocks,
        code_blocks,
        table_of_contents_depths,
        footnotes,
        has_footnote_block,
        bibliographies,
    }
}

// Helper functions

fn build_toc_list_element(
    incr: &mut Incrementer,
    list: DepthList<(), String>,
) -> Element<'static> {
    let build_item = |item| match item {
        DepthItem::List(_, list) => ListItem::SubList {
            element: Box::new(build_toc_list_element(incr, list)),
        },
        DepthItem::Item(name) => {
            let anchor = match incr.next() {
                None => Cow::Borrowed("javascript:;"),
                Some(index) => Cow::Owned(format!("#toc{index}")),
            };
            let link = Element::Link {
                ltype: LinkType::TableOfContents,
                link: LinkLocation::Url(anchor),
                label: LinkLabel::Text(Cow::Owned(name)),
                target: None,
            };

            ListItem::Elements {
                elements: vec![link],
                attributes: AttributeMap::new(),
            }
        }
    };

    let items = list.into_iter().map(build_item).collect();
    let attributes = AttributeMap::new();

    Element::List {
        ltype: ListType::Bullet,
        items,
        attributes,
    }
}

/// Represents the result of an internal parse.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnstructuredParseResult<'r, 't> {
    /// The returned result from parsing.
    pub result: ParseResult<'r, 't, Vec<Element<'t>>>,

    /// The list of HTML blocks to emit from this page.
    pub html_blocks: Vec<Cow<'t, str>>,

    /// The list of code blocks to emit from this page.
    pub code_blocks: Vec<CodeBlock<'t>>,

    /// The "depths" list for table of content entries.
    ///
    /// Each value is a zero-indexed depth of how
    pub table_of_contents_depths: Vec<(usize, String)>,

    /// The list of footnotes.
    ///
    /// Each entry is a series of elements, in combination
    /// they make the contents of one footnote.
    pub footnotes: Vec<Vec<Element<'t>>>,

    /// Whether a footnote block was placed during parsing.
    pub has_footnote_block: bool,

    /// The list of bibliographies.
    ///
    /// See `src/tree/bibliography.rs`.
    pub bibliographies: BibliographyList<'t>,
}
