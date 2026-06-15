/*
 * includes/includer/mod.rs
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

//! This module contains tools which format pages after they have been referenced in an include
//! block.

mod debug;
mod null;

mod prelude {
    pub use crate::data::PageRef;
    pub use crate::includes::{FetchedPage, IncludeRef, Includer};
    pub use std::borrow::Cow;
}

use crate::includes::{IncludeRef, PageRef};
use std::borrow::Cow;

pub use self::debug::DebugIncluder;
pub use self::null::NullIncluder;

/// A type used by [`Includer`] which represents a page that is ready to be included.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct FetchedPage<'t> {
    pub page_ref: PageRef,
    pub content: Option<Cow<'t, str>>,
}

/// A trait that handles the formatting of included pages.
pub trait Includer<'t> {
    type Error;

    /// Returns a list of the pages included.
    fn include_pages(
        &mut self,
        includes: &[IncludeRef<'t>],
    ) -> Result<Vec<FetchedPage<'t>>, Self::Error>;

    /// Handles the inclusion of a page not found.
    fn no_such_include(
        &mut self,
        page_ref: &PageRef,
    ) -> Result<Cow<'t, str>, Self::Error>;
}
