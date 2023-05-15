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
use crate::services::{PageRevisionService, PageService, RenderService, TextService};
use crate::web::Reference;
use crate::utils::split_category;
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
        site: &SiteModel,
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
        let config = ctx.config();
        let (slug, key) = match sp_page_type {
            SpecialPageType::Template => (cow!(config.special_page_template), ""),
            SpecialPageType::Missing => {
                let slug = match split_category(&config.special_page_missing) {
                    // Has category explicitly, only use this exact slug.
                    (Some(_), slug) => cow!(slug),

                    // Draw from the same category.
                    (None, slug) => match page_info.category {
                        Some(ref category) => Cow::Owned(format!("{category}:{slug}")),
                        None => cow!(slug),
                    }
                };

                (slug, "wiki-page-missing")
            }
            SpecialPageType::Private => {
                (cow!(config.special_page_private), "wiki-page-private")
            }
        };

        let wikitext = match PageService::get_optional(
            ctx,
            site.site_id,
            Reference::Slug(slug),
        )
        .await?
        {
            Some(page) => {
                // Fetch special page wikitext, it exists.

                let revision =
                    PageRevisionService::get_latest(ctx, site.site_id, page.page_id)
                        .await?;

                TextService::get(ctx, &revision.wikitext_hash).await?
            }
            None => {
                // Page is absent, use fallback string from localization.

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
                wikitext.into_owned()
            }
        };

        // Render here with relevant page context.
        // The "page" here is what would've been there in this case,
        // passed in by the caller.
        let settings = WikitextSettings::from_mode(WikitextMode::Page);
        let render_output =
            RenderService::render(ctx, wikitext.clone(), &page_info, &settings).await?;

        Ok(GetSpecialPageOutput {
            wikitext,
            render_output,
        })
    }
}
