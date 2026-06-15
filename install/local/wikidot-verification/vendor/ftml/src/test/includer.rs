/*
 * test/includer.rs
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

use crate::data::PageRef;
use crate::includes::{FetchedPage, IncludeRef, Includer};
use std::borrow::Cow;
use std::convert::Infallible;

const FRUIT_PAGE_SOURCE: &str = "
* Apple
* Banana
* Cherry
";

const COMPONENT_BASIC_PAGE_SOURCE: &str = "
My name is __{$name}__:

[[blockquote]]
{$contents}
[[/blockquote]]
";

const COMPONENT_FRUIT_PAGE_SOURCE: &str = "
[[div id=\"fruit\" class=\"{$class}\"]]
  [[ul]]
    [[li]] {$apple} [[/li]]
    [[li]] {$banana} [[/li]]
  [[/ul]]
[[/div]]
";

#[derive(Debug)]
pub struct TestIncluder;

impl<'t> Includer<'t> for TestIncluder {
    type Error = Infallible;

    #[inline]
    fn include_pages(
        &mut self,
        includes: &[IncludeRef<'t>],
    ) -> Result<Vec<FetchedPage<'t>>, Infallible> {
        let mut pages = Vec::new();

        for include in includes {
            let page_ref = include.page_ref().clone();
            let content = get_page_source(&page_ref);
            pages.push(FetchedPage { page_ref, content });
        }

        Ok(pages)
    }

    #[inline]
    fn no_such_include(
        &mut self,
        page_ref: &PageRef,
    ) -> Result<Cow<'t, str>, Infallible> {
        Ok(Cow::Owned(format!(
            "[[div class=\"wj-error\"]]\nNo such page '{page_ref}'\n[[/div]]",
        )))
    }
}

fn get_page_source(page_ref: &PageRef) -> Option<Cow<'static, str>> {
    macro_rules! cow {
        ($text:expr) => {
            Cow::Borrowed($text)
        };
    }

    if page_ref.site().is_some() {
        return Some(cow!("OFF-SITE INCLUDED PAGE"));
    }

    match page_ref.page() {
        "fruit" => Some(cow!(FRUIT_PAGE_SOURCE)),
        "component:basic" => Some(cow!(COMPONENT_BASIC_PAGE_SOURCE)),
        "component:fruit" => Some(cow!(COMPONENT_FRUIT_PAGE_SOURCE)),
        "fragment:page" => Some(cow!("INCLUDED FRAGMENT")),
        "missing" => None,
        _ => Some(cow!("INCLUDED PAGE")),
    }
}
