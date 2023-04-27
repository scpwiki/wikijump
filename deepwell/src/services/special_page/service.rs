/*
 * services/special_page/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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
use crate::services::{
    PageRevisionService, PageService, RenderService, SiteService, TextService,
};
use crate::web::Reference;
use either::Either;
use fluent::{FluentArgs, FluentValue};
use ftml::prelude::*;
use std::borrow::Cow;
use unic_langid::LanguageIdentifier;

#[derive(Debug)]
pub struct SpecialPageService;

impl SpecialPageService {
    /// Gets the specified special page, or the fallback if it doesn't exist.
    pub async fn get(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        sp_page_type: SpecialPageType,
        locale: &LanguageIdentifier,
        page_info: PageInfo<'_>,
    ) -> Result<()> {
        tide::log::info!("Getting special page {sp_page_type:?} for site ID {site_id}");

        // Stores site ID or the site model, to allow partial resolution for SpecialPageType::Site.
        let mut site = Either::Left(site_id);

        // Extract fields based on special page type.
        //
        // "key" refers to the translation key to read to get the default fallback.
        // If empty, then pull a constant string (not in the localization files).
        let config = ctx.config();
        let (slug, key) = match sp_page_type {
            SpecialPageType::Template => (&config.special_page_template, ""),
            SpecialPageType::Missing => {
                (&config.special_page_missing, "wiki-page-missing")
            }
            SpecialPageType::Private => {
                (&config.special_page_private, "wiki-page-private")
            }
            SpecialPageType::Site => {
                // A bit of special logic since this page can only exist in 'www'
                // So the site_id isn't used
                debug_assert_eq!(
                    site_id, 0,
                    "Site ID for missing site special page is not null",
                );

                site = Either::Right(
                    SiteService::get(ctx, Reference::Slug(cow!("www"))).await?,
                );

                (&config.special_page_site, "wiki-page-site")
            }
        };

        // Convert partial site Either into site
        let site = match site {
            Either::Right(site) => site,
            Either::Left(site_id) => {
                SiteService::get(ctx, Reference::Id(site_id)).await?
            }
        };

        // Fetch page and wikitext, if exists.
        // If missing, pull default from localization.
        let wikitext = match PageService::get_optional(
            ctx,
            site.site_id,
            Reference::Slug(cow!(slug)),
        )
        .await?
        {
            Some(page) => {
                let revision =
                    PageRevisionService::get_latest(ctx, site.site_id, page.page_id)
                        .await?;

                let wikitext = TextService::get(ctx, &revision.wikitext_hash).await?;
                wikitext
            }
            None => {
                let args = {
                    macro_rules! fluent_str {
                        ($value:expr) => {
                            FluentValue::String(Cow::Owned($value))
                        };
                    }

                    let page = &page_info.page;
                    let (category, full_slug) = match &page_info.category {
                        Some(category) => (str!(category), format!("{category}:{page}")),
                        None => (str!("_default"), str!(page)),
                    };

                    let mut args = FluentArgs::new();
                    args.set("slug", fluent_str!(full_slug));
                    args.set("page", fluent_str!(str!(page)));
                    args.set("category", fluent_str!(category));
                    args.set(
                        "domain",
                        FluentValue::String(cow!(&config.main_domain_no_dot)),
                    );

                    args
                };

                let wikitext = ctx.localization().translate(locale, key, &args)?;
                wikitext.into_owned()
            }
        };

        // Render here with relevant page context.
        // The "page" here is what would've been there in this case,
        // passed in by the caller.
        let settings = WikitextSettings::from_mode(WikitextMode::Page);
        let output = RenderService::render(ctx, wikitext, &page_info, &settings).await?;

        todo!()
    }
}
