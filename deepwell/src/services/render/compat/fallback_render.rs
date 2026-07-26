/*
 * services/render/compat/fallback_render.rs
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

use super::super::ftml_page_existence::{
    WikidotCompatLinkTitleMap, collect_fallback_page_references,
};
use super::super::service::{
    MAX_FTML_COMPAT_COLLAPSIBLE_BLOCKS, MAX_FTML_COMPAT_DENSE_PARSE_SCORE,
    MAX_FTML_COMPAT_PARSE_BYTES, MIN_DENSE_FTML_COMPAT_RENDER_TIMEOUT_SECS,
    MIN_FTML_COMPAT_TABBED_FALLBACK_BYTES, MIN_FTML_COMPAT_TABBED_FALLBACK_MARKERS,
    MIN_FTML_COMPAT_TABBED_MARKERS, MIN_FTML_COMPAT_TABBED_RENDER_BYTES, RenderService,
    WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX, WIKIDOT_RATE_ANCHOR_REGEX,
    WIKIDOT_RATE_ANCHOR_SENTINEL_PREFIX, WIKIDOT_TABVIEW_INIT_SCRIPT,
    WIKIDOT_TABVIEW_SCRIPT, collect_wikidot_compat_empty_label_link_slugs,
    escape_list_pages_html_attr, escape_list_pages_html_text, push_escaped_html,
    render_native_list_inline_html_with_titles, render_native_list_inline_wikidot_spans,
};
use super::wikidot_inline_markers::{
    WikidotCompatInlineMarkerKind, next_wikidot_compat_inline_marker,
};
use super::{
    WikidotCompatibilityFallbackOutput, sanitize_wikidot_compat_inline_tag,
    scan_compat_code_blocks,
};
use crate::config::Config;
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::page_revision;
use crate::models::site::Model as SiteModel;
use crate::services::ServiceContext;
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::{LinkService, PageService};
use crate::types::Reference;
use crate::types::{Action, Permission, Resource};
use ftml::data::PageInfo;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::time::Duration;

impl RenderService {
    pub(in crate::services::render) fn should_use_wikidot_compatibility_fallback(
        wikitext: &str,
        page_info: &PageInfo<'_>,
    ) -> bool {
        if wikitext.len() > MAX_FTML_COMPAT_PARSE_BYTES {
            return true;
        }

        if Self::wikidot_compat_has_many_collapsible_blocks(wikitext) {
            return true;
        }

        if Self::wikidot_compat_has_pathological_tabview_shape(wikitext) {
            return true;
        }

        let page_name = page_info.page.as_ref();
        let title = page_info.title.as_ref();
        if !page_name.contains("scp-style-resource")
            && !title.contains("スタイルリソース")
        {
            return false;
        }

        Self::wikidot_compat_parse_complexity_score(wikitext)
            > MAX_FTML_COMPAT_DENSE_PARSE_SCORE
    }

    fn wikidot_compat_has_many_collapsible_blocks(wikitext: &str) -> bool {
        wikitext.matches("[[collapsible").count() > MAX_FTML_COMPAT_COLLAPSIBLE_BLOCKS
    }

    fn wikidot_compat_has_pathological_tabview_shape(wikitext: &str) -> bool {
        wikitext.len() >= MIN_FTML_COMPAT_TABBED_FALLBACK_BYTES
            && wikitext.matches("[[tab").count()
                >= MIN_FTML_COMPAT_TABBED_FALLBACK_MARKERS
    }

    pub(in crate::services::render) fn ftml_compat_render_timeout(
        config: &Config,
        wikitext: &str,
    ) -> Duration {
        let configured_timeout = config
            .preprocess_timeout
            .checked_add(config.render_timeout)
            .unwrap_or(config.render_timeout);
        let dense_compat_timeout =
            Duration::from_secs(MIN_DENSE_FTML_COMPAT_RENDER_TIMEOUT_SECS);

        if Self::wikidot_compat_needs_extended_render_deadline(wikitext) {
            configured_timeout.max(dense_compat_timeout)
        } else {
            configured_timeout
        }
    }

    fn wikidot_compat_needs_extended_render_deadline(wikitext: &str) -> bool {
        if wikitext.len() >= MAX_FTML_COMPAT_PARSE_BYTES {
            return false;
        }

        if Self::wikidot_compat_parse_complexity_score(wikitext)
            > MAX_FTML_COMPAT_DENSE_PARSE_SCORE
        {
            return true;
        }

        wikitext.len() >= MIN_FTML_COMPAT_TABBED_RENDER_BYTES
            && wikitext.matches("[[tab").count() >= MIN_FTML_COMPAT_TABBED_MARKERS
    }

    pub(in crate::services::render) fn wikidot_compat_parse_complexity_score(
        wikitext: &str,
    ) -> usize {
        let mut score = 0;

        for line in wikitext.lines() {
            let trimmed_start = line.trim_start();
            if trimmed_start.starts_with('*')
                || trimmed_start.starts_with('-')
                || trimmed_start.starts_with('#')
            {
                score += 100;
            }

            if line.contains('{')
                || line.contains('}')
                || line.contains(';')
                || line.contains("--")
            {
                score += 200;
            }

            score += line.matches("[[include").count() * 500;
            score += line.matches("[[").count() * 50;
        }

        score
    }

    pub(in crate::services::render) async fn load_wikidot_compat_fallback_link_titles(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        site_slug: &str,
        wikitext: &str,
    ) -> Result<WikidotCompatLinkTitleMap> {
        let page_refs = collect_fallback_page_references(wikitext);
        let page_existence =
            LinkService::resolve_page_existence(ctx, site_id, site_slug, &page_refs)
                .await?;
        let mut titles = WikidotCompatLinkTitleMap::new();
        titles.set_page_existence(site_slug.to_owned(), page_existence);

        let slugs = collect_wikidot_compat_empty_label_link_slugs(wikitext);
        if slugs.is_empty() {
            return Ok(titles);
        }

        let references = slugs
            .iter()
            .map(|slug| Reference::Slug(Cow::Borrowed(slug.as_str())))
            .collect::<Vec<_>>();
        let pages = PageService::get_pages(ctx, site_id, &references).await?;
        let mut pages_by_slug = BTreeMap::<String, Vec<_>>::new();
        for page in pages {
            pages_by_slug
                .entry(page.slug.clone())
                .or_default()
                .push(page);
        }

        let mut selected_pages = Vec::with_capacity(pages_by_slug.len());
        for (slug, mut pages) in pages_by_slug {
            let page = if pages.len() == 1 {
                pages.pop()
            } else {
                // Active duplicate slugs are permitted by PostgreSQL's NULL treatment in the existing uniqueness constraint, so preserve the former singular lookup's page and permission decision for that exceptional case while retaining one batch query for ordinary slugs.
                PageService::get_optional(ctx, site_id, Reference::from(slug.as_str()))
                    .await?
            };
            if let Some(page) = page {
                selected_pages.push(page);
            }
        }

        let mut category_permissions = BTreeMap::new();
        let mut viewable_pages = Vec::with_capacity(selected_pages.len());
        for page in selected_pages {
            let can_view = if let Some(can_view) =
                category_permissions.get(&page.page_category_id)
            {
                *can_view
            } else {
                let can_view = PermissionService::check_user_can(
                    ctx,
                    &CheckPermissionContext {
                        user_id: None,
                        site_id,
                        page_reference: Some(Reference::Id(page.page_id)),
                    },
                    Permission {
                        resource_type: Resource::Page,
                        resource_category: Some(Reference::Id(page.page_category_id)),
                        action: Action::View,
                    },
                )
                .await?;
                category_permissions.insert(page.page_category_id, can_view);
                can_view
            };
            if can_view && page.latest_revision_id.is_some() {
                viewable_pages.push(page);
            }
        }

        let revision_ids = viewable_pages
            .iter()
            .filter_map(|page| page.latest_revision_id)
            .collect::<Vec<_>>();
        let revisions = page_revision::Entity::find()
            .filter(page_revision::Column::RevisionId.is_in(revision_ids))
            .all(ctx.transaction())
            .await
            .or_raise(|| {
                Error::new(
                    "failed to batch fallback link titles",
                    ErrorType::PageRevision,
                )
            })?
            .into_iter()
            .map(|revision| (revision.revision_id, revision.title))
            .collect::<BTreeMap<_, _>>();

        for page in viewable_pages {
            let Some(title) = page
                .latest_revision_id
                .and_then(|revision_id| revisions.get(&revision_id))
                .map(|title| title.trim())
                .filter(|title| !title.is_empty())
            else {
                continue;
            };
            titles.insert(page.slug, title.to_owned());
        }

        Ok(titles)
    }

    pub(in crate::services::render) fn render_oversized_wikidot_compatibility_fallback(
        wikitext: &str,
        current_site: Option<&SiteModel>,
        config: &Config,
        current_page: &str,
        link_titles: Option<&WikidotCompatLinkTitleMap>,
    ) -> WikidotCompatibilityFallbackOutput {
        let localized =
            Self::localize_wikidot_local_file_urls(wikitext, current_site, config);
        if localized.lines().any(|line| {
            let marker = line.trim_start().to_ascii_lowercase();
            marker.starts_with("[[code") || marker.starts_with("[[collapsible")
        }) {
            return Self::render_wikidot_compatibility_fallback_output_for_context(
                &localized,
                Some(current_page),
                current_site.map(|site| site.slug.as_str()),
                link_titles,
            );
        }

        if Self::wikidot_compat_text_has_markup(&localized) {
            return Self::render_wikidot_compatibility_fallback_output_for_context(
                &localized,
                Some(current_page),
                current_site.map(|site| site.slug.as_str()),
                link_titles,
            );
        }

        let mut body = String::with_capacity(localized.len() + 96);
        body.push_str("<div class=\"wikidot-compat-fallback\"><pre>");
        push_escaped_html(&mut body, &localized);
        body.push_str("</pre></div>");
        WikidotCompatibilityFallbackOutput::body(body)
    }

    #[allow(dead_code)]
    pub(in crate::services::render) fn render_wikidot_compatibility_fallback_with_code_blocks(
        wikitext: &str,
    ) -> String {
        Self::render_wikidot_compatibility_fallback_with_code_blocks_for_page(
            wikitext, None,
        )
    }

    #[allow(dead_code)]
    fn render_wikidot_compatibility_fallback_with_code_blocks_for_page(
        wikitext: &str,
        current_page: Option<&str>,
    ) -> String {
        Self::render_wikidot_compatibility_fallback_with_code_blocks_for_context(
            wikitext,
            current_page,
            None,
        )
    }

    pub(in crate::services::render) fn render_wikidot_compatibility_fallback_with_code_blocks_for_context(
        wikitext: &str,
        current_page: Option<&str>,
        local_file_site_slug: Option<&str>,
    ) -> String {
        Self::render_wikidot_compatibility_fallback_output_for_context(
            wikitext,
            current_page,
            local_file_site_slug,
            None,
        )
        .body
    }

    pub(in crate::services::render) fn render_wikidot_compatibility_fallback_output_for_context(
        wikitext: &str,
        current_page: Option<&str>,
        local_file_site_slug: Option<&str>,
        link_titles: Option<&WikidotCompatLinkTitleMap>,
    ) -> WikidotCompatibilityFallbackOutput {
        let has_code_or_collapsible = wikitext.lines().any(|line| {
            let marker = line.trim_start().to_ascii_lowercase();
            marker.starts_with("[[code") || marker.starts_with("[[collapsible")
        });
        if !has_code_or_collapsible {
            let mut html_block_texts = Vec::new();
            if Self::wikidot_compat_text_has_markup(wikitext) {
                let mut body = String::with_capacity(wikitext.len() + 96);
                body.push_str("<div class=\"wikidot-compat-fallback\">");
                body.push_str(
                    &Self::render_wikidot_compat_fallback_text_html_with_blocks(
                        wikitext,
                        current_page,
                        local_file_site_slug,
                        link_titles,
                        &mut html_block_texts,
                    ),
                );
                body.push_str("</div>");
                return WikidotCompatibilityFallbackOutput {
                    body,
                    html_block_texts,
                    code_blocks: Vec::new(),
                };
            }

            let mut body = String::with_capacity(wikitext.len() + 96);
            body.push_str("<div class=\"wikidot-compat-fallback\"><pre>");
            push_escaped_html(&mut body, wikitext);
            body.push_str("</pre></div>");
            return WikidotCompatibilityFallbackOutput::body(body);
        }

        let mut body = String::with_capacity(wikitext.len() + 256);
        body.push_str("<div class=\"wikidot-compat-fallback\">");
        let mut html_block_texts = Vec::new();

        let mut text_chunk = String::new();
        let mut collapsible_depth = 0usize;
        let mut code_blocks = Vec::new();
        let mut collapsible_blocks = 0;
        let parsed_code_blocks = match scan_compat_code_blocks(wikitext) {
            Ok(blocks) => blocks,
            Err(_) => {
                let mut literal = String::with_capacity(wikitext.len() + 96);
                literal.push_str("<div class=\"wikidot-compat-fallback\"><pre>");
                push_escaped_html(&mut literal, wikitext);
                literal.push_str("</pre></div>");
                return WikidotCompatibilityFallbackOutput::body(literal);
            }
        };
        let mut parsed_code_blocks = parsed_code_blocks.into_iter().peekable();
        let mut skip_code_through_line = None;

        for (line_index, line) in wikitext.lines().enumerate() {
            if skip_code_through_line.is_some_and(|end_line| line_index <= end_line) {
                continue;
            }
            let trimmed = line.trim_start();
            let marker = trimmed.to_ascii_lowercase();
            if parsed_code_blocks
                .peek()
                .is_some_and(|block| block.start_line == line_index)
            {
                Self::push_wikidot_compat_fallback_text_chunk_for_page(
                    &mut body,
                    &mut text_chunk,
                    current_page,
                    local_file_site_slug,
                    link_titles,
                    &mut html_block_texts,
                );
                let block = parsed_code_blocks.next().expect("peeked code block");
                body.push_str(r#"<div class="code"><pre><code>"#);
                push_escaped_html(&mut body, &block.contents);
                body.push_str("</code></pre></div>");
                skip_code_through_line = Some(block.end_line);
                code_blocks.push(block.into_ftml());
                continue;
            }

            if marker.starts_with("[[collapsible") {
                Self::push_wikidot_compat_fallback_text_chunk_for_page(
                    &mut body,
                    &mut text_chunk,
                    current_page,
                    local_file_site_slug,
                    link_titles,
                    &mut html_block_texts,
                );
                Self::push_wikidot_compat_fallback_collapsible_open(&mut body, trimmed);
                collapsible_depth += 1;
                collapsible_blocks += 1;
                continue;
            }

            if marker.starts_with("[[/collapsible]]") {
                if collapsible_depth > 0 {
                    Self::push_wikidot_compat_fallback_text_chunk_for_page(
                        &mut body,
                        &mut text_chunk,
                        current_page,
                        local_file_site_slug,
                        link_titles,
                        &mut html_block_texts,
                    );
                    body.push_str("</div></div></div>");
                    collapsible_depth -= 1;
                } else {
                    text_chunk.push_str(line);
                    text_chunk.push('\n');
                }
                continue;
            }

            text_chunk.push_str(line);
            text_chunk.push('\n');
        }

        Self::push_wikidot_compat_fallback_text_chunk_for_page(
            &mut body,
            &mut text_chunk,
            current_page,
            local_file_site_slug,
            link_titles,
            &mut html_block_texts,
        );
        while collapsible_depth > 0 {
            body.push_str("</div></div></div>");
            collapsible_depth -= 1;
        }
        body.push_str("</div>");

        if code_blocks.is_empty() && collapsible_blocks == 0 {
            if Self::wikidot_compat_text_has_markup(wikitext) {
                let mut fallback = String::with_capacity(wikitext.len() + 96);
                fallback.push_str("<div class=\"wikidot-compat-fallback\">");
                fallback.push_str(
                    &Self::render_wikidot_compat_fallback_text_html_with_blocks(
                        wikitext,
                        current_page,
                        local_file_site_slug,
                        link_titles,
                        &mut html_block_texts,
                    ),
                );
                fallback.push_str("</div>");
                return WikidotCompatibilityFallbackOutput {
                    body: fallback,
                    html_block_texts,
                    code_blocks,
                };
            }

            let mut fallback = String::with_capacity(wikitext.len() + 96);
            fallback.push_str("<div class=\"wikidot-compat-fallback\"><pre>");
            push_escaped_html(&mut fallback, wikitext);
            fallback.push_str("</pre></div>");
            return WikidotCompatibilityFallbackOutput::body(fallback);
        }

        WikidotCompatibilityFallbackOutput {
            body,
            html_block_texts,
            code_blocks,
        }
    }

    #[allow(dead_code)]
    fn push_wikidot_compat_fallback_text_chunk(body: &mut String, chunk: &mut String) {
        let mut html_block_texts = Vec::new();
        Self::push_wikidot_compat_fallback_text_chunk_for_page(
            body,
            chunk,
            None,
            None,
            None,
            &mut html_block_texts,
        );
    }

    fn push_wikidot_compat_fallback_text_chunk_for_page(
        body: &mut String,
        chunk: &mut String,
        current_page: Option<&str>,
        local_file_site_slug: Option<&str>,
        link_titles: Option<&WikidotCompatLinkTitleMap>,
        html_block_texts: &mut Vec<String>,
    ) {
        if chunk.is_empty() {
            return;
        }

        let cleaned = Self::strip_wikidot_comments_from_text(chunk);
        let text = cleaned.trim_end_matches('\n');
        if text.is_empty() {
            chunk.clear();
            return;
        }
        if Self::wikidot_compat_text_has_markup(text) {
            body.push_str(&Self::render_wikidot_compat_fallback_text_html_with_blocks(
                text,
                current_page,
                local_file_site_slug,
                link_titles,
                html_block_texts,
            ));
        } else {
            body.push_str("<pre>");
            push_escaped_html(body, text);
            body.push_str("</pre>");
        }
        chunk.clear();
    }

    fn wikidot_compat_text_has_markup(text: &str) -> bool {
        text.contains("[[div")
            || text.contains("[[/div]]")
            || text.contains("[[tab")
            || text.contains("[[/tab")
            || text.contains("[[size")
            || text.contains("[[/size")
            || text.contains("[[embed]]")
            || text.contains("[[html]]")
            || text.contains("[[=]]")
            || text.contains("[[/=]]")
            || text.lines().any(|line| {
                let trimmed = line.trim();
                trimmed == "////"
                    || trimmed == "@@ @@"
                    || Self::wikidot_compat_horizontal_rule_marker(trimmed)
            })
            || text.contains("[[=image")
            || text.contains("[[[")
            || text.contains("[[*")
            || text.contains("[http")
            || text.contains("**")
            || text.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX)
    }

    fn wikidot_compat_html_sentinel_marker(marker: &str) -> bool {
        let Some(token) = marker
            .strip_prefix(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX)
            .and_then(|value| value.strip_suffix('X'))
        else {
            return false;
        };

        token.len() == 32 && token.chars().all(|character| character.is_ascii_hexdigit())
            || token.split_once('I').is_some_and(|(namespace, index)| {
                namespace.len() == 32
                    && namespace
                        .chars()
                        .all(|character| character.is_ascii_hexdigit())
                    && !index.is_empty()
                    && index.chars().all(|character| character.is_ascii_digit())
            })
    }

    #[allow(dead_code)]
    fn render_wikidot_compat_fallback_text_html(text: &str) -> String {
        Self::render_wikidot_compat_fallback_text_html_for_page(text, None)
    }

    #[allow(dead_code)]
    fn render_wikidot_compat_fallback_text_html_for_page(
        text: &str,
        current_page: Option<&str>,
    ) -> String {
        Self::render_wikidot_compat_fallback_text_html_for_context(
            text,
            current_page,
            None,
        )
    }

    fn render_wikidot_compat_fallback_text_html_for_context(
        text: &str,
        current_page: Option<&str>,
        local_file_site_slug: Option<&str>,
    ) -> String {
        let mut html_block_texts = Vec::new();
        Self::render_wikidot_compat_fallback_text_html_with_blocks(
            text,
            current_page,
            local_file_site_slug,
            None,
            &mut html_block_texts,
        )
    }

    fn render_wikidot_compat_fallback_text_html_with_blocks(
        text: &str,
        current_page: Option<&str>,
        local_file_site_slug: Option<&str>,
        link_titles: Option<&WikidotCompatLinkTitleMap>,
        html_block_texts: &mut Vec<String>,
    ) -> String {
        let text = Self::strip_wikidot_comments_from_text(text);
        let mut output = String::with_capacity(text.len());
        let mut paragraph = String::new();
        let mut tabview_open = false;
        let mut tab_open = false;
        let mut size_depth = 0usize;
        let mut embed_body: Option<String> = None;
        let mut html_body: Option<String> = None;
        let mut tabview_body: Option<String> = None;
        let mut center_depth = 0usize;

        for line in text.lines() {
            let trimmed = line.trim();
            if tabview_body.is_some() {
                if trimmed.eq_ignore_ascii_case("[[/tabview]]") {
                    let body = tabview_body.take().unwrap_or_default();
                    Self::push_wikidot_compat_fallback_paragraph_for_page(
                        &mut output,
                        &mut paragraph,
                        current_page,
                        link_titles,
                    );
                    output.push_str(&Self::render_wikidot_compat_fallback_tabview_html(
                        &body,
                        current_page,
                        local_file_site_slug,
                        link_titles,
                        html_block_texts,
                    ));
                } else if let Some(body) = tabview_body.as_mut() {
                    body.push_str(line);
                    body.push('\n');
                }
                continue;
            }

            if html_body.is_some() {
                if trimmed.eq_ignore_ascii_case("[[/html]]") {
                    let body = html_body.take().unwrap_or_default();
                    if let Some(iframe) = Self::wikidot_compat_html_block_iframe(
                        current_page,
                        html_block_texts.len() + 1,
                    ) {
                        html_block_texts.push(body.trim_matches('\n').to_owned());
                        Self::push_wikidot_compat_fallback_paragraph_for_page(
                            &mut output,
                            &mut paragraph,
                            current_page,
                            link_titles,
                        );
                        output.push_str(&iframe);
                    } else {
                        paragraph.push_str("[[html]]\n");
                        paragraph.push_str(&body);
                        paragraph.push_str("[[/html]]\n");
                    }
                } else if let Some(body) = html_body.as_mut() {
                    body.push_str(line);
                    body.push('\n');
                }
                continue;
            }

            if embed_body.is_some() {
                if trimmed.eq_ignore_ascii_case("[[/embed]]") {
                    let body = embed_body.take().unwrap_or_default();
                    if let Some(iframe) = Self::allowed_wikidot_embed_iframe(body.trim())
                    {
                        output.push_str("<div>");
                        output.push_str(&iframe);
                        output.push_str("</div>");
                    } else {
                        paragraph.push_str("[[embed]]\n");
                        paragraph.push_str(&body);
                        paragraph.push_str("[[/embed]]\n");
                    }
                } else if let Some(body) = embed_body.as_mut() {
                    body.push_str(line);
                    body.push('\n');
                }
                continue;
            }

            if trimmed.eq_ignore_ascii_case("[[embed]]") {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                    link_titles,
                );
                embed_body = Some(String::new());
                continue;
            }

            if trimmed.eq_ignore_ascii_case("[[html]]") {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                    link_titles,
                );
                html_body = Some(String::new());
                continue;
            }

            if Self::wikidot_compat_html_sentinel_marker(trimmed) {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                    link_titles,
                );
                output.push_str(trimmed);
                continue;
            }

            if trimmed.eq_ignore_ascii_case("[[=]]") {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                    link_titles,
                );
                output.push_str(r#"<div style="text-align: center;">"#);
                center_depth += 1;
                continue;
            }

            if trimmed.eq_ignore_ascii_case("[[/=]]") {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                    link_titles,
                );
                if center_depth > 0 {
                    output.push_str("</div>");
                    center_depth -= 1;
                } else {
                    paragraph.push_str(line);
                    paragraph.push('\n');
                }
                continue;
            }

            if trimmed == "////" {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                    link_titles,
                );
                output.push_str("<br>");
                continue;
            }

            if Self::wikidot_compat_horizontal_rule_marker(trimmed) {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                    link_titles,
                );
                output.push_str("<hr>");
                continue;
            }

            if trimmed == "@@ @@" {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                    link_titles,
                );
                output.push_str(r#"<span style="white-space: pre-wrap;"> </span><br>"#);
                continue;
            }

            if trimmed.eq_ignore_ascii_case("[[tabview]]") {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                    link_titles,
                );
                tabview_body = Some(String::new());
                continue;
            }

            if let Some(title) = Self::wikidot_compat_tab_title(trimmed) {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                    link_titles,
                );
                if !tabview_open {
                    output.push_str(WIKIDOT_TABVIEW_SCRIPT);
                    output.push_str(
                        r#"<div class="yui-navset wikidot-compat-tabview"><div class="yui-content">"#,
                    );
                    tabview_open = true;
                }
                if tab_open {
                    output.push_str("</div>");
                }
                output.push_str(r#"<div class="wikidot-compat-tab"><h3>"#);
                output.push_str(
                    &Self::render_wikidot_compat_fallback_inline_html_for_page(
                        title,
                        current_page,
                        link_titles,
                    ),
                );
                output.push_str("</h3>");
                tab_open = true;
                continue;
            }

            if trimmed.eq_ignore_ascii_case("[[/tab]]") {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                    link_titles,
                );
                if tab_open {
                    output.push_str("</div>");
                    tab_open = false;
                }
                continue;
            }

            if trimmed.eq_ignore_ascii_case("[[/tabview]]") {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                    link_titles,
                );
                if tab_open {
                    output.push_str("</div>");
                    tab_open = false;
                }
                if tabview_open {
                    output.push_str("</div></div>");
                    output.push_str(WIKIDOT_TABVIEW_INIT_SCRIPT);
                    tabview_open = false;
                }
                continue;
            }

            if let Some(size) = Self::wikidot_compat_size_value(trimmed) {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                    link_titles,
                );
                output.push_str(r#"<span style="font-size: "#);
                output.push_str(&escape_list_pages_html_attr(&size));
                output.push_str(";\">");
                size_depth += 1;
                continue;
            }

            if trimmed.eq_ignore_ascii_case("[[/size]]") {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                    link_titles,
                );
                if size_depth > 0 {
                    output.push_str("</span>");
                    size_depth -= 1;
                }
                continue;
            }

            if let Some(image) = Self::render_wikidot_compat_fallback_image(
                trimmed,
                current_page,
                local_file_site_slug,
            ) {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                    link_titles,
                );
                output.push_str(&image);
                continue;
            }

            if let Some(rate_html) = Self::render_wikidot_compat_rate_widget_line(trimmed)
            {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                    link_titles,
                );
                output.push_str(&rate_html);
                continue;
            }

            if let Some(attributes) = Self::wikidot_compat_div_attributes(trimmed) {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                    link_titles,
                );
                output.push_str("<div");
                output.push_str(&attributes);
                output.push('>');
                continue;
            }

            if trimmed.eq_ignore_ascii_case("[[/div]]") {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                    link_titles,
                );
                output.push_str("</div>");
                continue;
            }

            paragraph.push_str(line);
            paragraph.push('\n');
        }

        if let Some(body) = embed_body {
            paragraph.push_str("[[embed]]\n");
            paragraph.push_str(&body);
        }
        if let Some(body) = html_body {
            paragraph.push_str("[[html]]\n");
            paragraph.push_str(&body);
        }
        if let Some(body) = tabview_body {
            paragraph.push_str("[[tabview]]\n");
            paragraph.push_str(&body);
        }
        Self::push_wikidot_compat_fallback_paragraph_for_page(
            &mut output,
            &mut paragraph,
            current_page,
            link_titles,
        );
        while size_depth > 0 {
            output.push_str("</span>");
            size_depth -= 1;
        }
        if tab_open {
            output.push_str("</div>");
        }
        if tabview_open {
            output.push_str("</div></div>");
            output.push_str(WIKIDOT_TABVIEW_INIT_SCRIPT);
        }
        while center_depth > 0 {
            output.push_str("</div>");
            center_depth -= 1;
        }
        output
    }

    fn render_wikidot_compat_fallback_tabview_html(
        text: &str,
        current_page: Option<&str>,
        local_file_site_slug: Option<&str>,
        link_titles: Option<&WikidotCompatLinkTitleMap>,
        html_block_texts: &mut Vec<String>,
    ) -> String {
        let Some(tabs) = Self::parse_wikidot_compat_fallback_tabs(text) else {
            let mut output = String::new();
            output.push_str("<p>");
            output.push_str(&Self::render_wikidot_compat_fallback_inline_html_for_page(
                "[[tabview]]",
                current_page,
                link_titles,
            ));
            output.push_str("</p>");
            output.push_str(&Self::render_wikidot_compat_fallback_text_html_with_blocks(
                text,
                current_page,
                local_file_site_slug,
                link_titles,
                html_block_texts,
            ));
            output.push_str("<p>");
            output.push_str(&Self::render_wikidot_compat_fallback_inline_html_for_page(
                "[[/tabview]]",
                current_page,
                link_titles,
            ));
            output.push_str("</p>");
            return output;
        };

        let mut output = String::new();
        output.push_str(WIKIDOT_TABVIEW_SCRIPT);
        output.push_str(
            r#"<div class="yui-navset yui-navset-top wikidot-compat-tabview">"#,
        );
        output.push_str(r#"<ul class="yui-nav">"#);
        for (index, (title, _)) in tabs.iter().enumerate() {
            if index == 0 {
                output.push_str(
                    r#"<li class="selected" title="active"><a href="javascript:;"><em>"#,
                );
            } else {
                output.push_str(r#"<li><a href="javascript:;"><em>"#);
            }
            output.push_str(&Self::render_wikidot_compat_fallback_inline_html_for_page(
                title,
                current_page,
                link_titles,
            ));
            output.push_str("</em></a></li>");
        }
        output.push_str(r#"</ul><div class="yui-content">"#);

        for (index, (_, body)) in tabs.iter().enumerate() {
            if index == 0 {
                output.push_str(r#"<div style="display: block;">"#);
            } else {
                output.push_str(r#"<div style="display:none">"#);
            }
            output.push_str(&Self::render_wikidot_compat_fallback_text_html_with_blocks(
                body,
                current_page,
                local_file_site_slug,
                link_titles,
                html_block_texts,
            ));
            output.push_str("</div>");
        }

        output.push_str("</div></div>");
        output.push_str(WIKIDOT_TABVIEW_INIT_SCRIPT);
        output
    }

    fn parse_wikidot_compat_fallback_tabs(text: &str) -> Option<Vec<(String, String)>> {
        let mut tabs = Vec::new();
        let mut current_title: Option<String> = None;
        let mut current_body = String::new();
        let mut prelude = String::new();

        for line in text.lines() {
            let trimmed = line.trim();
            if let Some(title) = Self::wikidot_compat_tab_title(trimmed) {
                if let Some(title) = current_title.take() {
                    tabs.push((title, std::mem::take(&mut current_body)));
                } else if !prelude.trim().is_empty() {
                    return None;
                }
                current_title = Some(title.to_owned());
                continue;
            }

            if trimmed.eq_ignore_ascii_case("[[/tab]]") {
                let title = current_title.take()?;
                tabs.push((title, std::mem::take(&mut current_body)));
                continue;
            }

            if current_title.is_some() {
                current_body.push_str(line);
                current_body.push('\n');
            } else {
                prelude.push_str(line);
                prelude.push('\n');
            }
        }

        if let Some(title) = current_title {
            tabs.push((title, current_body));
        }

        if tabs.is_empty() || !prelude.trim().is_empty() {
            return None;
        }

        Some(tabs)
    }

    fn wikidot_compat_horizontal_rule_marker(marker: &str) -> bool {
        marker.len() >= 4 && marker.chars().all(|character| character == '-')
    }

    fn wikidot_compat_html_block_iframe(
        current_page: Option<&str>,
        block_index: usize,
    ) -> Option<String> {
        let current_page = current_page?;
        let src = format!("/{current_page}/html/{block_index}");
        Some(format!(
            r#"<iframe src="{}" allowtransparency="true" frameborder="0" class="html-block-iframe"></iframe>"#,
            escape_list_pages_html_attr(&src)
        ))
    }

    fn strip_wikidot_comments_from_text(text: &str) -> String {
        let mut output = String::with_capacity(text.len());
        let mut rest = text;
        let mut in_comment = false;

        while !rest.is_empty() {
            if in_comment {
                let Some(end) = rest.find("--]") else {
                    break;
                };
                rest = &rest[end + "--]".len()..];
                in_comment = false;
                continue;
            }

            let Some(start) = rest.find("[!--") else {
                output.push_str(rest);
                break;
            };
            output.push_str(&rest[..start]);
            rest = &rest[start + "[!--".len()..];
            in_comment = true;
        }

        output
    }

    fn wikidot_compat_tab_title(marker: &str) -> Option<&str> {
        let lower = marker.to_ascii_lowercase();
        if !lower.starts_with("[[tab ") || !marker.ends_with("]]") {
            return None;
        }

        Some(marker[6..marker.len() - 2].trim())
    }

    fn wikidot_compat_size_value(marker: &str) -> Option<String> {
        let lower = marker.to_ascii_lowercase();
        if !lower.starts_with("[[size ") || !marker.ends_with("]]") {
            return None;
        }

        let value = marker[7..marker.len() - 2].trim();
        Self::wikidot_compat_valid_css_size(value).then(|| value.to_owned())
    }

    fn wikidot_compat_valid_css_size(value: &str) -> bool {
        !value.is_empty()
            && value.chars().all(|character| {
                character.is_ascii_alphanumeric()
                    || matches!(character, '.' | '%' | '-' | '_')
            })
    }

    fn render_wikidot_compat_fallback_image(
        marker: &str,
        current_page: Option<&str>,
        local_file_site_slug: Option<&str>,
    ) -> Option<String> {
        let lower = marker.to_ascii_lowercase();
        let (center, rest) = if lower.starts_with("[[=image ") {
            (true, &marker[9..])
        } else if lower.starts_with("[[image ") {
            (false, &marker[8..])
        } else {
            return None;
        };
        if !marker.ends_with("]]") {
            return None;
        }

        let inner = &rest[..rest.len() - 2];
        let split_at = inner.find(char::is_whitespace).unwrap_or(inner.len());
        let target = inner[..split_at].trim();
        if target.is_empty()
            || target
                .chars()
                .any(|character| matches!(character, '<' | '>' | '"' | '\''))
        {
            return None;
        }

        let mut classes = vec!["image".to_owned()];
        if let Some(class) = Self::wikidot_marker_attr(marker, "class")
            && class.chars().all(|character| {
                character.is_ascii_alphanumeric()
                    || matches!(character, ' ' | '-' | '_' | ':')
            })
        {
            classes.push(class);
        }
        if let Some(size) = Self::wikidot_marker_attr(marker, "size")
            && size.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
            })
        {
            classes.push(format!("image-size-{size}"));
        }

        let src = Self::wikidot_compat_fallback_image_src(
            target,
            current_page,
            local_file_site_slug,
        );
        let mut image = format!(
            r#"<img src="{src}" class="{class}">"#,
            src = escape_list_pages_html_attr(&src),
            class = escape_list_pages_html_attr(&classes.join(" ")),
        );
        if let Some(style) = Self::wikidot_marker_attr(marker, "style") {
            image = image.replace(
                ">",
                &format!(r#" style="{}">"#, escape_list_pages_html_attr(&style)),
            );
        }

        if center {
            Some(format!(
                r#"<div class="image-container aligncenter">{image}</div>"#
            ))
        } else {
            Some(image)
        }
    }

    fn wikidot_compat_fallback_image_src(
        target: &str,
        current_page: Option<&str>,
        local_file_site_slug: Option<&str>,
    ) -> String {
        if target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with('/')
        {
            return target.to_owned();
        }

        let page = current_page.unwrap_or(".");
        if current_page.is_some()
            && let Some(site_slug) = local_file_site_slug
            && Self::wikidot_compat_valid_local_file_site_slug(site_slug)
        {
            return format!(
                "https://{site_slug}.wdfiles.com/local--files/{page}/{target}"
            );
        }

        format!("/local--files/{page}/{target}")
    }

    fn wikidot_compat_valid_local_file_site_slug(value: &str) -> bool {
        !value.is_empty()
            && value
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-')
    }

    fn render_wikidot_compat_rate_widget_line(line: &str) -> Option<String> {
        let inner = line
            .strip_prefix("[[div class=\"page-rate-widget-box\"]]")?
            .strip_suffix("[[/div]]")?;
        let mut anchors = Vec::new();
        let protected = WIKIDOT_RATE_ANCHOR_REGEX
            .replace_all(inner, |captures: &regex::Captures<'_>| {
                let marker =
                    format!("{WIKIDOT_RATE_ANCHOR_SENTINEL_PREFIX}{}X", anchors.len());
                anchors.push(format!(
                    r#"<a href="javascript:;" onclick="{}" title="{}">{}</a>"#,
                    escape_list_pages_html_attr(&captures["onclick"]),
                    escape_list_pages_html_attr(&captures["title"]),
                    escape_list_pages_html_text(&captures["label"]),
                ));
                marker
            })
            .into_owned();
        let mut inner_html = render_native_list_inline_wikidot_spans(&protected);
        for (index, anchor) in anchors.iter().enumerate() {
            let marker = format!("{WIKIDOT_RATE_ANCHOR_SENTINEL_PREFIX}{index}X");
            inner_html = inner_html.replace(&marker, anchor);
        }

        Some(format!(
            r#"<div class="page-rate-widget-box">{inner_html}</div>"#
        ))
    }

    fn wikidot_compat_div_attributes(marker: &str) -> Option<String> {
        Self::wikidot_div_attributes(marker, true)
    }

    pub(in crate::services::render) fn wikidot_residual_div_attributes(
        marker: &str,
    ) -> Option<String> {
        Self::wikidot_div_attributes(marker, false)
    }

    fn wikidot_div_attributes(marker: &str, prefix_ids: bool) -> Option<String> {
        if !marker.ends_with("]]") {
            return None;
        }

        let lower = marker.to_ascii_lowercase();
        if lower != "[[div]]"
            && lower != "[[div_]]"
            && !lower.starts_with("[[div ")
            && !lower.starts_with("[[div_ ")
        {
            return None;
        }

        let inner = marker
            .strip_prefix("[[div")
            .and_then(|value| value.strip_suffix("]]"))?
            .trim();

        if inner.is_empty() || inner == "_" {
            return Some(String::new());
        }

        let mut attributes = String::new();
        if let Some(class) = Self::wikidot_marker_attr(marker, "class") {
            if !class.chars().all(|character| {
                character.is_ascii_alphanumeric()
                    || matches!(character, ' ' | '-' | '_' | ':')
            }) {
                return None;
            }
            attributes.push_str(r#" class=""#);
            attributes.push_str(&escape_list_pages_html_attr(&class));
            attributes.push('"');
        }

        if let Some(id) = Self::wikidot_marker_attr(marker, "id") {
            if !id.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
            }) {
                return None;
            }
            attributes.push_str(r#" id=""#);
            if prefix_ids {
                attributes.push_str("u-");
            }
            attributes.push_str(&escape_list_pages_html_attr(&id));
            attributes.push('"');
        }

        if let Some(style) = Self::wikidot_marker_attr(marker, "style") {
            attributes.push_str(r#" style=""#);
            attributes.push_str(&escape_list_pages_html_attr(&style));
            attributes.push('"');
        }

        (!attributes.is_empty()).then_some(attributes)
    }

    #[allow(dead_code)]
    fn push_wikidot_compat_fallback_paragraph(body: &mut String, paragraph: &mut String) {
        Self::push_wikidot_compat_fallback_paragraph_for_page(
            body, paragraph, None, None,
        );
    }

    fn push_wikidot_compat_fallback_paragraph_for_page(
        body: &mut String,
        paragraph: &mut String,
        current_page: Option<&str>,
        link_titles: Option<&WikidotCompatLinkTitleMap>,
    ) {
        let text = paragraph.trim_matches('\n');
        if !text.trim().is_empty() {
            body.push_str("<p>");
            body.push_str(&Self::render_wikidot_compat_fallback_inline_html_for_page(
                text,
                current_page,
                link_titles,
            ));
            body.push_str("</p>");
        }
        paragraph.clear();
    }

    #[allow(dead_code)]
    fn render_wikidot_compat_fallback_inline_html(value: &str) -> String {
        Self::render_wikidot_compat_fallback_inline_html_for_page(value, None, None)
    }

    fn render_wikidot_compat_fallback_inline_html_for_page(
        value: &str,
        _current_page: Option<&str>,
        link_titles: Option<&WikidotCompatLinkTitleMap>,
    ) -> String {
        let mut output = String::with_capacity(value.len());
        let mut strong = false;
        for segment in value.split("**") {
            if strong {
                output.push_str("<strong>");
            }
            Self::push_wikidot_compat_fallback_inline_segment(
                &mut output,
                segment,
                link_titles,
            );
            if strong {
                output.push_str("</strong>");
            }
            strong = !strong;
        }
        output
    }

    fn push_wikidot_compat_fallback_inline_segment(
        output: &mut String,
        value: &str,
        link_titles: Option<&WikidotCompatLinkTitleMap>,
    ) {
        let mut rest = value;
        while let Some(start) = rest.find('<') {
            let (before, after_start) = rest.split_at(start);
            output.push_str(&Self::render_wikidot_compat_inline_text_segment(
                before,
                link_titles,
            ));
            if let Some(end) = after_start.find('>') {
                let (tag, after_tag) = after_start.split_at(end + 1);
                if let Some(tag) = sanitize_wikidot_compat_inline_tag(tag) {
                    output.push_str(&tag);
                } else {
                    output.push_str(&escape_list_pages_html_text(tag));
                }
                rest = after_tag;
            } else {
                output.push_str(&Self::render_wikidot_compat_inline_text_segment(
                    after_start,
                    link_titles,
                ));
                return;
            }
        }
        output.push_str(&Self::render_wikidot_compat_inline_text_segment(
            rest,
            link_titles,
        ));
    }

    fn render_wikidot_compat_inline_text_segment(
        value: &str,
        link_titles: Option<&WikidotCompatLinkTitleMap>,
    ) -> String {
        let html = Self::render_wikidot_compat_fallback_inline_markup(value, link_titles);
        Self::render_wikidot_compat_inline_size_markers(&html)
    }

    pub(in crate::services::render) fn render_wikidot_compat_fallback_inline_markup(
        value: &str,
        link_titles: Option<&WikidotCompatLinkTitleMap>,
    ) -> String {
        let mut output = String::with_capacity(value.len());
        let mut rest = value;

        while let Some(marker) = next_wikidot_compat_inline_marker(rest) {
            let (before, marker_start) = rest.split_at(marker.start);
            output.push_str(&render_native_list_inline_html_with_titles(
                before,
                link_titles,
            ));
            let marker_len = marker.end - marker.start;

            match marker.kind {
                WikidotCompatInlineMarkerKind::Color => {
                    let Some(pipe_offset) = marker_start[..marker_len].find('|') else {
                        output.push_str(&render_native_list_inline_html_with_titles(
                            &marker_start[..marker_len],
                            link_titles,
                        ));
                        rest = &marker_start[marker_len..];
                        continue;
                    };
                    let color = marker_start[2..pipe_offset].trim();
                    let inner = &marker_start[pipe_offset + 1..marker_len - 2];
                    output.push_str(r#"<span style="color: "#);
                    output.push_str(&escape_list_pages_html_attr(
                        &Self::wikidot_compat_color_value(color),
                    ));
                    output.push_str(r#";">"#);
                    output.push_str(&Self::render_wikidot_compat_fallback_inline_markup(
                        inner,
                        link_titles,
                    ));
                    output.push_str("</span>");
                }
                WikidotCompatInlineMarkerKind::Italic => {
                    let inner = &marker_start[2..marker_len - 2];
                    output.push_str("<em>");
                    output.push_str(&Self::render_wikidot_compat_fallback_inline_markup(
                        inner,
                        link_titles,
                    ));
                    output.push_str("</em>");
                }
                WikidotCompatInlineMarkerKind::Underline => {
                    let inner = &marker_start[2..marker_len - 2];
                    output.push_str("<u>");
                    output.push_str(&Self::render_wikidot_compat_fallback_inline_markup(
                        inner,
                        link_titles,
                    ));
                    output.push_str("</u>");
                }
            }

            rest = &marker_start[marker_len..];
        }

        output.push_str(&render_native_list_inline_html_with_titles(
            rest,
            link_titles,
        ));
        output
    }

    fn wikidot_compat_color_value(value: &str) -> String {
        let color = value.trim();
        if color.starts_with('#') || !Self::wikidot_compat_is_hex_color(color) {
            color.to_owned()
        } else {
            format!("#{color}")
        }
    }

    fn wikidot_compat_is_hex_color(value: &str) -> bool {
        matches!(value.len(), 3 | 6)
            && value.chars().all(|character| character.is_ascii_hexdigit())
    }

    fn render_wikidot_compat_inline_size_markers(value: &str) -> String {
        let mut output = String::with_capacity(value.len());
        let mut rest = value;

        while let Some(start) = rest.find("[[") {
            let (before, marker_start) = rest.split_at(start);
            output.push_str(before);
            let lower = marker_start.to_ascii_lowercase();
            if lower.starts_with("[[/size]]") {
                output.push_str("</span>");
                rest = &marker_start[9..];
                continue;
            }
            if lower.starts_with("[[size ")
                && let Some(end) = marker_start.find("]]")
            {
                let marker = &marker_start[..end + 2];
                if let Some(size) = Self::wikidot_compat_size_value(marker) {
                    output.push_str(r#"<span style="font-size: "#);
                    output.push_str(&escape_list_pages_html_attr(&size));
                    output.push_str(";\">");
                    rest = &marker_start[end + 2..];
                    continue;
                }
            }
            output.push_str("[[");
            rest = &marker_start[2..];
        }

        output.push_str(rest);
        output
    }

    fn push_wikidot_compat_fallback_collapsible_open(body: &mut String, marker: &str) {
        let show = Self::wikidot_marker_attr(marker, "show")
            .unwrap_or_else(|| "+ show block".to_owned());
        let hide = Self::wikidot_marker_attr(marker, "hide")
            .unwrap_or_else(|| "– hide block".to_owned());
        let folded = Self::wikidot_marker_attr(marker, "folded")
            .map(|value| !value.eq_ignore_ascii_case("no") && value != "0")
            .unwrap_or(true);

        body.push_str(r#"<div class="collapsible-block">"#);
        if folded {
            body.push_str(r#"<div class="collapsible-block-folded"><a class="collapsible-block-link" href="javascript:;">"#);
        } else {
            body.push_str(r#"<div class="collapsible-block-folded" style="display:none"><a class="collapsible-block-link" href="javascript:;">"#);
        }
        push_escaped_html(body, &show);
        body.push_str("</a></div>");
        if folded {
            body.push_str(
                r#"<div class="collapsible-block-unfolded" style="display:none">"#,
            );
        } else {
            body.push_str(r#"<div class="collapsible-block-unfolded">"#);
        }
        body.push_str(r#"<div class="collapsible-block-unfolded-link"><a class="collapsible-block-link" href="javascript:;">"#);
        push_escaped_html(body, &hide);
        body.push_str("</a></div><div class=\"collapsible-block-content\">");
    }

    fn wikidot_marker_attr(marker: &str, name: &str) -> Option<String> {
        let pattern = format!("{name}=\"");
        let start = marker.find(&pattern)? + pattern.len();
        let rest = &marker[start..];
        let end = rest.find('"')?;
        Some(rest[..end].to_owned())
    }
}
