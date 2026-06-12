/*
 * services/render/service.rs
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

use super::prelude::*;
use crate::hash::TextHash;
use crate::models::site::Model as SiteModel;
use crate::models::wikidot_user::{self, Entity as WikidotUser};
use crate::services::page_query::{
    CategoriesSelector, DateSelector, FoundPageFields, FoundPageRow, IncludedCategories,
    OrderBySelector, OrderProperty, PageParentSelector, PageQuery, PageTypeSelector,
    PaginationSelector, RangeSelector, TagCondition,
};
use crate::services::settings::{NavigationPageWikitext, SettingsService};
use crate::services::text_block::{
    MIME_HTML, TextBlock, TextBlockService, mime_for_language,
};
use crate::services::{PageQueryService, PageRevisionService, SiteService, TextService};
use crate::types::{PageId, TextBlockType};
use ftml::data::PageRef;
use ftml::includes::{FetchedPage, IncludeRef};
use ftml::prelude::*;
use ftml::tree::{CodeBlock, VariableMap};
use regex::Regex;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::LazyLock;
use tokio::time::timeout;

#[derive(Debug)]
pub struct RenderService;

const MAX_INCLUDE_EXPANSION_DEPTH: usize = 8;
const INCLUDE_VARIABLE_OPEN_SENTINEL: &str = "__WIKIJUMP_INCLUDE_VAR_OPEN__";
const INCLUDE_VARIABLE_CLOSE_SENTINEL: &str = "__WIKIJUMP_INCLUDE_VAR_CLOSE__";
const WIKIDOT_EMBED_IFRAME_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTEMBEDIFRAME";
const WIKIDOT_LOCAL_INTERWIKI_BASE: &str = "/-/wikidot-interwiki";

static INCLUDE_VARIABLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\$(?P<name>[a-zA-Z0-9_\-]+)\}").unwrap());
static LISTPAGES_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?is)\[\[module\s+ListPages(?P<head>[^\]]*)\]\](?P<body>.*?)\[\[/module\]\]",
    )
    .unwrap()
});
static RATE_MODULE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)\[\[module\s+Rate(?P<head>[^\]]*)\]\]").unwrap());
static LISTPAGES_ARGUMENT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)(?P<key>[A-Za-z][A-Za-z0-9_\-]*)\s*=\s*(?:"(?P<double>[^"]*)"|'(?P<single>[^']*)'|(?P<bare>[^\s\]]+))"#)
        .unwrap()
});
static LISTPAGES_VARIABLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"%%(?P<name>[A-Za-z0-9_]+)%%").unwrap());
static WIKIDOT_EMAIL_SPAN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"<span class="wiki-email" style="visibility: visible;"><a href="mailto:([^"]+)">([^<]+)</a></span>"#,
    )
    .unwrap()
});
static WIKIDOT_EMBED_PARAGRAPH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)<p>\[\[embed\]\]<br/?>(.*?)<br/?>\[\[/embed\]\]</p>"#).unwrap()
});
static WIKIDOT_RAW_EMBED_IFRAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)\[\[embed\]\]\s*(?P<iframe><iframe\b[^>]*></iframe>)\s*\[\[/embed\]\]"#,
    )
    .unwrap()
});
static WIKIDOT_RENDERED_ANCHOR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)<a href="[^"]+">(.*?)</a>"#).unwrap());
static WIKIDOT_STYLEFRAME_IFRAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"^<iframe src="(?P<src>//interwiki\.scpwiki\.com/styleFrame\.html\?[^"]+)" style="display: none"></iframe>$"#,
    )
    .unwrap()
});
static WIKIDOT_INTERWIKI_FRAME_IFRAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"^<iframe src="(?P<src>//interwiki\.scpwiki\.com/interwikiFrame\.html\?[^"]+)" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>$"#,
    )
    .unwrap()
});
static WIKIDOT_LOCAL_FILE_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?P<quote>["'])https?://(?P<host>[A-Za-z0-9.-]+)(?::[0-9]+)?(?P<path>/local--(?:files|code)/[^"'<>\s]+)"#,
    )
    .unwrap()
});
static WIKIDOT_LOCAL_FILE_CSS_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)(?P<prefix>url\(\s*["']?)https?://(?P<host>[A-Za-z0-9.-]+)(?::[0-9]+)?(?P<path>/local--(?:files|code)/[^"')<>\s]+)"#,
    )
    .unwrap()
});
static CSS_IMPORT_LINE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?im)^(?P<indent>[ \t]*)@import(?P<body>[^\n]*)$"#).unwrap()
});
static CSS_ABSOLUTE_URL_HOST_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)https?://(?P<host>[A-Za-z0-9.-]+)(?::[0-9]+)?"#).unwrap()
});
static CSS_EXTERNAL_URL_FUNCTION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)url\(\s*(?P<quote>["']?)https?://(?P<host>[A-Za-z0-9.-]+)(?::[0-9]+)?(?P<path>[^"')\s]*)["']?\s*\)"#,
    )
    .unwrap()
});

impl RenderService {
    pub async fn render(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
    ) -> Result<RenderOutput> {
        let wikitext_len = wikitext.len();
        let make_error = || {
            Error::new(
                format!(
                    "failed to run parse and render (wikitext {} bytes, info {:?}, settings {:?})",
                    wikitext_len, page_info, settings,
                ),
                ErrorType::Render,
            )
        };

        let RenderInnerOutput {
            html_output,
            errors,
            compiled_hash,
        } = Self::render_inner(ctx, wikitext, page_info, settings, RenderContext::none())
            .await
            .or_raise(make_error)?;

        Ok(RenderOutput {
            html_output,
            errors,
            compiled_hash,
            compiled_at: now(),
            compiled_generator: FTML_VERSION.clone(),
        })
    }

    pub async fn render_page(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        layout: Layout,
        PageId {
            site_id,
            category_id,
            page_id,
        }: PageId,
    ) -> Result<RenderPageOutput> {
        let page_settings = WikitextSettings::from_mode(WikitextMode::Page, layout);
        let nav_settings = WikitextSettings::from_mode(WikitextMode::PageNav, layout);

        let wikitext_len = wikitext.len();
        let make_error = || {
            Error::new(
                format!(
                    "failed to run parse and render for page ID {} in site ID {} (wikitext {} bytes, info {:?}, layout {})",
                    page_id,
                    site_id,
                    wikitext_len,
                    page_info,
                    layout.description(),
                ),
                ErrorType::Render,
            )
        };

        let RenderInnerOutput {
            html_output,
            errors,
            compiled_hash: compiled_body_html_hash,
        } = Self::render_inner(
            ctx,
            wikitext,
            page_info,
            &page_settings,
            RenderContext::page(site_id, page_id),
        )
        .await
        .or_raise(make_error)?;

        let NavigationPageWikitext {
            top_bar_page_wikitext,
            side_bar_page_wikitext,
        } = SettingsService::get_nav_page_wikitext(ctx, site_id, Some(category_id))
            .await
            .or_raise(make_error)?;

        let render_nav_page = |wikitext| async {
            match wikitext {
                Some(wikitext) => {
                    // Navigation pages render in the context of the viewed page, but
                    // must not update the viewed page's hosted text blocks.
                    //
                    // Also note that the page_info for nav pages is the page being displayed,
                    // not the nav pages themselves. This means that any variables or blocks
                    // which depend on the current page (e.g. page slug, tags), which reflect
                    // the page being viewed.
                    let result = Self::render_inner(
                        ctx,
                        wikitext,
                        page_info,
                        &nav_settings,
                        RenderContext::page_nav(site_id, page_id),
                    )
                    .await;

                    match result {
                        Ok(RenderInnerOutput { compiled_hash, .. }) => {
                            Ok(Some(compiled_hash))
                        }
                        Err(error) => Err(error),
                    }
                }

                // No nav page
                None => Ok(None),
            }
        };

        let (top_bar_render_result, side_bar_render_result) = join!(
            render_nav_page(top_bar_page_wikitext),
            render_nav_page(side_bar_page_wikitext),
        );
        let (compiled_top_bar_html_hash, compiled_side_bar_html_hash) =
            raise_multiple!(top_bar_render_result, side_bar_render_result; make_error);

        Ok(RenderPageOutput {
            html_output,
            errors,
            compiled_body_html_hash,
            compiled_top_bar_html_hash,
            compiled_side_bar_html_hash,
            compiled_at: now(),
            compiled_generator: FTML_VERSION.clone(),
        })
    }

    async fn render_inner(
        ctx: &ServiceContext<'_>,
        mut wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        render_context: RenderContext,
    ) -> Result<RenderInnerOutput> {
        let config = ctx.config();
        let RenderContext {
            current_site_id,
            current_page_id,
            text_block_page_id,
        } = render_context;

        let make_error =
            || Error::new("failed to perform render operation", ErrorType::Render);
        let current_site = match current_site_id {
            Some(site_id) => Some(
                SiteService::get(ctx, Reference::Id(site_id))
                    .await
                    .or_raise(make_error)?,
            ),
            None => None,
        };

        Self::remove_preview_component_separator_markers(&mut wikitext);

        let IncludeExpansion {
            wikitext: expanded_wikitext,
            included_pages,
        } = Self::expand_includes(
            ctx,
            wikitext,
            page_info.site.as_ref(),
            settings,
            current_site_id,
        )
        .await
        .or_raise(make_error)?;
        wikitext = expanded_wikitext;
        Self::remove_unresolved_variable_iftags_blocks(&mut wikitext);
        wikitext = Self::expand_list_pages(
            ctx,
            wikitext,
            settings,
            current_site_id,
            current_page_id,
        )
        .await
        .or_raise(make_error)?;
        wikitext = Self::expand_rate_modules(wikitext, page_info, settings);
        let wikidot_embed_iframes = Self::protect_wikidot_embed_iframes(&mut wikitext);

        // We isolate the actual tasks for rendering,
        // allowing us to time it out if it takes too long.
        //
        // The preprocess step has to be distinct for borrowing reasons,
        // since we want to do the processing for non-ftml work
        // outside the timeout guards.

        let tokens = timeout(config.preprocess_timeout, async {
            // TODO include
            ftml::preprocess(&mut wikitext);
            ftml::tokenize(&wikitext)
        })
        .await
        .or_raise(|| {
            Error::new(
                "failed to preprocess and tokenize due to timeout",
                ErrorType::RenderTimeout,
            )
        })?;

        let (tree, html_output, errors) = timeout(config.render_timeout, async {
            let result = ftml::parse(&tokens, page_info, settings);
            let (tree, errors) = result.into();
            let mut html_output = HtmlRender.render(&tree, page_info, settings);
            html_output.body = Self::restore_protected_wikidot_embed_iframes(
                html_output.body,
                &wikidot_embed_iframes,
            );
            html_output.body = Self::restore_wikidot_render_compatibility(
                &html_output.body,
                current_site.as_ref(),
                config,
            );
            apply_basalt_shell_compatibility(&mut html_output.body);
            html_output.backlinks.included_pages.extend(included_pages);
            (tree, html_output, errors)
        })
        .await
        .or_raise(|| {
            Error::new(
                "failed to parse and render due to timeout",
                ErrorType::RenderTimeout,
            )
        })?;

        // Insert compiled HTML into text table
        let compiled_hash = TextService::create(ctx, html_output.body.clone())
            .await
            .or_raise(make_error)?;

        // Set up the hosted text blocks
        //
        // This only applies for published pages, in any other
        // rendering context and we should skip this step.

        if let Some(page_id) = text_block_page_id {
            // It's possible to render a page without doing text blocks
            // (e.g. blueprint pages), but all cases where text blocks
            // are done are pages.
            debug_assert_eq!(settings.mode, WikitextMode::Page);

            // [[html]]
            let html_blocks: Vec<TextBlock> = tree
                .html_blocks
                .iter()
                .map(|html| TextBlock {
                    text: html,
                    text_type: None,
                    mime: MIME_HTML,
                    name: None,
                })
                .collect();

            TextBlockService::add_blocks(ctx, page_id, TextBlockType::Html, &html_blocks)
                .await
                .or_raise(make_error)?;

            // [[code]]
            let code_block_texts: Vec<String> = tree
                .code_blocks
                .iter()
                .map(|CodeBlock { contents, .. }| {
                    Self::restore_wikidot_code_block_compatibility(
                        contents,
                        current_site.as_ref(),
                        config,
                    )
                })
                .collect();
            let code_blocks: Vec<TextBlock> = tree
                .code_blocks
                .iter()
                .zip(code_block_texts.iter())
                .map(|(CodeBlock { language, name, .. }, contents)| TextBlock {
                    text: contents,
                    text_type: language.as_deref(),
                    mime: mime_for_language(language),
                    name: name.as_deref(),
                })
                .collect();

            TextBlockService::add_blocks(ctx, page_id, TextBlockType::Code, &code_blocks)
                .await
                .or_raise(make_error)?;
        }

        // Build and return
        Ok(RenderInnerOutput {
            html_output,
            errors,
            compiled_hash,
        })
    }

    fn restore_wikidot_render_compatibility(
        html: &str,
        current_site: Option<&SiteModel>,
        config: &Config,
    ) -> String {
        let html = Self::restore_wikidot_rendered_embed_iframes(html);
        let html = Self::restore_wikidot_email_obfuscation(&html);
        Self::localize_wikidot_local_file_urls(&html, current_site, config)
    }

    fn restore_wikidot_code_block_compatibility(
        code: &str,
        current_site: Option<&SiteModel>,
        config: &Config,
    ) -> String {
        let code = Self::localize_wikidot_local_file_urls(code, current_site, config);
        Self::suppress_external_css_dependencies(&code, config)
    }

    fn remove_unresolved_variable_iftags_blocks(wikitext: &mut String) {
        while let Some(open_marker_start) = wikitext.find("[[ift{$") {
            let name_start = open_marker_start + "[[ift{$".len();
            let Some(name_end_offset) = wikitext[name_start..].find("}gs") else {
                break;
            };
            let name_end = name_start + name_end_offset;
            let name = &wikitext[name_start..name_end];

            if !is_include_variable_name(name) {
                break;
            }

            let open_end_search_start = name_end + "}gs".len();
            let Some(open_end_offset) = wikitext[open_end_search_start..].find("]]")
            else {
                break;
            };
            let body_start = open_end_search_start + open_end_offset + "]]".len();
            let close_marker = format!("[[/ift{{${name}}}gs]]");
            let Some(close_marker_offset) = wikitext[body_start..].find(&close_marker)
            else {
                break;
            };

            let block_start = wikitext[..open_marker_start]
                .rfind('\n')
                .map_or(0, |index| index + 1);
            let close_marker_start = body_start + close_marker_offset;
            let close_marker_end = close_marker_start + close_marker.len();
            let block_end = wikitext[close_marker_end..]
                .find('\n')
                .map_or(wikitext.len(), |offset| close_marker_end + offset + 1);

            let body_end =
                Self::quoted_marker_body_end(wikitext, body_start, close_marker_start);
            let body = &wikitext[body_start..body_end];
            let replacement = if body.contains("[[iftags]]") {
                String::new()
            } else {
                body.to_owned()
            };

            wikitext.replace_range(block_start..block_end, &replacement);
        }

        Self::remove_collapsed_basalt_iftags_blocks(wikitext);
    }

    fn remove_preview_component_separator_markers(wikitext: &mut String) {
        while let Some((before_start, before_end, after_start, after_end)) =
            Self::find_preview_component_separator_markers(wikitext)
        {
            let mut cleaned = String::with_capacity(
                wikitext.len() - (before_end - before_start) - (after_end - after_start),
            );
            cleaned.push_str(&wikitext[..before_start]);
            cleaned.push_str(&wikitext[before_end..after_start]);
            cleaned.push_str(&wikitext[after_end..]);
            *wikitext = cleaned;
        }
    }

    fn find_preview_component_separator_markers(
        wikitext: &str,
    ) -> Option<(usize, usize, usize, usize)> {
        let lines = Self::wikitext_line_ranges(wikitext);

        for include_start_line in 0..lines.len() {
            let include_start = Self::trim_wikitext_line(lines[include_start_line].2);
            if !include_start.starts_with("[[include") {
                continue;
            }

            let mut include_text = String::new();
            let mut include_end_line = include_start_line;
            loop {
                let line = lines[include_end_line].2;
                include_text.push_str(line);
                if line.contains("]]") {
                    break;
                }

                include_end_line += 1;
                if include_end_line >= lines.len() {
                    break;
                }
            }

            if !include_text
                .to_ascii_lowercase()
                .contains("component:preview")
            {
                continue;
            }

            if include_start_line == 0 || include_end_line + 1 >= lines.len() {
                continue;
            }

            let before = lines[include_start_line - 1];
            let after = lines[include_end_line + 1];
            if Self::trim_wikitext_line(before.2) == "====="
                && Self::trim_wikitext_line(after.2) == "====="
            {
                return Some((before.0, before.1, after.0, after.1));
            }
        }

        None
    }

    fn wikitext_line_ranges(wikitext: &str) -> Vec<(usize, usize, &str)> {
        let mut lines = Vec::new();
        let mut start = 0;

        for (index, character) in wikitext.char_indices() {
            if character == '\n' {
                let end = index + character.len_utf8();
                lines.push((start, end, &wikitext[start..end]));
                start = end;
            }
        }

        if start < wikitext.len() {
            lines.push((start, wikitext.len(), &wikitext[start..]));
        }

        lines
    }

    fn trim_wikitext_line(line: &str) -> &str {
        line.trim_end_matches(['\r', '\n']).trim()
    }

    fn remove_collapsed_basalt_iftags_blocks(wikitext: &mut String) {
        const ACTIVE_OPEN_MARKER: &str = "[[iftags -basalt-override]]";
        const ACTIVE_CLOSE_MARKER: &str = "[[/iftags]]";
        const INNER_OPEN_MARKER: &str = "[[iftags]]";

        while let Some(open_marker_start) = wikitext.find(ACTIVE_OPEN_MARKER) {
            let outer_body_start = open_marker_start + ACTIVE_OPEN_MARKER.len();
            let Some(first_close_offset) =
                wikitext[outer_body_start..].find(ACTIVE_CLOSE_MARKER)
            else {
                break;
            };
            let first_close_start = outer_body_start + first_close_offset;
            let next_close_start = first_close_start + ACTIVE_CLOSE_MARKER.len();
            let Some(second_close_offset) =
                wikitext[next_close_start..].find(ACTIVE_CLOSE_MARKER)
            else {
                break;
            };
            let second_close_start = next_close_start + second_close_offset;
            let second_close_end = second_close_start + ACTIVE_CLOSE_MARKER.len();
            let block_start = wikitext[..open_marker_start]
                .rfind('\n')
                .map_or(0, |index| index + 1);
            let block_end = wikitext[second_close_end..]
                .find('\n')
                .map_or(wikitext.len(), |offset| second_close_end + offset + 1);

            let first_body_end = Self::quoted_marker_body_end(
                wikitext,
                outer_body_start,
                first_close_start,
            );
            let outer_body = &wikitext[outer_body_start..first_body_end];
            let inner_body_start = outer_body
                .find(INNER_OPEN_MARKER)
                .map(|offset| outer_body_start + offset + INNER_OPEN_MARKER.len())
                .unwrap_or(outer_body_start);
            let replacement = wikitext[inner_body_start..first_body_end].to_owned();

            wikitext.replace_range(block_start..block_end, &replacement);
        }

        const MALFORMED_OPEN_MARKER: &str = "[[ifta gs -basalt-override]]";
        const MALFORMED_CLOSE_MARKER: &str = "[[/ifta gs]]";

        while let Some(open_marker_start) = wikitext.find(MALFORMED_OPEN_MARKER) {
            let body_start = open_marker_start + MALFORMED_OPEN_MARKER.len();
            let Some(close_marker_offset) =
                wikitext[body_start..].find(MALFORMED_CLOSE_MARKER)
            else {
                break;
            };
            let block_start = wikitext[..open_marker_start]
                .rfind('\n')
                .map_or(0, |index| index + 1);
            let close_marker_start = body_start + close_marker_offset;
            let close_marker_end = close_marker_start + MALFORMED_CLOSE_MARKER.len();
            let block_end = wikitext[close_marker_end..]
                .find('\n')
                .map_or(wikitext.len(), |offset| close_marker_end + offset + 1);

            wikitext.replace_range(block_start..block_end, "");
        }
    }

    fn quoted_marker_body_end(
        wikitext: &str,
        body_start: usize,
        marker_start: usize,
    ) -> usize {
        let bytes = wikitext.as_bytes();
        if marker_start > body_start && bytes.get(marker_start - 1) == Some(&b'>') {
            marker_start - 1
        } else {
            marker_start
        }
    }

    fn protect_wikidot_embed_iframes(wikitext: &mut String) -> Vec<String> {
        let mut iframes = Vec::new();
        let protected = WIKIDOT_RAW_EMBED_IFRAME_REGEX
            .replace_all(wikitext, |captures: &regex::Captures<'_>| {
                let Some(iframe_match) = captures.name("iframe") else {
                    return captures.get(0).map_or("", |m| m.as_str()).to_owned();
                };
                let iframe = iframe_match.as_str().trim();
                let Some(iframe) = Self::allowed_wikidot_embed_iframe(iframe) else {
                    return captures.get(0).map_or("", |m| m.as_str()).to_owned();
                };

                let marker =
                    format!("{WIKIDOT_EMBED_IFRAME_SENTINEL_PREFIX}{}X", iframes.len());
                iframes.push(iframe);
                marker
            })
            .into_owned();
        *wikitext = protected;
        iframes
    }

    fn restore_protected_wikidot_embed_iframes(
        mut html: String,
        iframes: &[String],
    ) -> String {
        for (index, iframe) in iframes.iter().enumerate() {
            let marker = format!("{WIKIDOT_EMBED_IFRAME_SENTINEL_PREFIX}{index}X");
            html = html.replace(&marker, iframe);
        }
        html
    }

    fn restore_wikidot_rendered_embed_iframes(html: &str) -> String {
        WIKIDOT_EMBED_PARAGRAPH_REGEX
            .replace_all(html, |captures: &regex::Captures<'_>| {
                let block = captures.get(1).map_or("", |m| m.as_str());
                let decoded = Self::decode_rendered_embed_block(block);

                let Some(iframe) = Self::allowed_wikidot_embed_iframe(&decoded) else {
                    return captures.get(0).map_or("", |m| m.as_str()).to_string();
                };

                format!("<p>{iframe}</p>")
            })
            .into_owned()
    }

    fn allowed_wikidot_embed_iframe(iframe: &str) -> Option<String> {
        if let Some(captures) = WIKIDOT_STYLEFRAME_IFRAME_REGEX.captures(iframe) {
            return Some(Self::rewrite_wikidot_interwiki_iframe_src(
                iframe,
                &captures["src"],
                "styleFrame.html",
            ));
        }

        if let Some(captures) = WIKIDOT_INTERWIKI_FRAME_IFRAME_REGEX.captures(iframe) {
            return Some(Self::rewrite_wikidot_interwiki_iframe_src(
                iframe,
                &captures["src"],
                "interwikiFrame.html",
            ));
        }

        None
    }

    fn rewrite_wikidot_interwiki_iframe_src(
        iframe: &str,
        original_src: &str,
        local_file_name: &str,
    ) -> String {
        let query = original_src.split_once('?').map_or("", |(_, query)| query);
        let local_src =
            format!("{WIKIDOT_LOCAL_INTERWIKI_BASE}/{local_file_name}?{query}");

        iframe.replace(original_src, &local_src)
    }

    fn decode_rendered_embed_block(block: &str) -> String {
        let without_anchors = WIKIDOT_RENDERED_ANCHOR_REGEX.replace_all(block, "$1");
        let text = without_anchors
            .replace("<br>", "")
            .replace("<br/>", "")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#34;", "\"")
            .replace("&#39;", "'")
            .replace("&amp;", "&");

        text.trim().to_owned()
    }

    fn restore_wikidot_email_obfuscation(html: &str) -> String {
        WIKIDOT_EMAIL_SPAN_REGEX
            .replace_all(html, |captures: &regex::Captures<'_>| {
                let href_email = captures.get(1).map_or("", |m| m.as_str());
                let visible_email = captures.get(2).map_or("", |m| m.as_str());

                if href_email != visible_email {
                    return captures.get(0).map_or("", |m| m.as_str()).to_string();
                }

                let (email, trailing) =
                    Self::split_trailing_email_punctuation(visible_email);
                let Some(obfuscated) = Self::wikidot_obfuscated_email(email) else {
                    return captures.get(0).map_or("", |m| m.as_str()).to_string();
                };

                format!(r#"<span class="wiki-email">{obfuscated}</span>{trailing}"#)
            })
            .into_owned()
    }

    fn wikidot_obfuscated_email(email: &str) -> Option<String> {
        let (user, domain) = email.split_once('@')?;

        if user.is_empty() || domain.is_empty() {
            return None;
        }

        let reversed_user: String = user.chars().rev().collect();
        let reversed_domain: String = domain.chars().rev().collect();

        Some(format!(
            "{reversed_domain}|{reversed_user}#{reversed_domain}|{reversed_user}"
        ))
    }

    fn split_trailing_email_punctuation(email: &str) -> (&str, &str) {
        let email_end = email
            .trim_end_matches(|character| {
                matches!(character, '.' | ',' | ';' | ':' | '!' | '?')
            })
            .len();

        email.split_at(email_end)
    }

    fn localize_wikidot_local_file_urls(
        html: &str,
        current_site: Option<&SiteModel>,
        config: &Config,
    ) -> String {
        let Some(current_site) = current_site else {
            return html.to_owned();
        };

        let html = WIKIDOT_LOCAL_FILE_URL_REGEX
            .replace_all(html, |captures: &regex::Captures<'_>| {
                let host = &captures["host"];
                let path = &captures["path"];
                let Some(localized) = Self::localized_wikidot_local_file_url(
                    host,
                    path,
                    current_site,
                    config,
                ) else {
                    return captures.get(0).map_or("", |m| m.as_str()).to_owned();
                };

                format!("{}{}", &captures["quote"], localized)
            })
            .into_owned();

        WIKIDOT_LOCAL_FILE_CSS_URL_REGEX
            .replace_all(&html, |captures: &regex::Captures<'_>| {
                let host = &captures["host"];
                let path = &captures["path"];
                let Some(localized) = Self::localized_wikidot_local_file_url(
                    host,
                    path,
                    current_site,
                    config,
                ) else {
                    return captures.get(0).map_or("", |m| m.as_str()).to_owned();
                };

                format!("{}{}", &captures["prefix"], localized)
            })
            .into_owned()
    }

    fn localized_wikidot_local_file_url(
        host: &str,
        path: &str,
        current_site: &SiteModel,
        config: &Config,
    ) -> Option<String> {
        let site_slug = local_file_host_site_slug(host, config)?;
        if !site_accepts_wikidot_local_asset_slug(current_site, &site_slug) {
            return None;
        }

        Some(format!(
            "https://{}{}{}",
            current_site.slug, config.files_domain, path,
        ))
    }

    fn suppress_external_css_dependencies(css: &str, config: &Config) -> String {
        let css = CSS_IMPORT_LINE_REGEX
            .replace_all(css, |captures: &regex::Captures<'_>| {
                let body = &captures["body"];
                let has_external_url = CSS_ABSOLUTE_URL_HOST_REGEX
                    .captures_iter(body)
                    .any(|url_captures| {
                        !css_dependency_host_is_local(&url_captures["host"], config)
                    });
                if !has_external_url {
                    return captures.get(0).map_or("", |m| m.as_str()).to_owned();
                }

                format!(
                    "{}/* wikijump local render: omitted external @import */",
                    &captures["indent"],
                )
            })
            .into_owned();

        CSS_EXTERNAL_URL_FUNCTION_REGEX
            .replace_all(&css, |captures: &regex::Captures<'_>| {
                let host = &captures["host"];
                if css_dependency_host_is_local(host, config) {
                    return captures.get(0).map_or("", |m| m.as_str()).to_owned();
                }

                r#"url("data:,")"#.to_owned()
            })
            .into_owned()
    }

    async fn expand_includes(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        current_site_slug: &str,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
    ) -> Result<IncludeExpansion> {
        let Some(current_site_id) = current_site_id else {
            return Ok(IncludeExpansion {
                wikitext,
                included_pages: Vec::new(),
            });
        };

        if !settings.enable_page_syntax {
            return Ok(IncludeExpansion {
                wikitext,
                included_pages: Vec::new(),
            });
        }

        let mut expansion = Self::expand_includes_for_site(
            ctx,
            wikitext,
            current_site_id,
            current_site_slug.to_owned(),
            settings,
            0,
        )
        .await?;
        unprotect_include_variables(&mut expansion.wikitext);

        Ok(expansion)
    }

    fn expand_includes_for_site<'a>(
        ctx: &'a ServiceContext<'_>,
        wikitext: String,
        current_site_id: i64,
        current_site_slug: String,
        settings: &'a WikitextSettings,
        depth: usize,
    ) -> Pin<Box<dyn Future<Output = Result<IncludeExpansion>> + Send + 'a>> {
        Box::pin(async move {
            let mut includes = Vec::new();
            ftml::include(
                &wikitext,
                settings,
                CollectingIncluder {
                    includes: &mut includes,
                },
                include_error,
            )?;

            if includes.is_empty() {
                let mut wikitext = wikitext;
                protect_include_variables(&mut wikitext);
                return Ok(IncludeExpansion {
                    wikitext,
                    included_pages: Vec::new(),
                });
            }

            if depth >= MAX_INCLUDE_EXPANSION_DEPTH {
                return Err(Error::new(
                    format!(
                        "include expansion exceeded maximum depth {}",
                        MAX_INCLUDE_EXPANSION_DEPTH,
                    ),
                    ErrorType::Render,
                )
                .into());
            }

            let mut fetched_pages = Vec::with_capacity(includes.len());
            let mut nested_included_pages = Vec::with_capacity(includes.len());

            for include in &includes {
                let source = Self::fetch_include_source(
                    ctx,
                    current_site_id,
                    &current_site_slug,
                    include.page_ref(),
                )
                .await?;

                let Some(mut source) = source else {
                    fetched_pages.push(None);
                    nested_included_pages.push(Vec::new());
                    continue;
                };

                apply_include_variables(&mut source.wikitext, include);
                Self::remove_unresolved_variable_iftags_blocks(&mut source.wikitext);

                let expansion = Self::expand_includes_for_site(
                    ctx,
                    source.wikitext,
                    source.site_id,
                    source.site_slug,
                    settings,
                    depth + 1,
                )
                .await?;

                fetched_pages.push(Some(expansion.wikitext));
                nested_included_pages.push(expansion.included_pages);
            }

            let (mut expanded, direct_included_pages) = ftml::include(
                &wikitext,
                settings,
                PreparedIncluder {
                    pages: fetched_pages,
                },
                include_error,
            )?;

            protect_include_variables(&mut expanded);

            let mut included_pages = Vec::new();
            for (page_ref, nested_pages) in
                direct_included_pages.into_iter().zip(nested_included_pages)
            {
                included_pages.push(page_ref);
                included_pages.extend(nested_pages);
            }

            Ok(IncludeExpansion {
                wikitext: expanded,
                included_pages,
            })
        })
    }

    async fn fetch_include_source(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        current_site_slug: &str,
        page_ref: &PageRef,
    ) -> Result<Option<IncludeSource>> {
        match page_ref.site() {
            Some(site_slug) if site_slug != current_site_slug => {
                let current_site = SiteService::get_optional(
                    ctx,
                    Reference::Id(current_site_id),
                )
                .await
                .or_raise(|| {
                    Error::new(
                        format!(
                            "failed to get current include site ID {current_site_id}"
                        ),
                        ErrorType::Site,
                    )
                })?;
                let current_site_matches = current_site
                    .as_ref()
                    .is_some_and(|site| site_matches_wikidot_slug(site, site_slug));

                if current_site_matches {
                    if let Some(source) = Self::fetch_include_source_from_site(
                        ctx,
                        current_site_id,
                        current_site_slug,
                        page_ref.page(),
                    )
                    .await?
                    {
                        return Ok(Some(source));
                    }
                }

                let Some(site) =
                    SiteService::get_optional(ctx, Reference::from(site_slug))
                        .await
                        .or_raise(|| {
                            Error::new(
                                format!("failed to get include site '{}'", site_slug),
                                ErrorType::Site,
                            )
                        })?
                else {
                    return Ok(None);
                };

                Self::fetch_include_source_from_site(
                    ctx,
                    site.site_id,
                    &site.slug,
                    page_ref.page(),
                )
                .await
            }
            _ => {
                Self::fetch_include_source_from_site(
                    ctx,
                    current_site_id,
                    current_site_slug,
                    page_ref.page(),
                )
                .await
            }
        }
    }

    async fn fetch_include_source_from_site(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        site_slug: &str,
        page_slug: &str,
    ) -> Result<Option<IncludeSource>> {
        if let Some(wikitext) = PageRevisionService::get_wikitext_optional(
            ctx,
            site_id,
            Reference::from(page_slug),
        )
        .await?
        {
            return Ok(Some(IncludeSource {
                site_id,
                site_slug: site_slug.to_owned(),
                wikitext,
            }));
        }

        Ok(None)
    }

    async fn expand_list_pages(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        current_page_id: Option<i64>,
    ) -> Result<String> {
        let (Some(current_site_id), Some(current_page_id)) =
            (current_site_id, current_page_id)
        else {
            return Ok(wikitext);
        };

        if !settings.enable_page_syntax {
            return Ok(wikitext);
        }

        let mut expanded = String::with_capacity(wikitext.len());
        let mut cursor = 0;

        for captures in LISTPAGES_MODULE_REGEX.captures_iter(&wikitext) {
            let mtch = captures.get(0).unwrap();
            expanded.push_str(&wikitext[cursor..mtch.start()]);

            let Some(arguments) =
                parse_list_pages_arguments(captures.name("head").unwrap().as_str())
            else {
                expanded.push_str(mtch.as_str());
                cursor = mtch.end();
                continue;
            };

            let body = captures.name("body").unwrap().as_str();
            if !list_pages_body_variables_supported(body) {
                expanded.push_str(mtch.as_str());
                cursor = mtch.end();
                continue;
            }

            let replacement = Self::render_list_pages_block(
                ctx,
                current_site_id,
                current_page_id,
                arguments,
                body,
            )
            .await?;
            expanded.push_str(&replacement);
            cursor = mtch.end();
        }

        expanded.push_str(&wikitext[cursor..]);
        Ok(expanded)
    }

    fn expand_rate_modules(
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
    ) -> String {
        if !settings.enable_page_syntax {
            return wikitext;
        }

        let replacement = render_read_only_rate_module(page_info.score);
        RATE_MODULE_REGEX
            .replace_all(&wikitext, replacement.as_str())
            .into_owned()
    }

    async fn render_list_pages_block(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        current_page_id: i64,
        arguments: ListPagesArguments,
        body: &str,
    ) -> Result<String> {
        let ListPagesArguments {
            category_all,
            categories,
            excluded_categories,
            any_tags,
            all_tags,
            no_tags,
            created_by,
            order,
            limit,
            per_page,
            range,
        } = arguments;
        let included_categories = if category_all {
            IncludedCategories::All
        } else {
            IncludedCategories::List(&categories)
        };

        let query = PageQuery {
            current_page_id,
            current_site_id,
            queried_site_id: None,
            page_type: PageTypeSelector::All,
            categories: CategoriesSelector {
                included_categories,
                excluded_categories: &excluded_categories,
            },
            tags: TagCondition {
                any_present: &any_tags,
                all_present: &all_tags,
                none_present: &no_tags,
            },
            page_parent: PageParentSelector::DifferentParents,
            contains_outgoing_links: &[],
            creation_date: DateSelector::FromPresent {
                start: time::OffsetDateTime::UNIX_EPOCH,
            },
            update_date: DateSelector::FromPresent {
                start: time::OffsetDateTime::UNIX_EPOCH,
            },
            author: &created_by,
            score: &[],
            votes: &[],
            offset: 0,
            range,
            name: None,
            slug: None,
            data_form_fields: &[],
            order,
            pagination: PaginationSelector {
                limit,
                per_page,
                reversed: false,
            },
            variables: &[],
            fields: FoundPageFields {
                title: true,
                slug: true,
                created_by: list_pages_body_uses_variable(body, "created_by"),
                score: list_pages_body_uses_variable(body, "rating"),
                ..Default::default()
            },
        };

        let pages = PageQueryService::find(ctx, query).await?;
        let total = pages.total();
        let created_by_names = if list_pages_body_uses_variable(body, "created_by") {
            Self::load_wikidot_user_names(ctx, &pages.pages).await?
        } else {
            BTreeMap::new()
        };
        let mut output = String::from("[[div class=\"list-pages-box\"]]\n");

        for (index, page) in pages.pages.iter().enumerate() {
            output.push_str("[[div class=\"list-pages-item\"]]\n");
            output.push_str(&substitute_list_pages_variables(
                body,
                page,
                index + 1,
                total,
                &created_by_names,
            ));
            output.push_str("\n[[/div]]\n");
        }

        output.push_str("[[/div]]");
        Ok(output)
    }

    async fn load_wikidot_user_names(
        ctx: &ServiceContext<'_>,
        pages: &[FoundPageRow],
    ) -> Result<BTreeMap<i64, String>> {
        let make_error = || {
            Error::new(
                "failed to load Wikidot user names for ListPages render",
                ErrorType::Render,
            )
        };

        let wikidot_user_ids = pages
            .iter()
            .filter_map(|page| page.created_by)
            .filter_map(|user_id| i32::try_from(user_id).ok())
            .collect::<BTreeSet<_>>();

        if wikidot_user_ids.is_empty() {
            return Ok(BTreeMap::new());
        }

        let users = WikidotUser::find()
            .filter(wikidot_user::Column::UserId.is_in(wikidot_user_ids))
            .all(ctx.transaction())
            .await
            .or_raise(make_error)?;

        Ok(users
            .into_iter()
            .filter_map(|user| {
                user.name
                    .or(user.slug)
                    .map(|name| (i64::from(user.user_id), name))
            })
            .collect())
    }
}

