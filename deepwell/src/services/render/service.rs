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

mod native_list;
mod post_render_compat;
mod wikidot_preprocess;

#[cfg(test)]
use native_list::{
    MAX_NATIVE_LIST_WIKIDOT_SPAN_NESTING, find_balanced_ul_end,
    native_list_page_link_default_label, render_list_pages_table_inline_html,
    render_native_list_page_link,
};
pub(super) use native_list::{
    collect_wikidot_compat_empty_label_link_slugs, native_list_page_link_slug,
    native_numbered_list_content, render_list_pages_numbered_rows_with_titles,
    render_list_pages_table_rows, render_native_list_inline_html,
    render_native_list_inline_html_with_titles, render_native_list_inline_wikidot_spans,
    render_native_list_inline_wikidot_strong,
    render_native_list_inline_wikidot_underlines, wikidot_inline_span_marker_open,
};
use native_list::{
    native_bullet_list_item, render_native_bullet_list_with_wikipedia_links,
};

use super::compat::CompatHtmlFragments;
use super::compat::color_and_inline_protection::{
    ProtectedWikidotColorSpans, ProtectedWikidotInlineHtml, decode_numeric_html_entity,
    protect_wikidot_color_spans, protect_wikidot_inline_html_spans,
    restore_protected_wikidot_color_spans, restore_protected_wikidot_inline_html,
    sanitize_wikidot_compat_inline_tag,
};
#[cfg(test)]
use super::compat::color_and_inline_protection::{
    ProtectedWikidotCompatHtml, parse_wikidot_compat_color_descriptor,
    render_wikidot_protected_inline_body_html,
    substitute_wikidot_protected_inline_dashes,
    substitute_wikidot_protected_inline_dashes_with_scan_count,
    wikidot_compat_html_marker,
};
#[cfg(test)]
use super::compat::preparation::extract_css_modules;
use super::compat::preparation::{
    RuntimeCssInsertion, extract_css_modules_with_runtime_insertions,
    neutralize_authored_markers, protect_css_modules_before_first_list_pages,
};
use super::compat::text_fragments::CompatTextFragments;
use super::compat::wikidot_iframe::{
    expand_wikidot_iframe_syntax, has_wikidot_iframe_syntax,
};
use super::compat::wikidot_link_protection::{
    WikidotWikipediaLink, build_wikidot_wikipedia_link,
};
use super::diagnostics::{
    CorpusRenderDimension, CorpusRenderScope, CorpusRenderStage, CorpusRenderTrace,
    StageGuard,
};
use super::forum_modules::resolve_typed_root_recent_threads_runtime_modules;
use super::ftml_render::{
    FtmlRenderOutput, InnerPreparedRenderWikitext, ParsedFtmlRender,
    WikidotCompatLinkTitleMap, native_list_page_link_ref,
};
use super::ftml_user_info::UserInfoSnapshot;
use super::generator::COMPILED_GENERATOR;
use super::iftags::{
    resolve_outermost_wikidot_iftags,
    resolve_outermost_wikidot_iftags_before_include_expansion,
    resolve_outermost_wikidot_iftags_before_include_expansion_for_page_preview,
};
use super::include_attachment_owners::{
    AttachmentOwner, AttachmentProvenanceRegistry, AttachmentVariableOwners,
    find_wikidot_directive_end, owned_url, parse_wikidot_include_argument,
    preserve_argument_quotes, protect_forwarded_attachment_variables,
    qualify_included_relative_image_attachments,
    qualify_relative_image_variable_attachments, relative, semantic_attachment_value,
    split_wikidot_include_argument_segments, wikidot_include_directive_ranges,
    wikidot_include_segment_is_space,
};
use super::include_comment_branches::{
    remove_nested_include_boundaries, remove_unresolved_include_comment_branches,
};
use super::include_resolution::{
    PreparedIncluder, collect_include_display_pages,
    collect_missing_include_replacements, expand_malformed_include_targets,
    wikidot_no_such_include_replacement,
};
use super::include_variable_iftags::resolve_unbound_include_variable_iftags;
#[cfg(test)]
use super::include_variables::{
    apply_include_variables, apply_include_variables_before_resolving_iftags,
    default_include_variable_value, trim_include_variable_value,
};
use super::include_variables::{
    is_include_variable_name, prepare_include_source_variables_and_comment_branches,
    protect_include_variables, unprotect_include_variables,
};
use super::legacy_actions::LegacyActionRegistry;
#[cfg(test)]
use super::list_pages::ResolvedListPagesAuthors;
use super::list_pages::{
    ListPagesExpansion, ListPagesExpansionOptions,
    build_wikidot_list_pages_module_request, protect_ajax_module_literal_markers,
    protect_generated_parser_function_comment_gates,
    replace_recursive_list_pages_with_error,
    resolve_wikidot_parser_functions_outside_list_pages,
};
use super::literal_regions::LiteralRegionIndex;
use super::metacomponent::{
    MetacomponentSourceContext, select_metacomponent_documentation,
};
use super::module_arguments::wikidot_module_argument;
use super::native_list_context::NativeListSourceContext;
use super::next_previous_page::NextPreviousPageExpansion;
use super::pages::expand_page_index_modules;
use super::percent_encoding::percent_encode_path_segment;
use super::render_budget::RenderCostBudget;
use super::render_options::{
    RenderContext, RenderExpansionOptions, RenderInnerOptions, RenderLifecycle,
    RenderPageOptions,
};
use super::runtime::{IncludeSource, IncludeSourceCache, RenderRuntime};
use super::runtime_modules::{RateModuleContext, SecondaryRuntimeModuleExpansionOptions};
use super::site_utility_modules::resolve_typed_root_site_grid_runtime_modules;
use super::structs::{RenderOutput, RenderPageOutput};
use super::url_arguments::UrlArguments;
use super::wikidot_hosts::{
    direct_wdfiles_local_file_url, local_file_host_site_slug, preferred_domain_host,
    preferred_domain_wikidot_slug,
};
use crate::config::Config;
use crate::error::prelude::{Error, ErrorType, ExnError, OptionExt, Result, ResultExt};
use crate::hash::{TextHash, k12_hash};
use crate::models::site::Model as SiteModel;
use crate::services::ServiceContext;
use crate::services::settings::{
    NavigationPageWikitext, PageRatingType, SettingsService,
};
use crate::services::text_block::{
    MIME_HTML, TextBlock, TextBlockService, mime_for_language,
};
use crate::services::{
    LinkService, PageQueryService, ScoreService, SiteService, TextService,
};
use crate::types::Reference;
use crate::types::{PageId, TextBlockType};
use crate::utils::now;
use crate::utils::{locale_for_ftml, parse_locales};
use ftml::data::PageRef;
use ftml::includes::{FetchedPage, IncludeRef};
use ftml::prelude::{
    Includer, Layout, PageInfo, ParseError, ScoreValue, WikitextMode, WikitextSettings,
};
use ftml::render::html::HtmlOutput;
use ftml::tree::{CodeBlock, VariableMap};
use ftml::{self};
use regex::Regex;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::ops::Range;
use std::pin::Pin;
use std::sync::LazyLock;
use std::time::Instant;
use tokio::task;
use tokio::time::timeout;

#[derive(Debug)]
pub struct RenderService;

/// Runtime-expanded page input for the corpus render replayer.
///
/// This is intentionally produced by the same expansion path used by
/// `render_inner()`, stopping immediately before the pure Wikidot
/// normalization and protection steps.  Keeping the owned FTML context with
/// the text lets a worker process finish preparation without rebuilding page
/// metadata independently.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CorpusReplayExpandedWikitext {
    pub wikitext: String,
    pub page_info: PageInfo<'static>,
    pub settings: WikitextSettings,
    pub id: PageId,
    pub included_pages: Vec<PageRef>,
    pub(super) wikidot_compat_html: CompatHtmlFragments,
    pub(super) wikidot_compat_text: CompatTextFragments,
}

impl CorpusReplayExpandedWikitext {
    #[inline]
    pub fn included_page_count(&self) -> usize {
        self.included_pages.len()
    }
}

/// Timings for the pure portion of corpus replay preparation.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CorpusReplayStageTimings {
    pub normalization_us: u64,
    pub outer_protection_us: u64,
    pub fallback_check_us: u64,
    pub inner_protection_us: u64,
    pub preprocess_us: u64,
}

/// Cheap syntax features recorded beside a replay input for clustering.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CorpusReplaySyntaxFeatures {
    pub bytes: usize,
    pub lines: usize,
    pub max_line_bytes: usize,
    pub block_markers: usize,
    pub quote_prefixed_lines: usize,
    pub ordered_list_lines: usize,
    pub unordered_list_lines: usize,
    pub table_lines: usize,
    pub inline_delimiter_markers: usize,
}

/// Start notifications for the pure replay preparation stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CorpusReplayPreparationStage {
    Normalization,
    OuterProtection,
    FallbackCheck,
    InnerProtection,
    Preprocess,
}

/// Pure worker-ready output from corpus replay preparation.
///
/// `preprocessed` is false only when Deepwell's compatibility fallback is the
/// production path.  Such pages deliberately never enter FTML preprocessing
/// or parsing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CorpusReplayPreparedWikitext {
    pub wikitext: String,
    pub page_info: PageInfo<'static>,
    pub settings: WikitextSettings,
    pub id: PageId,
    pub included_pages: Vec<PageRef>,
    pub compatibility_fallback: bool,
    pub preprocessed: bool,
    pub timings: CorpusReplayStageTimings,
    pub features: CorpusReplaySyntaxFeatures,
    pub(super) wikidot_css_modules: Vec<String>,
    pub(super) wikidot_compat_html: CompatHtmlFragments,
    pub(super) wikidot_compat_text: CompatTextFragments,
    native_list_wikipedia_links: Vec<WikidotWikipediaLink>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CountPagesRequiredTagBatchResult {
    Exact(usize),
    PreserveLiteral,
}

#[derive(Debug)]
struct WikidotImageBlockArgument {
    value: String,
    attachment_owner: Option<AttachmentOwner>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProtectedWikidotCompatLink {
    pub(super) anchor: String,
    pub(super) marker: String,
}

#[derive(Debug)]
struct ExpandedRenderWikitext {
    wikitext: String,
    theme_previewer_blank: bool,
    included_pages: Vec<PageRef>,
    expanded_include_count: usize,
    url_offset_list_pages_content_bytes: usize,
    wikidot_compat_html: CompatHtmlFragments,
    wikidot_compat_text: CompatTextFragments,
    runtime_css_insertions: Vec<RuntimeCssInsertion>,
}

/// Owns the protection registries created during render preparation.
///
/// The registries must stay alive until every rendered body, HTML block, and
/// code block has crossed the Wikidot restoration boundary.  Callers borrow
/// the registries only inside [`RenderProtectionBundle::restore`]; dropping the bundle without
/// that consuming handoff is a programming error.
#[must_use = "restore the Wikidot protection bundle before dropping it"]
#[derive(Debug)]
pub(super) struct RenderProtectionBundle {
    wikidot_css_modules: Vec<String>,
    wikidot_inline_html: Vec<ProtectedWikidotInlineHtml>,
    wikidot_color_spans: ProtectedWikidotColorSpans,
    wikidot_compat_html: CompatHtmlFragments,
    wikidot_compat_text: CompatTextFragments,
    native_list_wikipedia_links: Vec<WikidotWikipediaLink>,
    restored: bool,
}

#[derive(Debug)]
struct RenderProtectionParts {
    wikidot_css_modules: Vec<String>,
    wikidot_compat_html: CompatHtmlFragments,
    wikidot_compat_text: CompatTextFragments,
    native_list_wikipedia_links: Vec<WikidotWikipediaLink>,
}

impl RenderProtectionBundle {
    fn new(
        wikidot_css_modules: Vec<String>,
        wikidot_inline_html: Vec<ProtectedWikidotInlineHtml>,
        wikidot_color_spans: ProtectedWikidotColorSpans,
        wikidot_compat_html: CompatHtmlFragments,
        wikidot_compat_text: CompatTextFragments,
        native_list_wikipedia_links: Vec<WikidotWikipediaLink>,
    ) -> Self {
        Self {
            wikidot_css_modules,
            wikidot_inline_html,
            wikidot_color_spans,
            wikidot_compat_html,
            wikidot_compat_text,
            native_list_wikipedia_links,
            restored: false,
        }
    }

    /// Run the restoration boundary and consume the guard.
    fn restore<R>(mut self, restore: impl FnOnce(&mut Self) -> R) -> R {
        let result = restore(&mut self);
        self.restored = true;
        result
    }

    /// Transfer registries to the replay transport representation.
    ///
    /// Replay preparation serializes these registries for a later worker, so
    /// the worker remains responsible for the eventual HTML restoration.
    fn into_parts(mut self) -> RenderProtectionParts {
        self.restored = true;
        RenderProtectionParts {
            wikidot_css_modules: std::mem::take(&mut self.wikidot_css_modules),
            wikidot_compat_html: std::mem::replace(
                &mut self.wikidot_compat_html,
                CompatHtmlFragments::new(""),
            ),
            wikidot_compat_text: std::mem::replace(
                &mut self.wikidot_compat_text,
                CompatTextFragments::new(""),
            ),
            native_list_wikipedia_links: std::mem::take(
                &mut self.native_list_wikipedia_links,
            ),
        }
    }

    fn take_css_modules(&mut self) -> Vec<String> {
        std::mem::take(&mut self.wikidot_css_modules)
            .into_iter()
            .map(|css| self.wikidot_compat_text.restore(&css))
            .collect()
    }

    fn take_native_list_wikipedia_links(&mut self) -> Vec<WikidotWikipediaLink> {
        std::mem::take(&mut self.native_list_wikipedia_links)
    }

    fn inline_html(&self) -> &[ProtectedWikidotInlineHtml] {
        &self.wikidot_inline_html
    }

    fn color_spans(&self) -> &ProtectedWikidotColorSpans {
        &self.wikidot_color_spans
    }

    fn compat_html(&self) -> &CompatHtmlFragments {
        &self.wikidot_compat_html
    }

    fn compat_text(&self) -> &CompatTextFragments {
        &self.wikidot_compat_text
    }

