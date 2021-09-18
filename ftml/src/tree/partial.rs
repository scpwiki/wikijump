/*
 * tree/partial.rs
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

use super::{
    Collection, Element, Elements, ListItem, ParagraphSafe, TableCell, TableRow,
};
use crate::parsing::ParseWarningKind;
use std::convert::{TryFrom, TryInto};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", tag = "element", content = "data")]
pub enum PartialElement<'t> {
    /// A full, complete element.
    ///
    /// This can be passed as-is to the final syntax tree.
    Complete(Element<'t>),

    /// A row within a table.
    TableRow(TableRow<'t>),

    /// A cell within a table row.
    TableCell(TableCell<'t>),

    /// An item within a list.
    ListItem(Box<ListItem<'t>>),
}

impl<'t> PartialElement<'t> {
    pub fn to_owned(&self) -> PartialElement<'static> {
        match self {
            PartialElement::Complete(element) => {
                PartialElement::Complete(element.to_owned())
            }
            PartialElement::TableRow(row) => PartialElement::TableRow(row.to_owned()),
            PartialElement::TableCell(cell) => PartialElement::TableCell(cell.to_owned()),
            PartialElement::ListItem(boxed_list_item) => {
                let list_item: &ListItem = &*boxed_list_item;

                PartialElement::ListItem(Box::new(list_item.to_owned()))
            }
        }
    }
}

impl ParagraphSafe for PartialElement<'_> {
    fn paragraph_safe(&self) -> bool {
        match self {
            PartialElement::Complete(element) => element.paragraph_safe(),
            PartialElement::TableRow(_)
            | PartialElement::TableCell(_)
            | PartialElement::ListItem(_) => false,
        }
    }
}

impl<'t> From<Element<'t>> for PartialElement<'t> {
    #[inline]
    fn from(element: Element<'t>) -> PartialElement<'t> {
        PartialElement::Complete(element)
    }
}

impl<'t> TryFrom<PartialElement<'t>> for Element<'t> {
    type Error = ParseWarningKind;

    fn try_from(partial: PartialElement<'t>) -> Result<Element<'t>, ParseWarningKind> {
        match partial {
            PartialElement::Complete(element) => Ok(element),
            PartialElement::TableRow(_) => Err(ParseWarningKind::TableRowOutsideTable),
            PartialElement::TableCell(_) => Err(ParseWarningKind::TableCellOutsideTable),
            PartialElement::ListItem(_) => Err(ParseWarningKind::ListItemOutsideList),
        }
    }
}

/// Wrapper for the result of producing element(s).
///
/// This has an enum instead of a simple `Vec<PartialElement>`
/// since the most common output is a single element,
/// and it makes little sense to heap allocate for every
/// single return if we can easily avoid it.
///
/// It also contains a field marking whether all of the
/// contents are paragraph-safe or not, used by `ParagraphStack`.
pub type PartialElements<'t> = Collection<PartialElement<'t>>;

/// Convert a complete `Element` to a `PartialElements` collection.
impl<'t> From<Element<'t>> for PartialElements<'t> {
    #[inline]
    fn from(element: Element<'t>) -> PartialElements<'t> {
        PartialElements::Single(PartialElement::Complete(element))
    }
}

/// Unwrap `PartialElements` into `Elements`.
impl<'t> TryFrom<PartialElements<'t>> for Elements<'t> {
    type Error = ParseWarningKind;

    fn try_from(partials: PartialElements<'t>) -> Result<Elements<'t>, ParseWarningKind> {
        match partials {
            PartialElements::None => Ok(Elements::None),
            PartialElements::Single(partial) => partial.try_into().map(Elements::Single),
            PartialElements::Multiple(partials) => {
                let mut elements = Vec::new();

                for partial in partials {
                    elements.push(partial.try_into()?);
                }

                Ok(Elements::Multiple(elements))
            }
        }
    }
}
