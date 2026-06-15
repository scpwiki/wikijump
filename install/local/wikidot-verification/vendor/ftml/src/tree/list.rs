/*
 * tree/list.rs
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

use super::Element;
use super::attribute::AttributeMap;
use super::clone::elements_to_owned;
use strum_macros::IntoStaticStr;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", tag = "item-type")]
pub enum ListItem<'t> {
    /// This item is a series of elements.
    ///
    /// It's just an item in the list, which may have multiple elements
    /// similar to any other container.
    Elements {
        attributes: AttributeMap<'t>,
        elements: Vec<Element<'t>>,
    },

    /// This item in the list is a sub-list.
    ///
    /// That is, it's another, deeper list within the list.
    SubList {
        #[serde(flatten)]
        element: Box<Element<'t>>,
    },
}

impl ListItem<'_> {
    pub fn to_owned(&self) -> ListItem<'static> {
        match self {
            ListItem::Elements {
                attributes,
                elements,
            } => ListItem::Elements {
                attributes: attributes.to_owned(),
                elements: elements_to_owned(elements),
            },
            ListItem::SubList { element } => {
                let element: &Element = element;

                ListItem::SubList {
                    element: Box::new(element.to_owned()),
                }
            }
        }
    }
}

#[derive(
    Serialize, Deserialize, IntoStaticStr, Debug, Copy, Clone, Hash, PartialEq, Eq,
)]
#[serde(rename_all = "kebab-case")]
pub enum ListType {
    /// Bullet lists, or "unordered lists" in HTML.
    ///
    /// Corresponds to the tag `<ul>`.
    Bullet,

    /// Numbered lists, or "ordered lists" in HTML.
    ///
    /// Corresponds to the tag `<ol>`.
    Numbered,

    /// Generic list, which does not have a preferred list type.
    ///
    /// This can be implemented in HTML with either
    /// `<ul>` or `<ol>`, as these should not have any
    /// list items that are not sub-lists.
    Generic,
}

impl ListType {
    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }

    #[inline]
    pub fn html_tag(self) -> &'static str {
        match self {
            ListType::Bullet | ListType::Generic => "ul",
            ListType::Numbered => "ol",
        }
    }
}
