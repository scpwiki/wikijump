/*
 * services/render/includer.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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
use crate::services::{PageService, SiteService, TextService};
use fluent::FluentArgs;
use ftml::data::PageRef;
use ftml::includes::{FetchedPage, IncludeRef, Includer};
use std::borrow::Cow;
use unic_langid::LanguageIdentifier;

#[derive(Debug)]
pub struct PageIncluder<'ctx> {
    ctx: &'ctx ServiceContext<'ctx>,
    site_id: i64,
    locale: LanguageIdentifier,
}

impl<'ctx> PageIncluder<'ctx> {
    #[inline]
    pub fn new(
        ctx: &'ctx ServiceContext<'ctx>,
        site_id: i64,
        locale: LanguageIdentifier,
    ) -> Self {
        PageIncluder {
            ctx,
            site_id,
            locale,
        }
    }
}

impl<'t, 'ctx> Includer<'t> for PageIncluder<'ctx> {
    type Error = Error;

    fn include_pages(
        &mut self,
        includes: &[IncludeRef<'t>],
    ) -> Result<Vec<FetchedPage<'t>>> {
        for include in includes {
            todo!();
        }

        todo!()
    }

    fn no_such_include(&mut self, page_ref: &PageRef<'t>) -> Result<Cow<'t, str>> {
        let locales = &self.ctx.state().localizations;
        let args = {
            let mut args = FluentArgs::new();
            args.set(
                "site",
                match page_ref.site.as_ref() {
                    Some(site) => site.as_ref(),
                    None => "",
                },
            );
            args.set("page", page_ref.page.as_ref());
            args
        };

        let message = {
            let mut message =
                locales.translate(&self.locale, "wikitext-missing-include", &args)?;
            let mut buffer = message.to_mut();
            buffer.insert_str(0, r#"[[div class="wj-error"]"#);
            buffer.push_str("[[/div]]");
            Cow::Owned(message.into_owned())
        };

        Ok(message)
    }
}
