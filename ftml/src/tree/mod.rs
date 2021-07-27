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
mod clone;
mod condition;
mod container;
mod element;
mod heading;
mod image;
mod link;
mod list;
mod module;
mod tag;

pub use self::align::*;
pub use self::anchor::*;
pub use self::attribute::AttributeMap;
pub use self::condition::*;
pub use self::container::*;
pub use self::element::*;
pub use self::heading::*;
pub use self::image::*;
pub use self::link::*;
pub use self::list::*;
pub use self::module::*;
pub use self::tag::*;

use self::clone::{elements_to_owned, strings_to_owned};
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
    /// however the recommendation is to combine them all into one large style
    /// rule list.
    pub styles: Vec<Cow<'t, str>>,
}

impl<'t> SyntaxTree<'t> {
    pub(crate) fn from_element_result(
        elements: Vec<Element<'t>>,
        warnings: Vec<ParseWarning>,
        styles: Vec<Cow<'t, str>>,
    ) -> ParseOutcome<Self> {
        let tree = SyntaxTree { elements, styles };
        ParseOutcome::new(tree, warnings)
    }

    pub fn to_owned(&self) -> SyntaxTree<'static> {
        SyntaxTree {
            elements: elements_to_owned(&self.elements),
            styles: strings_to_owned(&self.styles),
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