#[derive(Debug)]
struct ListPagesArguments {
    category_all: bool,
    categories: Vec<Cow<'static, str>>,
    excluded_categories: Vec<Cow<'static, str>>,
    any_tags: Vec<Cow<'static, str>>,
    all_tags: Vec<Cow<'static, str>>,
    no_tags: Vec<Cow<'static, str>>,
    created_by: Vec<Cow<'static, str>>,
    order: Option<OrderBySelector>,
    limit: Option<u64>,
    per_page: u8,
    range: RangeSelector,
}

fn parse_list_pages_arguments(head: &str) -> Option<ListPagesArguments> {
    let unparsed = LISTPAGES_ARGUMENT_REGEX.replace_all(head, "");
    if !unparsed.trim().is_empty() {
        return None;
    }

    let mut category_all = true;
    let mut categories = Vec::new();
    let mut excluded_categories = Vec::new();
    let mut any_tags = Vec::new();
    let mut all_tags = Vec::new();
    let mut no_tags = Vec::new();
    let mut created_by = Vec::new();
    let mut order = None;
    let mut limit = None;
    let mut per_page = PaginationSelector::default().per_page;
    let mut range = RangeSelector::Current;

    for captures in LISTPAGES_ARGUMENT_REGEX.captures_iter(head) {
        let key = captures["key"].to_ascii_lowercase();
        let value = captures
            .name("double")
            .or_else(|| captures.name("single"))
            .or_else(|| captures.name("bare"))
            .unwrap()
            .as_str()
            .trim();

        match key.as_str() {
            "category" => {
                if value == "*" {
                    category_all = true;
                    categories.clear();
                    excluded_categories.clear();
                } else {
                    category_all = true;
                    categories.clear();
                    excluded_categories.clear();

                    for category in split_list_pages_values(value) {
                        if category == "*" {
                            category_all = true;
                            categories.clear();
                        } else if let Some(category) = category.strip_prefix('-') {
                            excluded_categories.push(Cow::Owned(category.to_owned()));
                        } else {
                            category_all = false;
                            categories.push(Cow::Owned(
                                category
                                    .strip_prefix('+')
                                    .unwrap_or(&category)
                                    .to_owned(),
                            ));
                        }
                    }
                }
            }
            "tags" => {
                for tag in split_list_pages_values(value) {
                    if let Some(tag) = tag.strip_prefix('-') {
                        no_tags.push(Cow::Owned(tag.to_owned()));
                    } else if let Some(tag) = tag.strip_prefix('+') {
                        all_tags.push(Cow::Owned(tag.to_owned()));
                    } else {
                        any_tags.push(Cow::Owned(tag));
                    }
                }
            }
            "tag" => {
                for tag in split_list_pages_values(value) {
                    if let Some(tag) = tag.strip_prefix('-') {
                        no_tags.push(Cow::Owned(tag.to_owned()));
                    } else {
                        all_tags.push(Cow::Owned(
                            tag.strip_prefix('+').unwrap_or(&tag).to_owned(),
                        ));
                    }
                }
            }
            "created_by" | "createdby" => {
                let values = split_list_pages_values(value);
                if values.is_empty() || !values.iter().all(|value| value == "=") {
                    return None;
                }

                created_by.extend(values.into_iter().map(Cow::Owned));
            }
            "limit" => {
                limit = Some(value.parse().ok()?);
            }
            "perpage" | "per_page" => {
                per_page = value.parse().ok()?;
            }
            "order" => {
                order = Some(parse_list_pages_order(value)?);
            }
            "range" => {
                range = parse_list_pages_range(value)?;
            }
            // These inputs need additional data or Wikidot semantics that are not
            // implemented by PageQueryService yet. Leaving the module untouched is
            // safer than silently returning a wrong list.
            "created_at" | "createdat" | "updated_at" | "updatedat" | "rating"
            | "score" | "votes" | "form" | "parent" | "link_to" | "linkto" => {
                return None;
            }
            _ => return None,
        }
    }

    Some(ListPagesArguments {
        category_all,
        categories,
        excluded_categories,
        any_tags,
        all_tags,
        no_tags,
        created_by,
        order,
        limit,
        per_page,
        range,
    })
}