    fn compat_text_mut(&mut self) -> &mut CompatTextFragments {
        &mut self.wikidot_compat_text
    }
}

impl Drop for RenderProtectionBundle {
    fn drop(&mut self) {
        if !self.restored && !std::thread::panicking() {
            panic!("render protection bundle dropped before restoration");
        }
    }
}

#[derive(Debug)]
struct OuterPreparedRenderWikitext {
    wikitext: String,
    included_pages: Vec<PageRef>,
    url_offset_list_pages_content_bytes: usize,
    protection: RenderProtectionBundle,
    compatibility_fallback: bool,
    timings: CorpusReplayStageTimings,
}

pub(super) const MAX_INCLUDE_EXPANSION_DEPTH: usize = 8;
pub(super) const MAX_INCLUDE_EXPANSION_TOTAL: usize = 256;
// The frozen EN corpus contains a page with 1,266 direct includes. Only the
// trusted corpus finalizer receives this higher ceiling; user-controlled
// render paths retain the ordinary limit above.
const MAX_CORPUS_INCLUDE_EXPANSION_TOTAL: usize = 4096;
pub(super) const DEFAULT_LISTPAGES_PER_PAGE: u64 = 20;
pub(super) const MAX_LISTPAGES_RENDER_LIMIT: u64 = 250;
// Keep runtime-owned content expansion within the largest supported ListPages
// page. The render-scoped module, generated-output, include, and scan budgets
// still bound revision loading and nested expansion across the whole render.
pub(super) const MAX_LISTPAGES_CONTENT_ROWS_PER_RENDER: usize =
    MAX_LISTPAGES_RENDER_LIMIT as usize;
// Content-backed ListPages modules with selected rows can trigger revision
// loading and nested include expansion. Zero-row queries never load content
// and therefore do not consume this budget. Three nonempty modules cover the
// common corpus shape while stopping dense author-page compositions before
// they exhaust the render budget.
pub(super) const MAX_LISTPAGES_CONTENT_MODULES_PER_RENDER: usize = 3;
pub(super) const MAX_LISTPAGES_RENDER_OFFSET: u32 = 1_000;
pub(super) const MAX_LISTPAGES_RENDER_SCAN_ROWS: u32 = 50_000;
pub(super) const MAX_WIKIDOT_AJAX_MODULE_BODY_BYTES: usize = 65_536;
pub(super) const MAX_WIKIDOT_AJAX_MODULE_PARAMETERS: usize = 64;
pub(super) const MAX_WIKIDOT_AJAX_MODULE_PARAMETER_BYTES: usize = 4_096;
const LONG_NATIVE_LIST_RENDER_MIN_ITEMS: usize = 8;
const MAX_NATIVE_LIST_COMPAT_DEPTH: usize = 64;
pub(super) const MAX_FTML_COMPAT_PARSE_BYTES: usize = 768_000;
pub(super) const MAX_FTML_COMPAT_DENSE_PARSE_SCORE: usize = 180_000;
pub(super) const MAX_FTML_COMPAT_COLLAPSIBLE_BLOCKS: usize = 48;
pub(super) const MIN_FTML_COMPAT_TABBED_FALLBACK_BYTES: usize = 64_000;
pub(super) const MIN_FTML_COMPAT_TABBED_FALLBACK_MARKERS: usize = 12;
pub(super) const MIN_FTML_COMPAT_TABBED_RENDER_BYTES: usize = 100_000;
pub(super) const MIN_FTML_COMPAT_TABBED_MARKERS: usize = 10;
pub(super) const MIN_DENSE_FTML_COMPAT_RENDER_TIMEOUT_SECS: u64 = 150;
// Large ListPages content selected by a valid `/offset/<n>` is rendered for
// the page view rather than stored ahead of time. SCP-8980's observed fragment
// is large enough to exceed the ordinary standing timeout.
pub(super) const MIN_URL_OFFSET_LISTPAGES_CONTENT_BYTES: usize = 100_000;
pub(super) const MIN_URL_OFFSET_LISTPAGES_RENDER_TIMEOUT_SECS: u64 = 10;
pub(super) const INCLUDE_VARIABLE_OPEN_SENTINEL: &str = "__WIKIJUMP_INCLUDE_VAR_OPEN__";
pub(super) const INCLUDE_VARIABLE_CLOSE_SENTINEL: &str = "__WIKIJUMP_INCLUDE_VAR_CLOSE__";
const WIKIDOT_LITERAL_INCLUDE_SENTINEL: &str = "__WIKIJUMP_LITERAL_INCLUDE__";
pub(super) const WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTCOMPATHTML";
pub(super) const WIKIDOT_COMPAT_LINK_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTCOMPATLINK";
pub(super) const WIKIDOT_WIKIPEDIA_LINK_SENTINEL_PREFIX: &str =
    "WIKIJUMPWIKIDOTWIKIPEDIALINK";
pub(super) const WIKIDOT_WIKIPEDIA_LINK_SENTINEL_NONCE_LEN: usize = 32;
#[cfg(test)]
const WIKIDOT_COLOR_SPAN_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTCOLORSPAN";
pub(super) const WIKIDOT_INLINE_HTML_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTINLINEHTML";
pub(super) const WIKIDOT_RATE_ANCHOR_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTRATEANCHOR";
pub(super) const WIKIDOT_TABVIEW_SCRIPT: &str = "";
pub(super) const WIKIDOT_TABVIEW_INIT_SCRIPT: &str = r#"<script type="text/javascript" src="/wikidot/scripts/tabview-compat.js"></script>"#;
const MAX_WIKIDOT_COMPAT_FALLBACK_TITLE_LINKS: usize = 128;

pub(super) static INCLUDE_VARIABLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\$(?P<name>[a-zA-Z0-9_\-]+)\}").unwrap());
pub(super) static COUNTPAGES_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)\[\[module\s+CountPages(?P<head>(?:"[^"]*"|'[^']*'|[^\]])*)\]\](?:(?P<body>.*?)\[\[/module\]\])?"#,
    )
    .unwrap()
});
pub(super) static RATE_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+Rate(?P<head>(?:\s[^\]]*)?)\]\]").unwrap()
});
pub(super) static WIKIDOT_RATE_ANCHOR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"\[\[a href="javascript:;" onclick="(?P<onclick>WIKIDOT\.modules\.PageRateWidgetModule\.listeners\.(?:rate\(event, -?1\)|cancelVote\(event\)))" title="(?P<title>[^"]*)"\]\](?P<label>[^\[]*)\[\[/a\]\]"#,
    )
    .unwrap()
});
pub(super) static TAGCLOUD_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+TagCloud(?P<head>[^\]]*)\]\]").unwrap()
});
pub(super) static RATEDPAGES_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?im)^(?P<module>\[\[module[\t ]+RatedPages(?P<head>(?:[\t ]+[^\]\r\n]*)?)\]\])[\t ]*\r?$",
    )
    .unwrap()
});
pub(super) static PAGECALENDAR_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+PageCalendar(?P<head>[^\]]*)\]\]").unwrap()
});
pub(super) static REGISTRY_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)\[\[module\s+(?P<name>NewPage|Clone|Join)\b(?P<head>(?:[^\]"]+|"[^"]*")*)\]\]"#,
    )
    .unwrap()
});
pub(super) static GENERATED_LISTPAGES_HTML_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)<table class="wiki-content-table" data-wikijump-compat-listpages="1">.*?</table>|<ol data-wikijump-compat-listpages="1">.*?</ol>|<div class="feedinfo" data-wikijump-compat-listpages-feed="1">.*?</div>|<span class="odate time_-?[0-9]+ format_[A-Za-z0-9%_.-]+" data-wikijump-compat-date="1">[^<>]*</span>|<span class="printuser avatarhover" data-wikijump-compat-listpages-user="1">.*?</span>|<span class="page-rate-list-pages-start" data-rating="[^"]*" data-wikijump-compat-listpages-rating="1">[^<>]*</span>|<span data-wikijump-compat-listpages-preview="1" style="white-space: pre-wrap;">[^<>]*</span>"#,
    )
    .unwrap()
});
static WIKIDOT_DIRECT_HTML_IFRAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)^\s*(?P<iframe><iframe\b(?P<attrs>[^<>]*)>\s*</iframe\s*>)\s*$"#)
        .expect("direct Wikidot HTML iframe expression is valid")
});
static WIKIDOT_DIRECT_HTML_IFRAME_ATTRIBUTE_REGEX: LazyLock<Regex> = LazyLock::new(
    || {
        Regex::new(
            r#"(?is)\s+(?P<name>[a-z][a-z0-9:-]*)(?:\s*=\s*(?:"[^"<>]*"|'[^'<>]*'|[^\s"'=<>`]+))?"#,
        )
        .expect("direct Wikidot HTML iframe attribute expression is valid")
    },
);

#[cfg(test)]
static GENERATED_COMPAT_TABLE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)<table class="wiki-content-table" data-wikijump-compat-listpages="1">.*?</table>|<ol data-wikijump-compat-listpages="1">.*?</ol>|<div class="feedinfo" data-wikijump-compat-listpages-feed="1">.*?</div>|<div id="ml-[0-9]+" data-wikijump-compat-members="1"[^>]*>.*?</div>|<div class="backlinks-module-box" data-wikijump-compat-backlinks="1"[^>]*>.*?</div>|<div class="new-page-box" data-wikijump-compat-new-page="1"[^>]*>.*?</div>|<a class="button" data-wikijump-compat-clone="1"[^>]*>.*?</a>|<span class="odate time_-?[0-9]+ format_[A-Za-z0-9%_.-]+" data-wikijump-compat-date="1">[^<>]*</span>|<span class="page-rate-list-pages-start" data-rating="[^"]*" data-wikijump-compat-listpages-rating="1">[^<>]*</span>|<span data-wikijump-compat-listpages-preview="1" style="white-space: pre-wrap;">[^<>]*</span>"#,
    )
    .unwrap()
});
pub(super) static WIKIDOT_RESIDUAL_DIV_PARAGRAPH_REGEX: LazyLock<Regex> =
    LazyLock::new(|| {
        Regex::new(
        r#"(?is)<p>\s*(?:(?P<open>\[\[div[^\]]*\]\])|(?P<close>\[\[/div\]\]))\s*</p>"#,
    )
    .unwrap()
    });
pub(super) static WIKIJUMP_FOOTNOTE_MARKER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)<wj-footnote-ref-marker(?P<attrs>[^>]*)>(?P<label>.*?)</wj-footnote-ref-marker>"#,
    )
    .unwrap()
});
pub(super) static WIKIJUMP_FOOTNOTE_REF_SPAN_WRAPPER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| {
        Regex::new(
        r#"(?is)<span class="wj-footnote-ref">\s*(?P<body><sup class="footnoteref">.*?</sup>)\s*</span>"#,
    )
    .unwrap()
    });
pub(super) static WIKIJUMP_FOOTNOTE_REF_LEADING_SPACE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| {
        Regex::new(r#"(?s)(?P<before>\S)\s+(?P<footnote><span class="wj-footnote-ref">)"#)
            .unwrap()
    });
pub(super) static WIKIJUMP_FOOTNOTE_DATA_ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"data-id="(?P<id>[0-9]+)""#).unwrap());
static WIKIDOT_USER_INLINE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[\*user\s+(?P<name>[^\]]+)\]\]").unwrap());
pub(super) static WIKIDOT_CURRENT_PAGE_LINK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| {
        Regex::new(r"\[#(?:(?P<fragment>[^\s\]\n]+))?\s+(?P<label>[^\]\n]+)\]").unwrap()
    });
pub(super) static WIKIDOT_LABELED_EMAIL_LINK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| {
        Regex::new(
            r"\[(?P<email>[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+)\s+(?P<label>[^\]\n]+)\]",
        )
        .unwrap()
    });
pub(super) static WIKIDOT_STAR_LOCAL_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\*/(?P<target>[^\s\]\n]+)\s+(?P<label>[^\]\n]+)\]").unwrap()
});
pub(super) static WIKIDOT_LABELED_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\[\[(?P<target>[^\]|\n]+)\|(?P<label>[^\]\n]*)\]\]\]").unwrap()
});
static WIKIDOT_MULTILINE_LABELED_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)\[\[\[(?P<target>[^\]|\n]+)\|(?P<label>[^\]]*\n[^\]]*)\]\]\]")
        .unwrap()
});
pub(super) static WIKIDOT_QUADRUPLE_LINK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[\[\[(?P<target>[^\]\n]+)\]\]\]\]").unwrap());
pub(super) static WIKIDOT_UNLABELED_LINK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[\[(?P<target>[^\]\n]+)\]\]\]").unwrap());
static WIKIDOT_LOCAL_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[/(?P<target>[^\s\]\n]+)\s+(?P<label>[^\]\n]+)\]").unwrap()
});
static WIKIDOT_EXTERNAL_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\*?(?P<url>https?://[^\s\]]+)\s+(?P<label>[^\]]+)\]").unwrap()
});
pub(super) static WIKIDOT_WIKIPEDIA_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[wikipedia:(?P<target>[^\s\]\n]+)(?:\s+(?P<label>[^\]\n]+))?\]")
        .unwrap()
});
pub(super) static WIKIDOT_COLOR_SPAN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?P<hashes>#{2,})(?P<color>[A-Za-z0-9_-]+)\s*\|(?P<body>.*?)##").unwrap()
});
pub(super) static WIKIDOT_BOLD_UNDERLINE_SPAN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*\*__(?P<body>[^\n]*?)(?:__\*\*|\*\*__)").unwrap());
pub(super) static WIKIDOT_BOLD_OUTER_COLOR_SPAN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| {
        Regex::new(
            r"\*\*(?P<hashes>#{2,})(?P<color>[A-Za-z0-9_-]+)\s*\|(?P<body>[^\n]*?)##\*\*",
        )
        .unwrap()
    });
pub(super) static WIKIDOT_BOLD_COLOR_SPAN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\*\*(?P<hashes>#{2,})(?P<color>[A-Za-z0-9_-]+)\s*\|(?P<body>[^\n]*?)\*\*##",
    )
    .unwrap()
});
pub(super) static WIKIDOT_ESCAPED_NBSP_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"@<(?P<html>&nbsp;)>@").unwrap());
pub(super) static WIKIJUMP_CODE_BLOCK_PANEL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?is)<div class="wj-code-panel">.*?</div>"#).unwrap());
pub(super) static WIKIJUMP_CODE_BLOCK_OPEN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)<wj-code class="wj-code(?:\s+wj-language-(?P<language>[^"\s]+))?">"#,
    )
    .unwrap()
});
pub(super) static WIKIJUMP_TAB_BUTTON_LIST_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<div class="wj-tabs-button-list"[^>]*>(?P<body>.*?)</div>"#)
        .unwrap()
});
pub(super) static WIKIJUMP_TAB_PANEL_LIST_OPEN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| {
        Regex::new(r#"(?is)<div class="wj-tabs-panel-list"[^>]*>"#).unwrap()
    });
pub(super) static WIKIJUMP_SELECTED_TAB_BUTTON_REGEX: LazyLock<Regex> =
    LazyLock::new(|| {
        Regex::new(
        r#"(?is)<wj-tabs-button class="wj-tabs-button"[^>]*aria-selected="true"[^>]*>"#,
    )
    .unwrap()
    });
