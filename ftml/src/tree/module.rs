/*
 * tree/module.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Ammon Smith
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
        id: Option<Cow<'t, str>>,
        class: Option<Cow<'t, str>>,
        style: Option<Cow<'t, str>>,
    },

    /// Meta-element for modules which perform no action.
    Null,

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
}