fn split_list_pages_values(value: &str) -> Vec<String> {
    value
        .split(|ch: char| ch.is_whitespace() || ch == ',')
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect()
}

fn parse_list_pages_order(value: &str) -> Option<OrderBySelector> {
    let (value, ascending) = match value.strip_prefix('-') {
        Some(value) => (value, false),
        None => (value, true),
    };

    let property = match value.to_ascii_lowercase().as_str() {
        "name" | "slug" | "fullname" | "fullslug" | "full_slug" => {
            OrderProperty::FullSlug
        }
        "title" => OrderProperty::Title,
        "created_at" | "createdat" | "date" | "created" => OrderProperty::CreatedAt,
        "updated_at" | "updatedat" | "updated" => OrderProperty::UpdatedAt,
        "random" => OrderProperty::Random,
        _ => return None,
    };

    Some(OrderBySelector {
        property,
        ascending,
    })
}

fn parse_list_pages_range(value: &str) -> Option<RangeSelector> {
    match value.to_ascii_lowercase().as_str() {
        "." | "current" => Some(RangeSelector::Current),
        "others" | "other" => Some(RangeSelector::Others),
        "before" => Some(RangeSelector::Before),
        "after" => Some(RangeSelector::After),
        _ => None,
    }
}

fn list_pages_body_variables_supported(body: &str) -> bool {
    LISTPAGES_VARIABLE_REGEX
        .captures_iter(body)
        .all(
            |captures| match captures["name"].to_ascii_lowercase().as_str() {
                "title_linked" | "title" | "name" | "slug" | "page_unix_name"
                | "fullname" | "full_slug" | "created_by" | "rating" | "index"
                | "total" => true,
                _ => false,
            },
        )
}

