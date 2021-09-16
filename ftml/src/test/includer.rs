/*
 * test/includer.rs
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

use crate::data::PageRef;
use crate::includes::{FetchedPage, IncludeRef, Includer};
use std::borrow::Cow;
use void::Void;

const FRUIT_PAGE_SOURCE: &str = "
[[blockquote]]
* Apple: {$apple}
* Banana: {$banana}
* Cherry: {$cherry}
[[/blockquote]]
";

const COMPONENT_PAGE_SOURCE: &str = "
[[div class=\"{$class}\"]]
  [[ul]]
    [[li]] {$item-1} [[/li]]
    [[li]] {$item-2} [[/li]]
    [[li]] {$item-3} [[/li]]
    [[li]] {$item-4} [[/li]]
  [[/ul]]
[[/div]]
";

#[derive(Debug)]
pub struct TestIncluder;

impl<'t> Includer<'t> for TestIncluder {
    type Error = Void;

    #[inline]
    fn include_pages(
        &mut self,
        includes: &[IncludeRef<'t>],
    ) -> Result<Vec<FetchedPage<'t>>, Void> {
        let mut pages = Vec::new();

        for include in includes {
            let content = match *&include.page_ref().page() {
                "fruit" => Some(Cow::Borrowed(FRUIT_PAGE_SOURCE)),
                "component:thing" => Some(Cow::Borrowed(COMPONENT_PAGE_SOURCE)),
                "missing" => None,
                _ => Some(Cow::Borrowed("INCLUDED PAGE")),
            };

            let page_ref = include.page_ref().clone();

            pages.push(FetchedPage { page_ref, content });
        }

        Ok(pages)
    }

    #[inline]
    fn no_such_include(&mut self, page_ref: &PageRef<'t>) -> Result<Cow<'t, str>, Void> {
        Ok(Cow::Owned(format!(
            "[[div class=\"wj-error\"]]\nNo such page '{}'\n[[/div]]",
            page_ref,
        )))
    }
}
