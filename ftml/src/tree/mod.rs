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

mod anchor;
mod attribute;
mod container;
mod element;
mod heading;
mod link;
mod list;
mod module;
mod tag;

pub use self::anchor::*;
pub use self::attribute::AttributeMap;
pub use self::container::*;
pub use self::element::*;
pub use self::heading::*;
pub use self::link::*;
pub use self::list::*;
pub use self::module::*;
pub use self::tag::*;

use crate::parsing::{ParseOutcome, ParseWarning};
use std::borrow::{Borrow, Cow};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
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
}

/// Wrapper structure to implement `ToOwned` for `SyntaxTree`.
///
/// The `ToOwned` implementation converts all the underlying
/// `Cow`s into their owned version, thus the `'static` lifetime.
///
/// However because `Borrow<T>` for `T` is already implemented, we need
/// to implement for a wrapper to avoid this.
/// Then we just unwrap it and we have the cloned tree.
///
/// A `Borrow` implementation is also needed for `ToOwned`.
#[derive(Debug, Default)]
pub struct SyntaxTreeOwned(pub SyntaxTree<'static>);

impl<'t> Borrow<SyntaxTree<'t>> for SyntaxTreeOwned {
    #[inline]
    fn borrow(&self) -> &SyntaxTree<'t> {
        &self.0
    }
}

impl<'t> ToOwned for SyntaxTreeOwned {
    type Owned = SyntaxTreeOwned;

    fn to_owned(&self) -> SyntaxTreeOwned {
        let inner = SyntaxTree {
            elements: self.0.elements.iter().map(ToOwned::to_owned).collect(),
            styles: self.0.styles.iter().map(ToOwned::to_owned).collect(),
        };

        SyntaxTreeOwned(inner)
    }
}