fn list_pages_body_uses_variable(body: &str, variable: &str) -> bool {
    LISTPAGES_VARIABLE_REGEX
        .captures_iter(body)
        .any(|captures| captures["name"].eq_ignore_ascii_case(variable))
}

fn substitute_list_pages_variables(
    template: &str,
    page: &FoundPageRow,
    index: usize,
    total: usize,
    created_by_names: &BTreeMap<i64, String>,
) -> String {
    let slug = page.slug.as_deref().unwrap_or("");
    let title = page.title.as_deref().unwrap_or(slug);
    let title_linked = if slug.is_empty() {
        title.to_owned()
    } else {
        format!("[/{slug} {title}]")
    };
    let created_by = page
        .created_by
        .map(|user_id| {
            created_by_names
                .get(&user_id)
                .cloned()
                .unwrap_or_else(|| user_id.to_string())
        })
        .unwrap_or_default();
    let rating = format_list_pages_rating(page.score);

    template
        .replace("%%title_linked%%", &title_linked)
        .replace("%%title%%", title)
        .replace("%%name%%", slug)
        .replace("%%slug%%", slug)
        .replace("%%page_unix_name%%", slug)
        .replace("%%fullname%%", slug)
        .replace("%%full_slug%%", slug)
        .replace("%%created_by%%", &created_by)
        .replace("%%rating%%", &rating)
        .replace("%%index%%", &index.to_string())
        .replace("%%total%%", &total.to_string())
}

