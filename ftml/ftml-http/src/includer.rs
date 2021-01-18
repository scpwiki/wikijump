/*
 * includer.rs
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

use crate::Error;
use ftml::{FetchedPages, IncludeRef, Includer, PageRef};
use std::borrow::Cow;
use tera::{Context, Tera};

#[derive(Debug)]
pub struct HttpIncluder<'a> {
    callback_url: &'a str,
    templates: Tera,
}

impl<'a> HttpIncluder<'a> {
    pub fn new(
        callback_url: &'a str,
        missing_include_template: &'a str,
    ) -> Result<Self, Error> {
        let mut templates = Tera::default();
        templates.add_raw_template("missing.ftml", missing_include_template)?;

        Ok(HttpIncluder {
            callback_url,
            templates,
        })
    }
}

impl<'t> Includer<'t> for HttpIncluder<'_> {
    type Error = Error;

    fn include_pages(
        &mut self,
        includes: &[IncludeRef<'t>],
    ) -> Result<FetchedPages<'t>, Self::Error> {
        let body = reqwest::blocking::get(self.callback_url)?
            .json()?;

        todo!()
    }

    fn no_such_include(
        &mut self,
        page_ref: &PageRef<'t>,
    ) -> Result<Cow<'t, str>, Self::Error> {
        let context = {
            let mut context = Context::new();

            if let Some(site) = page_ref.site() {
                context.insert("site", site);
            }

            context.insert("page", page_ref.page());
            context.insert("path", page_ref);
            context
        };

        let message = self.templates.render("missing.ftml", &context)?;
        Ok(Cow::Owned(message))
    }
}
