/*
 * tree/module.rs
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

//! Representation of Wikidot modules, along with their context.

use super::AttributeMap;
use super::clone::option_string_to_owned;
use std::borrow::Cow;
use std::num::NonZeroU32;
use strum_macros::IntoStaticStr;

#[derive(Serialize, Deserialize, IntoStaticStr, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", tag = "module", content = "data")]
pub enum Module<'t> {
    /// Lists all the backlinks on the given page.
    ///
    /// If no page is listed, the backlinks are returned for the current page.
    Backlinks { page: Option<Cow<'t, str>> },

    /// Lists all categories on the site, along with the pages they contain.
    #[serde(rename_all = "kebab-case")]
    Categories { include_hidden: bool },

    /// Allows a user to join a site.
    #[serde(rename_all = "kebab-case")]
    Join {
        button_text: Option<Cow<'t, str>>,
        attributes: AttributeMap<'t>,
    },

    /// Lists the structure of pages as connected by parenthood.
    ///
    /// Shows the hierarchy of parent relationships present on the given page.
    /// If no root page is listed, the tree returned is for the current page.
    #[serde(rename_all = "kebab-case")]
    PageTree {
        root: Option<Cow<'t, str>>,
        show_root: bool,
        depth: Option<NonZeroU32>,
    },

    /// A rating module, which can be used to vote on the page.
    Rate,
}

impl Module<'_> {
    #[inline]
    pub fn name(&self) -> &'static str {
        self.into()
    }

    pub fn to_owned(&self) -> Module<'static> {
        match self {
            Module::Backlinks { page } => Module::Backlinks {
                page: option_string_to_owned(page),
            },
            Module::Categories { include_hidden } => Module::Categories {
                include_hidden: *include_hidden,
            },
            Module::Join {
                button_text,
                attributes,
            } => Module::Join {
                button_text: option_string_to_owned(button_text),
                attributes: attributes.to_owned(),
            },
            Module::PageTree {
                root,
                show_root,
                depth,
            } => Module::PageTree {
                root: option_string_to_owned(root),
                show_root: *show_root,
                depth: *depth,
            },
            Module::Rate => Module::Rate,
        }
    }
}