fn format_list_pages_rating(score: Option<f32>) -> String {
    let Some(score) = score else {
        return String::new();
    };

    if score.fract() == 0.0 {
        format!("{score:.0}")
    } else {
        score.to_string()
    }
}

fn render_read_only_rate_module(score: ftml::data::ScoreValue) -> String {
    let score = format_score_value(score);

    format!(
        concat!(
            "[[div class=\"page-rate-widget-box\"]]",
            "[[span class=\"rate-points\"]]rating: ",
            "[[span class=\"number prw54353\"]]{}[[/span]]",
            "[[/span]]",
            "[[span class=\"rateup btn btn-default\"]]",
            "[[a href=\"javascript:;\" title=\"I like it\"]]+[[/a]]",
            "[[/span]]",
            "[[span class=\"ratedown btn btn-default\"]]",
            "[[a href=\"javascript:;\" title=\"I don't like it\"]]–[[/a]]",
            "[[/span]]",
            "[[span class=\"cancel btn btn-default\"]]",
            "[[a href=\"javascript:;\" title=\"Cancel my vote\"]]x[[/a]]",
            "[[/span]]",
            "[[/div]]"
        ),
        score,
    )
}

fn format_score_value(score: ftml::data::ScoreValue) -> String {
    match score {
        ftml::data::ScoreValue::Integer(value) if value > 0 => format!("+{value}"),
        ftml::data::ScoreValue::Integer(value) => value.to_string(),
        ftml::data::ScoreValue::Float(value) if value > 0.0 && value.fract() == 0.0 => {
            format!("+{value:.0}")
        }
        ftml::data::ScoreValue::Float(value) if value > 0.0 => {
            format!("+{value}")
        }
        ftml::data::ScoreValue::Float(value) if value.fract() == 0.0 => {
            format!("{value:.0}")
        }
        ftml::data::ScoreValue::Float(value) => value.to_string(),
    }
}

#[derive(Debug)]
struct RenderInnerOutput {
    html_output: HtmlOutput,
    errors: Vec<ParseError>,
    compiled_hash: TextHash,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RenderContext {
    current_site_id: Option<i64>,
    current_page_id: Option<i64>,
    text_block_page_id: Option<i64>,
}

impl RenderContext {
    fn none() -> Self {
        Self {
            current_site_id: None,
            current_page_id: None,
            text_block_page_id: None,
        }
    }

    fn page(site_id: i64, page_id: i64) -> Self {
        Self {
            current_site_id: Some(site_id),
            current_page_id: Some(page_id),
            text_block_page_id: Some(page_id),
        }
    }

    fn page_nav(site_id: i64, current_page_id: i64) -> Self {
        Self {
            current_site_id: Some(site_id),
            current_page_id: Some(current_page_id),
            text_block_page_id: None,
        }
    }
}

#[derive(Debug)]
struct IncludeExpansion {
    wikitext: String,
    included_pages: Vec<PageRef>,
}

#[derive(Debug)]
struct IncludeSource {
    site_id: i64,
    site_slug: String,
    wikitext: String,
}

#[derive(Debug)]
struct CollectingIncluder<'a> {
    includes: &'a mut Vec<IncludeRef<'static>>,
}

impl<'a, 't> Includer<'t> for CollectingIncluder<'a> {
    type Error = ExnError;

    fn include_pages(
        &mut self,
        includes: &[IncludeRef<'t>],
    ) -> Result<Vec<FetchedPage<'t>>> {
        self.includes.extend(includes.iter().map(own_include_ref));

        Ok(includes
            .iter()
            .map(|include| FetchedPage {
                page_ref: include.page_ref().clone(),
                content: Some(Cow::Borrowed("")),
            })
            .collect())
    }

    fn no_such_include(&mut self, page_ref: &PageRef) -> Result<Cow<'t, str>> {
        Ok(Cow::Owned(format!("No such page: {page_ref}")))
    }
}

#[derive(Debug)]
struct PreparedIncluder {
    pages: Vec<Option<String>>,
}

impl<'t> Includer<'t> for PreparedIncluder {
    type Error = ExnError;

    fn include_pages(
        &mut self,
        includes: &[IncludeRef<'t>],
    ) -> Result<Vec<FetchedPage<'t>>> {
        if includes.len() != self.pages.len() {
            return Err(include_error());
        }

        let pages = std::mem::take(&mut self.pages);

        Ok(includes
            .iter()
            .zip(pages)
            .map(|(include, content)| {
                let page_ref = include.page_ref().clone();
                let content = content.map(Cow::Owned);

                FetchedPage { page_ref, content }
            })
            .collect())
    }

