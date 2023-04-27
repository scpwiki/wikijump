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
use crate::models::site::Model as SiteModel;
use crate::services::view::PageAndRevision;
use crate::services::{
    PageRevisionService, PageService, RenderService, SiteService, TextService,
};
use crate::web::Reference;
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
        mut site: &SiteModel,
        sp_page_type: SpecialPageType,
        locale: &LanguageIdentifier,
        page_info: PageInfo<'_>,
    ) -> Result<GetSpecialPageOutput> {
        tide::log::info!(
            "Getting special page {sp_page_type:?} for site ID {}",
            site.site_id,
        );

        // Extract fields based on special page type.
        //
        // "key" refers to the translation key to read to get the default fallback.
        // If empty, then pull a constant string (not in the localization files).
        let site_owned;
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

                site_owned = SiteService::get(ctx, Reference::Slug(cow!("www"))).await?;
                site = &site_owned;

                (&config.special_page_site, "wiki-page-site")
            }
        };

        let (page_and_revision, wikitext) = match PageService::get_optional(
            ctx,
            site.site_id,
            Reference::Slug(cow!(slug)),
        )
        .await?
        {
            Some(page) => {
                // Fetch special page wikitext, it exists.

                let page_revision =
                    PageRevisionService::get_latest(ctx, site.site_id, page.page_id)
                        .await?;

                let wikitext =
                    TextService::get(ctx, &page_revision.wikitext_hash).await?;

                let page_and_revision = Some(PageAndRevision {
                    page,
                    page_revision,
                });

                (page_and_revision, wikitext)
            }
            None => {
                // Page is absent, use fallback string from localization.

                macro_rules! fluent_str {
                    ($value:expr) => {
                        FluentValue::String(cow!(&$value))
                    };
                }

                let page = &page_info.page;
                let (category, full_slug) = match &page_info.category {
                    Some(category) => (str!(category), format!("{category}:{page}")),
                    None => (str!("_default"), str!(page)),
                };

                let mut args = FluentArgs::new();
                args.set("slug", FluentValue::String(Cow::Owned(full_slug)));
                args.set("page", fluent_str!(page));
                args.set("category", fluent_str!(category));
                args.set("domain", fluent_str!(config.main_domain_no_dot));

                let wikitext = ctx.localization().translate(locale, key, &args)?;
                (None, wikitext.into_owned())
            }
        };

        // Render here with relevant page context.
        // The "page" here is what would've been there in this case,
        // passed in by the caller.
        let settings = WikitextSettings::from_mode(WikitextMode::Page);
        let render_output =
            RenderService::render(ctx, wikitext.clone(), &page_info, &settings).await?;

        Ok(GetSpecialPageOutput {
            page_and_revision,
            wikitext,
            render_output,
        })
    }
}