pub(super) static WIKIJUMP_TAB_BUTTON_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<wj-tabs-button class="wj-tabs-button"[^>]*>"#).unwrap()
});
pub(super) static WIKIJUMP_TAB_PANEL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?is)<div class="wj-tabs-panel"[^>]*>"#).unwrap());
pub(super) static WIKIDOT_TABVIEW_PANEL_ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<div id="wiki-tab-0-(?P<index>[0-9]+)">"#).unwrap());
static WIKIDOT_IMAGE_BLOCK_INCLUDE_START_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r#"(?is)\[\[include(?:[ \t\r\n]+|\[!--.*?--\])+"#,
        r#"(?::(?P<site>[A-Za-z0-9_-]+):)?component:image-block"#,
        r#"(?P<after>(?:[ \t\r\n]+|\[!--.*?--\])+|\||\]\])"#,
    ))
    .unwrap()
});
static WIKIDOT_INCLUDE_OPEN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\[\[\s*(?P<keyword>include)(?P<after>\s+|\[!--)").unwrap()
});
pub(super) static WIKIDOT_COMPAT_STYLE_BLOCK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| {
        Regex::new(r#"(?is)<style\b[^>]*\btype\s*=\s*["']text/css["'][^>]*>.*?</style>"#)
            .unwrap()
    });
pub(super) static WIKIJUMP_INLINE_MATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)<span class="wj-math wj-math-inline"><code class="wj-math-source wj-hidden"[^>]*>(?P<source>.*?)</code><wj-math-ml class="wj-math-ml">.*?</wj-math-ml></span>"#,
    )
    .unwrap()
});
pub(super) static WIKIDOT_EMAIL_SPAN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"<span class="wiki-email" style="visibility: visible;"><a href="mailto:([^"]+)">([^<]+)</a></span>"#,
    )
    .unwrap()
});
static WIKIDOT_EMAIL_CLASS_SPAN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<span class="wiki-email">(?P<body>[^<]*)</span>"#).unwrap()
});
static WIKIDOT_OBFUSCATED_EMAIL_BODY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^[A-Za-z0-9.-]+\|[A-Za-z0-9._%+-]+#[^<\r\n]+$"#).unwrap()
});
static WIKIDOT_RECOVERABLE_REVERSED_EMAIL_BODY_REGEX: LazyLock<Regex> = LazyLock::new(
    || {
        Regex::new(
            r#";tg&(?P<domain1>[A-Za-z0-9.-]+)\|(?P<user1>[A-Za-z0-9._%+-]+);tl&#.*?;tg&(?P<domain2>[A-Za-z0-9.-]+)\|(?P<user2>[A-Za-z0-9._%+-]+);tl&"#,
        )
        .unwrap()
    },
);
pub(super) static WIKIDOT_EMBED_PARAGRAPH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)<p>\[\[embed\]\]<br/?>(.*?)<br/?>\[\[/embed\]\]</p>"#).unwrap()
});
static WIKIDOT_LOCAL_FILE_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?P<quote>["'])(?:https?:)?//(?P<host>[A-Za-z0-9.-]+)(?::[0-9]+)?(?P<path>/local--(?:files|code|resized-images)/[^"'<>\s]+)"#,
    )
    .unwrap()
});
static WIKIDOT_LOCAL_FILE_CSS_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)(?P<prefix>url\(\s*["']?)(?:https?:)?//(?P<host>[A-Za-z0-9.-]+)(?::[0-9]+)?(?P<path>/local--(?:files|code|resized-images)/[^"')<>\s]+)"#,
    )
    .unwrap()
});