    fn no_such_include(&mut self, page_ref: &PageRef) -> Result<Cow<'t, str>> {
        Ok(Cow::Owned(format!("No such page: {page_ref}")))
    }
}

fn include_error() -> ExnError {
    Error::new(
        "include expansion returned mismatched page references",
        ErrorType::Render,
    )
    .into()
}

fn own_include_ref(include: &IncludeRef<'_>) -> IncludeRef<'static> {
    let variables = include
        .variables()
        .iter()
        .map(|(key, value)| (Cow::Owned(key.to_string()), Cow::Owned(value.to_string())))
        .collect::<VariableMap<'static>>();

    IncludeRef::new(include.page_ref().clone(), variables)
}

fn apply_include_variables(content: &mut String, include: &IncludeRef<'_>) {
    let mut matches = Vec::new();

    for capture in INCLUDE_VARIABLE_REGEX.captures_iter(content) {
        let mtch = capture.get(0).unwrap();
        let name = &capture["name"];

        if let Some(value) = include
            .variables()
            .get(name)
            .map(|value| trim_include_variable_value(value).to_owned())
            .or_else(|| default_include_variable_value(name))
        {
            matches.push((value, mtch.range()));
        }
    }

    matches.reverse();
    for (value, range) in matches {
        content.replace_range(range, &value);
    }
}

fn trim_include_variable_value(value: &str) -> &str {
    value.trim_end_matches([' ', '\t', '\r', '\n'])
}

fn default_include_variable_value(name: &str) -> Option<String> {
    match name.to_ascii_lowercase().as_str() {
        "author" => Some("%%created_by%%".to_owned()),
        _ => None,
    }
}

fn is_include_variable_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
}

fn protect_include_variables(content: &mut String) {
    let protected = INCLUDE_VARIABLE_REGEX
        .replace_all(content, |capture: &regex::Captures<'_>| {
            format!(
                "{}{}{}",
                INCLUDE_VARIABLE_OPEN_SENTINEL,
                &capture["name"],
                INCLUDE_VARIABLE_CLOSE_SENTINEL,
            )
        })
        .to_string();

    *content = protected;
}

fn unprotect_include_variables(content: &mut String) {
    *content = content
        .replace(INCLUDE_VARIABLE_OPEN_SENTINEL, "{$")
        .replace(INCLUDE_VARIABLE_CLOSE_SENTINEL, "}");
}

fn apply_basalt_shell_compatibility(html: &mut String) {
    if !html.contains("theme%3Abasalt") && !html.contains("basalt-bedrock-min.css") {
        return;
    }

    html.push_str(
        r#"<style>
#side-bar {
    left: calc(var(--side-bar-width, 17rem) * -1) !important;
}
#main-content {
    margin-left: auto !important;
    margin-right: auto !important;
}
</style>"#,
    );
}

fn site_matches_wikidot_slug(site: &SiteModel, site_slug: &str) -> bool {
    if site.slug.eq_ignore_ascii_case(site_slug) {
        return true;
    }

    let Some(preferred_domain) = site.preferred_domain.as_deref() else {
        return false;
    };

    preferred_domain_matches_wikidot_slug(preferred_domain, site_slug)
}

fn site_accepts_wikidot_local_asset_slug(site: &SiteModel, site_slug: &str) -> bool {
    site_matches_wikidot_slug(site, site_slug)
        || translated_scp_site_uses_scp_wiki_source_assets(site, site_slug)
}

fn translated_scp_site_uses_scp_wiki_source_assets(
    site: &SiteModel,
    site_slug: &str,
) -> bool {
    if !site_slug.eq_ignore_ascii_case("scp-wiki")
        || site.locale.eq_ignore_ascii_case("en")
    {
        return false;
    }

    let Some(preferred_domain) = site.preferred_domain.as_deref() else {
        return false;
    };
    let Some(preferred_slug) = preferred_domain_wikidot_slug(preferred_domain) else {
        return false;
    };

    !preferred_slug.eq_ignore_ascii_case("scp-wiki") && preferred_slug.starts_with("scp")
}

fn preferred_domain_matches_wikidot_slug(
    preferred_domain: &str,
    site_slug: &str,
) -> bool {
    let Some(host) = preferred_domain_host(preferred_domain) else {
        return false;
    };

    if host.eq_ignore_ascii_case(site_slug) {
        return true;
    }

    let Some(wikidot_slug) = host.strip_suffix(".wikidot.com") else {
        return false;
    };

    wikidot_slug.eq_ignore_ascii_case(site_slug)
}

fn preferred_domain_wikidot_slug(preferred_domain: &str) -> Option<String> {
    let host = preferred_domain_host(preferred_domain)?;
    let wikidot_slug = host.strip_suffix(".wikidot.com").unwrap_or(host.as_str());

    (!wikidot_slug.is_empty()).then(|| wikidot_slug.to_owned())
}

fn preferred_domain_host(preferred_domain: &str) -> Option<String> {
    let trimmed = preferred_domain.trim().trim_end_matches('.');
    let host = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(trimmed)
        .split('/')
        .next()
        .unwrap_or(trimmed)
        .split(':')
        .next()
        .unwrap_or(trimmed)
        .trim_end_matches('.');

    (!host.is_empty()).then(|| host.to_ascii_lowercase())
}

fn local_file_host_site_slug(host: &str, config: &Config) -> Option<String> {
    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();

    [".wikidot.com", ".wdfiles.com", ".wjfiles.com"]
        .iter()
        .find_map(|suffix| host.strip_suffix(suffix).map(ToOwned::to_owned))
        .filter(|slug| !slug.is_empty())
        .or_else(|| {
            let suffix = config.files_domain.to_ascii_lowercase();
            host.strip_suffix(&suffix)
                .map(ToOwned::to_owned)
                .filter(|slug| !slug.is_empty())
        })
}

fn css_dependency_host_is_local(host: &str, config: &Config) -> bool {
    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();
    if host == "localhost" || host == "127.0.0.1" || host.ends_with(".localhost") {
        return true;
    }

    let files_domain = config.files_domain.trim().to_ascii_lowercase();
    if !files_domain.is_empty() && host.ends_with(&files_domain) {
        return true;
    }

    let files_domain_no_dot = config.files_domain_no_dot.trim().to_ascii_lowercase();
    !files_domain_no_dot.is_empty() && host == files_domain_no_dot
}

#[cfg(test)]
mod tests {
    use super::{CollectingIncluder, RenderContext, RenderService, include_error};
    use crate::config::Config;
    use crate::models::site::Model as SiteModel;
    use crate::types::License;
    use crate::utils::now;
    use ftml::data::PageRef;
    use ftml::includes::IncludeRef;
    use ftml::layout::Layout;
    use ftml::settings::{WikitextMode, WikitextSettings};
    use std::borrow::Cow;

    #[test]
    fn restores_wikidot_email_obfuscation() {
        let html = concat!(
            r#"<p><strong>Email:</strong> "#,
            r#"<span class="wiki-email" style="visibility: visible;">"#,
            r#"<a href="mailto:info@nfsi.gov">info@nfsi.gov</a></span><br /></p>"#,
        );

        assert_eq!(
            RenderService::restore_wikidot_email_obfuscation(html),
            concat!(
                r#"<p><strong>Email:</strong> "#,
                r#"<span class="wiki-email">vog.isfn|ofni#vog.isfn|ofni</span>"#,
                r#"<br /></p>"#,
            ),
        );
    }

    #[test]
    fn moves_sentence_punctuation_outside_wikidot_email_span() {
        let html = concat!(
            r#"<p>For more information, contact "#,
            r#"<span class="wiki-email" style="visibility: visible;">"#,
            r#"<a href="mailto:training@nfsi.gov.">training@nfsi.gov.</a></span></p>"#,
        );

        assert_eq!(
            RenderService::restore_wikidot_email_obfuscation(html),
            concat!(
                r#"<p>For more information, contact "#,
                r#"<span class="wiki-email">vog.isfn|gniniart#vog.isfn|gniniart</span>."#,
                r#"</p>"#,
            ),
        );
    }

    #[test]
    fn leaves_non_matching_email_spans_unchanged() {
        let html = concat!(
            r#"<span class="wiki-email" style="visibility: visible;">"#,
            r#"<a href="mailto:info@nfsi.gov">different@nfsi.gov</a></span>"#,
        );

        assert_eq!(RenderService::restore_wikidot_email_obfuscation(html), html);
    }

    #[test]
    fn localizes_matching_wikidot_local_file_urls() {
        let site = wikidot_site(
            "scp-wiki-en-corpus-scp9506-slice-v2",
            Some("scp-wiki.wikidot.com"),
        );
        let mut config = Config::integration_testing();
        config.files_domain = ".wjfiles.localhost".to_owned();
        config.files_domain_no_dot = "wjfiles.localhost".to_owned();
        let html = concat!(
            r#"<p><img src="http://scp-wiki.wikidot.com/local--files/scp-9506/NFSI.png?download=true#frag">"#,
            r#"<a href='https://scp-wiki.wdfiles.com:443/local--files/scp-9506/NAME%20HERE.png'>"#,
            r#"file</a>"#,
            r#"<img class="image crom-thumbnail" src="https://scp-wiki-en-corpus-scp9506-slice-v2.wjfiles.com/local--files/scp-9506/NFSI.png">"#,
            r#"<style>:root{--logo:url(http://scp-wiki.wikidot.com/local--files/scp-9506/NFSI.png)}</style>"#,
            r#"<style>.quoted{background:url('http://scp-wiki.wikidot.com/local--files/scp-9506/BG.png')}</style>"#,
            r#"<style>@import "https://scp-wiki.wdfiles.com/local--code/theme%3Abasalt/1";</style>"#,
            r#"<style>@import url(https://scp-wiki.wdfiles.com/local--code/component:betterfootnotes/1)</style>"#,
            r#"</p>"#,
        );

        assert_eq!(
            RenderService::localize_wikidot_local_file_urls(html, Some(&site), &config,),
            concat!(
                r#"<p><img src="https://scp-wiki-en-corpus-scp9506-slice-v2.wjfiles.localhost/local--files/scp-9506/NFSI.png?download=true#frag">"#,
                r#"<a href='https://scp-wiki-en-corpus-scp9506-slice-v2.wjfiles.localhost/local--files/scp-9506/NAME%20HERE.png'>"#,
                r#"file</a>"#,
                r#"<img class="image crom-thumbnail" src="https://scp-wiki-en-corpus-scp9506-slice-v2.wjfiles.localhost/local--files/scp-9506/NFSI.png">"#,
                r#"<style>:root{--logo:url(https://scp-wiki-en-corpus-scp9506-slice-v2.wjfiles.localhost/local--files/scp-9506/NFSI.png)}</style>"#,
                r#"<style>.quoted{background:url('https://scp-wiki-en-corpus-scp9506-slice-v2.wjfiles.localhost/local--files/scp-9506/BG.png')}</style>"#,
                r#"<style>@import "https://scp-wiki-en-corpus-scp9506-slice-v2.wjfiles.localhost/local--code/theme%3Abasalt/1";</style>"#,
                r#"<style>@import url(https://scp-wiki-en-corpus-scp9506-slice-v2.wjfiles.localhost/local--code/component:betterfootnotes/1)</style>"#,
                r#"</p>"#,
            ),
        );
    }

