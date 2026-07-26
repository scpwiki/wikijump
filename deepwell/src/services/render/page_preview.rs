/*
 * services/render/page_preview.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use super::generator::COMPILED_GENERATOR;
use super::render_options::{RenderContext, RenderInnerOptions};
use super::service::{MAX_INCLUDE_EXPANSION_TOTAL, RenderInnerOutput, RenderService};
use super::{RenderOutput, UrlArguments};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::services::{ServiceContext, SiteService};
use crate::types::Reference;
use crate::utils::{locale_for_ftml, now};
use ftml::prelude::{Layout, PageInfo, ScoreValue, WikitextMode, WikitextSettings};
use std::borrow::Cow;

impl RenderService {
    pub async fn render_wikidot_page_preview(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        title: &str,
        wikitext: String,
    ) -> Result<RenderOutput> {
        let make_error = || {
            Error::new(
                format!("failed to render Wikidot page preview in site ID {site_id}"),
                ErrorType::Render,
            )
        };
        let site = SiteService::get(ctx, Reference::Id(site_id))
            .await
            .or_raise(make_error)?;
        let page_info = PageInfo {
            page: Cow::Borrowed(""),
            category: None,
            site: Cow::Owned(site.slug),
            title: Cow::Borrowed(title),
            alt_title: None,
            score: ScoreValue::Integer(0),
            tags: Vec::new(),
            language: Cow::Owned(locale_for_ftml(&site.locale).to_owned()),
        };
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let RenderInnerOutput {
            html_output,
            errors,
            compiled_hash,
        } = Box::pin(Self::render_inner(
            ctx,
            wikitext,
            &page_info,
            &settings,
            RenderInnerOptions {
                render_context: RenderContext::page_preview(site_id),
                max_include_expansions: MAX_INCLUDE_EXPANSION_TOTAL,
                trace: None,
                persist_compiled_text: false,
                url: UrlArguments::default(),
            },
        ))
        .await
        .or_raise(make_error)?;

        Ok(RenderOutput {
            html_output,
            errors,
            compiled_hash,
            compiled_at: now(),
            compiled_generator: COMPILED_GENERATOR.clone(),
        })
    }
}