impl RenderService {
    fn owned_page_info(page_info: &PageInfo<'_>) -> PageInfo<'static> {
        PageInfo {
            page: Cow::Owned(page_info.page.to_string()),
            category: page_info
                .category
                .as_ref()
                .map(|category| Cow::Owned(category.to_string())),
            site: Cow::Owned(page_info.site.to_string()),
            title: Cow::Owned(page_info.title.to_string()),
            alt_title: page_info
                .alt_title
                .as_ref()
                .map(|title| Cow::Owned(title.to_string())),
            score: page_info.score,
            tags: page_info
                .tags
                .iter()
                .map(|tag| Cow::Owned(tag.to_string()))
                .collect(),
            language: Cow::Owned(page_info.language.to_string()),
        }
    }

    pub async fn render(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
    ) -> Result<RenderOutput> {
        let wikitext_len = wikitext.len();
        let render_cost_budget = RenderCostBudget::new_default();
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
            ..
        } = Box::pin(Self::render_inner(
            ctx,
            wikitext,
            page_info,
            settings,
            RenderInnerOptions {
                render_context: RenderContext::none(),
                viewer_user_id: None,
                current_page_data_form_values: None,
                max_include_expansions: MAX_INCLUDE_EXPANSION_TOTAL,
                render_cost_budget: render_cost_budget.clone(),
                trace: None,
                persist_compiled_text: true,
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

    /// Renders a Wikidot fragment in an existing page's site context without
    /// persisting compiled text or replacing that page's hosted text blocks.
    pub async fn render_wikidot_fragment_for_page(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        PageId {
            site_id,
            category_id,
            page_id,
        }: PageId,
    ) -> Result<RenderOutput> {
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let RenderInnerOutput {
            html_output,
            errors,
            compiled_hash,
            ..
        } = Box::pin(Self::render_inner(
            ctx,
            wikitext,
            page_info,
            &settings,
            RenderInnerOptions {
                render_context: RenderContext::page_nav(site_id, category_id, page_id),
                viewer_user_id: None,
                current_page_data_form_values: None,
                max_include_expansions: MAX_INCLUDE_EXPANSION_TOTAL,
                render_cost_budget: RenderCostBudget::new_default(),
                trace: None,
                persist_compiled_text: false,
                url: UrlArguments::default(),
            },
        ))
        .await?;

        Ok(RenderOutput {
            html_output,
            errors,
            compiled_hash,
            compiled_at: now(),
            compiled_generator: COMPILED_GENERATOR.clone(),
        })
    }

    pub async fn render_wikidot_list_pages_module(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        module_body: String,
        parameters: &BTreeMap<String, String>,
    ) -> Result<RenderOutput> {
        let make_error = || {
            Error::new(
                format!("failed to render Wikidot ListPages module in site ID {site_id}"),
                ErrorType::Render,
            )
        };
        let site = SiteService::get(ctx, Reference::Id(site_id))
            .await
            .or_raise(make_error)?;
        let request = build_wikidot_list_pages_module_request(module_body, parameters)
            .ok_or_raise(make_error)?;
        let page = request.page;
        let wikitext = request.source;
        let page_info = PageInfo {
            page: Cow::Borrowed("_ajax-module-connector"),
            category: None,
            site: Cow::Owned(site.slug),
            title: Cow::Borrowed(""),
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
            ..
        } = Box::pin(Self::render_inner(
            ctx,
            wikitext,
            &page_info,
            &settings,
            RenderInnerOptions {
                render_context: RenderContext::ajax_module(site_id),
                // The random ListPages cache retains only an order seed.
                // Re-evaluate candidates and visibility for the current AMC
                // actor on every request.
                viewer_user_id: ctx.request().user_id,
                current_page_data_form_values: None,
                max_include_expansions: MAX_INCLUDE_EXPANSION_TOTAL,
                render_cost_budget: RenderCostBudget::new_default(),
                trace: None,
                persist_compiled_text: false,
                // AMC carries pager state as `p`; selectors remain in the
                // synthesized module head while `p` follows the ordinary page
                // view path through UrlArguments.
                url: UrlArguments {
                    page,
                    ..UrlArguments::default()
                },
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

    /// Render a page.
    ///
    /// `url` carries the Wikidot URL path arguments when this render is
    /// serving a page view. Pass the default when producing a revision's
    /// stored HTML, which has no request behind it.
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
        url: UrlArguments<'_>,
    ) -> Result<RenderPageOutput> {
        Box::pin(Self::render_page_with_include_limit(
            ctx,
            wikitext,
            page_info,
            layout,
            PageId {
                site_id,
                category_id,
                page_id,
            },
            RenderPageOptions {
                max_include_expansions: MAX_INCLUDE_EXPANSION_TOTAL,
                viewer_user_id: None,
                current_page_data_form_values: None,
                persist_page_artifacts: true,
                url,
                trace: None,
            },
        ))
        .await
    }

    /// Render a stored revision from the same raw candidate values that are
    /// about to be committed.
    ///
    /// Category templates may contain ListPages selectors derived from the
    /// current page's data-form fields. Revision creation renders before the
    /// new revision row exists, so these values must travel with the candidate
    /// instead of being read back from the previous committed revision.
    pub(crate) async fn render_page_with_candidate_data_form_values(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        layout: Layout,
        id: PageId,
        current_page_data_form_values: BTreeMap<String, String>,
    ) -> Result<RenderPageOutput> {
        Box::pin(Self::render_page_with_include_limit(
            ctx,
            wikitext,
            page_info,
            layout,
            id,
            RenderPageOptions {
                max_include_expansions: MAX_INCLUDE_EXPANSION_TOTAL,
                viewer_user_id: None,
                current_page_data_form_values: Some(current_page_data_form_values),
                persist_page_artifacts: true,
                url: UrlArguments::default(),
                trace: None,
            },
        ))
        .await
    }

    /// Render a page for a request-specific viewer.
    ///
    /// This is used for page-view rerenders whose output depends on runtime
    /// request state. Revision compilation uses `render_page()` so
    /// viewer-specific modules cannot bake an editor's identity into stored
    /// HTML.
    pub async fn render_page_for_viewer(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        layout: Layout,
        id: PageId,
        viewer_user_id: Option<i64>,
        url: UrlArguments<'_>,
    ) -> Result<RenderPageOutput> {
        Box::pin(Self::render_page_with_include_limit(
            ctx,
            wikitext,
            page_info,
            layout,
            id,
            RenderPageOptions {
                max_include_expansions: MAX_INCLUDE_EXPANSION_TOTAL,
                viewer_user_id,
                current_page_data_form_values: None,
                persist_page_artifacts: false,
                url,
                trace: None,
            },
        ))
        .await
    }

    /// Render a trusted corpus-import page with its evidence-backed include ceiling.
    ///
    /// Callers must not expose this path to user-controlled page rendering.
    pub async fn render_corpus_page(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        layout: Layout,
        id: PageId,
    ) -> Result<RenderPageOutput> {
        Box::pin(Self::render_page_with_include_limit(
            ctx,
            wikitext,
            page_info,
            layout,
            id,
            RenderPageOptions {
                max_include_expansions: MAX_CORPUS_INCLUDE_EXPANSION_TOTAL,
                viewer_user_id: None,
                current_page_data_form_values: None,
                persist_page_artifacts: true,
                url: UrlArguments::default(),
                trace: None,
            },
        ))
        .await
    }

    pub(crate) async fn render_corpus_page_traced(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        layout: Layout,
        id: PageId,
        trace: &CorpusRenderTrace,
    ) -> Result<RenderPageOutput> {
        Box::pin(Self::render_page_with_include_limit(
            ctx,
            wikitext,
            page_info,
            layout,
            id,
            RenderPageOptions {
                max_include_expansions: MAX_CORPUS_INCLUDE_EXPANSION_TOTAL,
                viewer_user_id: None,
                current_page_data_form_values: None,
                persist_page_artifacts: true,
                url: UrlArguments::default(),
                trace: Some(trace),
            },
        ))
        .await
    }

    async fn render_page_with_include_limit(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        layout: Layout,
        PageId {
            site_id,
            category_id,
            page_id,
        }: PageId,
        RenderPageOptions {
            max_include_expansions,
            viewer_user_id,
            current_page_data_form_values,
            persist_page_artifacts,
            url,
            trace,
        }: RenderPageOptions<'_>,
    ) -> Result<RenderPageOutput> {
        let page_settings = WikitextSettings::from_mode(WikitextMode::Page, layout);
        let nav_settings = WikitextSettings::from_mode(WikitextMode::PageNav, layout);

        let wikitext_len = wikitext.len();
        let render_cost_budget = RenderCostBudget::new_default();
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
            mut html_output,
            errors,
            compiled_hash: compiled_body_html_hash,
            theme_previewer_blank,
            ..
        } = Self::render_inner(
            ctx,
            wikitext,
            page_info,
            &page_settings,
            RenderInnerOptions {
                render_context: if persist_page_artifacts {
                    RenderContext::page(site_id, category_id, page_id)
                } else {
                    RenderContext::page_view(site_id, category_id, page_id)
                },
                viewer_user_id,
                current_page_data_form_values: current_page_data_form_values.clone(),
                max_include_expansions,
                render_cost_budget: render_cost_budget.clone(),
                trace: trace.map(|trace| (trace, CorpusRenderScope::Body)),
                persist_compiled_text: persist_page_artifacts,
                url,
            },
        )
        .await
        .or_raise(make_error)?;

        let NavigationPageWikitext {
            top_bar_page_wikitext,
            side_bar_page_wikitext,
        } = SettingsService::get_nav_page_wikitext(ctx, site_id, Some(category_id))
            .await
            .or_raise(make_error)?;

        let nav_settings = &nav_settings;
        let render_nav_page = |wikitext, scope| {
            let render_cost_budget = render_cost_budget.clone();
            let current_page_data_form_values = current_page_data_form_values.clone();
            async move {
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
                            nav_settings,
                            RenderInnerOptions {
                                render_context: RenderContext::page_nav(
                                    site_id,
                                    category_id,
                                    page_id,
                                ),
                                viewer_user_id,
                                current_page_data_form_values,
                                max_include_expansions,
                                render_cost_budget: render_cost_budget.clone(),
                                trace: trace.map(|trace| (trace, scope)),
                                persist_compiled_text: persist_page_artifacts,
                                // Whether a nav bar's own modules read the viewed
                                // page's URL arguments is not captured, so they
                                // keep rendering as they do without one.
                                url: UrlArguments::default(),
                            },
                        )
                        .await;

                        match result {
                            Ok(RenderInnerOutput {
                                html_output,
                                compiled_hash,
                                ..
                            }) => Ok(Some((compiled_hash, html_output.styles))),
                            Err(error) => Err(error),
                        }
                    }

                    // No nav page
                    None => Ok(None),
                }
            }
        };

        let (top_bar_render_result, side_bar_render_result) = join!(
            render_nav_page(top_bar_page_wikitext, CorpusRenderScope::TopNav),
            render_nav_page(side_bar_page_wikitext, CorpusRenderScope::SideNav),
        );
        let (top_bar_render, side_bar_render) =
            raise_multiple!(top_bar_render_result, side_bar_render_result; make_error);
        let (compiled_top_bar_html_hash, top_bar_styles) = top_bar_render
            .map(|(hash, styles)| (Some(hash), styles))
            .unwrap_or_default();
        let (compiled_side_bar_html_hash, side_bar_styles) = side_bar_render
            .map(|(hash, styles)| (Some(hash), styles))
            .unwrap_or_default();

        let body_styles = std::mem::take(&mut html_output.styles);
        html_output.styles = top_bar_styles;
        html_output.styles.extend(side_bar_styles);
        html_output.styles.extend(body_styles);
        let styles_json =
            super::compiled_styles::encode(&html_output.styles, theme_previewer_blank)
                .or_raise(make_error)?;
        let compiled_body_styles_hash = Self::persist_or_hash_compiled_text(
            ctx,
            None,
            &styles_json,
            persist_page_artifacts,
            make_error,
        )
        .await?;

        Ok(RenderPageOutput {
            html_output,
            errors,
            compiled_body_html_hash,
            compiled_body_styles_hash,
            compiled_top_bar_html_hash,
            compiled_side_bar_html_hash,
            compiled_at: now(),
            compiled_generator: COMPILED_GENERATOR.clone(),
        })
    }

    /// Expand trusted corpus page wikitext exactly as the production page
    /// renderer does, stopping before the pure normalization/protection pass.
    ///
    /// The returned value is intentionally owned and serializable so a replay
    /// controller can hand it to an isolated worker without giving that worker
    /// database or service credentials.
    pub(crate) async fn expand_corpus_replay_wikitext(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: PageInfo<'static>,
        settings: WikitextSettings,
        id: PageId,
    ) -> Result<CorpusReplayExpandedWikitext> {
        let expanded = Self::expand_render_wikitext(
            ctx,
            wikitext,
            &page_info,
            &settings,
            RenderExpansionOptions {
                current_site_id: Some(id.site_id),
                current_category_id: Some(id.category_id),
                current_page_id: Some(id.page_id),
                viewer_user_id: None,
                current_page_data_form_values: None,
                max_include_expansions: MAX_CORPUS_INCLUDE_EXPANSION_TOTAL,
                render_cost_budget: RenderCostBudget::new_default(),
                trace: None,
                // Corpus replay renders stored wikitext, never a live request.
                url: UrlArguments::default(),
                list_pages_pager_route: super::list_pages::ListPagesPagerRoute::SavedPage,
                page_preview: false,
                suppress_nested_list_pages: false,
            },
        )
        .await?;

        Ok(CorpusReplayExpandedWikitext {
            wikitext: expanded.wikitext,
            page_info,
            settings,
            id,
            included_pages: expanded.included_pages,
            wikidot_compat_html: expanded.wikidot_compat_html,
            wikidot_compat_text: expanded.wikidot_compat_text,
        })
    }

    /// Finish the pure portion of production render preparation for an
    /// isolated corpus replay worker.
    #[allow(dead_code)]
    pub(crate) fn prepare_corpus_replay_wikitext(
        input: CorpusReplayExpandedWikitext,
    ) -> CorpusReplayPreparedWikitext {
        Self::prepare_corpus_replay_wikitext_with_observer(input, |_| {})
    }

    pub(crate) fn prepare_corpus_replay_wikitext_with_observer(
        input: CorpusReplayExpandedWikitext,
        mut observer: impl FnMut(CorpusReplayPreparationStage),
    ) -> CorpusReplayPreparedWikitext {
        let CorpusReplayExpandedWikitext {
            wikitext,
            page_info,
            settings,
            id,
            included_pages,
            wikidot_compat_html,
            wikidot_compat_text,
        } = input;
        let expanded = ExpandedRenderWikitext {
            theme_previewer_blank: false,
            wikitext,
            included_pages,
            expanded_include_count: 0,
            url_offset_list_pages_content_bytes: 0,
            wikidot_compat_html,
            wikidot_compat_text,
            runtime_css_insertions: Vec::new(),
        };
        let outer = Self::prepare_outer_render_wikitext_observed(
            expanded,
            &page_info,
            &settings,
            &mut observer,
        );

        if outer.compatibility_fallback {
            let RenderProtectionParts {
                wikidot_css_modules,
                wikidot_compat_html,
                wikidot_compat_text,
                native_list_wikipedia_links,
                ..
            } = outer.protection.into_parts();
            let features = corpus_replay_syntax_features(&outer.wikitext);
            return CorpusReplayPreparedWikitext {
                wikitext: outer.wikitext,
                page_info,
                settings,
                id,
                included_pages: outer.included_pages,
                compatibility_fallback: true,
                preprocessed: false,
                timings: outer.timings,
                features,
                wikidot_css_modules,
                wikidot_compat_html,
                wikidot_compat_text,
                native_list_wikipedia_links,
            };
        }

        let inner =
            Self::prepare_inner_render_wikitext_observed(outer, &settings, &mut observer);
        let RenderProtectionParts {
            wikidot_css_modules,
            wikidot_compat_html,
            wikidot_compat_text,
            native_list_wikipedia_links,
            ..
        } = inner.protection.into_parts();
        let features = corpus_replay_syntax_features(&inner.wikitext);
        CorpusReplayPreparedWikitext {
            wikitext: inner.wikitext,
            page_info,
            settings,
            id,
            included_pages: inner.included_pages,
            compatibility_fallback: false,
            preprocessed: true,
            timings: inner.timings,
            features,
            wikidot_css_modules,
            wikidot_compat_html,
            wikidot_compat_text,
            native_list_wikipedia_links,
        }
    }

    async fn expand_render_wikitext(
        ctx: &ServiceContext<'_>,
        mut wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        options: RenderExpansionOptions<'_>,
    ) -> Result<ExpandedRenderWikitext> {
        let RenderExpansionOptions {
            current_site_id,
            current_category_id,
            current_page_id,
            viewer_user_id,
            current_page_data_form_values,
            max_include_expansions,
            render_cost_budget,
            trace,
            url,
            list_pages_pager_route,
            page_preview,
            suppress_nested_list_pages,
        } = options;
        let backlinks_page_id = current_page_id;
        // PagePreviewModule carries `page_unix_name`/`pageId`, and the legacy
        // NextPage/PreviousPage modules resolve their adjacency from that
        // identity. Other preview modules keep rendering without a current
        // page until their own preview contract is evidenced.
        let next_previous_page_id = current_page_id;
        let current_page_id = (!page_preview).then_some(current_page_id).flatten();
        let make_error =
            || Error::new("failed to perform render operation", ErrorType::Render);
        let mut include_budget = IncludeExpansionBudget::new(max_include_expansions);
        let initial_include_expansions = include_budget.remaining;
        let mut include_source_cache = IncludeSourceCache::default();
        let metacomponent_context = if page_info
            .tags
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case("component"))
        {
            MetacomponentSourceContext::RootComponent
        } else {
            MetacomponentSourceContext::RootNonComponent
        };
        select_metacomponent_documentation(&mut wikitext, metacomponent_context);
        let mut wikidot_compat_text = CompatTextFragments::new(&wikitext);
        let mut included_pages = {
            let _stage = StageGuard::new(trace, CorpusRenderStage::ImagePrelude);
            if settings.enable_page_syntax {
                Self::expand_wikidot_image_block_includes(&mut wikitext, page_info, None)
            } else {
                Vec::new()
            }
        };
        let IncludeExpansion {
            wikitext: expanded_wikitext,
            included_pages: expanded_included_pages,
            expanded_include_count,
        } = {
            let _stage = StageGuard::new(trace, CorpusRenderStage::Includes);
            Self::expand_includes(
                ctx,
                wikitext,
                page_info,
                page_info.site.as_ref(),
                settings,
                IncludeExpansionOptions {
                    current_site_id,
                    source_attachment_owner: None,
                    source_cache: &mut include_source_cache,
                    compat_text: &mut wikidot_compat_text,
                    expand_wikidot_image_blocks: true,
                    page_preview,
                    budget: include_budget,
                    render_cost_budget: render_cost_budget.clone(),
                },
            )
            .await
            .or_raise(make_error)?
        };
        wikitext = expanded_wikitext;
        included_pages.extend(expanded_included_pages);
        include_budget.consume(expanded_include_count);
        {
            let _stage = StageGuard::new(trace, CorpusRenderStage::PostInclude);
            // Include-branch cleanup treats a standalone `[!-- --]` line as
            // a generic boundary. Generated parser-function gates use the
            // same Wikidot token as their delayed branch closer, so protect
            // only the structurally recognized gate before cleanup and put
            // the boundary back immediately afterwards.
            let generated_comment_gates =
                protect_generated_parser_function_comment_gates(&mut wikitext);
            remove_unresolved_include_comment_branches(&mut wikitext);
            if let Some((opening, closing, standalone_closing)) = generated_comment_gates
            {
                wikitext = wikitext
                    .replace(&opening, "[!--")
                    .replace(&closing, "--]")
                    .replace(&standalone_closing, "[!-- --]");
            }
            Self::prepare_wikidot_conditionals_for_include_expansion(
                &mut wikitext,
                page_info,
                &mut wikidot_compat_text,
            );
            neutralize_authored_markers(&mut wikitext);
        }
        let mut wikidot_compat_html = CompatHtmlFragments::new(&wikitext);
        if suppress_nested_list_pages {
            wikitext = replace_recursive_list_pages_with_error(
                &wikitext,
                &mut wikidot_compat_html,
                &render_cost_budget,
            );
        }
        let ListPagesExpansion {
            wikitext: expanded_wikitext,
            included_pages: list_pages_included_pages,
            expanded_include_count: list_pages_expanded_include_count,
            url_offset_content_bytes,
            runtime_css_insertions,
        } = {
            let _stage = StageGuard::new(trace, CorpusRenderStage::ListPages);
            let protected_css =
                protect_css_modules_before_first_list_pages(&mut wikitext, settings);
            let mut expansion = Self::expand_list_pages(
                ctx,
                wikitext,
                page_info,
                settings,
                &mut wikidot_compat_html,
                &mut include_source_cache,
                &mut wikidot_compat_text,
                ListPagesExpansionOptions {
                    current_site_id,
                    current_page_id,
                    current_page_data_form_values,
                    page_preview,
                    viewer_user_id,
                    include_budget,
                    render_cost_budget: render_cost_budget.clone(),
                    url,
                    pager_route: list_pages_pager_route,
                },
            )
            .await
            .or_raise(make_error)?;
            if let Some(protected_css) = protected_css {
                expansion.wikitext = protected_css.restore(&expansion.wikitext);
            }
            expansion
        };
        wikitext = expanded_wikitext;
        included_pages.extend(list_pages_included_pages);
        include_budget.consume(list_pages_expanded_include_count);
        let theme_previewer_blank =
            !page_preview && super::runtime_modules::has_theme_previewer_blank(&wikitext);
        wikitext = Box::pin(Self::expand_secondary_runtime_modules(
            ctx,
            wikitext,
            page_info,
            settings,
            SecondaryRuntimeModuleExpansionOptions {
                current_site_id,
                current_page_id,
                viewer_user_id,
                page_preview,
                url,
                trace,
            },
            &mut wikidot_compat_text,
            &mut wikidot_compat_html,
        ))
        .await
        .or_raise(make_error)?;
        wikitext = {
            let _stage = StageGuard::new(trace, CorpusRenderStage::Backlinks);
            let wikitext = Self::expand_backlinks_modules(
                ctx,
                wikitext,
                settings,
                current_site_id,
                backlinks_page_id,
                &mut wikidot_compat_html,
            )
            .await
            .or_raise(make_error)?;
            Self::expand_link_listing_modules(
                ctx,
                wikitext,
                settings,
                current_site_id,
                &mut wikidot_compat_html,
            )
            .await
            .or_raise(make_error)?
        };
        wikitext = Self::expand_categories_and_page_tree_modules(
            ctx,
            wikitext,
            settings,
            (current_site_id, current_page_id),
            viewer_user_id,
            &mut wikidot_compat_html,
        )
        .await
        .or_raise(make_error)?;
        wikitext = {
            let _stage = StageGuard::new(trace, CorpusRenderStage::ChildPages);
            Self::expand_child_pages_modules(
                ctx,
                wikitext,
                settings,
                current_site_id,
                current_page_id,
                &mut wikidot_compat_html,
            )
            .await
            .or_raise(make_error)?
        };
        let NextPreviousPageExpansion {
            wikitext: expanded_wikitext,
            included_pages: next_previous_included_pages,
        } = {
            let _stage = StageGuard::new(trace, CorpusRenderStage::NextPreviousPage);
            Self::expand_next_previous_page_modules(
                ctx,
                wikitext,
                page_info,
                settings,
                current_site_id,
                next_previous_page_id,
                &mut include_budget,
                &render_cost_budget,
                url,
                &mut wikidot_compat_html,
                &mut wikidot_compat_text,
            )
            .await
            .or_raise(make_error)?
        };
        wikitext = expanded_wikitext;
        included_pages.extend(next_previous_included_pages);
        wikitext = expand_page_index_modules(
            ctx,
            wikitext,
            page_info,
            settings,
            current_site_id,
            url,
            trace,
            &mut wikidot_compat_html,
        )
        .await
        .or_raise(make_error)?;
        {
            let _stage = StageGuard::new(trace, CorpusRenderStage::RegistryModules);
            wikitext = Box::pin(Self::expand_new_page_modules_with_registry(
                ctx,
                wikitext,
                settings,
                page_info.language.as_ref(),
                current_site_id,
                &mut wikidot_compat_html,
            ))
            .await
            .or_raise(make_error)?;
            wikitext = Self::expand_registry_modules_with_registry(
                wikitext,
                settings,
                &mut wikidot_compat_html,
            );
            let has_rate_module = RATE_MODULE_REGEX.is_match(&wikitext);
            let rating_type =
                match (has_rate_module, current_site_id, current_category_id) {
                    (true, Some(site_id), Some(category_id)) => {
                        SettingsService::get_page_rating_settings(
                            ctx,
                            site_id,
                            category_id,
                        )
                        .await
                        .or_raise(make_error)?
                        .rating_type
                    }
                    _ => PageRatingType::PlusMinus,
                };
            let rate_score = match (has_rate_module, current_page_id, page_preview) {
                (true, Some(page_id), false) => ScoreService::score(ctx, page_id)
                    .await
                    .or_raise(make_error)?,
                _ => page_info.score,
            };
            let rating_votes = match (has_rate_module, current_page_id, page_preview) {
                (true, Some(page_id), false) => Some(
                    PageQueryService::effective_vote_count(ctx, page_id)
                        .await
                        .or_raise(make_error)?,
                ),
                _ => None,
            };
            wikitext = Self::expand_rate_modules_with_registry(
                wikitext,
                page_info,
                settings,
                RateModuleContext {
                    rating_type,
                    score: rate_score,
                    rating_votes,
                },
                &mut wikidot_compat_html,
                &mut wikidot_compat_text,
            );
        }
        if settings.enable_page_syntax && has_wikidot_iframe_syntax(&wikitext) {
            wikitext = expand_wikidot_iframe_syntax(wikitext, &mut wikidot_compat_html);
        }
        wikitext = Self::finalize_runtime_module_residuals(
            wikitext,
            settings,
            page_preview,
            &mut wikidot_compat_text,
            &mut wikidot_compat_html,
        );

        if let Some((trace, CorpusRenderScope::Body)) = trace {
            trace.set_dimension(CorpusRenderDimension::ExpandedBytes, wikitext.len());
            trace.set_dimension(
                CorpusRenderDimension::IncludedPages,
                included_pages.len(),
            );
        }

        Ok(ExpandedRenderWikitext {
            theme_previewer_blank,
            wikitext,
            included_pages,
            expanded_include_count: initial_include_expansions
                .saturating_sub(include_budget.remaining),
            url_offset_list_pages_content_bytes: url_offset_content_bytes,
            wikidot_compat_html,
            wikidot_compat_text,
            runtime_css_insertions,
        })
    }

    fn prepare_outer_render_wikitext(
        expanded: ExpandedRenderWikitext,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
    ) -> OuterPreparedRenderWikitext {
        Self::prepare_outer_render_wikitext_observed(
            expanded,
            page_info,
            settings,
            &mut |_| {},
        )
    }

    fn prepare_outer_render_wikitext_observed(
        mut expanded: ExpandedRenderWikitext,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        observer: &mut impl FnMut(CorpusReplayPreparationStage),
    ) -> OuterPreparedRenderWikitext {
        let mut timings = CorpusReplayStageTimings::default();
        let mut wikidot_compat_text = expanded.wikidot_compat_text;

        observer(CorpusReplayPreparationStage::Normalization);
        let started = Instant::now();
        if settings.enable_page_syntax {
            // Includes and runtime modules can introduce literal Wikidot
            // conditionals after the pre-expansion pass. Resolve that generated
            // context-free syntax before FTML sees it as anchor markup.
            Self::prepare_wikidot_conditionals_for_include_expansion(
                &mut expanded.wikitext,
                page_info,
                &mut wikidot_compat_text,
            );
            Self::normalize_wikidot_cross_closed_div_collapsibles(&mut expanded.wikitext);
            Self::normalize_wikidot_div_style_url_quotes(&mut expanded.wikitext);
            Self::protect_wikidot_unbound_include_variables(
                &mut expanded.wikitext,
                &mut wikidot_compat_text,
            );
            Self::normalize_wikidot_multiline_page_links(&mut expanded.wikitext);
        }
        timings.normalization_us = elapsed_micros(started);

        observer(CorpusReplayPreparationStage::OuterProtection);
        let started = Instant::now();
        let wikidot_inline_html =
            Self::protect_wikidot_inline_html_spans(&mut expanded.wikitext, settings);
        let wikidot_color_spans =
            Self::protect_wikidot_color_spans(&mut expanded.wikitext, settings);
        expanded.wikitext = Self::protect_unrendered_wikidot_color_markers(
            expanded.wikitext,
            settings,
            &mut wikidot_compat_text,
        );
        let (rendered, native_list_wikipedia_links) =
            Self::render_long_native_list_runs_with_registry(
                expanded.wikitext,
                &mut expanded.wikidot_compat_html,
            );
        expanded.wikitext = rendered;
        let wikidot_css_modules = extract_css_modules_with_runtime_insertions(
            &mut expanded.wikitext,
            page_info,
            settings,
            &mut expanded.wikidot_compat_html,
            std::mem::take(&mut expanded.runtime_css_insertions),
        );
        timings.outer_protection_us = elapsed_micros(started);

        observer(CorpusReplayPreparationStage::FallbackCheck);
        let started = Instant::now();
        let compatibility_fallback = Self::should_use_wikidot_compatibility_fallback(
            &expanded.wikitext,
            page_info,
        );
        timings.fallback_check_us = elapsed_micros(started);

        OuterPreparedRenderWikitext {
            wikitext: expanded.wikitext,
            included_pages: expanded.included_pages,
            url_offset_list_pages_content_bytes: expanded
                .url_offset_list_pages_content_bytes,
            protection: RenderProtectionBundle::new(
                wikidot_css_modules,
                wikidot_inline_html,
                wikidot_color_spans,
                expanded.wikidot_compat_html,
                wikidot_compat_text,
                native_list_wikipedia_links,
            ),
            compatibility_fallback,
            timings,
        }
    }

    fn prepare_inner_render_wikitext(
        outer: OuterPreparedRenderWikitext,
        settings: &WikitextSettings,
    ) -> InnerPreparedRenderWikitext {
        Self::prepare_inner_render_wikitext_observed(outer, settings, &mut |_| {})
    }

    fn prepare_inner_render_wikitext_observed(
        mut outer: OuterPreparedRenderWikitext,
        settings: &WikitextSettings,
        observer: &mut impl FnMut(CorpusReplayPreparationStage),
    ) -> InnerPreparedRenderWikitext {
        debug_assert!(!outer.compatibility_fallback);

        observer(CorpusReplayPreparationStage::InnerProtection);
        let started = Instant::now();
        let wikidot_compat_links =
            Self::protect_wikidot_compat_links(&mut outer.wikitext, settings);
        let wikidot_wikipedia_links =
            Self::protect_wikidot_wikipedia_links(&mut outer.wikitext, settings);
        let wikidot_embed_iframes =
            Self::protect_wikidot_embed_iframes(&mut outer.wikitext);
        outer.timings.inner_protection_us = elapsed_micros(started);

        observer(CorpusReplayPreparationStage::Preprocess);
        let started = Instant::now();
        ftml::preprocess_for_layout(&mut outer.wikitext, settings.layout);
        outer.timings.preprocess_us = elapsed_micros(started);

        InnerPreparedRenderWikitext {
            wikitext: outer.wikitext,
            included_pages: outer.included_pages,
            protection: outer.protection,
            wikidot_compat_links,
            wikidot_wikipedia_links,
            wikidot_embed_iframes,
            timings: outer.timings,
        }
    }

    pub(in crate::services::render) fn rewrite_wikidot_html_block_iframe_urls(
        mut body: String,
        page_info: &PageInfo<'_>,
        html_blocks: &[String],
    ) -> String {
        const PLACEHOLDER: &str = concat!(
            r#"<iframe src="https://example.com/" allowtransparency="true" "#,
            r#"frameborder="0" class="html-block-iframe"></iframe>"#,
        );
        if html_blocks.is_empty() || !body.contains(PLACEHOLDER) {
            return body;
        }

        let full_slug = Self::page_info_full_slug(page_info);
        // PagePreviewModule uses `search:site` for the otherwise anonymous
        // preview page identity. Keep that route fallback local to HTML-block
        // registration: treating it as the query's current page changes
        // ListPages selection and exclusion semantics.
        let full_slug = if full_slug.is_empty() {
            "search:site".to_owned()
        } else {
            full_slug
        };
        let mut search_start = 0;
        for index in 1..=html_blocks.len() {
            let Some(relative_start) = body[search_start..].find(PLACEHOLDER) else {
                break;
            };
            let start = search_start + relative_start;
            let end = start + PLACEHOLDER.len();
            let iframe = Self::wikidot_direct_html_iframe(&html_blocks[index - 1])
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    format!(
                        concat!(
                            r#"<iframe src="/{}/html/{}" allowtransparency="true" "#,
                            r#"frameborder="0" class="html-block-iframe"></iframe>"#,
                        ),
                        escape_list_pages_html_attr(&full_slug),
                        index,
                    )
                });
            body.replace_range(start..end, &iframe);
            search_start = start + iframe.len();
        }
        body
    }