    #[test]
    fn localizes_when_site_slug_is_wikidot_slug() {
        let site = wikidot_site("scp-wiki", None);
        let config = Config::integration_testing();
        let html =
            r#"<img src="http://scp-wiki.wikidot.com/local--files/scp-9506/NFSI.png">"#;

        assert_eq!(
            RenderService::localize_wikidot_local_file_urls(html, Some(&site), &config,),
            r#"<img src="https://scp-wiki.wjfiles.com/local--files/scp-9506/NFSI.png">"#,
        );
    }

    #[test]
    fn localizes_scp_wiki_source_assets_for_translated_scp_sites() {
        let mut site = wikidot_site(
            "scp-wiki-cn-corpus-scp9506-translation-seed",
            Some("scp-wiki-cn.wikidot.com"),
        );
        site.locale = "cn".to_owned();
        let mut config = Config::integration_testing();
        config.files_domain = ".wjfiles.localhost".to_owned();
        config.files_domain_no_dot = "wjfiles.localhost".to_owned();
        let html = concat!(
            r#"<img src="http://scp-wiki.wikidot.com/local--files/scp-9506/NFSI.png">"#,
            r#"<style>@import "https://scp-wiki.wdfiles.com/local--code/theme%3Abasalt/1";</style>"#,
        );

        assert_eq!(
            RenderService::localize_wikidot_local_file_urls(html, Some(&site), &config,),
            concat!(
                r#"<img src="https://scp-wiki-cn-corpus-scp9506-translation-seed.wjfiles.localhost/local--files/scp-9506/NFSI.png">"#,
                r#"<style>@import "https://scp-wiki-cn-corpus-scp9506-translation-seed.wjfiles.localhost/local--code/theme%3Abasalt/1";</style>"#,
            ),
        );
    }

    #[test]
    fn leaves_nonmatching_wikidot_local_file_urls_unchanged() {
        let site = wikidot_site(
            "scp-wiki-en-corpus-scp9506-slice-v2",
            Some("scp-wiki.wikidot.com"),
        );
        let config = Config::integration_testing();
        let html = concat!(
            r#"<img src="https://wanderers-library.wikidot.com/local--files/the-page/image.png">"#,
            r#"<img src="https://wanderers-library.wikidot.com/local--code/theme:basalt/1">"#,
            r#"<style>:root{--logo:url(http://wanderers-library.wikidot.com/local--files/the-page/image.png)}</style>"#,
            r#"<style>@import url(http://wanderers-library.wikidot.com/local--code/the-page/1)</style>"#,
            r#"<img src="https://example.com/local--files/scp-9506/NFSI.png">"#,
        );

        assert_eq!(
            RenderService::localize_wikidot_local_file_urls(html, Some(&site), &config,),
            html,
        );
        assert_eq!(
            RenderService::localize_wikidot_local_file_urls(html, None, &config),
            html,
        );
    }

