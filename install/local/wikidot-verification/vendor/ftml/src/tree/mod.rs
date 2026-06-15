/*
 * tree/mod.rs
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

pub mod attribute;

mod align;
mod anchor;
mod bibliography;
mod clear_float;
mod clone;
mod code;
mod container;
mod date;
mod definition_list;
mod element;
mod embed;
mod file_source;
mod heading;
mod link;
mod list;
mod module;
mod partial;
mod ruby;
mod tab;
mod table;
mod tag;
mod variables;

pub use self::align::*;
pub use self::anchor::*;
pub use self::attribute::AttributeMap;
pub use self::bibliography::*;
pub use self::clear_float::*;
pub use self::code::CodeBlock;
pub use self::container::*;
pub use self::date::DateItem;
pub use self::definition_list::*;
pub use self::element::*;
pub use self::embed::*;
pub use self::file_source::*;
pub use self::heading::*;
pub use self::link::*;
pub use self::list::*;
pub use self::module::*;
pub use self::partial::*;
pub use self::ruby::*;
pub use self::tab::*;
pub use self::table::*;
pub use self::tag::*;
pub use self::variables::*;

use self::clone::{elements_lists_to_owned, elements_to_owned, string_to_owned};
use crate::parsing::{ParseError, ParseOutcome};
use std::borrow::Cow;
use std::ops::Not;

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct SyntaxTree<'t> {
    /// The list of elements that compose this tree.
    ///
    /// Note that each `Element<'t>` can contain other elements within it,
    /// and these as well, etc. This structure composes the depth of the
    /// syntax tree.
    pub elements: Vec<Element<'t>>,

    /// The full table of contents for this page.
    ///
    /// Depth list conversion happens here, so that depths on the table
    /// match the heading level.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub table_of_contents: Vec<Element<'t>>,

    /// The full list of HTML blocks for this page.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub html_blocks: Vec<Cow<'t, str>>,

    /// The full list of code blocks for this page.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub code_blocks: Vec<CodeBlock<'t>>,

    /// The full footnote list for this page.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub footnotes: Vec<Vec<Element<'t>>>,

    /// Whether the renderer should add its own footnote block.
    ///
    /// This is true if there is no footnote block in the element
    /// list above, *and* there are footnotes to render.
    // NOTE: Not::not() here is effectively saying "don't serialize if !value"
    //       which is just "is false".
    #[serde(default, skip_serializing_if = "Not::not")]
    pub needs_footnote_block: bool,

    /// The full list of bibliographies for this page.
    #[serde(default, skip_serializing_if = "BibliographyList::is_empty")]
    pub bibliographies: BibliographyList<'t>,

    /// Hint for the size of the wikitext input.
    ///
    /// This is an optimization to make rendering large parges slightly faster.
    #[serde(skip)]
    pub wikitext_len: usize,
}

impl<'t> SyntaxTree<'t> {
    pub(crate) fn from_element_result(
        elements: Vec<Element<'t>>,
        errors: Vec<ParseError>,
        (html_blocks, code_blocks): (Vec<Cow<'t, str>>, Vec<CodeBlock<'t>>),
        table_of_contents: Vec<Element<'t>>,
        (footnotes, needs_footnote_block): (Vec<Vec<Element<'t>>>, bool),
        bibliographies: BibliographyList<'t>,
        wikitext_len: usize,
    ) -> ParseOutcome<Self> {
        let tree = SyntaxTree {
            elements,
            table_of_contents,
            html_blocks,
            code_blocks,
            footnotes,
            needs_footnote_block,
            bibliographies,
            wikitext_len,
        };
        ParseOutcome::new(tree, errors)
    }

    pub fn to_owned(&self) -> SyntaxTree<'static> {
        SyntaxTree {
            elements: elements_to_owned(&self.elements),
            table_of_contents: elements_to_owned(&self.table_of_contents),
            html_blocks: self
                .html_blocks
                .iter()
                .map(|html| string_to_owned(html))
                .collect(),
            code_blocks: self
                .code_blocks
                .iter()
                .map(|code| code.to_owned())
                .collect(),
            footnotes: elements_lists_to_owned(&self.footnotes),
            needs_footnote_block: self.needs_footnote_block,
            bibliographies: self.bibliographies.to_owned(),
            wikitext_len: self.wikitext_len,
        }
    }
}

#[test]
fn borrowed_to_owned() {
    use std::mem;

    let tree_1: SyntaxTree<'_> = SyntaxTree::default();
    let tree_2: SyntaxTree<'static> = tree_1.to_owned();

    mem::drop(tree_1);

    let tree_3: SyntaxTree<'static> = tree_2.clone();

    mem::drop(tree_3);
}