    fn wikidot_direct_html_iframe(html_block: &str) -> Option<&str> {
        const ALLOWED_ATTRIBUTES: &[&str] = &[
            "align",
            "allow",
            "allowfullscreen",
            "class",
            "frameborder",
            "height",
            "loading",
            "referrerpolicy",
            "sandbox",
            "scrolling",
            "src",
            "style",
            "title",
            "width",
        ];

        let captures = WIKIDOT_DIRECT_HTML_IFRAME_REGEX.captures(html_block)?;
        let attributes = captures
            .name("attrs")
            .expect("direct iframe capture has attributes")
            .as_str();
        let mut cursor = 0;
        let mut has_attribute = false;
        for attribute in
            WIKIDOT_DIRECT_HTML_IFRAME_ATTRIBUTE_REGEX.captures_iter(attributes)
        {
            let whole = attribute
                .get(0)
                .expect("direct iframe attribute capture has whole match");
            if !attributes[cursor..whole.start()].trim().is_empty() {
                return None;
            }
            let name = attribute
                .name("name")
                .expect("direct iframe attribute capture has name")
                .as_str()
                .to_ascii_lowercase();
            if !ALLOWED_ATTRIBUTES.contains(&name.as_str()) {
                return None;
            }
            has_attribute = true;
            cursor = whole.end();
        }
        if !attributes[cursor..].trim().is_empty() || !has_attribute {
            return None;
        }
        captures.name("iframe").map(|iframe| iframe.as_str())
    }

    pub(super) async fn render_inner(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        options: RenderInnerOptions<'_>,
    ) -> Result<RenderInnerOutput> {
        // Recursive ListPages/content renders can otherwise complete several
        // ready futures synchronously on the caller's stack. Yield once at
        // this boundary so each nested render resumes from its heap-owned
        // future instead of accumulating poll frames.
        tokio::task::yield_now().await;
        let RenderInnerOptions {
            render_context,
            viewer_user_id,
            current_page_data_form_values,
            max_include_expansions,
            render_cost_budget,
            trace,
            persist_compiled_text,
            url,
        } = options;
        let trusted_corpus_render =
            max_include_expansions == MAX_CORPUS_INCLUDE_EXPANSION_TOTAL;
        let list_pages_pager_route = render_context.list_pages_pager_route();
        let RenderContext {
            current_site_id,
            current_category_id,
            current_page_id,
            text_block_page_id,
            lifecycle,
            suppress_nested_list_pages,
        } = render_context;
        let allow_wikidot_styleframe = matches!(
            lifecycle,
            RenderLifecycle::SavedPage | RenderLifecycle::PagePreview
        );

        if let Some((trace, CorpusRenderScope::Body)) = trace {
            trace.set_dimension(CorpusRenderDimension::SourceBytes, wikitext.len());
        }

        let make_error =
            || Error::new("failed to perform render operation", ErrorType::Render);
        let current_site = {
            let _stage = StageGuard::new(trace, CorpusRenderStage::SiteLoad);
            match current_site_id {
                Some(site_id) => Some(
                    SiteService::get(ctx, Reference::Id(site_id))
                        .await
                        .or_raise(make_error)?,
                ),
                None => None,
            }
        };

        let expanded = Self::expand_render_wikitext(
            ctx,
            wikitext,
            page_info,
            settings,
            RenderExpansionOptions {
                current_site_id,
                current_category_id,
                current_page_id,
                viewer_user_id,
                current_page_data_form_values,
                max_include_expansions,
                render_cost_budget,
                trace,
                url,
                list_pages_pager_route,
                page_preview: lifecycle == RenderLifecycle::PagePreview,
                suppress_nested_list_pages,
            },
        )
        .await?;
        let render_current_page_id = (lifecycle != RenderLifecycle::PagePreview)
            .then_some(current_page_id)
            .flatten();
        Box::pin(Self::render_inner_expanded(
            ctx,
            expanded,
            page_info,
            settings,
            current_site_id,
            render_current_page_id,
            viewer_user_id,
            text_block_page_id,
            allow_wikidot_styleframe,
            lifecycle == RenderLifecycle::PagePreview,
            current_site,
            trace,
            persist_compiled_text,
            trusted_corpus_render,
        ))
        .await
    }