    #[test]
    fn code_block_compatibility_suppresses_external_css_dependencies() {
        let mut site = wikidot_site(
            "scp-wiki-cn-corpus-scp9506-translation-seed",
            Some("scp-wiki-cn.wikidot.com"),
        );
        site.locale = "cn".to_owned();
        let mut config = Config::integration_testing();
        config.files_domain = ".wjfiles.localhost".to_owned();
        config.files_domain_no_dot = "wjfiles.localhost".to_owned();
        let css = concat!(
            "@import url('https://cdn.scpwiki.com/theme/en/basalt/normalize-min.css');\n",
            "@import url('https://fonts.googleapis.com/css2?family=Sofia+Sans:ital,wght@0,100;0,200;1,900&display=swap');\n",
            "@import url(\"https://scp-wiki-cn-corpus-scp9506-translation-seed.wjfiles.localhost/local--code/theme:basalt/1\");\n",
            "@font-face { src: url('https://cdn.jsdelivr.net/font.woff2') format('woff2'); }\n",
            ":root { --logo: url('http://scp-wiki.wikidot.com/local--files/scp-9506/NFSI.png'); }\n",
        );

        let restored = RenderService::restore_wikidot_code_block_compatibility(
            css,
            Some(&site),
            &config,
        );

        assert!(restored.contains("omitted external @import"));
        assert!(!restored.contains("cdn.scpwiki.com"));
        assert!(!restored.contains("fonts.googleapis.com"));
        assert!(!restored.contains("display=swap"));
        assert!(!restored.contains("cdn.jsdelivr.net"));
        assert!(restored.contains(
            "https://scp-wiki-cn-corpus-scp9506-translation-seed.wjfiles.localhost/local--code/theme:basalt/1"
        ));
        assert!(restored.contains(
            "https://scp-wiki-cn-corpus-scp9506-translation-seed.wjfiles.localhost/local--files/scp-9506/NFSI.png"
        ));
        assert!(restored.contains(r#"url("data:,")"#));
    }

    #[test]
    fn page_nav_render_context_keeps_current_page_without_text_block_target() {
        assert_eq!(
            RenderContext::page_nav(7, 11),
            RenderContext {
                current_site_id: Some(7),
                current_page_id: Some(11),
                text_block_page_id: None,
            },
        );
    }

    #[test]
    fn page_render_context_uses_current_page_as_text_block_target() {
        assert_eq!(
            RenderContext::page(7, 11),
            RenderContext {
                current_site_id: Some(7),
                current_page_id: Some(11),
                text_block_page_id: Some(11),
            },
        );
    }

    #[test]
    fn protects_wikidot_interwiki_embed_iframe_before_ftml() {
        let mut wikitext = concat!(
            "[[embed]]\n",
            r#"<iframe src="//interwiki.scpwiki.com/interwikiFrame.html?lang=en&community=scp&pagename=scp-9506" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#,
            "\n[[/embed]]",
        )
        .to_owned();

        let iframes = RenderService::protect_wikidot_embed_iframes(&mut wikitext);
        assert_eq!(wikitext, "WIKIJUMPWIKIDOTEMBEDIFRAME0X");
        assert_eq!(
            iframes,
            vec![
                r#"<iframe src="/-/wikidot-interwiki/interwikiFrame.html?lang=en&community=scp&pagename=scp-9506" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#
                    .to_owned()
            ],
        );
        assert_eq!(
            RenderService::restore_protected_wikidot_embed_iframes(
                "<p>WIKIJUMPWIKIDOTEMBEDIFRAME0X</p>".to_owned(),
                &iframes,
            ),
            r#"<p><iframe src="/-/wikidot-interwiki/interwikiFrame.html?lang=en&community=scp&pagename=scp-9506" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe></p>"#,
        );
    }

    #[test]
    fn leaves_unsupported_raw_wikidot_embed_iframe_unprotected() {
        let original = concat!(
            "[[embed]]\n",
            r#"<iframe src="//example.com/widget" style="display: none"></iframe>"#,
            "\n[[/embed]]",
        );
        let mut wikitext = original.to_owned();

        let iframes = RenderService::protect_wikidot_embed_iframes(&mut wikitext);
        assert!(iframes.is_empty());
        assert_eq!(wikitext, original);
    }

    #[test]
    fn restores_wikidot_styleframe_embed_iframe() {
        let html = concat!(
            r#"<p>[[embed]]<br/>"#,
            r#"&lt;iframe src="//interwiki.scpwiki.com/styleFrame.html?priority=1<br/>"#,
            r#"&amp;theme=<a href="https://cdn.scpwiki.com/theme/en/basalt/normalize-min.css">"#,
            r#"https://cdn.scpwiki.com/theme/en/basalt/normalize-min.css</a><br/>"#,
            r#"&amp;css={$css}" style="display: none"&gt;&lt;/iframe&gt;"#,
            r#"<br/>[[/embed]]</p>"#,
        );

        assert_eq!(
            RenderService::restore_wikidot_rendered_embed_iframes(html),
            concat!(
                r#"<p><iframe src="/-/wikidot-interwiki/styleFrame.html?priority=1"#,
                r#"&theme=https://cdn.scpwiki.com/theme/en/basalt/normalize-min.css"#,
                r#"&css={$css}" style="display: none"></iframe></p>"#,
            ),
        );
    }

    #[test]
    fn restores_wikidot_interwiki_rendered_embed_iframe() {
        let html = concat!(
            r#"<p>[[embed]]<br/>"#,
            r#"&lt;iframe src="//interwiki.scpwiki.com/interwikiFrame.html?lang=en&amp;community=scp&amp;pagename=scp-9506" "#,
            r#"allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"&gt;&lt;/iframe&gt;"#,
            r#"<br/>[[/embed]]</p>"#,
        );

        assert_eq!(
            RenderService::restore_wikidot_rendered_embed_iframes(html),
            r#"<p><iframe src="/-/wikidot-interwiki/interwikiFrame.html?lang=en&community=scp&pagename=scp-9506" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe></p>"#,
        );
    }

    #[test]
    fn leaves_bare_embed_and_non_styleframe_embed_literal() {
        let bare = "<p>[[embed]]</p>";
        assert_eq!(
            RenderService::restore_wikidot_rendered_embed_iframes(bare),
            bare
        );

        let non_styleframe = concat!(
            r#"<p>[[embed]]<br/>"#,
            r#"&lt;iframe src="//example.com/widget" style="display: none"&gt;&lt;/iframe&gt;"#,
            r#"<br/>[[/embed]]</p>"#,
        );
        assert_eq!(
            RenderService::restore_wikidot_rendered_embed_iframes(non_styleframe),
            non_styleframe,
        );
    }

    #[test]
    fn removes_preview_component_separator_markers() {
        let mut wikitext = concat!(
            "[[image NFSI.png class=\"crom-thumbnail\" style=\"display: none\"]]\n",
            "=====\n",
            "[[include component:preview text=Official fog safety hub.]]\n",
            "=====\n",
            "after\n",
        )
        .to_owned();

        RenderService::remove_preview_component_separator_markers(&mut wikitext);

        assert_eq!(
            wikitext,
            concat!(
                "[[image NFSI.png class=\"crom-thumbnail\" style=\"display: none\"]]\n",
                "[[include component:preview text=Official fog safety hub.]]\n",
                "after\n",
            ),
        );
    }

    #[test]
    fn removes_site_qualified_preview_component_separator_markers() {
        let mut wikitext = concat!(
            "before\n",
            "=====\n",
            "[[include :scp-wiki:component:preview text=Official fog safety hub.]]\n",
            "=====\n",
            "after\n",
        )
        .to_owned();

        RenderService::remove_preview_component_separator_markers(&mut wikitext);

        assert_eq!(
            wikitext,
            concat!(
                "before\n",
                "[[include :scp-wiki:component:preview text=Official fog safety hub.]]\n",
                "after\n",
            ),
        );
    }

    #[test]
    fn removes_multiline_preview_component_separator_markers() {
        let mut wikitext = concat!(
            "before\n",
            "=====\n",
            "[[include component:preview\n",
            "| text=Official fog safety hub.\n",
            "]]\n",
            "=====\n",
            "after\n",
        )
        .to_owned();

        RenderService::remove_preview_component_separator_markers(&mut wikitext);

        assert_eq!(
            wikitext,
            concat!(
                "before\n",
                "[[include component:preview\n",
                "| text=Official fog safety hub.\n",
                "]]\n",
                "after\n",
            ),
        );
    }

    #[test]
    fn leaves_plain_content_separators() {
        let mut wikitext = "Before\n=====\nAfter\n".to_owned();

        RenderService::remove_preview_component_separator_markers(&mut wikitext);

        assert_eq!(wikitext, "Before\n=====\nAfter\n");
    }

    #[test]
    fn leaves_non_preview_include_separator_markers() {
        let mut wikitext = concat!(
            "before\n",
            "=====\n",
            "[[include component:license-box]]\n",
            "=====\n",
            "after\n",
        )
        .to_owned();

        RenderService::remove_preview_component_separator_markers(&mut wikitext);

        assert_eq!(
            wikitext,
            concat!(
                "before\n",
                "=====\n",
                "[[include component:license-box]]\n",
                "=====\n",
                "after\n",
            ),
        );
    }

    #[test]
    fn removes_unresolved_variable_iftags_block() {
        let mut wikitext = concat!(
            "before\n",
            ">[[ift{$darkmode}gs -basalt-override]]\n",
            ">[[iftags]]\n",
            "[[module CSS]]\n",
            "@import url(https://scp-wiki.wdfiles.com/local--code/theme%3Abasalt/2)\n",
            "[[/module]]\n",
            "[[include :scp-wiki:component:interwiki-style\n",
            "| priority=4\n",
            "| theme=https://scp-wiki.wdfiles.com/local--code/theme%3Abasalt/2\n",
            "]]\n",
            ">[[/iftags]]\n",
            ">[[/ift{$darkmode}gs]]\n",
            "after\n",
        )
        .to_owned();

        RenderService::remove_unresolved_variable_iftags_blocks(&mut wikitext);

        assert_eq!(wikitext, "before\nafter\n");
    }

    #[test]
    fn unwraps_active_collapsed_basalt_iftags_block() {
        let mut wikitext = concat!(
            "before\n",
            ">[[iftags -basalt-override]]\n",
            ">[[iftags]]\n",
            "[[module CSS]]\n",
            "@import url(https://scp-wiki.wdfiles.com/local--code/theme%3Abasalt/3)\n",
            "[[/module]]\n",
            ">[[/iftags]]\n",
            ">[[/iftags]]\n",
            "after\n",
        )
        .to_owned();

        RenderService::remove_unresolved_variable_iftags_blocks(&mut wikitext);

        assert_eq!(
            wikitext,
            concat!(
                "before\n",
                "\n",
                "[[module CSS]]\n",
                "@import url(https://scp-wiki.wdfiles.com/local--code/theme%3Abasalt/3)\n",
                "[[/module]]\n",
                "after\n",
            ),
        );
    }

    #[test]
    fn unwraps_unresolved_variable_iftags_body_without_nested_condition() {
        let mut wikitext = concat!(
            "before\n",
            ">[[ift{$disable-acs-anim}gs +theme]]\n",
            "[[include :scp-wiki:component:acs-animation]]\n",
            ">[[/ift{$disable-acs-anim}gs]]\n",
            "after\n",
        )
        .to_owned();

        RenderService::remove_unresolved_variable_iftags_blocks(&mut wikitext);

        assert_eq!(
            wikitext,
            concat!(
                "before\n",
                "\n",
                "[[include :scp-wiki:component:acs-animation]]\n",
                "after\n",
            ),
        );
    }

    #[test]
    fn included_source_cleanup_exposes_nested_acs_include_before_expansion() {
        let mut wikitext = concat!(
            ">[[ift{$disable-acs-anim}gs +theme]]\n",
            "[[include :scp-wiki:component:acs-animation]]\n",
            ">[[/ift{$disable-acs-anim}gs]]\n",
        )
        .to_owned();
        let include =
            IncludeRef::page_only(PageRef::page_and_site("scp-wiki", "theme:basalt"));

        super::apply_include_variables(&mut wikitext, &include);
        RenderService::remove_unresolved_variable_iftags_blocks(&mut wikitext);

        assert_eq!(
            wikitext,
            concat!("\n", "[[include :scp-wiki:component:acs-animation]]\n",),
        );
    }

    #[test]
    fn collects_single_line_wikidot_include_variables() {
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut includes = Vec::new();
        ftml::include(
            "[[include :scp-wiki:theme:basalt | wide=a | hidetitle=a]]\n",
            &settings,
            CollectingIncluder {
                includes: &mut includes,
            },
            include_error,
        )
        .expect("include collection should parse Wikidot single-line variables");

        assert_eq!(includes.len(), 1);
        assert_eq!(
            includes[0].page_ref(),
            &PageRef::page_and_site("scp-wiki", "theme:basalt"),
        );
        assert_eq!(
            includes[0].variables().get("wide").map(Cow::as_ref),
            Some("a "),
        );
        assert_eq!(
            includes[0].variables().get("hidetitle").map(Cow::as_ref),
            Some("a"),
        );

        let mut source = concat!(
            ">[[ift{$hidetitle}gs -basalt-override]]\n",
            ">[[iftags]]\n",
            "@import url(https://scp-wiki.wdfiles.com/local--code/theme%3Abasalt/3)\n",
            ">[[/iftags]]\n",
            ">[[/ift{$hidetitle}gs]]\n",
            ">[[ift{$wide}gs -basalt-override]]\n",
            ">[[iftags]]\n",
            "@import url(https://scp-wiki.wdfiles.com/local--code/theme%3Abasalt/6)\n",
            ">[[/iftags]]\n",
            ">[[/ift{$wide}gs]]\n",
            ">[[ift{$disable-acs-anim}gs +theme]]\n",
            "[[include :scp-wiki:component:acs-animation]]\n",
            ">[[/ift{$disable-acs-anim}gs]]\n",
        )
        .to_owned();

        super::apply_include_variables(&mut source, &includes[0]);
        RenderService::remove_unresolved_variable_iftags_blocks(&mut source);

        assert!(source.contains("theme%3Abasalt/3"));
        assert!(source.contains("theme%3Abasalt/6"));
        assert!(source.contains("[[include :scp-wiki:component:acs-animation]]"));
        assert!(!source.contains("[[ifta gs -basalt-override]]"));
    }

    #[test]
    fn removes_malformed_collapsed_basalt_iftags_block() {
        let mut wikitext = concat!(
            "before\n",
            ">[[ifta gs -basalt-override]]\n",
            ">[[iftags]]\n",
            "[[module CSS]]\n",
            "@import url(https://scp-wiki.wdfiles.com/local--code/theme%3Abasalt/6)\n",
            "[[/module]]\n",
            ">[[/iftags]]\n",
            ">[[/ifta gs]]\n",
            "after\n",
        )
        .to_owned();

        RenderService::remove_unresolved_variable_iftags_blocks(&mut wikitext);

        assert_eq!(wikitext, "before\nafter\n");
    }

    #[test]
    fn leaves_plain_iftags_block() {
        let mut wikitext = "[[iftags +theme]]\nbody\n[[/iftags]]\n".to_owned();

        RenderService::remove_unresolved_variable_iftags_blocks(&mut wikitext);

        assert_eq!(wikitext, "[[iftags +theme]]\nbody\n[[/iftags]]\n");
    }

    fn wikidot_site(slug: &str, preferred_domain: Option<&str>) -> SiteModel {
        SiteModel {
            site_id: 1,
            created_at: now(),
            updated_at: None,
            deleted_at: None,
            from_wikidot: true,
            slug: slug.to_owned(),
            name: slug.to_owned(),
            tagline: String::new(),
            description: String::new(),
            locale: "en".to_owned(),
            default_page: "main".to_owned(),
            top_bar_page: "nav:top".to_owned(),
            side_bar_page: "nav:side".to_owned(),
            preferred_domain: preferred_domain.map(ToOwned::to_owned),
            layout: None,
            license: License::CcBySa30,
        }
    }
}
