/*
 * services/blueprint/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
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

use super::structs::{BlueprintPageType, GetBlueprintPageOutput};
use super::template::{compose_template, exact_template_slug};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::site::Model as SiteModel;
use crate::services::ServiceContext;
use crate::services::{PageRevisionService, PageService, RenderService, TextService};
use crate::types::Reference;
use crate::utils::{split_category, strip_fluent_control_chars};
use fluent::{FluentArgs, FluentValue};
use ftml::prelude::{Layout, PageInfo, WikitextMode, WikitextSettings};
use ref_map::OptionRefMap;
use std::borrow::Cow;
use unic_langid::LanguageIdentifier;

// TODO: check config fields for blueprint pages starts with the page prefix
// TODO: deny write ability for normal users for pages that start with the page prefix
// TODO: don't set or update nav pages for pages that start with the page prefix

#[derive(Debug)]
pub struct BlueprintPageService;

impl BlueprintPageService {
    /// Gets the specified blueprint page, or the fallback if it doesn't exist.
    pub async fn get(
        ctx: &ServiceContext<'_>,
        site: &SiteModel,
        blueprint_type: BlueprintPageType,
        locales: &[LanguageIdentifier],
        layout: Layout,
        page_info: PageInfo<'_>,
    ) -> Result<GetBlueprintPageOutput> {
        info!(
            "Getting blueprint page {:?} for site ID {}",
            blueprint_type, site.site_id,
        );

        let make_error = || {
            Error::new(
                format!(
                    "failed to get blueprint page type {:?} with layout {}, info {:?}",
                    blueprint_type,
                    layout.description(),
                    page_info,
                ),
                ErrorType::BlueprintPage,
            )
        };

        // Extract fields based on blueprint page type.
        //
        // "key" refers to the translation key to read to get the default fallback.
        // If empty, then pull a constant string (not in the localization files).
        //
        // Produces a list of slugs to use as a page template, the first one that
        // exists is the one that's used.
        let config = ctx.config();
        let (slugs, translate_key) = match blueprint_type {
            BlueprintPageType::Template => {
                let slug = exact_template_slug(
                    page_info.category.as_deref(),
                    page_info.page.as_ref(),
                    &config.blueprint_page_template,
                )
                .ok_or_else(&make_error)?;

                (vec![slug], "")
            }
            BlueprintPageType::Missing => {
                let slugs = Self::slugs_with_category(
                    &config.blueprint_page_missing,
                    page_info.category.ref_map(|s| s.as_ref()),
                );

                (slugs, "wiki-page-missing")
            }
            BlueprintPageType::Private => (
                vec![cow!(config.blueprint_page_private)],
                "wiki-page-private",
            ),
            BlueprintPageType::Banned => {
                (vec![cow!(config.blueprint_page_banned)], "wiki-page-banned")
            }
            BlueprintPageType::Unauthorized => (vec![], "admin-unauthorized"),
        };

        // Look through each option to get the blueprint page wikitext.
        let wikitext = Self::get_wikitext(
            ctx,
            &slugs,
            translate_key,
            site.site_id,
            locales,
            &page_info,
        )
        .await
        .or_raise(make_error)?;

        // Render here with relevant page context.
        //
        // The "page" here is what would've been there in this case,
        // passed in by the caller.
        //
        // For this reason, we are not using render_page(), as there is
        // no "real" page ID.
        let settings = WikitextSettings::from_mode(WikitextMode::Page, layout);
        let render_output =
            RenderService::render(ctx, wikitext.clone(), &page_info, &settings)
                .await
                .or_raise(make_error)?;

        Ok(GetBlueprintPageOutput {
            wikitext,
            render_output,
        })
    }

    /// Applies the exact category template to raw page source before rendering.
    ///
    /// This does not modify the stored revision source. A missing exact template
    /// is the identity operation, including when a named category has a default
    /// `_template` page available.
    pub(crate) async fn apply_page_template(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        category: Option<&str>,
        page: &str,
        wikitext: String,
    ) -> Result<String> {
        let Some(template) =
            Self::get_page_template(ctx, site_id, category, page).await?
        else {
            return Ok(wikitext);
        };

        Ok(compose_template(&template, &wikitext))
    }

    /// Gets the stored source for the exact template that applies to a page.
    pub(crate) async fn get_page_template(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        category: Option<&str>,
        page: &str,
    ) -> Result<Option<String>> {
        let template_page = &ctx.config().blueprint_page_template;
        let Some(template_slug) = exact_template_slug(category, page, template_page)
        else {
            return Ok(None);
        };

        Self::get_page_wikitext_optional(ctx, site_id, template_slug.as_ref())
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to get page template '{}' on site ID {}",
                        template_slug, site_id,
                    ),
                    ErrorType::BlueprintPage,
                )
            })
    }

    fn slugs_with_category<'a>(
        base_slug: &'a str,
        page_category: Option<&'a str>,
    ) -> Vec<Cow<'a, str>> {
        match split_category(base_slug) {
            // Has category explicitly, only use this exact slug.
            (Some(_), slug) => vec![cow!(slug)],

            // See if we can add a specific category.
            (None, slug) => {
                let mut slugs = Vec::with_capacity(2);
                slugs.push(cow!(slug));

                // If not in _default, add category-specific template to check first.
                if let Some(ref category) = page_category {
                    slugs.insert(0, Cow::Owned(format!("{category}:{slug}")));
                }

                slugs
            }
        }
    }

    async fn get_wikitext(
        ctx: &ServiceContext<'_>,
        slugs: &[Cow<'_, str>],
        translate_key: &str,
        site_id: i64,
        locales: &[LanguageIdentifier],
        page_info: &PageInfo<'_>,
    ) -> Result<String> {
        debug!(
            "Getting wikitext for blueprint page ({} slugs)",
            slugs.len(),
        );

        let make_error = || {
            Error::new(
                format!(
                    "failed to get blueprint wikitext for translation key '{}' for site ID {}",
                    translate_key, site_id,
                ),
                ErrorType::BlueprintPage,
            )
        };

        // Try all the pages listed.
        for slug in slugs {
            if let Some(text) =
                Self::get_page_wikitext_optional(ctx, site_id, slug.as_ref())
                    .await
                    .or_raise(make_error)?
            {
                return Ok(text);
            }
        }

        if translate_key.is_empty() {
            return Err(make_error().into());
        }

        // Use fallback string from localization
        let page = &page_info.page;
        let (category, full_slug) = match &page_info.category {
            Some(category) => (str!(category), format!("{category}:{page}")),
            None => (str!("_default"), str!(page)),
        };

        let mut args = FluentArgs::new();
        args.set("slug", FluentValue::String(Cow::Owned(full_slug)));
        args.set("page", fluent_str!(page));
        args.set("category", fluent_str!(category));
        args.set("domain", fluent_str!(ctx.config().main_domain_no_dot));

        let mut wikitext = ctx
            .localization()
            .translate(locales, translate_key, &args)?
            .into_owned();

        // Remove control chars because this string is intended to be wikitext
        strip_fluent_control_chars(&mut wikitext);

        Ok(wikitext)
    }

    async fn get_page_wikitext_optional(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        slug: &str,
    ) -> Result<Option<String>> {
        let Some(page) =
            PageService::get_optional(ctx, site_id, Reference::Slug(cow!(slug))).await?
        else {
            return Ok(None);
        };

        let revision =
            PageRevisionService::get_latest(ctx, site_id, page.page_id).await?;
        let text = TextService::get(ctx, &revision.wikitext_hash).await?;
        Ok(Some(text))
    }
}
