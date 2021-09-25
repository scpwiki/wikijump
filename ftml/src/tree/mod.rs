/*
 * tree/mod.rs
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

pub mod attribute;

mod align;
mod anchor;
mod clear_float;
mod clone;
mod container;
mod definition_list;
mod element;
mod heading;
mod image;
mod link;
mod list;
mod module;
mod partial;
mod table;
mod tag;
mod variables;

pub use self::align::*;
pub use self::anchor::*;
pub use self::attribute::AttributeMap;
pub use self::clear_float::*;
pub use self::container::*;
pub use self::definition_list::*;
pub use self::element::*;
pub use self::heading::*;
pub use self::image::*;
pub use self::link::*;
pub use self::list::*;
pub use self::module::*;
pub use self::partial::*;
pub use self::table::*;
pub use self::tag::*;
pub use self::variables::*;

use self::clone::{elements_lists_to_owned, elements_to_owned, strings_to_owned};
use crate::parsing::{ParseOutcome, ParseWarning};
use std::borrow::Cow;

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct SyntaxTree<'t> {
    /// The list of elements that compose this tree.
    ///
    /// Note that each `Element<'t>` can contain other elements within it,
    /// and these as well, etc. This structure composes the depth of the
    /// syntax tree.
    pub elements: Vec<Element<'t>>,

    /// The list of CSS styles added in this page, in order.
    ///
    /// How the renderer decides to consume these is up to the implementation,
    /// however the recommendation is to add these as separate style tags.
    pub styles: Vec<Cow<'t, str>>,

    /// The full table of contents for this page.
    ///
    /// Depth list conversion happens here, so that depths on the table
    /// match the heading level.
    pub table_of_contents: Vec<Element<'t>>,

    /// The full footnote list for this page.
    pub footnotes: Vec<Vec<Element<'t>>>,
}

impl<'t> SyntaxTree<'t> {
    pub(crate) fn from_element_result(
        elements: Vec<Element<'t>>,
        warnings: Vec<ParseWarning>,
        styles: Vec<Cow<'t, str>>,
        table_of_contents: Vec<Element<'t>>,
        footnotes: Vec<Vec<Element<'t>>>,
    ) -> ParseOutcome<Self> {
        let tree = SyntaxTree {
            elements,
            styles,
            table_of_contents,
            footnotes,
        };
        ParseOutcome::new(tree, warnings)
    }

    pub fn to_owned(&self) -> SyntaxTree<'static> {
        SyntaxTree {
            elements: elements_to_owned(&self.elements),
            styles: strings_to_owned(&self.styles),
            table_of_contents: elements_to_owned(&self.table_of_contents),
            footnotes: elements_lists_to_owned(&self.footnotes),
        }
    }
}

#[test]
fn borrowed_to_owned<'a>() {
    use std::mem;

    let tree_1: SyntaxTree<'a> = SyntaxTree::default();
    let tree_2: SyntaxTree<'static> = tree_1.to_owned();

    mem::drop(tree_1);

    let tree_3: SyntaxTree<'static> = tree_2.clone();

    mem::drop(tree_3);
}
