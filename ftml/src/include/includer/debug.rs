/*
 * include/includer/debug.rs
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

use super::prelude::*;
use void::Void;

#[derive(Debug)]
pub struct DebugIncluder;

impl<'t> Includer<'t> for DebugIncluder {
    type Error = Void;

    #[inline]
    fn include_pages(
        &mut self,
        includes: &[IncludeRef<'t>],
    ) -> Result<FetchedPages<'t>, Void> {
        let mut first = true;
        let mut pages = HashMap::new();

        for include in includes {
            if first && includes.len() > 1 {
                // If the requested inclusions are greater than one,
                // then have the list be a missing page.
                //
                // This lets us test the no_such_include() method,
                // without it affecting typical single-include test cases.

                first = false;
                continue;
            }

            let content = format!(
                "<INCLUDED-PAGE {} {:?}>",
                include.page(),
                include.variables(),
            );

            pages.insert(include.page().clone(), Cow::Owned(content));
        }

        Ok(pages)
    }

    #[inline]
    fn no_such_include(&mut self, page_ref: &PageRef<'t>) -> Cow<'t, str> {
        Cow::Owned(format!("<MISSING-PAGE {}>", page_ref))
    }
}
