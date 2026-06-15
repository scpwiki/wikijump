/*
 * includes/includer/null.rs
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

use super::prelude::*;
use std::convert::Infallible;

/// An [`Includer`] that replaces include blocks with nothing.
#[derive(Debug)]
pub struct NullIncluder;

impl<'t> Includer<'t> for NullIncluder {
    type Error = Infallible;

    #[inline]
    fn include_pages(
        &mut self,
        _includes: &[IncludeRef<'t>],
    ) -> Result<Vec<FetchedPage<'t>>, Infallible> {
        Ok(Vec::new())
    }

    #[inline]
    fn no_such_include(
        &mut self,
        _page_ref: &PageRef,
    ) -> Result<Cow<'t, str>, Infallible> {
        Ok(Cow::Borrowed(""))
    }
}