    pub(super) async fn render_wikidot_syntax_inner(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        current_site_id: i64,
        viewer_user_id: Option<i64>,
        current_site: SiteModel,
    ) -> Result<RenderInnerOutput> {
        Self::render_inner_expanded(
            ctx,
            ExpandedRenderWikitext {
                theme_previewer_blank: false,
                wikidot_compat_html: CompatHtmlFragments::new(&wikitext),
                wikidot_compat_text: CompatTextFragments::new(&wikitext),
                wikitext,
                included_pages: Vec::new(),
                expanded_include_count: 0,
                url_offset_list_pages_content_bytes: 0,
                runtime_css_insertions: Vec::new(),
            },
            page_info,
            settings,
            Some(current_site_id),
            None,
            viewer_user_id,
            None,
            true,
            true,
            Some(current_site),
            None,
            false,
            false,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn render_inner_expanded(
        ctx: &ServiceContext<'_>,
        expanded: ExpandedRenderWikitext,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        current_page_id: Option<i64>,
        viewer_user_id: Option<i64>,
        text_block_page_id: Option<i64>,
        allow_wikidot_styleframe: bool,
        page_preview: bool,
        current_site: Option<SiteModel>,
        trace: Option<(&CorpusRenderTrace, CorpusRenderScope)>,
        persist_compiled_text: bool,
        trusted_corpus_render: bool,
    ) -> Result<RenderInnerOutput> {
        let config = ctx.config();
        let make_error =
            || Error::new("failed to perform render operation", ErrorType::Render);
        let theme_previewer_blank = expanded.theme_previewer_blank && !page_preview;
        let expanded_include_count = expanded.expanded_include_count;
        let mut expanded = expanded;
        if !allow_wikidot_styleframe {
            Self::neutralize_untrusted_wikidot_styleframe_embeds(&mut expanded.wikitext);
        }
        let outer = Self::prepare_outer_render_wikitext(expanded, page_info, settings);
        if let Some((trace, scope)) = trace {
            trace.add_us(
                scope,
                CorpusRenderStage::Normalization,
                outer.timings.normalization_us,
            );
            trace.add_us(
                scope,
                CorpusRenderStage::OuterProtect,
                outer.timings.outer_protection_us,
            );
            trace.add_us(
                scope,
                CorpusRenderStage::FallbackCheck,
                outer.timings.fallback_check_us,
            );
        }
        if outer.compatibility_fallback {
            let OuterPreparedRenderWikitext {
                wikitext,
                included_pages,
                url_offset_list_pages_content_bytes: _,
                protection,
                compatibility_fallback: _,
                timings: _,
            } = outer;
            let fallback_link_titles = {
                let _stage = StageGuard::new(trace, CorpusRenderStage::FallbackTitles);
                if let (Some(site_id), Some(site)) =
                    (current_site_id, current_site.as_ref())
                {
                    Self::load_wikidot_compat_fallback_link_titles(
                        ctx,
                        site_id,
                        &site.slug,
                        &wikitext,
                        viewer_user_id,
                    )
                    .await
                    .or_raise(make_error)?
                } else {
                    WikidotCompatLinkTitleMap::new()
                }
            };
            let fallback_render_stage =
                StageGuard::new(trace, CorpusRenderStage::FallbackRender);
            let fallback_output = Self::render_oversized_wikidot_compatibility_fallback(
                &wikitext,
                current_site.as_ref(),
                config,
                settings
                    .enable_html_blocks
                    .then_some(page_info.page.as_ref()),
                Some(&fallback_link_titles),
            );
            let (
                wikidot_css_modules,
                fallback_html_block_texts,
                fallback_code_blocks,
                body,
                backlinks,
            ) = protection.restore(|protection| {
                let mut backlinks = ftml::data::Backlinks::new();
                backlinks.included_pages.extend(included_pages);
                let native_list_wikipedia_links =
                    protection.take_native_list_wikipedia_links();
                Self::record_wikidot_wikipedia_backlinks(
                    &mut backlinks,
                    &native_list_wikipedia_links,
                );
                let mut wikidot_css_modules = protection.take_css_modules();
                Self::localize_wikidot_generated_styles(
                    &mut wikidot_css_modules,
                    current_site.as_ref(),
                    config,
                );
                let fallback_html_block_texts: Vec<String> = fallback_output
                    .html_block_texts
                    .iter()
                    .map(|html| {
                        let html = protection.compat_html().restore(html);
                        let html = Self::restore_protected_wikidot_inline_html(
                            Self::restore_protected_wikidot_color_spans(
                                html,
                                protection.color_spans(),
                            ),
                            protection.inline_html(),
                        );
                        let html = Self::localize_wikidot_local_file_urls(
                            &html,
                            current_site.as_ref(),
                            config,
                        );
                        protection.compat_text().restore(&html)
                    })
                    .collect();
                let fallback_code_blocks: Vec<CodeBlock<'static>> = fallback_output
                    .code_blocks
                    .iter()
                    .map(
                        |CodeBlock {
                             contents,
                             language,
                             name,
                         }| CodeBlock {
                            contents: Cow::Owned(protection.compat_text().restore(
                                &Self::restore_wikidot_code_block_compatibility(
                                    &protection.compat_html().restore_plain(contents),
                                    current_site.as_ref(),
                                    config,
                                ),
                            )),
                            language: language
                                .as_ref()
                                .map(|language| Cow::Owned(language.to_string())),
                            name: name.as_ref().map(|name| Cow::Owned(name.to_string())),
                        },
                    )
                    .collect();
                let body = protection.compat_html().restore(&fallback_output.body);
                let body = Self::restore_protected_wikidot_inline_html(
                    Self::restore_protected_wikidot_color_spans(
                        body,
                        protection.color_spans(),
                    ),
                    protection.inline_html(),
                );
                let body = Self::localize_wikidot_local_file_urls(
                    &body,
                    current_site.as_ref(),
                    config,
                );
                let body = protection.compat_text().restore(&body);
                (
                    wikidot_css_modules,
                    fallback_html_block_texts,
                    fallback_code_blocks,
                    body,
                    backlinks,
                )
            });
            let mut html_output = HtmlOutput {
                body,
                meta: Vec::new(),
                styles: wikidot_css_modules,
                resource_requirements: Vec::new(),
                backlinks,
            };
            if !Self::resolve_wikidot_embed_video_requirements(&mut html_output) {
                return Err(Error::new(
                    "failed to resolve typed Wikidot embedvideo requirements",
                    ErrorType::Render,
                )
                .into());
            }
            if !Self::resolve_wikidot_social_requirements(
                &mut html_output,
                page_info,
                current_site.as_ref(),
                page_preview,
            ) {
                return Err(Error::new(
                    "failed to resolve typed Wikidot social requirements",
                    ErrorType::Render,
                )
                .into());
            }
            if !Self::resolve_wikidot_gallery_requirements(
                ctx,
                &mut html_output,
                current_site_id,
                current_page_id,
                viewer_user_id,
                current_site.as_ref(),
                trusted_corpus_render,
            )
            .await?
            {
                return Err(Error::new(
                    "failed to resolve typed Wikidot gallery requirements",
                    ErrorType::Render,
                )
                .into());
            }
            drop(fallback_render_stage);
            if let Some((trace, CorpusRenderScope::Body)) = trace {
                trace.set_dimension(
                    CorpusRenderDimension::OutputBytes,
                    html_output.body.len(),
                );
            }
            let compiled_hash = Self::persist_or_hash_compiled_text(
                ctx,
                trace,
                &html_output.body,
                persist_compiled_text,
                make_error,
            )
            .await?;
            if let Some(page_id) = text_block_page_id {
                TextBlockService::validate_page_block_counts(
                    fallback_html_block_texts.len(),
                    fallback_code_blocks.len(),
                )
                .or_raise(make_error)?;
                let html_blocks: Vec<TextBlock> = fallback_html_block_texts
                    .iter()
                    .map(|html| TextBlock {
                        text: html,
                        text_type: None,
                        mime: MIME_HTML,
                        name: None,
                    })
                    .collect();

                let _stage = StageGuard::new(trace, CorpusRenderStage::HtmlBlocks);
                TextBlockService::add_blocks(
                    ctx,
                    page_id,
                    TextBlockType::Html,
                    &html_blocks,
                )
                .await
                .or_raise(make_error)?;

                let code_blocks: Vec<TextBlock> = fallback_code_blocks
                    .iter()
                    .map(
                        |CodeBlock {
                             contents,
                             language,
                             name,
                         }| TextBlock {
                            text: contents,
                            text_type: language.as_deref(),
                            mime: mime_for_language(language),
                            name: name.as_deref(),
                        },
                    )
                    .collect();
                TextBlockService::add_blocks(
                    ctx,
                    page_id,
                    TextBlockType::Code,
                    &code_blocks,
                )
                .await
                .or_raise(make_error)?;
            }

            return Ok(RenderInnerOutput {
                theme_previewer_blank,
                html_output,
                errors: Vec::new(),
                compiled_hash,
                expanded_include_count,
            });
        }
        let render_page_info = Self::owned_page_info(page_info);
        let render_settings = settings.clone();
        let render_config = config.clone();
        let render_current_site = current_site.clone();
        let render_timeout = Self::ftml_compat_render_timeout(
            &render_config,
            &outer.wikitext,
            outer.url_offset_list_pages_content_bytes,
        );
        let worker_trace = trace.map(|(trace, scope)| (trace.clone(), scope));
        let parse_worker_trace = worker_trace.clone();
        let parser_page_info = render_page_info.clone();
        let parse_settings = render_settings.clone();
        let parse_queued_at = parse_worker_trace.as_ref().map(|_| Instant::now());
        let parse_started = Instant::now();
        let parse_task = task::spawn_blocking(move || {
            let trace = parse_worker_trace
                .as_ref()
                .map(|(trace, scope)| (trace, *scope));
            if let (Some((trace, scope)), Some(queued_at)) = (trace, parse_queued_at) {
                trace.record_elapsed(scope, CorpusRenderStage::WorkerQueue, queued_at);
            }
            let prepared = Self::prepare_inner_render_wikitext(outer, &parse_settings);
            let mut parsed = ParsedFtmlRender::parse(
                prepared,
                &parser_page_info,
                &parse_settings,
                trace,
            );
            resolve_typed_root_site_grid_runtime_modules(&mut parsed.tree);
            resolve_typed_root_recent_threads_runtime_modules(&mut parsed.tree);
            parsed
        });
        let parsed = timeout(render_timeout, parse_task)
            .await
            .or_raise(|| {
                Error::new("failed to parse due to timeout", ErrorType::RenderTimeout)
            })?
            .or_raise(|| Error::new("failed to join parse task", ErrorType::Render))?;
        let parse_elapsed = parse_started.elapsed();
        let page_references = parsed.tree.page_references();
        let user_references = parsed.tree.user_references();
        let page_existence = match current_site_id {
            Some(site_id) => Some(
                LinkService::resolve_page_existence(
                    ctx,
                    site_id,
                    &render_page_info.site,
                    &page_references,
                    viewer_user_id,
                )
                .await
                .or_raise(make_error)?,
            ),
            None => None,
        };
        let user_info = UserInfoSnapshot::load(ctx, &user_references).await?;
        let render_queued_at = worker_trace.as_ref().map(|_| Instant::now());
        let render_task = task::spawn_blocking(move || {
            let trace = worker_trace.as_ref().map(|(trace, scope)| (trace, *scope));
            if let (Some((trace, scope)), Some(queued_at)) = (trace, render_queued_at) {
                trace.record_elapsed(scope, CorpusRenderStage::WorkerQueue, queued_at);
            }
            let mut html_output = parsed.render(
                &render_page_info,
                &render_settings,
                page_existence.as_ref(),
                &user_info,
                trace,
            );
            let legacy_action_registry = LegacyActionRegistry::from_resource_requirements(
                &html_output.resource_requirements,
            );
            let wikidot_tabview_ids: Vec<String> = html_output
                .resource_requirements
                .iter()
                .filter_map(|requirement| {
                    requirement
                        .wikidot_tab_view_requirement()
                        .map(|requirement| requirement.id().to_owned())
                })
                .collect();
            let ParsedFtmlRender {
                prepared,
                tree,
                errors,
            } = parsed;
            let InnerPreparedRenderWikitext {
                wikitext: _,
                included_pages,
                protection,
                wikidot_compat_links,
                wikidot_wikipedia_links,
                wikidot_embed_iframes,
                timings: _,
            } = prepared;
            let (html_block_texts, code_blocks) = {
                let _stage = StageGuard::new(trace, CorpusRenderStage::HtmlCompat);
                protection.restore(|protection| {
                    // Deepwell's Wikidot compatibility scanner identifies actual CSS module
                    // syntax before raw HTML fragments are restored. Keeping that typed
                    // provenance separate ensures authored <style> HTML stays in the body.
                    let mut wikidot_css_modules = protection.take_css_modules();
                    if !wikidot_css_modules.is_empty() {
                        wikidot_css_modules.append(&mut html_output.styles);
                        Self::localize_wikidot_generated_styles(
                            &mut wikidot_css_modules,
                            render_current_site.as_ref(),
                            &render_config,
                        );
                        html_output.styles = wikidot_css_modules;
                    } else {
                        Self::localize_wikidot_generated_styles(
                            &mut html_output.styles,
                            render_current_site.as_ref(),
                            &render_config,
                        );
                    }
                    html_output.body = Self::restore_protected_wikidot_embed_iframes(
                        std::mem::take(&mut html_output.body),
                        &wikidot_embed_iframes,
                    );
                    html_output.body = Self::restore_protected_wikidot_color_spans(
                        std::mem::take(&mut html_output.body),
                        protection.color_spans(),
                    );
                    html_output.body = Self::restore_protected_wikidot_inline_html(
                        std::mem::take(&mut html_output.body),
                        protection.inline_html(),
                    );
                    html_output.body =
                        protection.compat_html().restore(&html_output.body);
                    if render_page_info.page.as_ref() == "_ajax-module-connector" {
                        html_output.body = protect_ajax_module_literal_markers(
                            std::mem::take(&mut html_output.body),
                            protection.compat_text_mut(),
                        );
                    }
                    html_output.body = Self::restore_protected_wikidot_wikipedia_links(
                        std::mem::take(&mut html_output.body),
                        &wikidot_wikipedia_links,
                    );
                    html_output.body = Self::restore_protected_wikidot_compat_links(
                        std::mem::take(&mut html_output.body),
                        &wikidot_compat_links,
                    );
                    Self::record_protected_wikidot_wikipedia_backlinks(
                        &mut html_output.backlinks,
                        &wikidot_wikipedia_links,
                    );
                    let native_list_wikipedia_links =
                        protection.take_native_list_wikipedia_links();
                    Self::record_wikidot_wikipedia_backlinks(
                        &mut html_output.backlinks,
                        &native_list_wikipedia_links,
                    );
                    html_output.body =
                        Self::restore_wikidot_render_compatibility_for_context_with_resources(
                            &html_output.body,
                            render_current_site.as_ref(),
                            &render_config,
                            allow_wikidot_styleframe,
                            &wikidot_tabview_ids,
                        );
                    html_output.body = Self::restore_wikidot_include_block_break(&html_output.body);
                    html_output.body =
                        protection.compat_text().restore(&html_output.body);
                    if render_settings.layout == Layout::Wikidot {
                        legacy_action_registry
                            .remove_renderer_ids_from_wikidot_html(&mut html_output.body);
                    }
                    html_output.backlinks.included_pages.extend(included_pages);
                    let html_block_texts: Vec<String> = tree
                        .html_blocks
                        .iter()
                        .map(|html| {
                            let html = protection.compat_html().restore(html);
                            let html = Self::localize_wikidot_local_file_urls(
                                &html,
                                render_current_site.as_ref(),
                                &render_config,
                            );
                            protection.compat_text().restore(&html)
                        })
                        .collect();
                    if render_settings.layout == Layout::Wikidot
                        && render_settings.enable_html_blocks
                        && !html_block_texts.is_empty()
                    {
                        html_output.body = Self::rewrite_wikidot_html_block_iframe_urls(
                            std::mem::take(&mut html_output.body),
                            &render_page_info,
                            &html_block_texts,
                        );
                    }
                    let code_blocks = tree
                        .code_blocks
                        .iter()
                        .map(
                            |CodeBlock {
                                 contents,
                                 language,
                                 name,
                             }| CodeBlock {
                                contents: Cow::Owned(protection.compat_text().restore(
                                    &Self::restore_wikidot_code_block_compatibility(
                                        &protection.compat_html().restore_plain(contents),
                                        render_current_site.as_ref(),
                                        &render_config,
                                    ),
                                )),
                                language: language
                                    .as_ref()
                                    .map(|language| Cow::Owned(language.to_string())),
                                name: name
                                    .as_ref()
                                    .map(|name| Cow::Owned(name.to_string())),
                            },
                        )
                        .collect();
                    (html_block_texts, code_blocks)
                })
            };

            FtmlRenderOutput {
                html_output,
                errors,
                html_block_texts,
                code_blocks,
            }
        });

        let FtmlRenderOutput {
            html_output,
            errors,
            html_block_texts,
            code_blocks,
        } = timeout(render_timeout.saturating_sub(parse_elapsed), render_task)
            .await
            .or_raise(|| {
                Error::new("failed to render due to timeout", ErrorType::RenderTimeout)
            })?
            .or_raise(|| Error::new("failed to join render task", ErrorType::Render))?;

        let (mut html_output, embed_video_resolved) = timeout(
            render_timeout,
            task::spawn_blocking(move || {
                let mut html_output = html_output;
                let resolved =
                    Self::resolve_wikidot_embed_video_requirements(&mut html_output);
                (html_output, resolved)
            }),
        )
        .await
        .or_raise(|| {
            Error::new(
                "failed to resolve typed Wikidot embedvideo requirements due to timeout",
                ErrorType::RenderTimeout,
            )
        })?
        .or_raise(|| {
            Error::new(
                "failed to join Wikidot embedvideo resolution task",
                ErrorType::Render,
            )
        })?;
        if !embed_video_resolved {
            return Err(Error::new(
                "failed to resolve typed Wikidot embedvideo requirements",
                ErrorType::Render,
            )
            .into());
        }
        if !Self::resolve_wikidot_social_requirements(
            &mut html_output,
            page_info,
            current_site.as_ref(),
            page_preview,
        ) {
            return Err(Error::new(
                "failed to resolve typed Wikidot social requirements",
                ErrorType::Render,
            )
            .into());
        }
        if !Self::resolve_wikidot_gallery_requirements(
            ctx,
            &mut html_output,
            current_site_id,
            current_page_id,
            viewer_user_id,
            current_site.as_ref(),
            trusted_corpus_render,
        )
        .await?
        {
            return Err(Error::new(
                "failed to resolve typed Wikidot gallery requirements",
                ErrorType::Render,
            )
            .into());
        }

        if let Some((trace, CorpusRenderScope::Body)) = trace {
            trace.set_dimension(
                CorpusRenderDimension::OutputBytes,
                html_output.body.len(),
            );
        }

        // Both hosted block collections must be valid before either one can
        // write to S3. Each add_blocks call also validates its own slice.
        {
            let _stage = StageGuard::new(trace, CorpusRenderStage::BlocksValidate);
            if text_block_page_id.is_some() {
                TextBlockService::validate_page_block_counts(
                    html_block_texts.len(),
                    code_blocks.len(),
                )
                .or_raise(make_error)?;
            }
        }

        let compiled_hash = Self::persist_or_hash_compiled_text(
            ctx,
            trace,
            &html_output.body,
            persist_compiled_text,
            make_error,
        )
        .await?;

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
            {
                let _stage = StageGuard::new(trace, CorpusRenderStage::HtmlBlocks);
                let html_blocks: Vec<TextBlock> = html_block_texts
                    .iter()
                    .map(|html| TextBlock {
                        text: html,
                        text_type: None,
                        mime: MIME_HTML,
                        name: None,
                    })
                    .collect();

                TextBlockService::add_blocks(
                    ctx,
                    page_id,
                    TextBlockType::Html,
                    &html_blocks,
                )
                .await
                .or_raise(make_error)?;
            }

            // [[code]]
            {
                let _stage = StageGuard::new(trace, CorpusRenderStage::CodeBlocks);
                let code_text_blocks: Vec<TextBlock> = code_blocks
                    .iter()
                    .map(
                        |CodeBlock {
                             contents,
                             language,
                             name,
                         }| TextBlock {
                            text: contents,
                            text_type: language.as_deref(),
                            mime: mime_for_language(language),
                            name: name.as_deref(),
                        },
                    )
                    .collect();

                TextBlockService::add_blocks(
                    ctx,
                    page_id,
                    TextBlockType::Code,
                    &code_text_blocks,
                )
                .await
                .or_raise(make_error)?;
            }
        }

        // Build and return
        Ok(RenderInnerOutput {
            theme_previewer_blank,
            html_output,
            errors,
            compiled_hash,
            expanded_include_count,
        })
    }

    async fn persist_or_hash_compiled_text(
        ctx: &ServiceContext<'_>,
        trace: Option<(&CorpusRenderTrace, CorpusRenderScope)>,
        html: &str,
        persist_compiled_text: bool,
        make_error: impl Fn() -> Error,
    ) -> Result<TextHash> {
        let _stage = StageGuard::new(trace, CorpusRenderStage::CompiledText);
        if persist_compiled_text {
            TextService::create(ctx, html.to_owned())
                .await
                .or_raise(make_error)
        } else {
            Ok(k12_hash(html.as_bytes()))
        }
    }

    pub(super) fn wikidot_obfuscated_email(email: &str) -> Option<String> {
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

    pub(super) fn split_trailing_email_punctuation(email: &str) -> (&str, &str) {
        let email_end = email
            .trim_end_matches(|character| {
                matches!(character, '.' | ',' | ';' | ':' | '!' | '?')
            })
            .len();

        email.split_at(email_end)
    }

    pub(super) fn localize_wikidot_local_file_urls(
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

    fn localize_wikidot_generated_styles(
        styles: &mut [String],
        current_site: Option<&SiteModel>,
        config: &Config,
    ) {
        for style in styles {
            *style = Self::localize_wikidot_local_file_urls(style, current_site, config);
        }
    }

    fn localized_wikidot_local_file_url(
        host: &str,
        path: &str,
        current_site: &SiteModel,
        config: &Config,
    ) -> Option<String> {
        let site_slug = local_file_host_site_slug(host, config)?;
        let target_site_slug =
            if site_accepts_wikidot_local_asset_slug(current_site, &site_slug)
                || site_accepts_cross_site_wdfiles_local_file(current_site, host, path)
            {
                current_site.slug.as_str()
            } else if local_lab_has_reserved_scp_asset_mirror(config, &site_slug) {
                site_slug.as_str()
            } else {
                return direct_wdfiles_local_file_url(host, path);
            };

        Some(format!(
            "https://{}{}{}",
            target_site_slug, config.files_domain, path,
        ))
    }

    pub(in crate::services::render) async fn expand_includes(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        current_site_slug: &str,
        settings: &WikitextSettings,
        options: IncludeExpansionOptions<'_>,
    ) -> Result<IncludeExpansion> {
        let IncludeExpansionOptions {
            current_site_id,
            source_attachment_owner,
            source_cache,
            compat_text,
            expand_wikidot_image_blocks,
            page_preview,
            budget,
            render_cost_budget,
        } = options;
        let Some(current_site_id) = current_site_id else {
            return Ok(IncludeExpansion {
                wikitext,
                included_pages: Vec::new(),
                expanded_include_count: 0,
            });
        };

        if !settings.enable_page_syntax {
            return Ok(IncludeExpansion {
                wikitext,
                included_pages: Vec::new(),
                expanded_include_count: 0,
            });
        }

        let mut wikitext = wikitext;
        if let Some(owner) = source_attachment_owner.as_ref() {
            qualify_included_relative_image_attachments(
                &mut wikitext,
                &owner.site_slug,
                &owner.page_slug,
            );
        }

        let mut expansion = Self::expand_includes_for_site(
            ctx,
            wikitext,
            IncludeExpansionContext {
                current_site_id,
                current_site_slug: current_site_slug.to_owned(),
                attachment_owner: source_attachment_owner,
                page_info,
                settings,
                page_preview,
                expand_wikidot_image_blocks,
                max_total_includes: budget.maximum,
                render_cost_budget,
            },
            source_cache,
            compat_text,
            0,
            budget.remaining,
        )
        .await?;
        source_cache
            .attachment_provenance
            .restore_unresolved(&mut expansion.wikitext);
        unprotect_include_variables(&mut expansion.wikitext);

        Ok(expansion)
    }

    fn expand_includes_for_site<'a>(
        ctx: &'a ServiceContext<'_>,
        wikitext: String,
        expansion_context: IncludeExpansionContext<'a>,
        include_source_cache: &'a mut IncludeSourceCache,
        compat_text: &'a mut CompatTextFragments,
        depth: usize,
        mut remaining_includes: usize,
    ) -> Pin<Box<dyn Future<Output = Result<IncludeExpansion>> + Send + 'a>> {
        Box::pin(async move {
            let mut wikitext = wikitext;
            Self::normalize_wikidot_multiline_includes(&mut wikitext);
            Self::normalize_wikidot_alignment_markers(&mut wikitext);
            if expansion_context.settings.layout.legacy() {
                expand_malformed_include_targets(&mut wikitext);
            }
            if expansion_context.page_preview {
                Self::prepare_wikidot_conditionals_before_include_expansion_for_page_preview(
                    &mut wikitext,
                    compat_text,
                    depth,
                );
            } else {
                Self::prepare_wikidot_conditionals_before_include_expansion(
                    &mut wikitext,
                    expansion_context.page_info,
                    compat_text,
                    depth,
                );
            }
            Self::mask_wikidot_literal_include_markers(&mut wikitext);
            let image_block_included_pages = if expansion_context
                .expand_wikidot_image_blocks
                && expansion_context.current_site_slug
                    == expansion_context.page_info.site.as_ref()
            {
                Self::expand_wikidot_image_block_includes_with_provenance(
                    &mut wikitext,
                    expansion_context.page_info,
                    expansion_context.attachment_owner.as_ref().map(|owner| {
                        (owner.site_slug.as_str(), owner.page_slug.as_str())
                    }),
                    Some(&include_source_cache.attachment_provenance),
                )
            } else {
                Vec::new()
            };

            if !has_include_opening_candidate(&wikitext) {
                Self::unmask_wikidot_literal_include_markers(&mut wikitext);
                protect_include_variables(&mut wikitext);
                return Ok(IncludeExpansion {
                    wikitext,
                    included_pages: image_block_included_pages,
                    expanded_include_count: 0,
                });
            }

            let mut include_display_pages = collect_include_display_pages(&wikitext);
            let mut includes = Vec::new();
            ftml::include(
                &wikitext,
                expansion_context.settings,
                CollectingIncluder {
                    includes: &mut includes,
                },
                include_error,
            )?;

            if includes.is_empty() {
                let mut wikitext = wikitext;
                Self::unmask_wikidot_literal_include_markers(&mut wikitext);
                protect_include_variables(&mut wikitext);
                return Ok(IncludeExpansion {
                    wikitext,
                    included_pages: image_block_included_pages,
                    expanded_include_count: 0,
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
            let mut nested_include_counts = Vec::with_capacity(includes.len());

            for include in &includes {
                if let Err(error) = expansion_context
                    .render_cost_budget
                    .charge(1, "include expansion")
                {
                    return Err(Error::new(error.to_string(), ErrorType::Render).into());
                }
                if remaining_includes == 0 {
                    return Err(Error::new(
                        format!(
                            "include expansion exceeded maximum total includes {}",
                            expansion_context.max_total_includes,
                        ),
                        ErrorType::Render,
                    )
                    .into());
                }
                remaining_includes -= 1;

                let callsite_owner = Self::include_attachment_owner(&expansion_context);
                let mut attachment_variable_owners = AttachmentVariableOwners::new();
                let variables = include
                    .variables()
                    .iter()
                    .map(|(name, value)| {
                        if let Some((decoded, owner)) =
                            include_source_cache.attachment_provenance.decode(value)
                        {
                            attachment_variable_owners
                                .insert(name.to_string(), owner.clone());
                            (Cow::Owned(name.to_string()), Cow::Owned(decoded.clone()))
                        } else {
                            attachment_variable_owners
                                .insert(name.to_string(), callsite_owner.clone());
                            (Cow::Owned(name.to_string()), Cow::Owned(value.to_string()))
                        }
                    })
                    .collect::<VariableMap<'static>>();
                let include = IncludeRef::new(include.page_ref().clone(), variables);

                let source = RenderRuntime::new(ctx)
                    .fetch_include_source(
                        expansion_context.current_site_id,
                        &expansion_context.current_site_slug,
                        include.page_ref(),
                        include_source_cache,
                    )
                    .await?;

                let Some(mut source) = source else {
                    fetched_pages.push(None);
                    nested_included_pages.push(Vec::new());
                    nested_include_counts.push(0);
                    continue;
                };

                qualify_relative_image_variable_attachments(
                    &mut source.wikitext,
                    include.variables(),
                    &attachment_variable_owners,
                );
                protect_forwarded_attachment_variables(
                    &mut source.wikitext,
                    include.variables(),
                    &attachment_variable_owners,
                    &mut include_source_cache.attachment_provenance,
                );
                prepare_include_source_variables_and_comment_branches(
                    &mut source.wikitext,
                    &include,
                    expansion_context.page_info,
                    compat_text,
                )?;
                qualify_included_relative_image_attachments(
                    &mut source.wikitext,
                    &source.site_slug,
                    &source.page_slug,
                );
                select_metacomponent_documentation(
                    &mut source.wikitext,
                    MetacomponentSourceContext::Included,
                );

                let nested_context = expansion_context.for_nested_source(&source);
                let expansion = Self::expand_includes_for_site(
                    ctx,
                    source.wikitext,
                    nested_context,
                    include_source_cache,
                    compat_text,
                    depth + 1,
                    remaining_includes,
                )
                .await?;
                if expansion.expanded_include_count > remaining_includes {
                    return Err(Error::new(
                        format!(
                            "include expansion exceeded maximum total includes {}",
                            expansion_context.max_total_includes,
                        ),
                        ErrorType::Render,
                    )
                    .into());
                }
                remaining_includes -= expansion.expanded_include_count;
                nested_include_counts.push(expansion.expanded_include_count);

                let mut nested_wikitext = expansion.wikitext;
                Self::prepare_wikidot_conditionals_for_include_expansion(
                    &mut nested_wikitext,
                    expansion_context.page_info,
                    compat_text,
                );
                remove_nested_include_boundaries(&mut nested_wikitext);
                fetched_pages.push(Some(nested_wikitext));
                nested_included_pages.push(expansion.included_pages);
            }

            let missing_replacements = collect_missing_include_replacements(
                &includes,
                &fetched_pages,
                &mut include_display_pages,
                compat_text,
            );
            let (mut expanded, direct_included_pages) = ftml::include(
                &wikitext,
                expansion_context.settings,
                PreparedIncluder {
                    pages: fetched_pages,
                    missing_replacements,
                },
                include_error,
            )?;

            Self::unmask_wikidot_literal_include_markers(&mut expanded);
            protect_include_variables(&mut expanded);

            let mut included_pages = image_block_included_pages;
            let expanded_include_count = direct_included_pages.len()
                + nested_include_counts.into_iter().sum::<usize>();
            for (page_ref, nested_pages) in
                direct_included_pages.into_iter().zip(nested_included_pages)
            {
                included_pages.push(page_ref);
                included_pages.extend(nested_pages);
            }

            Ok(IncludeExpansion {
                wikitext: expanded,
                included_pages,
                expanded_include_count,
            })
        })
    }

    fn include_attachment_owner(
        context: &IncludeExpansionContext<'_>,
    ) -> AttachmentOwner {
        context.attachment_owner.clone().unwrap_or_else(|| {
            let page_slug = match context.page_info.category.as_deref() {
                Some(category) => format!("{category}:{}", context.page_info.page),
                None => context.page_info.page.to_string(),
            };
            AttachmentOwner {
                site_slug: context.current_site_slug.clone(),
                page_slug,
            }
        })
    }

    #[cfg(test)]
    fn protect_wikidot_css_modules_before_first_list_pages(
        wikitext: &mut String,
        settings: &WikitextSettings,
    ) -> Option<CompatTextFragments> {
        protect_css_modules_before_first_list_pages(wikitext, settings)
    }

    #[cfg(test)]
    fn neutralize_authored_wikidot_compat_markers(wikitext: &mut String) {
        neutralize_authored_markers(wikitext);
    }

    #[cfg(test)]
    fn protect_generated_wikidot_compat_html(
        wikitext: &mut String,
        settings: &WikitextSettings,
    ) -> Vec<ProtectedWikidotCompatHtml> {
        if !settings.enable_page_syntax {
            return Vec::new();
        }

        let mut fragments = Vec::new();
        *wikitext =
            Self::protect_generated_wikidot_compat_lists(wikitext, &mut fragments);
        let literal_regions = LiteralRegionIndex::new(wikitext);
        let protected = GENERATED_COMPAT_TABLE_REGEX
            .replace_all(wikitext, |captures: &regex::Captures<'_>| {
                let full_match = captures.get(0).expect("compat fragment capture exists");
                if literal_regions.contains(full_match.start()) {
                    return full_match.as_str().to_owned();
                }
                let marker = wikidot_compat_html_marker();
                let html = captures[0]
                    .replace(r#" data-wikijump-compat-listpages="1""#, "")
                    .replace(r#" data-wikijump-compat-list="1""#, "")
                    .replace(r#" data-wikijump-compat-members="1""#, "")
                    .replace(r#" data-wikijump-compat-backlinks="1""#, "")
                    .replace(r#" data-wikijump-compat-new-page="1""#, "")
                    .replace(r#" data-wikijump-compat-clone="1""#, "")
                    .replace(r#" data-wikijump-compat-date="1""#, "");
                fragments.push(ProtectedWikidotCompatHtml {
                    marker: marker.clone(),
                    html,
                });
                marker
            })
            .into_owned();
        *wikitext = protected;
        fragments
    }

    #[cfg(test)]
    fn protect_generated_wikidot_compat_lists(
        wikitext: &str,
        fragments: &mut Vec<ProtectedWikidotCompatHtml>,
    ) -> String {
        let mut output = String::with_capacity(wikitext.len());
        let mut rest = wikitext;
        let mut rest_offset = 0usize;
        let list_start = r#"<ul data-wikijump-compat-list="1">"#;
        let literal_regions = LiteralRegionIndex::new(wikitext);

        while let Some(start) = rest.find(list_start) {
            let (before, from_start) = rest.split_at(start);
            output.push_str(before);
            let absolute_start = rest_offset + start;

            if literal_regions.contains(absolute_start) {
                output.push_str(list_start);
                rest = &from_start[list_start.len()..];
                rest_offset = absolute_start + list_start.len();
                continue;
            }

            if let Some(end) = find_balanced_ul_end(from_start) {
                let fragment = &from_start[..end];
                let marker = wikidot_compat_html_marker();
                fragments.push(ProtectedWikidotCompatHtml {
                    marker: marker.clone(),
                    html: fragment.replace(r#" data-wikijump-compat-list="1""#, ""),
                });
                output.push_str(&marker);
                rest = &from_start[end..];
                rest_offset = absolute_start + end;
            } else {
                output.push_str(list_start);
                rest = &from_start[list_start.len()..];
                rest_offset = absolute_start + list_start.len();
            }
        }

        output.push_str(rest);
        output
    }

    #[cfg(test)]
    fn restore_protected_generated_wikidot_compat_html(
        mut html: String,
        fragments: &[ProtectedWikidotCompatHtml],
    ) -> String {
        for fragment in fragments {
            html = html.replace(&fragment.marker, &fragment.html);
        }
        html
    }

    fn protect_wikidot_color_spans(
        wikitext: &mut String,
        settings: &WikitextSettings,
    ) -> ProtectedWikidotColorSpans {
        protect_wikidot_color_spans(wikitext, settings)
    }

    fn protect_wikidot_inline_html_spans(
        wikitext: &mut String,
        settings: &WikitextSettings,
    ) -> Vec<ProtectedWikidotInlineHtml> {
        protect_wikidot_inline_html_spans(wikitext, settings)
    }

    fn restore_protected_wikidot_color_spans(
        html: String,
        spans: &ProtectedWikidotColorSpans,
    ) -> String {
        restore_protected_wikidot_color_spans(html, spans)
    }

    fn restore_protected_wikidot_inline_html(
        html: String,
        spans: &[ProtectedWikidotInlineHtml],
    ) -> String {
        restore_protected_wikidot_inline_html(html, spans)
    }

    fn protect_unrendered_wikidot_color_markers(
        wikitext: String,
        settings: &WikitextSettings,
        fragments: &mut CompatTextFragments,
    ) -> String {
        if !settings.enable_page_syntax {
            return wikitext;
        }

        let literal_regions = LiteralRegionIndex::new_wikidot_protection(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for (start, _) in wikitext.match_indices("##") {
            output.push_str(&wikitext[cursor..start]);
            if literal_regions.contains(start) {
                output.push_str("##");
            } else {
                output.push_str(&fragments.push("##"));
            }
            cursor = start + 2;
        }
        output.push_str(&wikitext[cursor..]);
        output
    }

    fn render_long_native_list_runs_with_registry(
        wikitext: String,
        compat_html: &mut CompatHtmlFragments,
    ) -> (String, Vec<WikidotWikipediaLink>) {
        let lines = wikitext.split_inclusive('\n').collect::<Vec<_>>();
        let mut line_starts = Vec::with_capacity(lines.len());
        let mut line_start = 0usize;
        for line in &lines {
            line_starts.push(line_start);
            line_start += line.len();
        }
        let mut source_context = None;
        let mut output = String::with_capacity(wikitext.len());
        let mut wikipedia_links = Vec::new();
        let mut index = 0;

        while index < lines.len() {
            let mut end = index;
            while end < lines.len() && native_bullet_list_item(lines[end]).is_some() {
                end += 1;
            }

            if end - index >= LONG_NATIVE_LIST_RENDER_MIN_ITEMS
                && source_context
                    .get_or_insert_with(|| NativeListSourceContext::new(&wikitext))
                    .allows_block_run(&line_starts[index..end])
            {
                let rendered = render_native_bullet_list_with_wikipedia_links(
                    &lines[index..end],
                    &mut wikipedia_links,
                );
                output.push_str(&compat_html.push_block_html(rendered));
                index = end;
            } else {
                output.push_str(lines[index]);
                index += 1;
            }
        }

        (output, wikipedia_links)
    }

    #[cfg(test)]
    fn render_long_native_list_runs(wikitext: String) -> String {
        let mut fragments = CompatHtmlFragments::new(&wikitext);
        let (protected, _) =
            Self::render_long_native_list_runs_with_registry(wikitext, &mut fragments);
        fragments.restore(&protected)
    }
}

pub(super) fn format_wikidot_list_pages_date(
    created_at: time::OffsetDateTime,
    format: &str,
) -> String {
    let month = [
        "", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov",
        "Dec",
    ][created_at.month() as usize];
    let mut output = String::new();
    let mut chars = format.chars();
    while let Some(ch) = chars.next() {
        if ch != '%' {
            output.push(ch);
            continue;
        }
        match chars.next() {
            Some('d') => output.push_str(&format!("{:02}", created_at.day())),
            Some('e') => output.push_str(&created_at.day().to_string()),
            Some('b') => output.push_str(month),
            Some('m') => output.push_str(&format!("{:02}", created_at.month() as u8)),
            Some('Y') => output.push_str(&created_at.year().to_string()),
            Some('H') => output.push_str(&format!("{:02}", created_at.hour())),
            Some('M') => output.push_str(&format!("{:02}", created_at.minute())),
            Some('R') => output.push_str(&format!(
                "{:02}:{:02}",
                created_at.hour(),
                created_at.minute(),
            )),
            Some('%') => output.push('%'),
            Some(other) => {
                output.push('%');
                output.push(other);
            }
            None => output.push('%'),
        }
    }
    output
}

pub(super) fn escape_list_pages_html_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(super) fn escape_list_pages_html_attr(value: &str) -> String {
    escape_list_pages_html_text(value).replace('"', "&quot;")
}

pub(super) fn localized_runtime_module_label(
    ctx: &ServiceContext<'_>,
    locale: &str,
    message_key: &str,
    fallback: &str,
) -> String {
    localized_runtime_module_label_from_localizations(
        ctx.localization(),
        locale,
        message_key,
        fallback,
    )
}

pub(super) fn localized_runtime_module_label_from_localizations(
    localizations: &crate::locales::Localizations,
    locale: &str,
    message_key: &str,
    fallback: &str,
) -> String {
    let Ok(locales) = parse_locales(&[locale.to_owned()]) else {
        return fallback.to_owned();
    };
    let arguments = fluent::FluentArgs::new();
    localizations
        .translate_option(&locales, message_key, &arguments)
        .ok()
        .flatten()
        .map(|translation| translation.into_owned())
        .unwrap_or_else(|| fallback.to_owned())
}

pub(super) fn decode_wikidot_email_html_entities(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(start) = rest.find('&') {
        output.push_str(&rest[..start]);
        let entity_start = start + 1;
        let Some(relative_end) = rest[entity_start..].find(';') else {
            output.push_str(&rest[start..]);
            return output;
        };

        let entity_end = entity_start + relative_end;
        let entity = &rest[entity_start..entity_end];
        let decoded = match entity {
            "amp" => Some('&'),
            "quot" => Some('"'),
            "#39" | "apos" => Some('\''),
            "lt" => Some('<'),
            "gt" => Some('>'),
            _ => decode_numeric_html_entity(entity),
        };

        if let Some(character) = decoded {
            output.push(character);
        } else {
            output.push_str(&rest[start..=entity_end]);
        }

        rest = &rest[entity_end + 1..];
    }

    output.push_str(rest);
    output
}

pub(super) fn format_list_pages_rating(score: Option<f32>) -> String {
    let Some(score) = score else {
        return String::new();
    };

    if score.fract() == 0.0 {
        format!("{score:.0}")
    } else {
        score.to_string()
    }
}

#[cfg(test)]
/// Captured legacy HTML used only to exercise generic fragment protection.
pub(super) fn members_compat_html_fixture(group: &str) -> String {
    let group_attr = escape_list_pages_html_attr(group);
    let group_script = escape_javascript_single_quoted(group);
    let body = if group.eq_ignore_ascii_case("moderators") {
        concat!(
            r#"<table><tr><td><span class="printuser avatarhover">"#,
            r#"<a href="http://www.wikidot.com/user:info/lambert-eggman" onclick="WIKIDOT.page.listeners.userInfo(10382670); return false;">"#,
            r#"<img alt="lambert-eggman" class="small" src="http://www.wikidot.com/avatar.php?userid=10382670&amp;size=small&amp;timestamp=1782003747" style="background-image:url(http://www.wikidot.com/userkarma.php?u=10382670)"/></a>"#,
            r#"<a href="http://www.wikidot.com/user:info/lambert-eggman" onclick="WIKIDOT.page.listeners.userInfo(10382670); return false;">lambert-eggman</a>"#,
            r#"</span></td></tr></table>"#,
        )
    } else {
        ""
    };

    format!(
        r#"<div id="ml-607935" data-wikijump-compat-members="1" data-group="{group_attr}">{body}<script type="text/javascript">function updateMemberList607935(pageNo) {{var p = {{}};p.group = '{group_script}';p.order = 'joined';p.page = pageNo;OZONE.ajax.requestModule("membership/MembersListModule", p, function(r) {{}});}}</script></div>"#,
    )
}

pub(super) fn render_clone_module(head: &str) -> String {
    let button = wikidot_module_argument(head, "button")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Clone this site");

    format!(
        r#"<a class="button" data-wikijump-compat-clone="1" href="javascript:;">{button}</a>"#,
        button = escape_list_pages_html_text(button),
    )
}

#[cfg(test)]
fn escape_javascript_single_quoted(value: &str) -> String {
    value
        .replace('\\', r"\\")
        .replace('\'', r"\'")
        .replace('<', r"\x3C")
        .replace('>', r"\x3E")
        .replace('&', r"\x26")
        .replace('\u{2028}', r"\u2028")
        .replace('\u{2029}', r"\u2029")
        .replace('\n', r"\n")
        .replace('\r', r"\r")
}

pub(super) fn push_escaped_html(output: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(character),
        }
    }
}

fn elapsed_micros(started: Instant) -> u64 {
    started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
}

fn corpus_replay_syntax_features(wikitext: &str) -> CorpusReplaySyntaxFeatures {
    let mut features = CorpusReplaySyntaxFeatures {
        bytes: wikitext.len(),
        block_markers: wikitext.matches("[[").count(),
        inline_delimiter_markers: ["**", "//", "__", "^^", ",,"]
            .into_iter()
            .map(|marker| wikitext.matches(marker).count())
            .sum(),
        ..CorpusReplaySyntaxFeatures::default()
    };

    for line in wikitext.lines() {
        features.lines += 1;
        features.max_line_bytes = features.max_line_bytes.max(line.len());
        let trimmed = line.trim_start_matches([' ', '\t']);
        features.quote_prefixed_lines += usize::from(trimmed.starts_with('>'));
        features.ordered_list_lines += usize::from(trimmed.starts_with("# "));
        features.unordered_list_lines += usize::from(trimmed.starts_with("* "));
        features.table_lines += usize::from(trimmed.starts_with("||"));
    }

    features
}

#[derive(Debug)]
pub(super) struct RenderInnerOutput {
    pub(super) theme_previewer_blank: bool,
    pub(super) html_output: HtmlOutput,
    pub(super) errors: Vec<ParseError>,
    pub(super) compiled_hash: TextHash,
    pub(super) expanded_include_count: usize,
}

#[derive(Debug)]
pub(super) struct IncludeExpansion {
    pub(super) wikitext: String,
    pub(super) included_pages: Vec<PageRef>,
    pub(super) expanded_include_count: usize,
}

#[derive(Debug)]
pub(super) struct IncludeExpansionOptions<'a> {
    pub(super) current_site_id: Option<i64>,
    /// Canonical attachment owner for source text originating on a page other
    /// than the page currently being rendered, such as a ListPages content row.
    pub(super) source_attachment_owner: Option<AttachmentOwner>,
    pub(super) source_cache: &'a mut IncludeSourceCache,
    pub(super) compat_text: &'a mut CompatTextFragments,
    pub(super) expand_wikidot_image_blocks: bool,
    pub(super) page_preview: bool,
    pub(super) budget: IncludeExpansionBudget,
    pub(super) render_cost_budget: super::render_budget::SharedRenderCostBudget,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct IncludeExpansionBudget {
    pub(super) maximum: usize,
    pub(super) remaining: usize,
}

impl IncludeExpansionBudget {
    pub(super) fn new(maximum: usize) -> Self {
        Self {
            maximum,
            remaining: maximum,
        }
    }

    pub(super) fn consume(&mut self, count: usize) {
        self.remaining = self.remaining.saturating_sub(count);
    }
}

#[derive(Debug)]
struct IncludeExpansionContext<'a> {
    current_site_id: i64,
    current_site_slug: String,
    /// Canonical owner of relative attachments in this source. The ordinary
    /// render root remains `None` so its `PageInfo` identity is used.
    attachment_owner: Option<AttachmentOwner>,
    page_info: &'a PageInfo<'a>,
    settings: &'a WikitextSettings,
    page_preview: bool,
    expand_wikidot_image_blocks: bool,
    max_total_includes: usize,
    render_cost_budget: super::render_budget::SharedRenderCostBudget,
}

impl<'a> IncludeExpansionContext<'a> {
    fn for_nested_source(&self, source: &IncludeSource) -> Self {
        Self {
            // Wikidot resolves an unqualified include inside a cross-site include against the original callsite site. The fetched source still owns its relative attachments.
            current_site_id: self.current_site_id,
            current_site_slug: self.current_site_slug.clone(),
            attachment_owner: Some(AttachmentOwner {
                site_slug: source.site_slug.clone(),
                page_slug: source.page_slug.clone(),
            }),
            page_info: self.page_info,
            settings: self.settings,
            page_preview: self.page_preview,
            expand_wikidot_image_blocks: self.expand_wikidot_image_blocks,
            max_total_includes: self.max_total_includes,
            render_cost_budget: self.render_cost_budget.clone(),
        }
    }
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
        Ok(wikidot_no_such_include_replacement(page_ref))
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
        .with_spaced_empty_separator(include.has_spaced_empty_separator())
}

pub(in crate::services::render) fn has_include_opening_candidate(content: &str) -> bool {
    let bytes = content.as_bytes();
    let mut search = 0;
    while search + 1 < bytes.len() {
        let Some(offset) = bytes[search..].windows(2).position(|pair| pair == b"[[")
        else {
            return false;
        };
        let mut cursor = search + offset + 2;
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if bytes
            .get(cursor..cursor.saturating_add(b"include".len()))
            .is_some_and(|name| name.eq_ignore_ascii_case(b"include"))
        {
            return true;
        }
        search = cursor.max(search + offset + 2);
    }
    false
}

pub(super) fn site_matches_wikidot_slug(site: &SiteModel, site_slug: &str) -> bool {
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
        || corpus_site_slug_matches_wikidot_slug(site, site_slug)
        || translated_scp_site_uses_scp_wiki_source_assets(site, site_slug)
}

fn site_accepts_cross_site_wdfiles_local_file(
    site: &SiteModel,
    host: &str,
    path: &str,
) -> bool {
    if !path.starts_with("/local--files/") {
        return false;
    }

    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();
    host.ends_with(".wdfiles.com") && site_is_wikidot_local_asset_mirror(site)
}

fn local_lab_has_reserved_scp_asset_mirror(config: &Config, site_slug: &str) -> bool {
    config
        .files_domain_no_dot
        .to_ascii_lowercase()
        .ends_with(".localhost")
        && matches!(
            site_slug.to_ascii_lowercase().as_str(),
            "scp-wiki" | "scp-jp"
        )
}

fn site_is_wikidot_local_asset_mirror(site: &SiteModel) -> bool {
    site.from_wikidot
        || site.slug.eq_ignore_ascii_case("scp-wiki")
        || site
            .preferred_domain
            .as_deref()
            .and_then(preferred_domain_wikidot_slug)
            .is_some()
}

fn corpus_site_slug_matches_wikidot_slug(site: &SiteModel, site_slug: &str) -> bool {
    if !site.from_wikidot {
        return false;
    }

    let site_slug = site_slug.to_ascii_lowercase();
    let slug = site.slug.to_ascii_lowercase();
    let Some(remainder) = slug.strip_prefix(&format!("{site_slug}-")) else {
        return false;
    };

    remainder == "corpus"
        || remainder.starts_with("corpus-")
        || remainder.contains("-corpus-")
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

#[cfg(test)]
mod tests;
