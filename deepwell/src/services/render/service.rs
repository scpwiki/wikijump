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

use super::compat_html_fragments::CompatHtmlFragments;
use super::html_text::html_data_segments;
use super::include_comment_branches::remove_unresolved_include_comment_branches;
use super::literal_regions::LiteralRegionIndex;
use super::prelude::*;
use super::wikidot_expression::resolve_parser_functions;
use crate::hash::TextHash;
use crate::models::page::{self, Entity as Page};
use crate::models::page_revision;
use crate::models::site::Model as SiteModel;
use crate::models::user::{self, Entity as UserTable};
use crate::models::wikidot_user::{self, Entity as WikidotUser};
use crate::services::page_query::{
    CategoriesSelector, CountPagesExactCountEligibilityDiagnostics,
    CountPagesExactCountEligibilityInput, DataFormSelector, DateSelector,
    FoundPageFields, FoundPageRow, FoundPages, IncludedCategories,
    ListPagesRenderDiagnosticsInput, OrderBySelector, OrderProperty, PageParentSelector,
    PageQuery, PageQueryResultMetadata, PageTypeSelector, PaginationSelector,
    RangeSelector, TagCondition, count_pages_exact_count_eligibility_diagnostics,
    list_pages_render_diagnostics, parse_static_wikidot_data_form_values,
    static_wikidot_data_form_matches,
};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::settings::{NavigationPageWikitext, SettingsService};
use crate::services::text_block::{
    MIME_HTML, TextBlock, TextBlockService, mime_for_language,
};
use crate::services::{
    CategoryService, PageQueryService, PageRevisionService, PageService, SiteService,
    TextService,
};
use crate::types::{Action, PageId, Permission, Resource, TextBlockType};
use ftml::data::PageRef;
use ftml::includes::{FetchedPage, IncludeRef};
use ftml::prelude::*;
use ftml::tree::{CodeBlock, VariableMap};
use regex::Regex;
use sea_orm::{FromQueryResult, Statement};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::ops::Range;
use std::pin::Pin;
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use tokio::task;
use tokio::time::timeout;
use uuid::Uuid;

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
}

impl CorpusReplayExpandedWikitext {
    #[inline]
    #[allow(dead_code)] // Consumed by the stacked render-replay action.
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
    pub(super) wikidot_compat_html: CompatHtmlFragments,
}

#[derive(Debug)]
struct ViewableCountPagesRows {
    pages: FoundPages,
    metadata: PageQueryResultMetadata,
    view_permission_filtering_applied: bool,
}

#[derive(Debug)]
struct ViewableListPagesRows {
    pages: FoundPages,
    metadata: PageQueryResultMetadata,
    view_permission_filtering_applied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProtectedWikidotWikipediaLink {
    anchor: String,
    href: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProtectedWikidotCompatLink {
    anchor: String,
    marker: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProtectedWikidotColorSpan {
    marker: String,
    html: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProtectedWikidotInlineHtml {
    marker: String,
    html: String,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct ProtectedWikidotCompatHtml {
    marker: String,
    html: String,
}

#[derive(Debug)]
struct ExpandedRenderWikitext {
    wikitext: String,
    included_pages: Vec<PageRef>,
    wikidot_compat_html: CompatHtmlFragments,
}

#[derive(Debug)]
struct OuterPreparedRenderWikitext {
    wikitext: String,
    included_pages: Vec<PageRef>,
    wikidot_inline_html: Vec<ProtectedWikidotInlineHtml>,
    wikidot_color_spans: Vec<ProtectedWikidotColorSpan>,
    wikidot_compat_html: CompatHtmlFragments,
    compatibility_fallback: bool,
    timings: CorpusReplayStageTimings,
}

#[derive(Debug)]
struct InnerPreparedRenderWikitext {
    wikitext: String,
    included_pages: Vec<PageRef>,
    wikidot_inline_html: Vec<ProtectedWikidotInlineHtml>,
    wikidot_color_spans: Vec<ProtectedWikidotColorSpan>,
    wikidot_compat_links: Vec<ProtectedWikidotCompatLink>,
    wikidot_wikipedia_links: Vec<ProtectedWikidotWikipediaLink>,
    wikidot_compat_html: CompatHtmlFragments,
    wikidot_embed_iframes: Vec<String>,
    timings: CorpusReplayStageTimings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WikidotCompatInlineMarkerKind {
    Color,
    Italic,
    Underline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WikidotCompatInlineMarker {
    start: usize,
    end: usize,
    kind: WikidotCompatInlineMarkerKind,
}

const MAX_INCLUDE_EXPANSION_DEPTH: usize = 8;
const MAX_INCLUDE_EXPANSION_TOTAL: usize = 256;
// The frozen EN corpus contains a page with 1,266 direct includes. Only the
// trusted corpus finalizer receives this higher ceiling; user-controlled
// render paths retain the ordinary limit above.
const MAX_CORPUS_INCLUDE_EXPANSION_TOTAL: usize = 4096;
const DEFAULT_LISTPAGES_RENDER_LIMIT: u64 = 100;
const MAX_LISTPAGES_RENDER_LIMIT: u64 = 250;
const MAX_LISTPAGES_RENDER_OFFSET: u32 = 1_000;
const MAX_LISTPAGES_RENDER_SCAN_ROWS: u32 = 5_000;
const MAX_BACKLINKS_MODULE_ROWS: usize = 500;
const LONG_NATIVE_LIST_RENDER_MIN_ITEMS: usize = 8;
const MAX_FTML_COMPAT_PARSE_BYTES: usize = 768_000;
const MAX_FTML_COMPAT_DENSE_PARSE_SCORE: usize = 180_000;
const MAX_FTML_COMPAT_COLLAPSIBLE_BLOCKS: usize = 48;
const MIN_FTML_COMPAT_TABBED_FALLBACK_BYTES: usize = 64_000;
const MIN_FTML_COMPAT_TABBED_FALLBACK_MARKERS: usize = 12;
const MIN_FTML_COMPAT_TABBED_RENDER_BYTES: usize = 100_000;
const MIN_FTML_COMPAT_TABBED_MARKERS: usize = 10;
const MIN_DENSE_FTML_COMPAT_RENDER_TIMEOUT_SECS: u64 = 150;
const MAX_WIKIDOT_SIMPLE_IF_PASSES: usize = 32;
const LISTPAGES_NO_MATCH_AUTHOR_ID: &str = "-9223372036854775808";
const INCLUDE_VARIABLE_OPEN_SENTINEL: &str = "__WIKIJUMP_INCLUDE_VAR_OPEN__";
const INCLUDE_VARIABLE_CLOSE_SENTINEL: &str = "__WIKIJUMP_INCLUDE_VAR_CLOSE__";
const WIKIDOT_COMMENT_INCLUDE_SENTINEL: &str = "__WIKIJUMP_COMMENT_INCLUDE__";
const WIKIDOT_CLASS_INCLUDE_VARIABLE_SENTINEL_PREFIX: &str = "wikijump-include-var-";
const WIKIDOT_EMBED_IFRAME_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTEMBEDIFRAME";
const WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTCOMPATHTML";
const WIKIDOT_COMPAT_LINK_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTCOMPATLINK";
const WIKIDOT_WIKIPEDIA_LINK_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTWIKIPEDIALINK";
const WIKIDOT_COLOR_SPAN_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTCOLORSPAN";
const WIKIDOT_INLINE_HTML_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTINLINEHTML";
const WIKIDOT_RATE_ANCHOR_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTRATEANCHOR";
const WIKIDOT_LISTPAGES_LITERAL_ELLIPSIS_SENTINEL_PREFIX: &str =
    "WIKIJUMPWIKIDOTLISTPAGESELLIPSIS";
const WIKIDOT_LOCAL_INTERWIKI_BASE: &str = "/-/wikidot-interwiki";
const WIKIDOT_TABVIEW_SCRIPT: &str = "";
const WIKIDOT_TABVIEW_INIT_SCRIPT: &str = r#"<script type="text/javascript"></script>"#;
const MAX_WIKIDOT_COMPAT_FALLBACK_TITLE_LINKS: usize = 128;

type WikidotCompatLinkTitleMap = BTreeMap<String, String>;

static INCLUDE_VARIABLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\$(?P<name>[a-zA-Z0-9_\-]+)\}").unwrap());
static WIKIDOT_CLASS_INCLUDE_VARIABLE_SENTINEL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| {
        Regex::new(&format!(
            r"{}(?P<name>[a-zA-Z0-9_\-]+)",
            regex::escape(WIKIDOT_CLASS_INCLUDE_VARIABLE_SENTINEL_PREFIX)
        ))
        .unwrap()
    });
static LISTPAGES_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)\[\[module\s+ListPages(?P<head>(?:"[^"]*"|'[^']*'|[^\]])*)\]\](?P<body>.*?)\[\[/module\]\]"#,
    )
    .unwrap()
});
static WIKIDOT_LISTPAGES_LITERAL_ELLIPSIS_SENTINEL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| {
        Regex::new(&format!(
            r"{WIKIDOT_LISTPAGES_LITERAL_ELLIPSIS_SENTINEL_PREFIX}[0-9a-f]{{32}}X"
        ))
        .unwrap()
    });
static COUNTPAGES_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)\[\[module\s+CountPages(?P<head>(?:"[^"]*"|'[^']*'|[^\]])*)\]\](?P<body>.*?)\[\[/module\]\]"#,
    )
    .unwrap()
});
static RATE_MODULE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)\[\[module\s+Rate(?P<head>[^\]]*)\]\]").unwrap());
static WIKIDOT_RATE_ANCHOR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"\[\[a href="javascript:;" onclick="(?P<onclick>WIKIDOT\.modules\.PageRateWidgetModule\.listeners\.(?:rate\(event, -?1\)|cancelVote\(event\)))" title="(?P<title>[^"]*)"\]\](?P<label>[^\[]*)\[\[/a\]\]"#,
    )
    .unwrap()
});
static TAGCLOUD_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+TagCloud(?P<head>[^\]]*)\]\]").unwrap()
});
static BACKLINKS_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+Backlinks(?P<head>[^\]]*)\]\]").unwrap()
});
static MEMBERS_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+Members(?P<head>[^\]]*)\]\]").unwrap()
});
static NEWPAGE_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+NewPage(?P<head>[^\]]*)\]\]").unwrap()
});
static CLONE_MODULE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)\[\[module\s+Clone(?P<head>[^\]]*)\]\]").unwrap());
static CSS_MODULE_OPEN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)\[\[module\s+css[^\]]*\]\]").unwrap());
static MODULE_CLOSE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)\[\[/module\]\]").unwrap());
static AUTHORED_WIKIDOT_COMPAT_MARKER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)data-wikijump-compat-(?P<kind>listpages|list|members|backlinks|new-page|clone|date|css-module)",
    )
    .unwrap()
});
static AUTHORED_WIKIDOT_COMPAT_OPEN_TAG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)<table class="wiki-content-table" data-wikijump-compat-listpages="1">|<ul data-wikijump-compat-list="1">|<div id="ml-[0-9]+" data-wikijump-compat-members="1"[^>]*>|<div class="backlinks-module-box" data-wikijump-compat-backlinks="1"[^>]*>|<form class="new-page-box" data-wikijump-compat-new-page="1"[^>]*>|<a class="button" data-wikijump-compat-clone="1"[^>]*>|<span class="odate time_-?[0-9]+ format_[A-Za-z0-9%_.-]+" data-wikijump-compat-date="1" style="cursor: help; display: inline;">|<style data-wikijump-compat-css-module="1">"#,
    )
    .unwrap()
});
static GENERATED_LISTPAGES_HTML_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)<table class="wiki-content-table" data-wikijump-compat-listpages="1">.*?</table>|<span class="odate time_-?[0-9]+ format_[A-Za-z0-9%_.-]+" data-wikijump-compat-date="1" style="cursor: help; display: inline;">[^<>]*</span>"#,
    )
    .unwrap()
});
#[cfg(test)]
static GENERATED_COMPAT_TABLE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)<table class="wiki-content-table" data-wikijump-compat-listpages="1">.*?</table>|<div id="ml-[0-9]+" data-wikijump-compat-members="1"[^>]*>.*?</div>|<div class="backlinks-module-box" data-wikijump-compat-backlinks="1"[^>]*>.*?</div>|<form class="new-page-box" data-wikijump-compat-new-page="1"[^>]*>.*?</form>|<a class="button" data-wikijump-compat-clone="1"[^>]*>.*?</a>|<span class="odate time_-?[0-9]+ format_[A-Za-z0-9%_.-]+" data-wikijump-compat-date="1" style="cursor: help; display: inline;">[^<>]*</span>"#,
    )
    .unwrap()
});
static WIKIDOT_RESIDUAL_DIV_PARAGRAPH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)<p>\s*(?:(?P<open>\[\[div[^\]]*\]\])|(?P<close>\[\[/div\]\]))\s*</p>"#,
    )
    .unwrap()
});
static WIKIJUMP_FOOTNOTE_MARKER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)<wj-footnote-ref-marker(?P<attrs>[^>]*)>(?P<label>.*?)</wj-footnote-ref-marker>"#,
    )
    .unwrap()
});
static WIKIJUMP_FOOTNOTE_REF_SPAN_WRAPPER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)<span class="wj-footnote-ref">\s*(?P<body><sup class="footnoteref">.*?</sup>)\s*</span>"#,
    )
    .unwrap()
});
static WIKIJUMP_FOOTNOTE_REF_LEADING_SPACE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)(?P<before>\S)\s+(?P<footnote><span class="wj-footnote-ref">)"#)
        .unwrap()
});
static WIKIJUMP_FOOTNOTE_DATA_ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"data-id="(?P<id>[0-9]+)""#).unwrap());
static LISTPAGES_ARGUMENT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)(?P<key>[A-Za-z_][A-Za-z0-9_\-]*)\s*(?P<op>!?=)\s*(?:"(?P<double>[^"]*)"|'(?P<single>[^']*)'|(?P<bare>[^\s\]]+))"#)
        .unwrap()
});
static LISTPAGES_VARIABLE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"%%(?P<name>[A-Za-z0-9_]+)(?:\{(?P<argument>[A-Za-z0-9_-]+)\})?(?:\|(?P<format>.*?))?%%",
    )
    .unwrap()
});
static WIKIDOT_LISTPAGES_SIGNED_ABS_EXPR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[#ifexpr\s+(?P<test>-?[0-9]+(?:\.[0-9]+)?)\s*>\s*-1\s*\|\s*\+\s*\|\s*-\s*\]\]\s*\[\[#expr\s+abs\(\s*(?P<abs>-?[0-9]+(?:\.[0-9]+)?)\s*\)\s*\]\]").unwrap()
});
static WIKIDOT_NUMERIC_IFEXPR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[#ifexpr\s+(?P<left>-?[0-9]+(?:\.[0-9]+)?)\s*(?P<op>>=|<=|==|!=|=|>|<)\s*(?P<right>-?[0-9]+(?:\.[0-9]+)?)\s*\|\s*(?P<when_true>.*?)\s*\|\s*(?P<when_false>.*?)\s*\]\]").unwrap()
});
static WIKIDOT_USER_INLINE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[\*user\s+(?P<name>[^\]]+)\]\]").unwrap());
static WIKIDOT_ANCHOR_MARKER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[#\s+(?P<name>[^\]\n]+)\]\]").unwrap());
static WIKIDOT_CURRENT_PAGE_LINK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[#\s+(?P<label>[^\]\n]+)\]").unwrap());
static WIKIDOT_STAR_LOCAL_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\*/(?P<target>[^\s\]\n]+)\s+(?P<label>[^\]\n]+)\]").unwrap()
});
static WIKIDOT_LABELED_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\[\[(?P<target>[^\]|\n]+)\|(?P<label>[^\]\n]*)\]\]\]").unwrap()
});
static WIKIDOT_MULTILINE_LABELED_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)\[\[\[(?P<target>[^\]|\n]+)\|(?P<label>[^\]]*\n[^\]]*)\]\]\]")
        .unwrap()
});
static WIKIDOT_QUADRUPLE_LINK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[\[\[(?P<target>[^\]\n]+)\]\]\]\]").unwrap());
static WIKIDOT_UNLABELED_LINK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[\[(?P<target>[^\]\n]+)\]\]\]").unwrap());
static WIKIDOT_LOCAL_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[/(?P<target>[^\s\]\n]+)\s+(?P<label>[^\]\n]+)\]").unwrap()
});
static WIKIDOT_EXTERNAL_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\*?(?P<url>https?://[^\s\]]+)\s+(?P<label>[^\]]+)\]").unwrap()
});
static WIKIDOT_WIKIPEDIA_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[wikipedia:(?P<target>[^\s\]\n]+)(?:\s+(?P<label>[^\]\n]+))?\]")
        .unwrap()
});
static WIKIDOT_COLOR_SPAN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?P<hashes>#{2,})(?P<color>[A-Za-z0-9_-]+)\s*\|(?P<body>.*?)##").unwrap()
});
static WIKIDOT_BOLD_UNDERLINE_SPAN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*\*__(?P<body>[^\n]*?)(?:__\*\*|\*\*__)").unwrap());
static WIKIDOT_BOLD_OUTER_COLOR_SPAN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\*\*(?P<hashes>#{2,})(?P<color>[A-Za-z0-9_-]+)\s*\|(?P<body>[^\n]*?)##\*\*",
    )
    .unwrap()
});
static WIKIDOT_BOLD_COLOR_SPAN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\*\*(?P<hashes>#{2,})(?P<color>[A-Za-z0-9_-]+)\s*\|(?P<body>[^\n]*?)\*\*##",
    )
    .unwrap()
});
static WIKIDOT_ESCAPED_NBSP_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"@<(?P<html>&nbsp;)>@").unwrap());
static WIKIJUMP_CODE_BLOCK_PANEL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?is)<div class="wj-code-panel">.*?</div>"#).unwrap());
static WIKIJUMP_CODE_BLOCK_OPEN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<wj-code class="wj-code(?:\s+wj-language-[^"]*)?">"#).unwrap()
});
static WIKIJUMP_TAB_BUTTON_LIST_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<div class="wj-tabs-button-list"[^>]*>(?P<body>.*?)</div>"#)
        .unwrap()
});
static WIKIJUMP_TAB_PANEL_LIST_OPEN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<div class="wj-tabs-panel-list"[^>]*>"#).unwrap()
});
static WIKIJUMP_SELECTED_TAB_BUTTON_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)<wj-tabs-button class="wj-tabs-button"[^>]*aria-selected="true"[^>]*>"#,
    )
    .unwrap()
});
static WIKIJUMP_TAB_BUTTON_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<wj-tabs-button class="wj-tabs-button"[^>]*>"#).unwrap()
});
static WIKIJUMP_TAB_PANEL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?is)<div class="wj-tabs-panel"[^>]*>"#).unwrap());
static WIKIDOT_RESIDUAL_IFTAGS_INLINE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)\[\[iftags[^\]\n]*\]\][^\[\]\n]*\[\[/iftags\]\]"#).unwrap()
});
static WIKIDOT_SINGLE_LINE_IFTAGS_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)\[\[iftags(?P<spec>[^\]\n]*)\]\](?P<body>[^\[\]\n]*)\[\[/iftags\]\]"#,
    )
    .unwrap()
});
static WIKIDOT_SIMPLE_IFTAGS_BLOCK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)\[\[iftags(?P<spec>\s+[^\]\n]+)\]\](?P<body>.*?)\[\[/iftags\]\]"#)
        .unwrap()
});
static WIKIDOT_SIMPLE_IF_OPEN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)\[\[#if\s+(?P<cond>1|0|true|false)\s*\|"#).unwrap()
});
static WIKIDOT_IMAGE_BLOCK_INCLUDE_START_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)\[\[include\s+(?::(?P<site>[A-Za-z0-9_-]+):)?component:image-block(?P<after>\s|\||\]\])"#,
    )
    .unwrap()
});
static WIKIDOT_INCLUDE_OPEN_LINE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?im)^\[\[\s*(?P<keyword>include)(?P<after>\s+)").unwrap()
});
static WIKIDOT_COMPAT_STYLE_BLOCK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<style\b[^>]*\btype\s*=\s*["']text/css["'][^>]*>.*?</style>"#)
        .unwrap()
});
static WIKIDOT_USERKARMA_BACKGROUND_STYLE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\sstyle="background-image:\s*url\(https?://www\.wikidot\.com/userkarma\.php\?u=[0-9]+\)""#)
        .unwrap()
});
static WIKIDOT_RENDERED_MAILFORM_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)<p>\[\[module\s+MailForm(?P<head>[^\]]*)\]\]</p>(?P<body>.*?)<p>\[\[/module\]\]</p>"#,
    )
    .unwrap()
});
static WIKIDOT_RENDERED_MAILFORM_FIELD_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?is)<ol>\s*<li>(?P<name>[^<]+)</li>"#).unwrap());
static WIKIDOT_RENDERED_MAILFORM_DEFAULT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<li>default:\s*(?P<default>[^<]*)</li>"#).unwrap()
});
static WIKIDOT_RENDERED_MAILFORM_MAX_LENGTH_REGEX: LazyLock<Regex> =
    LazyLock::new(|| {
        Regex::new(r#"(?is)<li>maxLength:\s*(?P<max>[0-9]+)</li>"#).unwrap()
    });
static WIKIJUMP_INLINE_MATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)<span class="wj-math wj-math-inline"><code class="wj-math-source wj-hidden"[^>]*>(?P<source>.*?)</code><wj-math-ml class="wj-math-ml">.*?</wj-math-ml></span>"#,
    )
    .unwrap()
});
static WIKIDOT_EMAIL_SPAN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"<span class="wiki-email" style="visibility: visible;"><a href="mailto:([^"]+)">([^<]+)</a></span>"#,
    )
    .unwrap()
});
static WIKIDOT_EMAIL_CLASS_SPAN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<span class="wiki-email">(?P<body>[^<]*)</span>"#).unwrap()
});
static WIKIDOT_OBFUSCATED_EMAIL_BODY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^[A-Za-z0-9.-]+\|[A-Za-z0-9._%+-]+#[A-Za-z0-9.-]+\|[A-Za-z0-9._%+-]+$"#)
        .unwrap()
});
static WIKIDOT_RECOVERABLE_REVERSED_EMAIL_BODY_REGEX: LazyLock<Regex> = LazyLock::new(
    || {
        Regex::new(
            r#";tg&(?P<domain1>[A-Za-z0-9.-]+)\|(?P<user1>[A-Za-z0-9._%+-]+);tl&#.*?;tg&(?P<domain2>[A-Za-z0-9.-]+)\|(?P<user2>[A-Za-z0-9._%+-]+);tl&"#,
        )
        .unwrap()
    },
);
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
        r#"^<iframe src="(?P<src>//interwiki\.(?:scpwiki\.com|scp-jp\.org)/styleFrame\.html\?[^"]+)" style="display: none"></iframe>$"#,
    )
    .unwrap()
});
static WIKIDOT_INTERWIKI_FRAME_IFRAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"^<iframe src="(?P<src>//interwiki\.(?:scpwiki\.com|scp-jp\.org)/interwikiFrame\.html\?[^"]+)" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>$"#,
    )
    .unwrap()
});
static WIKIDOT_LOCAL_FILE_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?P<quote>["'])(?:https?:)?//(?P<host>[A-Za-z0-9.-]+)(?::[0-9]+)?(?P<path>/local--(?:files|code)/[^"'<>\s]+)"#,
    )
    .unwrap()
});
static WIKIDOT_LOCAL_FILE_CSS_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)(?P<prefix>url\(\s*["']?)(?:https?:)?//(?P<host>[A-Za-z0-9.-]+)(?::[0-9]+)?(?P<path>/local--(?:files|code)/[^"')<>\s]+)"#,
    )
    .unwrap()
});
static CSS_IMPORT_LINE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?im)^(?P<indent>[ \t]*)@import(?P<body>[^\n]*)$"#).unwrap()
});
static CSS_ABSOLUTE_URL_HOST_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)(?:https?:)?//(?P<host>[A-Za-z0-9.-]+)(?::[0-9]+)?"#).unwrap()
});
static CSS_EXTERNAL_URL_FUNCTION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)url\(\s*(?P<quote>["']?)(?:https?:)?//(?P<host>[A-Za-z0-9.-]+)(?::[0-9]+)?(?P<path>[^"')\s]*)["']?\s*\)"#,
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
        } = Self::render_inner(
            ctx,
            wikitext,
            page_info,
            settings,
            RenderContext::none(),
            MAX_INCLUDE_EXPANSION_TOTAL,
        )
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
        Self::render_page_with_include_limit(
            ctx,
            wikitext,
            page_info,
            layout,
            PageId {
                site_id,
                category_id,
                page_id,
            },
            MAX_INCLUDE_EXPANSION_TOTAL,
        )
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
        Self::render_page_with_include_limit(
            ctx,
            wikitext,
            page_info,
            layout,
            id,
            MAX_CORPUS_INCLUDE_EXPANSION_TOTAL,
        )
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
        max_include_expansions: usize,
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
            max_include_expansions,
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
                        max_include_expansions,
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

    /// Expand trusted corpus page wikitext exactly as the production page
    /// renderer does, stopping before the pure normalization/protection pass.
    ///
    /// The returned value is intentionally owned and serializable so a replay
    /// controller can hand it to an isolated worker without giving that worker
    /// database or service credentials.
    #[allow(dead_code)] // Consumed by the stacked render-replay action.
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
            Some(id.site_id),
            Some(id.page_id),
            MAX_CORPUS_INCLUDE_EXPANSION_TOTAL,
        )
        .await?;

        Ok(CorpusReplayExpandedWikitext {
            wikitext: expanded.wikitext,
            page_info,
            settings,
            id,
            included_pages: expanded.included_pages,
            wikidot_compat_html: expanded.wikidot_compat_html,
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
        } = input;
        let expanded = ExpandedRenderWikitext {
            wikitext,
            included_pages,
            wikidot_compat_html,
        };
        let outer = Self::prepare_outer_render_wikitext_observed(
            expanded,
            &page_info,
            &settings,
            &mut observer,
        );

        if outer.compatibility_fallback {
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
                wikidot_compat_html: outer.wikidot_compat_html,
            };
        }

        let inner =
            Self::prepare_inner_render_wikitext_observed(outer, &settings, &mut observer);
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
            wikidot_compat_html: inner.wikidot_compat_html,
        }
    }

    async fn expand_render_wikitext(
        ctx: &ServiceContext<'_>,
        mut wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        current_page_id: Option<i64>,
        max_include_expansions: usize,
    ) -> Result<ExpandedRenderWikitext> {
        let make_error =
            || Error::new("failed to perform render operation", ErrorType::Render);

        Self::remove_preview_component_separator_markers(&mut wikitext);
        let mut included_pages = if settings.enable_page_syntax {
            Self::expand_wikidot_image_block_includes(&mut wikitext, page_info)
        } else {
            Vec::new()
        };

        let IncludeExpansion {
            wikitext: expanded_wikitext,
            included_pages: expanded_included_pages,
            ..
        } = Self::expand_includes(
            ctx,
            wikitext,
            page_info,
            page_info.site.as_ref(),
            settings,
            current_site_id,
            IncludeExpansionOptions {
                expand_wikidot_image_blocks: true,
                max_total_includes: max_include_expansions,
            },
        )
        .await
        .or_raise(make_error)?;
        wikitext = expanded_wikitext;
        included_pages.extend(expanded_included_pages);
        Self::remove_wikidot_metacomponent_documentation(&mut wikitext);
        remove_unresolved_include_comment_branches(&mut wikitext);
        Self::prepare_wikidot_conditionals_for_include_expansion(
            &mut wikitext,
            page_info,
        );
        Self::neutralize_authored_wikidot_compat_markers(&mut wikitext);
        let mut wikidot_compat_html = CompatHtmlFragments::new(&wikitext);
        let IncludeExpansion {
            wikitext: expanded_wikitext,
            included_pages: list_pages_included_pages,
            ..
        } = Self::expand_list_pages(
            ctx,
            wikitext,
            page_info,
            settings,
            current_site_id,
            current_page_id,
            &mut wikidot_compat_html,
        )
        .await
        .or_raise(make_error)?;
        wikitext = expanded_wikitext;
        included_pages.extend(list_pages_included_pages);
        wikitext = Self::expand_count_pages(
            ctx,
            wikitext,
            page_info,
            settings,
            current_site_id,
            current_page_id,
        )
        .await
        .or_raise(make_error)?;
        wikitext = Self::expand_tag_cloud_modules(
            ctx,
            wikitext,
            page_info,
            current_site_id,
            current_page_id,
        )
        .await
        .or_raise(make_error)?;
        wikitext = Self::expand_backlinks_modules(
            ctx,
            wikitext,
            settings,
            current_site_id,
            current_page_id,
            &mut wikidot_compat_html,
        )
        .await
        .or_raise(make_error)?;
        wikitext = Self::expand_members_modules_with_registry(
            wikitext,
            settings,
            &mut wikidot_compat_html,
        );
        wikitext = Self::expand_new_page_modules_with_registry(
            wikitext,
            settings,
            &mut wikidot_compat_html,
        );
        wikitext = Self::expand_clone_modules_with_registry(
            wikitext,
            settings,
            &mut wikidot_compat_html,
        );
        wikitext = Self::expand_rate_modules(wikitext, page_info, settings);

        Ok(ExpandedRenderWikitext {
            wikitext,
            included_pages,
            wikidot_compat_html,
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

        observer(CorpusReplayPreparationStage::Normalization);
        let started = Instant::now();
        if settings.enable_page_syntax {
            // Includes and runtime modules can introduce literal Wikidot
            // conditionals after the pre-expansion pass. Resolve that generated
            // context-free syntax before FTML sees it as anchor markup.
            Self::prepare_wikidot_conditionals_for_include_expansion(
                &mut expanded.wikitext,
                page_info,
            );
            Self::normalize_wikidot_div_style_url_quotes(&mut expanded.wikitext);
            Self::protect_wikidot_marker_class_include_variables(&mut expanded.wikitext);
            Self::normalize_wikidot_multiline_page_links(&mut expanded.wikitext);
        }
        timings.normalization_us = elapsed_micros(started);

        observer(CorpusReplayPreparationStage::OuterProtection);
        let started = Instant::now();
        let wikidot_inline_html =
            Self::protect_wikidot_inline_html_spans(&mut expanded.wikitext, settings);
        let wikidot_color_spans =
            Self::protect_wikidot_color_spans(&mut expanded.wikitext, settings);
        expanded.wikitext =
            Self::escape_unrendered_wikidot_color_markers(expanded.wikitext, settings);
        expanded.wikitext = Self::render_long_native_list_runs_with_registry(
            expanded.wikitext,
            &mut expanded.wikidot_compat_html,
        );
        Self::protect_wikidot_css_modules(
            &mut expanded.wikitext,
            settings,
            &mut expanded.wikidot_compat_html,
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
            wikidot_inline_html,
            wikidot_color_spans,
            wikidot_compat_html: expanded.wikidot_compat_html,
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
        // Protection turns ambiguous branch constructs into opaque markers.
        // Rerun the bounded conditional scanner so syntax that could not be
        // selected safely during outer preparation does not leak into FTML.
        outer.wikitext = Self::resolve_wikidot_simple_if_fragments(&outer.wikitext);
        outer.timings.inner_protection_us = elapsed_micros(started);

        observer(CorpusReplayPreparationStage::Preprocess);
        let started = Instant::now();
        ftml::preprocess(&mut outer.wikitext);
        outer.timings.preprocess_us = elapsed_micros(started);

        InnerPreparedRenderWikitext {
            wikitext: outer.wikitext,
            included_pages: outer.included_pages,
            wikidot_inline_html: outer.wikidot_inline_html,
            wikidot_color_spans: outer.wikidot_color_spans,
            wikidot_compat_html: outer.wikidot_compat_html,
            wikidot_compat_links,
            wikidot_wikipedia_links,
            wikidot_embed_iframes,
            timings: outer.timings,
        }
    }

    async fn render_inner(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        render_context: RenderContext,
        max_include_expansions: usize,
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

        let expanded = Self::expand_render_wikitext(
            ctx,
            wikitext,
            page_info,
            settings,
            current_site_id,
            current_page_id,
            max_include_expansions,
        )
        .await?;
        let outer = Self::prepare_outer_render_wikitext(expanded, page_info, settings);
        if outer.compatibility_fallback {
            let OuterPreparedRenderWikitext {
                wikitext,
                included_pages,
                wikidot_inline_html,
                wikidot_color_spans,
                wikidot_compat_html,
                compatibility_fallback: _,
                timings: _,
            } = outer;
            let mut backlinks = ftml::data::Backlinks::new();
            backlinks.included_pages.extend(included_pages);
            let fallback_link_titles = if let Some(site_id) = current_site_id {
                Self::load_wikidot_compat_fallback_link_titles(ctx, site_id, &wikitext)
                    .await
                    .or_raise(make_error)?
            } else {
                WikidotCompatLinkTitleMap::new()
            };
            let fallback_output = Self::render_oversized_wikidot_compatibility_fallback(
                &wikitext,
                current_site.as_ref(),
                config,
                page_info.page.as_ref(),
                Some(&fallback_link_titles),
            );
            let fallback_html_block_texts: Vec<String> = fallback_output
                .html_block_texts
                .iter()
                .map(|html| {
                    let html = wikidot_compat_html.restore(html);
                    let html = Self::restore_protected_wikidot_inline_html(
                        Self::restore_protected_wikidot_color_spans(
                            html,
                            &wikidot_color_spans,
                        ),
                        &wikidot_inline_html,
                    );
                    let html = restore_list_pages_literal_ellipsis_markers(&html);
                    Self::localize_wikidot_local_file_urls(
                        &html,
                        current_site.as_ref(),
                        config,
                    )
                })
                .collect();
            let html_output = HtmlOutput {
                body: {
                    let body = wikidot_compat_html.restore(&fallback_output.body);
                    let body = Self::restore_protected_wikidot_inline_html(
                        Self::restore_protected_wikidot_color_spans(
                            body,
                            &wikidot_color_spans,
                        ),
                        &wikidot_inline_html,
                    );
                    let body = restore_list_pages_literal_ellipsis_markers(&body);
                    Self::localize_wikidot_local_file_urls(
                        &body,
                        current_site.as_ref(),
                        config,
                    )
                },
                meta: Vec::new(),
                styles: Vec::new(),
                backlinks,
            };
            let compiled_hash = TextService::create(ctx, html_output.body.clone())
                .await
                .or_raise(make_error)?;
            if let Some(page_id) = text_block_page_id {
                let html_blocks: Vec<TextBlock> = fallback_html_block_texts
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

            return Ok(RenderInnerOutput {
                html_output,
                errors: Vec::new(),
                compiled_hash,
            });
        }
        let render_page_info = Self::owned_page_info(page_info);
        let render_settings = settings.clone();
        let render_config = config.clone();
        let render_current_site = current_site.clone();
        let render_timeout =
            Self::ftml_compat_render_timeout(&render_config, &outer.wikitext);

        let render_task = task::spawn_blocking(move || {
            let InnerPreparedRenderWikitext {
                wikitext,
                included_pages,
                wikidot_inline_html,
                wikidot_color_spans,
                wikidot_compat_links,
                wikidot_wikipedia_links,
                wikidot_compat_html,
                wikidot_embed_iframes,
                timings: _,
            } = Self::prepare_inner_render_wikitext(outer, &render_settings);
            let tokens = ftml::tokenize(&wikitext);
            let result = ftml::parse(&tokens, &render_page_info, &render_settings);
            let (tree, errors) = result.into();
            let mut html_output =
                HtmlRender.render(&tree, &render_page_info, &render_settings);
            html_output.body = Self::restore_protected_wikidot_embed_iframes(
                html_output.body,
                &wikidot_embed_iframes,
            );
            html_output.body = Self::restore_protected_wikidot_color_spans(
                html_output.body,
                &wikidot_color_spans,
            );
            html_output.body = Self::restore_protected_wikidot_inline_html(
                html_output.body,
                &wikidot_inline_html,
            );
            html_output.body = wikidot_compat_html.restore(&html_output.body);
            html_output.body = Self::restore_protected_wikidot_wikipedia_links(
                html_output.body,
                &wikidot_wikipedia_links,
            );
            html_output.body = Self::restore_protected_wikidot_compat_links(
                html_output.body,
                &wikidot_compat_links,
            );
            html_output.body =
                restore_list_pages_literal_ellipsis_markers(&html_output.body);
            Self::record_protected_wikidot_wikipedia_backlinks(
                &mut html_output.backlinks,
                &wikidot_wikipedia_links,
            );
            html_output.body = Self::restore_wikidot_render_compatibility(
                &html_output.body,
                render_current_site.as_ref(),
                &render_config,
            );
            apply_basalt_shell_compatibility(&mut html_output.body);
            apply_blankstyle_shell_compatibility(&mut html_output.body);
            html_output.body =
                Self::remove_wikidot_compat_style_blocks(&html_output.body);
            html_output.body = Self::localize_wikidot_local_file_urls(
                &html_output.body,
                render_current_site.as_ref(),
                &render_config,
            );
            html_output.backlinks.included_pages.extend(included_pages);
            let html_block_texts = tree
                .html_blocks
                .iter()
                .map(|html| {
                    let html = wikidot_compat_html.restore(html);
                    Self::localize_wikidot_local_file_urls(
                        &html,
                        render_current_site.as_ref(),
                        &render_config,
                    )
                })
                .collect();
            let code_blocks = tree
                .code_blocks
                .iter()
                .map(
                    |CodeBlock {
                         contents,
                         language,
                         name,
                     }| CodeBlock {
                        contents: Cow::Owned(
                            Self::restore_wikidot_code_block_compatibility(
                                contents,
                                render_current_site.as_ref(),
                                &render_config,
                            ),
                        ),
                        language: language
                            .as_ref()
                            .map(|language| Cow::Owned(language.to_string())),
                        name: name.as_ref().map(|name| Cow::Owned(name.to_string())),
                    },
                )
                .collect();

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
        } = timeout(render_timeout, render_task)
            .await
            .or_raise(|| {
                Error::new(
                    "failed to parse and render due to timeout",
                    ErrorType::RenderTimeout,
                )
            })?
            .or_raise(|| {
                Error::new("failed to join parse and render task", ErrorType::Render)
            })?;

        // Both hosted block collections must be valid before either one can
        // write to S3. Each add_blocks call also validates its own slice.
        if text_block_page_id.is_some() {
            TextBlockService::validate_page_block_counts(
                html_block_texts.len(),
                code_blocks.len(),
            )
            .or_raise(make_error)?;
        }

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
            let html_blocks: Vec<TextBlock> = html_block_texts
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
        let html = Self::remove_spurious_wikidot_email_classes(&html);
        let html = Self::restore_wikidot_collapsible_compatibility(&html);
        let html = Self::restore_wikidot_code_block_dom_compatibility(&html);
        let html = Self::restore_wikidot_tabview_dom_compatibility(&html);
        let html = Self::resolve_wikidot_simple_if_fragments(&html);
        let html = Self::restore_wikidot_mailform_compatibility(&html);
        let html = Self::restore_residual_wikidot_div_paragraph_markers(&html);
        let html = Self::restore_residual_wikidot_span_markers(&html);
        let html = Self::restore_residual_wikidot_alignment_markers(&html);
        let html = Self::restore_residual_wikidot_separator_markers(&html);
        let html = Self::restore_residual_wikidot_heading_markers(&html);
        let html = Self::remove_residual_wikidot_iftags_fragments(&html);
        let html = Self::remove_wikijump_table_body_wrappers(&html);
        let html = Self::remove_wikidot_compat_style_blocks(&html);
        let html = Self::restore_wikidot_inline_math_compatibility(&html);
        let html = Self::restore_wikidot_ta_badge_default_compatibility(&html);
        let html = Self::restore_wikidot_text_ellipsis_compatibility(&html);
        let html = Self::restore_wikidot_footnote_dom_compatibility(&html);
        let html = Self::remove_wikijump_underline_wrappers(&html);
        let html = Self::remove_wikidot_userkarma_background_styles(&html);
        let html = Self::restore_protected_wikidot_marker_class_include_variables(&html);
        Self::localize_wikidot_local_file_urls(&html, current_site, config)
    }

    fn restore_wikidot_collapsible_compatibility(html: &str) -> String {
        html.replace(r#"<details class="wj-collapsible""#, r#"<details class="collapsible-block collapsible-block-folded collapsible-block-unfolded""#)
            .replace(
                r#"<summary class="wj-collapsible-button wj-collapsible-button-top">"#,
                r#"<summary class="collapsible-block-link">"#,
            )
            .replace(
                r#"<wj-collapsible-button-bottom class="wj-collapsible-button wj-collapsible-button-bottom">"#,
                r#"<div class="collapsible-block-unfolded-link"><span class="collapsible-block-link">"#,
            )
            .replace("</wj-collapsible-button-bottom>", "</span></div>")
            .replace("wj-collapsible-content", "collapsible-block-content")
            .replace("wj-collapsible-show-text", "collapsible-block-link")
            .replace(
                "wj-collapsible-hide-text",
                "collapsible-block-link collapsible-block-unfolded-link",
            )
    }

    fn remove_spurious_wikidot_email_classes(html: &str) -> String {
        WIKIDOT_EMAIL_CLASS_SPAN_REGEX
            .replace_all(html, |captures: &regex::Captures<'_>| {
                let body = captures.name("body").map_or("", |mtch| mtch.as_str());
                if WIKIDOT_OBFUSCATED_EMAIL_BODY_REGEX.is_match(body) {
                    captures[0].to_owned()
                } else if let Some(body) = Self::recover_reversed_wikidot_email_body(body)
                {
                    format!(r#"<span class="wiki-email">{body}</span>"#)
                } else {
                    format!("<span>{body}</span>")
                }
            })
            .into_owned()
    }

    fn recover_reversed_wikidot_email_body(body: &str) -> Option<String> {
        let captures = WIKIDOT_RECOVERABLE_REVERSED_EMAIL_BODY_REGEX.captures(body)?;
        let domain1 = captures.name("domain1")?.as_str();
        let user1 = captures.name("user1")?.as_str();
        let domain2 = captures.name("domain2")?.as_str();
        let user2 = captures.name("user2")?.as_str();

        if domain1 == domain2 && user1 == user2 {
            Some(format!("{domain1}|{user1}#{domain2}|{user2}"))
        } else {
            None
        }
    }

    fn restore_wikidot_code_block_dom_compatibility(html: &str) -> String {
        let html = WIKIJUMP_CODE_BLOCK_PANEL_REGEX.replace_all(html, "");
        let html =
            WIKIJUMP_CODE_BLOCK_OPEN_REGEX.replace_all(&html, r#"<div class="code">"#);
        html.replace("</wj-code>", "</div>")
    }

    fn restore_wikidot_tabview_dom_compatibility(html: &str) -> String {
        let html = html.replace(
            r#"<wj-tabs class="wj-tabs">"#,
            &format!(r#"{WIKIDOT_TABVIEW_SCRIPT}<div class="yui-navset">"#),
        );
        let html = WIKIJUMP_TAB_BUTTON_LIST_REGEX
            .replace_all(&html, r#"<ul class="yui-nav">$body</ul>"#);
        let html = Self::restore_wikidot_tab_panel_visibility(&html);
        let html = WIKIJUMP_TAB_PANEL_LIST_OPEN_REGEX
            .replace_all(&html, r#"<div class="yui-content">"#);
        let html = WIKIJUMP_SELECTED_TAB_BUTTON_REGEX
            .replace_all(&html, r#"<li class="selected"><a href="javascript:;">"#);
        let html = WIKIJUMP_TAB_BUTTON_REGEX
            .replace_all(&html, r#"<li><a href="javascript:;">"#);
        html.replace("</wj-tabs-button>", "</a></li>\n").replace(
            "</wj-tabs>",
            &format!("</div>{WIKIDOT_TABVIEW_INIT_SCRIPT}"),
        )
    }

    fn restore_wikidot_tab_panel_visibility(html: &str) -> String {
        let mut panel_index = 0usize;
        let mut last_copied = 0usize;
        let mut group_scan_start = 0usize;
        let mut restored = String::with_capacity(html.len());

        for captures in WIKIJUMP_TAB_PANEL_REGEX.captures_iter(html) {
            let Some(panel_match) = captures.get(0) else {
                continue;
            };

            if WIKIJUMP_TAB_PANEL_LIST_OPEN_REGEX
                .is_match(&html[group_scan_start..panel_match.start()])
            {
                panel_index = 0;
            }

            restored.push_str(&html[last_copied..panel_match.start()]);
            let panel = panel_match.as_str();
            let hidden = Self::wikijump_tab_panel_is_hidden(panel);
            if panel_index == 0 && !hidden {
                restored.push_str(r#"<div style="display: block;">"#);
            } else {
                restored.push_str(r#"<div style="display:none">"#);
            }
            panel_index += 1;
            last_copied = panel_match.end();
            group_scan_start = panel_match.end();
        }

        restored.push_str(&html[last_copied..]);
        restored
    }

    fn wikijump_tab_panel_is_hidden(panel_open_tag: &str) -> bool {
        panel_open_tag
            .trim_end_matches('>')
            .split_ascii_whitespace()
            .any(|attribute| attribute == "hidden" || attribute.starts_with("hidden="))
    }

    fn restore_wikidot_footnote_dom_compatibility(html: &str) -> String {
        let html = WIKIJUMP_FOOTNOTE_MARKER_REGEX
            .replace_all(html, |captures: &regex::Captures<'_>| {
                let attrs = captures.name("attrs").map_or("", |mtch| mtch.as_str());
                let label = captures.name("label").map_or("", |mtch| mtch.as_str());
                let Some(id) = WIKIJUMP_FOOTNOTE_DATA_ID_REGEX
                    .captures(attrs)
                    .and_then(|data_id| data_id.name("id"))
                    .map(|mtch| mtch.as_str())
                else {
                    return captures.get(0).unwrap().as_str().to_owned();
                };

                format!(
                    r#"<sup class="footnoteref"><a id="footnoteref-{id}" href="javascript:;" class="footnoteref" onclick="WIKIDOT.page.utils.scrollToReference('footnote-{id}')">{label}</a></sup>"#
                )
            })
            .into_owned();
        let html = WIKIJUMP_FOOTNOTE_REF_LEADING_SPACE_REGEX
            .replace_all(&html, |captures: &regex::Captures<'_>| {
                format!("{}{}", &captures["before"], &captures["footnote"])
            })
            .into_owned();
        let html = WIKIJUMP_FOOTNOTE_REF_SPAN_WRAPPER_REGEX
            .replace_all(&html, |captures: &regex::Captures<'_>| {
                captures
                    .name("body")
                    .map_or("", |mtch| mtch.as_str())
                    .to_owned()
            })
            .into_owned();
        let html = Self::remove_wikijump_footnote_ref_tooltips(&html);
        html.replace(
            r#"<div class="wj-footnote-list">"#,
            r#"<div class="wj-footnote-list footnotes-footer">"#,
        )
        .replace(
            r#"<div class="wj-title">Footnotes</div>"#,
            r#"<div class="wj-title title">Footnotes</div>"#,
        )
    }

    fn remove_wikijump_footnote_ref_tooltips(html: &str) -> String {
        const DIV_TOOLTIP_OPEN: &str = r#"<div class="wj-footnote-ref-tooltip""#;
        const SPAN_TOOLTIP_OPEN: &str = r#"<span class="wj-footnote-ref-tooltip""#;
        let mut output = String::with_capacity(html.len());
        let mut cursor = 0usize;

        loop {
            let div_start = html[cursor..]
                .find(DIV_TOOLTIP_OPEN)
                .map(|offset| (cursor + offset, "<div", "</div>"));
            let span_start = html[cursor..]
                .find(SPAN_TOOLTIP_OPEN)
                .map(|offset| (cursor + offset, "<span", "</span>"));
            let Some((start, open_tag, close_tag)) = (match (div_start, span_start) {
                (Some(div), Some(span)) => Some(if div.0 < span.0 { div } else { span }),
                (Some(div), None) => Some(div),
                (None, Some(span)) => Some(span),
                (None, None) => None,
            }) else {
                break;
            };

            output.push_str(&html[cursor..start]);

            let Some(end) =
                Self::balanced_html_element_end(html, start, open_tag, close_tag)
            else {
                output.push_str(&html[start..]);
                return output;
            };
            cursor = end;
        }

        output.push_str(&html[cursor..]);
        output
    }

    fn balanced_html_element_end(
        html: &str,
        start: usize,
        open_tag: &str,
        close_tag: &str,
    ) -> Option<usize> {
        let mut cursor = start;
        let mut depth = 0usize;

        loop {
            let next_open = html[cursor..].find(open_tag).map(|offset| cursor + offset);
            let next_close = html[cursor..].find(close_tag).map(|offset| cursor + offset);

            match (next_open, next_close) {
                (Some(open), Some(close)) if open < close => {
                    depth += 1;
                    cursor = open + open_tag.len();
                }
                (Some(_), None) => return None,
                (_, Some(close)) => {
                    if depth == 0 {
                        return None;
                    }
                    depth -= 1;
                    cursor = close + close_tag.len();
                    if depth == 0 {
                        return Some(cursor);
                    }
                }
                (None, None) => return None,
            }
        }
    }

    fn restore_residual_wikidot_div_paragraph_markers(html: &str) -> String {
        let mut restored_open_count = 0usize;

        let restored = WIKIDOT_RESIDUAL_DIV_PARAGRAPH_REGEX
            .replace_all(html, |captures: &regex::Captures<'_>| {
                if let Some(marker) = captures.name("open") {
                    let marker = marker
                        .as_str()
                        .replace("&quot;", "\"")
                        .replace("&#34;", "\"");
                    if let Some(attributes) =
                        Self::wikidot_residual_div_attributes(&marker)
                    {
                        restored_open_count += 1;
                        return format!("<div{attributes}>");
                    }

                    return captures.get(0).unwrap().as_str().to_owned();
                }

                if restored_open_count == 0 {
                    return captures.get(0).unwrap().as_str().to_owned();
                }

                restored_open_count -= 1;
                "</div>".to_owned()
            })
            .into_owned();

        Self::restore_standalone_residual_wikidot_div_markers(&restored)
    }

    fn restore_standalone_residual_wikidot_div_markers(html: &str) -> String {
        let mut output = String::with_capacity(html.len());
        let mut restored_open_count = 0usize;
        let mut raw_text_depth = 0usize;

        for line in html.split_inclusive('\n') {
            let (line_body, line_end) = line
                .strip_suffix('\n')
                .map_or((line, ""), |body| (body, "\n"));
            let trimmed = line_body.trim();
            let protected = raw_text_depth > 0;

            if !protected && trimmed.starts_with("[[div") && trimmed.ends_with("]]") {
                let marker = trimmed.replace("&quot;", "\"").replace("&#34;", "\"");
                if let Some(attributes) = Self::wikidot_residual_div_attributes(&marker) {
                    restored_open_count += 1;
                    Self::push_replaced_standalone_wikidot_marker_line(
                        &mut output,
                        line_body,
                        line_end,
                        &format!("<div{attributes}>"),
                    );
                    raw_text_depth = Self::update_residual_div_raw_text_depth(
                        raw_text_depth,
                        line_body,
                    );
                    continue;
                }
            }

            if !protected
                && trimmed.eq_ignore_ascii_case("[[/div]]")
                && restored_open_count > 0
            {
                restored_open_count -= 1;
                Self::push_replaced_standalone_wikidot_marker_line(
                    &mut output,
                    line_body,
                    line_end,
                    "</div>",
                );
                raw_text_depth =
                    Self::update_residual_div_raw_text_depth(raw_text_depth, line_body);
                continue;
            }

            output.push_str(line_body);
            output.push_str(line_end);
            raw_text_depth =
                Self::update_residual_div_raw_text_depth(raw_text_depth, line_body);
        }

        output
    }

    fn push_replaced_standalone_wikidot_marker_line(
        output: &mut String,
        line_body: &str,
        line_end: &str,
        replacement: &str,
    ) {
        let prefix_len = line_body.len() - line_body.trim_start().len();
        let suffix_len = line_body.len() - line_body.trim_end().len();
        output.push_str(&line_body[..prefix_len]);
        output.push_str(replacement);
        output.push_str(&line_body[line_body.len() - suffix_len..]);
        output.push_str(line_end);
    }

    fn update_residual_div_raw_text_depth(mut depth: usize, line: &str) -> usize {
        let lower = line.to_ascii_lowercase();
        for tag in ["pre", "code", "textarea", "style", "script"] {
            depth += lower.matches(&format!("<{tag}")).count();
            depth = depth.saturating_sub(lower.matches(&format!("</{tag}>")).count());
        }
        depth
    }

    fn restore_residual_wikidot_span_markers(html: &str) -> String {
        let mut open_markers: Vec<(Range<usize>, String)> = Vec::new();
        let mut replacements: Vec<(Range<usize>, String)> = Vec::new();

        for segment in html_data_segments(html) {
            if !segment.continues_from_previous {
                open_markers.clear();
            }
            let data_range = segment.range;
            let data = &html[data_range.clone()];
            let mut cursor = 0;
            while cursor < data.len() {
                let open = data[cursor..].find("[[span");
                let close = data[cursor..].find("[[/span]]");
                let (offset, closing) = match (open, close) {
                    (Some(open), Some(close)) if close < open => (close, true),
                    (Some(open), _) => (open, false),
                    (None, Some(close)) => (close, true),
                    (None, None) => break,
                };
                let start = cursor + offset;

                if closing {
                    let end = start + "[[/span]]".len();
                    if let Some((open_range, open_tag)) = open_markers.pop() {
                        replacements.push((open_range, open_tag));
                        replacements.push((
                            data_range.start + start..data_range.start + end,
                            "</span>".to_owned(),
                        ));
                    }
                    cursor = end;
                    continue;
                }

                let marker_start = &data[start..];
                let Some(relative_end) = marker_start.find("]]") else {
                    break;
                };
                let end = start + relative_end + 2;
                let marker = &data[start..end];
                let decoded_marker = Self::decode_residual_wikidot_marker_quotes(marker);
                if let Some(open_tag) = wikidot_inline_span_marker_open(&decoded_marker) {
                    open_markers.push((
                        data_range.start + start..data_range.start + end,
                        open_tag,
                    ));
                }
                cursor = end;
            }
        }

        if replacements.is_empty() {
            return html.to_owned();
        }
        replacements.sort_by_key(|(range, _)| range.start);

        let mut output = String::with_capacity(html.len());
        let mut cursor = 0;
        for (range, replacement) in replacements {
            output.push_str(&html[cursor..range.start]);
            output.push_str(&replacement);
            cursor = range.end;
        }
        output.push_str(&html[cursor..]);
        output
    }

    fn decode_residual_wikidot_marker_quotes(marker: &str) -> String {
        marker
            .replace("&quot;", "\"")
            .replace("&#34;", "\"")
            .replace("&#x22;", "\"")
            .replace("&#X22;", "\"")
    }

    fn restore_residual_wikidot_alignment_markers(html: &str) -> String {
        let mut output = String::with_capacity(html.len());
        let mut alignment_stack: Vec<&'static str> = Vec::new();
        let mut raw_text_depth = 0usize;

        for line in html.split_inclusive('\n') {
            let (line_body, line_end) = line
                .strip_suffix('\n')
                .map_or((line, ""), |body| (body, "\n"));
            let trimmed = line_body.trim();
            let protected = raw_text_depth > 0;

            if !protected {
                if let Some((alignment, replacement)) =
                    Self::residual_wikidot_alignment_open_replacement(trimmed)
                {
                    alignment_stack.push(alignment);
                    Self::push_replaced_standalone_wikidot_marker_line(
                        &mut output,
                        line_body,
                        line_end,
                        replacement,
                    );
                    raw_text_depth = Self::update_residual_div_raw_text_depth(
                        raw_text_depth,
                        line_body,
                    );
                    continue;
                }

                if let Some(alignment) = Self::residual_wikidot_alignment_close(trimmed)
                    && alignment_stack.last().copied() == Some(alignment)
                {
                    alignment_stack.pop();
                    Self::push_replaced_standalone_wikidot_marker_line(
                        &mut output,
                        line_body,
                        line_end,
                        "</div>",
                    );
                    raw_text_depth = Self::update_residual_div_raw_text_depth(
                        raw_text_depth,
                        line_body,
                    );
                    continue;
                }
            }

            output.push_str(line_body);
            output.push_str(line_end);
            raw_text_depth =
                Self::update_residual_div_raw_text_depth(raw_text_depth, line_body);
        }

        Self::restore_residual_wikidot_alignment_html_markers(&output)
    }

    fn restore_residual_wikidot_alignment_html_markers(html: &str) -> String {
        const MARKERS: &[(&str, &str, &str, bool)] = &[
            (
                "<p>[[=]]</p>",
                "center",
                r#"<div style="text-align: center;">"#,
                false,
            ),
            (
                "<p>[[<]]</p>",
                "left",
                r#"<div style="text-align: left;">"#,
                false,
            ),
            (
                "<p>[[&lt;]]</p>",
                "left",
                r#"<div style="text-align: left;">"#,
                false,
            ),
            (
                "<p>[[>]]</p>",
                "right",
                r#"<div style="text-align: right;">"#,
                false,
            ),
            (
                "<p>[[&gt;]]</p>",
                "right",
                r#"<div style="text-align: right;">"#,
                false,
            ),
            ("<p>[[/=]]</p>", "center", "</div>", true),
            ("<br>[[/=]]<br>", "center", "</div><br>", true),
            ("<br/>[[/=]]<br/>", "center", "</div><br/>", true),
            ("<br />[[/=]]<br />", "center", "</div><br />", true),
            ("<p>[[/<]]</p>", "left", "</div>", true),
            ("<p>[[/&lt;]]</p>", "left", "</div>", true),
            ("<br>[[/<]]<br>", "left", "</div><br>", true),
            ("<br>[[/&lt;]]<br>", "left", "</div><br>", true),
            ("<p>[[/>]]</p>", "right", "</div>", true),
            ("<p>[[/&gt;]]</p>", "right", "</div>", true),
            ("<br>[[/>]]<br>", "right", "</div><br>", true),
            ("<br>[[/&gt;]]<br>", "right", "</div><br>", true),
        ];

        let mut output = String::with_capacity(html.len());
        let mut rest = html;
        let mut alignment_stack: Vec<&'static str> = Vec::new();

        loop {
            let Some((position, marker, alignment, replacement, is_close)) = MARKERS
                .iter()
                .filter_map(|(marker, alignment, replacement, is_close)| {
                    rest.find(marker).map(|position| {
                        (position, *marker, *alignment, *replacement, *is_close)
                    })
                })
                .min_by_key(|(position, ..)| *position)
            else {
                output.push_str(rest);
                return output;
            };

            output.push_str(&rest[..position]);
            if is_close {
                if alignment_stack.last().copied() == Some(alignment) {
                    alignment_stack.pop();
                    output.push_str(replacement);
                } else {
                    output.push_str(marker);
                }
            } else {
                alignment_stack.push(alignment);
                output.push_str(replacement);
            }
            rest = &rest[position + marker.len()..];
        }
    }

    fn residual_wikidot_alignment_open_replacement(
        marker: &str,
    ) -> Option<(&'static str, &'static str)> {
        match marker.to_ascii_lowercase().as_str() {
            "[[=]]" => Some(("center", r#"<div style="text-align: center;">"#)),
            "[[<]]" | "[[&lt;]]" => Some(("left", r#"<div style="text-align: left;">"#)),
            "[[>]]" | "[[&gt;]]" => {
                Some(("right", r#"<div style="text-align: right;">"#))
            }
            _ => None,
        }
    }

    fn residual_wikidot_alignment_close(marker: &str) -> Option<&'static str> {
        match marker.to_ascii_lowercase().as_str() {
            "[[/=]]" => Some("center"),
            "[[/<]]" | "[[/&lt;]]" => Some("left"),
            "[[/>]]" | "[[/&gt;]]" => Some("right"),
            _ => None,
        }
    }

    fn restore_residual_wikidot_separator_markers(html: &str) -> String {
        let mut output = String::with_capacity(html.len());
        let mut raw_text_depth = 0usize;

        for line in html.split_inclusive('\n') {
            let (line_body, line_end) = line
                .strip_suffix('\n')
                .map_or((line, ""), |body| (body, "\n"));
            let trimmed = line_body.trim();
            let protected = raw_text_depth > 0;

            if !protected {
                if Self::residual_wikidot_horizontal_rule_line(trimmed) {
                    Self::push_replaced_standalone_wikidot_marker_line(
                        &mut output,
                        line_body,
                        line_end,
                        "<hr>",
                    );
                    raw_text_depth = Self::update_residual_div_raw_text_depth(
                        raw_text_depth,
                        line_body,
                    );
                    continue;
                }

                if trimmed == "@@ @@" {
                    Self::push_replaced_standalone_wikidot_marker_line(
                        &mut output,
                        line_body,
                        line_end,
                        r#"<p><span style="white-space: pre-wrap;"> </span></p>"#,
                    );
                    raw_text_depth = Self::update_residual_div_raw_text_depth(
                        raw_text_depth,
                        line_body,
                    );
                    continue;
                }

                if trimmed == "~~~~" {
                    Self::push_replaced_standalone_wikidot_marker_line(
                        &mut output,
                        line_body,
                        line_end,
                        r#"<div style="clear:both; height: 0px; font-size: 1px"></div>"#,
                    );
                    raw_text_depth = Self::update_residual_div_raw_text_depth(
                        raw_text_depth,
                        line_body,
                    );
                    continue;
                }
            }

            output.push_str(line_body);
            output.push_str(line_end);
            raw_text_depth =
                Self::update_residual_div_raw_text_depth(raw_text_depth, line_body);
        }

        output
    }

    fn residual_wikidot_horizontal_rule_line(line: &str) -> bool {
        line.len() >= 4 && line.chars().all(|character| character == '-')
    }

    fn restore_residual_wikidot_heading_markers(html: &str) -> String {
        let mut output = String::with_capacity(html.len());
        let mut raw_text_depth = 0usize;

        for line in html.split_inclusive('\n') {
            let (line_body, line_end) = line
                .strip_suffix('\n')
                .map_or((line, ""), |body| (body, "\n"));
            let trimmed = line_body.trim();
            let protected = raw_text_depth > 0;

            if !protected {
                if Self::residual_wikidot_content_section_line(trimmed) {
                    Self::push_replaced_standalone_wikidot_marker_line(
                        &mut output,
                        line_body,
                        line_end,
                        "",
                    );
                    raw_text_depth = Self::update_residual_div_raw_text_depth(
                        raw_text_depth,
                        line_body,
                    );
                    continue;
                }

                if let Some((level, body)) =
                    Self::residual_wikidot_heading_replacement(trimmed)
                {
                    Self::push_replaced_standalone_wikidot_marker_line(
                        &mut output,
                        line_body,
                        line_end,
                        &format!("<h{level}><span>{body}</span></h{level}>"),
                    );
                    raw_text_depth = Self::update_residual_div_raw_text_depth(
                        raw_text_depth,
                        line_body,
                    );
                    continue;
                }
            }

            output.push_str(line_body);
            output.push_str(line_end);
            raw_text_depth =
                Self::update_residual_div_raw_text_depth(raw_text_depth, line_body);
        }

        output
    }

    fn residual_wikidot_content_section_line(line: &str) -> bool {
        line.len() >= 4 && line.chars().all(|character| character == '=')
    }

    fn residual_wikidot_heading_replacement(line: &str) -> Option<(usize, &str)> {
        let level = line.bytes().take_while(|byte| *byte == b'+').count();
        if !(1..=6).contains(&level) {
            return None;
        }

        let mut body = &line[level..];
        if body.starts_with('*') {
            body = &body[1..];
        }
        body = body.trim_start();
        if body.is_empty() {
            return None;
        }

        Some((level, body))
    }

    fn remove_residual_wikidot_iftags_fragments(html: &str) -> String {
        WIKIDOT_RESIDUAL_IFTAGS_INLINE_REGEX
            .replace_all(html, "")
            .into_owned()
    }

    fn prepare_wikidot_conditionals_for_include_expansion(
        wikitext: &mut String,
        page_info: &ftml::data::PageInfo<'_>,
    ) {
        Self::remove_unresolved_variable_iftags_blocks(wikitext);
        Self::resolve_single_line_wikidot_iftags_fragments(wikitext, page_info);
        Self::resolve_simple_wikidot_iftags_blocks(wikitext, page_info);
        *wikitext = Self::resolve_wikidot_simple_if_fragments(wikitext);
    }

    fn resolve_single_line_wikidot_iftags_fragments(
        wikitext: &mut String,
        page_info: &ftml::data::PageInfo<'_>,
    ) {
        let resolved = WIKIDOT_SINGLE_LINE_IFTAGS_REGEX.replace_all(
            wikitext,
            |captures: &regex::Captures<'_>| {
                if wikidot_tag_conditions_match(&captures["spec"], &page_info.tags) {
                    captures["body"].to_owned()
                } else {
                    String::new()
                }
            },
        );
        if let Cow::Owned(resolved) = resolved {
            *wikitext = resolved;
        }
    }

    fn resolve_simple_wikidot_iftags_blocks(
        wikitext: &mut String,
        page_info: &ftml::data::PageInfo<'_>,
    ) {
        loop {
            let resolved = WIKIDOT_SIMPLE_IFTAGS_BLOCK_REGEX.replace_all(
                wikitext,
                |captures: &regex::Captures<'_>| {
                    let body = captures.name("body").map_or("", |mtch| mtch.as_str());
                    if body.contains("[[iftags") {
                        return captures
                            .get(0)
                            .map_or("", |mtch| mtch.as_str())
                            .to_owned();
                    }

                    let spec = captures.name("spec").map_or("", |mtch| mtch.as_str());
                    if wikidot_tag_conditions_match(spec, &page_info.tags) {
                        body.to_owned()
                    } else {
                        String::new()
                    }
                },
            );

            match resolved {
                Cow::Borrowed(_) => return,
                Cow::Owned(resolved) if resolved == *wikitext => return,
                Cow::Owned(resolved) => *wikitext = resolved,
            }
        }
    }

    fn resolve_wikidot_simple_if_fragments(html: &str) -> String {
        let mut resolved = html.to_owned();

        for _ in 0..MAX_WIKIDOT_SIMPLE_IF_PASSES {
            let source = resolved.clone();
            let literal_regions = LiteralRegionIndex::new(&source);
            let mut replacements: Vec<(Range<usize>, String)> = Vec::new();
            let mut search_start = 0usize;

            while let Some(captures) =
                WIKIDOT_SIMPLE_IF_OPEN_REGEX.captures(&source[search_start..])
            {
                let full_open =
                    captures.get(0).expect("simple-if opening capture exists");
                let conditional_start = search_start + full_open.start();
                let branch_start = search_start + full_open.end();
                let Some((conditional_end, separator)) =
                    Self::find_wikidot_simple_if_end(&source, branch_start)
                else {
                    search_start = branch_start;
                    continue;
                };
                search_start = conditional_end;

                if literal_regions.contains(conditional_start) {
                    continue;
                }

                let condition = captures
                    .name("cond")
                    .expect("simple-if condition capture exists")
                    .as_str();
                let close_start = conditional_end - 2;
                let selected = if condition.eq_ignore_ascii_case("1")
                    || condition.eq_ignore_ascii_case("true")
                {
                    &source[branch_start..separator.unwrap_or(close_start)]
                } else {
                    separator.map_or("", |separator| &source[separator + 1..close_start])
                };
                replacements.push((
                    conditional_start..conditional_end,
                    selected.trim().to_owned(),
                ));
            }

            if replacements.is_empty() {
                return resolved;
            }
            for (range, replacement) in replacements.into_iter().rev() {
                resolved.replace_range(range, &replacement);
            }
        }

        resolved
    }

    fn find_wikidot_simple_if_end(
        source: &str,
        branch_start: usize,
    ) -> Option<(usize, Option<usize>)> {
        let bytes = source.as_bytes();
        let mut cursor = branch_start;
        let mut nested_wikidot_depth = 1usize;
        let mut separator = None;

        while cursor + 1 < bytes.len() {
            if bytes[cursor..].starts_with(b"[[") {
                nested_wikidot_depth += 1;
                cursor += 2;
                continue;
            }
            if bytes[cursor..].starts_with(b"]]") {
                if nested_wikidot_depth == 1 {
                    return Some((cursor + 2, separator));
                }
                nested_wikidot_depth -= 1;
                cursor += 2;
                continue;
            }
            if bytes[cursor] == b'|' && nested_wikidot_depth == 1 && separator.is_none() {
                separator = Some(cursor);
            }
            cursor += 1;
        }

        None
    }

    fn resolve_wikidot_parser_functions(value: &str) -> String {
        resolve_parser_functions(value)
    }

    fn normalize_wikidot_div_style_url_quotes(wikitext: &mut String) {
        let mut normalized = String::with_capacity(wikitext.len());
        let mut changed = false;

        for line in wikitext.split_inclusive('\n') {
            if !line.trim_start().starts_with("[[div") || !line.contains("url(\"") {
                normalized.push_str(line);
                continue;
            }

            let mut line = line.to_owned();
            let mut search_start = 0usize;
            while let Some(open_offset) = line[search_start..].find("url(\"") {
                let open_quote = search_start + open_offset + "url(".len();
                let value_start = open_quote + 1;
                let Some(close_offset) = line[value_start..].find("\")") else {
                    break;
                };
                let close_quote = value_start + close_offset;

                line.replace_range(close_quote..close_quote + 1, "'");
                line.replace_range(open_quote..open_quote + 1, "'");
                search_start = open_quote + "url('".len();
                changed = true;
            }

            normalized.push_str(&line);
        }

        if changed {
            *wikitext = normalized;
        }
    }

    fn protect_wikidot_marker_class_include_variables(wikitext: &mut String) {
        if !wikitext.contains("{$") {
            return;
        }

        let mut normalized = String::with_capacity(wikitext.len());
        let mut changed = false;

        for line in wikitext.split_inclusive('\n') {
            let trimmed = line.trim_start();
            if !(trimmed.starts_with("[[div") || trimmed.starts_with("[[span"))
                || !line.contains("class=\"")
                || !line.contains("{$")
            {
                normalized.push_str(line);
                continue;
            }

            let mut line = line.to_owned();
            let mut search_start = 0usize;
            while let Some(attr_offset) = line[search_start..].find("class=\"") {
                let value_start = search_start + attr_offset + "class=\"".len();
                let Some(value_end_offset) = line[value_start..].find('"') else {
                    break;
                };
                let value_end = value_start + value_end_offset;
                let value = &line[value_start..value_end];
                let protected = INCLUDE_VARIABLE_REGEX
                    .replace_all(value, |captures: &regex::Captures<'_>| {
                        format!(
                            "{WIKIDOT_CLASS_INCLUDE_VARIABLE_SENTINEL_PREFIX}{}",
                            &captures["name"]
                        )
                    })
                    .into_owned();

                if protected != value {
                    line.replace_range(value_start..value_end, &protected);
                    changed = true;
                    search_start = value_start + protected.len();
                } else {
                    search_start = value_end + 1;
                }
            }

            normalized.push_str(&line);
        }

        if changed {
            *wikitext = normalized;
        }
    }

    fn restore_protected_wikidot_marker_class_include_variables(html: &str) -> String {
        WIKIDOT_CLASS_INCLUDE_VARIABLE_SENTINEL_REGEX
            .replace_all(html, |captures: &regex::Captures<'_>| {
                format!("{{${}}}", &captures["name"])
            })
            .into_owned()
    }

    fn normalize_wikidot_multiline_page_links(wikitext: &mut String) {
        let source = wikitext.clone();
        let mut normalized = String::with_capacity(source.len());
        let mut last = 0usize;
        let mut changed = false;

        for captures in WIKIDOT_MULTILINE_LABELED_LINK_REGEX.captures_iter(&source) {
            let Some(link_match) = captures.get(0) else {
                continue;
            };

            normalized.push_str(&source[last..link_match.start()]);
            last = link_match.end();

            if Self::is_inside_wikidot_literal_region(&source, link_match.start()) {
                normalized.push_str(link_match.as_str());
                continue;
            }

            let Some(target) = captures
                .name("target")
                .map(|matched| matched.as_str().trim())
                .filter(|target| !target.is_empty())
            else {
                normalized.push_str(link_match.as_str());
                continue;
            };
            let Some(label) = captures
                .name("label")
                .map(|matched| Self::collapse_wikidot_inline_whitespace(matched.as_str()))
                .filter(|label| !label.is_empty())
            else {
                normalized.push_str(link_match.as_str());
                continue;
            };

            normalized.push_str(&format!("[[[{target}|{label}]]]"));
            changed = true;
        }

        if !changed {
            return;
        }

        normalized.push_str(&source[last..]);
        *wikitext = normalized;
    }

    fn collapse_wikidot_inline_whitespace(value: &str) -> String {
        value.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn restore_wikidot_mailform_compatibility(html: &str) -> String {
        WIKIDOT_RENDERED_MAILFORM_REGEX
            .replace_all(html, |captures: &regex::Captures<'_>| {
                let head = captures.name("head").map_or("", |mtch| mtch.as_str());
                let body = captures.name("body").map_or("", |mtch| mtch.as_str());
                let name = WIKIDOT_RENDERED_MAILFORM_FIELD_REGEX
                    .captures(body)
                    .and_then(|captures| captures.name("name"))
                    .map_or("field", |mtch| mtch.as_str().trim());
                let default = WIKIDOT_RENDERED_MAILFORM_DEFAULT_REGEX
                    .captures(body)
                    .and_then(|captures| captures.name("default"))
                    .map_or("", |mtch| mtch.as_str().trim());
                let max_length = WIKIDOT_RENDERED_MAILFORM_MAX_LENGTH_REGEX
                    .captures(body)
                    .and_then(|captures| captures.name("max"))
                    .map_or("256", |mtch| mtch.as_str().trim());
                let button =
                    rendered_wikidot_mailform_attribute(head, "button").unwrap_or_default();

                format!(
                    concat!(
                        r#"<div class="mailform-box">"#,
                        r#"<form class="form" action="javascript:;">"#,
                        r#"<table>"#,
                        r#"<tr><td>"#,
                        r#"<input class="text" type="text" name="{name}" value="{default}" maxlength="{max_length}" size="30">"#,
                        r#"</td><td><div class="field-error-message"></div></td></tr>"#,
                        r#"<tr><td colspan="2"><div class="buttons"><input type="submit" value="{button}"></div></td></tr>"#,
                        r#"</table>"#,
                        r#"</form>"#,
                        r#"</div>"#,
                    ),
                    name = name,
                    default = default,
                    max_length = max_length,
                    button = button,
                )
            })
            .into_owned()
    }

    fn remove_wikijump_table_body_wrappers(html: &str) -> String {
        html.replace("<tbody>", "").replace("</tbody>", "")
    }

    fn remove_wikidot_compat_style_blocks(html: &str) -> String {
        WIKIDOT_COMPAT_STYLE_BLOCK_REGEX
            .replace_all(html, "")
            .into_owned()
    }

    fn restore_wikidot_inline_math_compatibility(html: &str) -> String {
        WIKIJUMP_INLINE_MATH_REGEX
            .replace_all(html, r#"<span class="math-inline">$$${source}$$</span>"#)
            .into_owned()
    }

    fn restore_wikidot_ta_badge_default_compatibility(html: &str) -> String {
        html.replace("bg-shadow-{$bg-shadow}", "bg-shadow-true")
            .replace("plate-shadow-{$plate-shadow}", "plate-shadow-true")
            .replace(
                "item-mobile-mode-{$item-mobile-mode}",
                "item-mobile-mode-true",
            )
            .replace("item-align-{$item-align}", "item-align-true")
            .replace("{$badge-top-link}", "empty")
            .replace("{$badge-right-link}", "empty")
            .replace("{$badge-left-link}", "empty")
            .replace("{$item-lt-link}", "empty")
            .replace("{$item-lc-link}", "empty")
            .replace("{$item-lb-link}", "empty")
            .replace("{$item-rt-link}", "empty")
            .replace("{$item-rc-link}", "empty")
            .replace("{$item-rb-link}", "empty")
    }

    fn restore_wikidot_text_ellipsis_compatibility(html: &str) -> String {
        let mut output = String::with_capacity(html.len());
        let mut cursor = 0usize;
        let mut literal_depth = 0usize;

        while let Some(tag_start_offset) = html[cursor..].find('<') {
            let tag_start = cursor + tag_start_offset;
            Self::push_wikidot_text_ellipsis_segment(
                &mut output,
                &html[cursor..tag_start],
                literal_depth,
            );

            let Some(tag_end_offset) = html[tag_start..].find('>') else {
                output.push_str(&html[tag_start..]);
                return output;
            };
            let tag_end = tag_start + tag_end_offset + 1;
            let tag = &html[tag_start..tag_end];
            Self::update_wikidot_ellipsis_literal_depth(tag, &mut literal_depth);
            output.push_str(tag);
            cursor = tag_end;
        }

        Self::push_wikidot_text_ellipsis_segment(
            &mut output,
            &html[cursor..],
            literal_depth,
        );
        output
    }

    fn push_wikidot_text_ellipsis_segment(
        output: &mut String,
        segment: &str,
        literal_depth: usize,
    ) {
        if literal_depth == 0 {
            output.push_str(&segment.replace("...", "…"));
        } else {
            output.push_str(segment);
        }
    }

    fn update_wikidot_ellipsis_literal_depth(tag: &str, literal_depth: &mut usize) {
        let Some((name, closing, self_closing)) = Self::html_tag_name(tag) else {
            return;
        };
        if !matches!(
            name.as_str(),
            "code" | "pre" | "script" | "style" | "textarea"
        ) {
            return;
        }

        if closing {
            *literal_depth = literal_depth.saturating_sub(1);
        } else if !self_closing {
            *literal_depth += 1;
        }
    }

    fn html_tag_name(tag: &str) -> Option<(String, bool, bool)> {
        let inner = tag.strip_prefix('<')?.strip_suffix('>')?.trim();
        if inner.is_empty() || inner.starts_with('!') || inner.starts_with('?') {
            return None;
        }

        let closing = inner.starts_with('/');
        let inner = if closing {
            inner[1..].trim_start()
        } else {
            inner
        };
        let name = inner
            .split(|character: char| {
                character.is_ascii_whitespace() || character == '/' || character == '>'
            })
            .next()?
            .to_ascii_lowercase();
        if name.is_empty() {
            return None;
        }

        Some((name, closing, inner.ends_with('/')))
    }

    fn remove_wikijump_underline_wrappers(html: &str) -> String {
        // FTML uses semantic <s> elements for paired Wikidot --text--
        // strikethrough. Those are visible formatting, not plain wrappers.
        html.replace("<u>", "").replace("</u>", "")
    }

    fn remove_wikidot_userkarma_background_styles(html: &str) -> String {
        WIKIDOT_USERKARMA_BACKGROUND_STYLE_REGEX
            .replace_all(html, "")
            .into_owned()
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

        Self::remove_collapsed_empty_negative_iftags_blocks(wikitext);
        Self::remove_collapsed_basalt_iftags_blocks(wikitext);
    }

    fn remove_wikidot_component_iftags_documentation(wikitext: &mut String) {
        const OPEN_MARKER: &str = "[[iftags +component]]";
        const CLOSE_MARKER: &str = "[[/iftags]]";

        while let Some(open_start) = wikitext.find(OPEN_MARKER) {
            let body_start = open_start + OPEN_MARKER.len();
            let Some(close_offset) = wikitext[body_start..].find(CLOSE_MARKER) else {
                break;
            };
            let close_end = body_start + close_offset + CLOSE_MARKER.len();
            let block_start = wikitext[..open_start]
                .rfind('\n')
                .map_or(0, |index| index + 1);
            let block_end = wikitext[close_end..]
                .find('\n')
                .map_or(wikitext.len(), |offset| close_end + offset + 1);

            wikitext.replace_range(block_start..block_end, "");
        }
    }

    fn normalize_wikidot_ta_badge_multiline_includes(wikitext: &mut String) {
        const INCLUDE_PREFIX: &str = "[[include :scp-jp:user-component:ta-badge";

        let lines = Self::wikitext_line_ranges(wikitext);
        let mut replacements = Vec::new();
        let mut line_index = 0;

        while line_index < lines.len() {
            let (start, _, line) = lines[line_index];
            if !Self::trim_wikitext_line(line).starts_with(INCLUDE_PREFIX) {
                line_index += 1;
                continue;
            }

            let mut include_lines = vec![Self::trim_wikitext_line(line).to_owned()];
            let mut end_line_index = line_index;
            while !Self::trim_wikitext_line(lines[end_line_index].2).ends_with("]]") {
                end_line_index += 1;
                if end_line_index >= lines.len() {
                    break;
                }
                include_lines
                    .push(Self::trim_wikitext_line(lines[end_line_index].2).to_owned());
            }

            if end_line_index >= lines.len() {
                break;
            }

            let (_, end, _) = lines[end_line_index];
            let mut normalized = include_lines.join(" ");
            while normalized.contains("  ") {
                normalized = normalized.replace("  ", " ");
            }
            normalized = normalized.replace(" ]]", "]]");
            normalized.push('\n');
            replacements.push((start..end, normalized));
            line_index = end_line_index + 1;
        }

        for (range, replacement) in replacements.into_iter().rev() {
            wikitext.replace_range(range, &replacement);
        }
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

    fn remove_wikidot_metacomponent_documentation(wikitext: &mut String) {
        const BEGIN_MARKER: &str = "[!-- Begin metacomponent context detection --]";
        const END_MARKER: &str = "[!-- End metacomponent context detection --]";

        while let Some(begin_offset) = wikitext.find(BEGIN_MARKER) {
            let Some(end_offset) = wikitext[begin_offset..].find(END_MARKER) else {
                break;
            };
            let end = begin_offset + end_offset + END_MARKER.len();
            let replacement_start = wikitext[..begin_offset]
                .rfind('\n')
                .map_or(begin_offset, |index| index + 1);
            let replacement_end = wikitext[end..]
                .find('\n')
                .map_or(end, |offset| end + offset + 1);
            wikitext.replace_range(replacement_start..replacement_end, "");
        }
    }

    fn expand_wikidot_image_block_includes(
        wikitext: &mut String,
        page_info: &PageInfo<'_>,
    ) -> Vec<PageRef> {
        let source = wikitext.clone();
        let mut replacements: Vec<(Range<usize>, String)> = Vec::new();
        let mut included_pages = Vec::new();
        let mut search_start = 0;

        while let Some(captures) =
            WIKIDOT_IMAGE_BLOCK_INCLUDE_START_REGEX.captures(&source[search_start..])
        {
            let include_start =
                search_start + captures.get(0).expect("whole match exists").start();
            let match_end =
                search_start + captures.get(0).expect("whole match exists").end();
            let after = captures
                .name("after")
                .expect("after delimiter exists")
                .as_str();
            let (args_start, include_end) = if after == "]]" {
                (match_end - 2, match_end)
            } else {
                let args_start = match_end - after.len();
                let Some(include_end) =
                    Self::find_wikidot_include_end(&source, match_end)
                else {
                    search_start = match_end;
                    continue;
                };
                (args_start, include_end)
            };

            search_start = include_end;

            if Self::should_skip_wikidot_image_block_include_expansion(
                &source,
                include_start,
            ) || !Self::should_expand_wikidot_image_block_include(
                captures.name("site").map(|site| site.as_str()),
                page_info,
            ) {
                continue;
            }

            let args = Self::parse_wikidot_include_arguments(
                &source[args_start..include_end - 2],
            );
            let Some(name) = args.get("name").filter(|value| !value.is_empty()) else {
                continue;
            };

            let caption = args.get("caption").map_or("", String::as_str);
            let width = args.get("width").map_or("300px", String::as_str);
            let align = args.get("align").map_or("right", String::as_str);
            let link = args.get("link").map_or("#", String::as_str);
            let image_source = Self::wikidot_image_block_source(name, page_info);
            let image_attribute = args
                .get("alt")
                .filter(|attribute| is_include_variable_name(attribute))
                .zip(args.get("alt-text"))
                .map(|(attribute, value)| {
                    format!(r#" {attribute}="{}""#, value.replace('"', "&quot;"))
                })
                .unwrap_or_default();
            let link_attribute = if link == "#" {
                String::new()
            } else {
                format!(" link={link}")
            };

            let replacement = format!(
                concat!(
                    r#"[[div class="scp-image-block block-{align}" style="width:{width};"]]"#,
                    "\n",
                    r#"[[image {image_source}{image_attribute}{link_attribute}]]"#,
                    "\n",
                    r#"[[div class="scp-image-caption"]]"#,
                    "\n",
                    "{caption}\n",
                    "[[/div]]\n",
                    "[[/div]]"
                ),
                align = align,
                width = width,
                image_source = image_source,
                image_attribute = image_attribute,
                link_attribute = link_attribute,
                caption = caption,
            );

            Self::push_wikidot_image_block_include_refs(
                &mut included_pages,
                captures.name("site").map(|site| site.as_str()),
            );
            replacements.push((include_start..include_end, replacement));
        }

        for (range, replacement) in replacements.into_iter().rev() {
            wikitext.replace_range(range, &replacement);
        }

        included_pages
    }

    fn find_wikidot_include_end(source: &str, mut offset: usize) -> Option<usize> {
        let bytes = source.as_bytes();
        while offset + 1 < bytes.len() {
            if bytes[offset..].starts_with(b"[[[") {
                if let Some(close_offset) = source[offset + 3..].find("]]]") {
                    offset += 3 + close_offset + 3;
                    continue;
                }
            } else if bytes[offset..].starts_with(b"[[") {
                if let Some(close_offset) = source[offset + 2..].find("]]") {
                    offset += 2 + close_offset + 2;
                    continue;
                }
            } else if bytes[offset..].starts_with(b"]]") {
                return Some(offset + 2);
            }

            offset += 1;
        }

        None
    }

    fn push_wikidot_image_block_include_refs(
        included_pages: &mut Vec<PageRef>,
        site: Option<&str>,
    ) {
        included_pages.push(Self::wikidot_image_block_page_ref(
            site,
            "component:image-block",
        ));
        included_pages.push(Self::wikidot_image_block_page_ref(
            site,
            "component:image-block-base",
        ));
    }

    fn wikidot_image_block_page_ref(site: Option<&str>, page: &str) -> PageRef {
        match site {
            Some(site) => PageRef::page_and_site(site, page),
            None => PageRef::page_only(page),
        }
    }

    fn should_skip_wikidot_image_block_include_expansion(
        source: &str,
        start: usize,
    ) -> bool {
        Self::is_inside_wikidot_code_block(source, start)
            || Self::is_inside_wikidot_escape(source, start)
            || Self::is_inside_wikidot_html_block(source, start)
            || Self::is_inside_wikidot_comment(source, start)
    }

    fn should_expand_wikidot_image_block_include(
        include_site: Option<&str>,
        page_info: &PageInfo<'_>,
    ) -> bool {
        page_info.site.as_ref() == "scp-wiki"
            && include_site.is_none_or(|site| site == "scp-wiki")
    }

    fn is_inside_wikidot_code_block(source: &str, start: usize) -> bool {
        let mut in_code = false;
        for line in source[..start].lines() {
            let marker = line.trim_start().to_ascii_lowercase();
            if marker.starts_with("[[code") {
                in_code = true;
            } else if marker.starts_with("[[/code]]") {
                in_code = false;
            }
        }
        in_code
    }

    fn is_inside_wikidot_html_block(source: &str, start: usize) -> bool {
        let mut in_html = false;
        for line in source[..start].lines() {
            let marker = line.trim_start().to_ascii_lowercase();
            if marker.starts_with("[[html") {
                in_html = true;
            } else if marker.starts_with("[[/html]]") {
                in_html = false;
            }
        }
        in_html
    }

    fn is_inside_wikidot_escape(source: &str, start: usize) -> bool {
        source[..start].matches("@@").count() % 2 == 1
    }

    fn is_inside_wikidot_literal_region(source: &str, start: usize) -> bool {
        Self::is_inside_wikidot_code_block(source, start)
            || Self::is_inside_wikidot_escape(source, start)
            || Self::is_inside_wikidot_html_block(source, start)
            || Self::is_inside_wikidot_comment(source, start)
    }

    fn is_inside_wikidot_comment(source: &str, start: usize) -> bool {
        let before = &source[..start];
        let last_open = before.rfind("[!--");
        let last_close = before.rfind("--]");
        match (last_open, last_close) {
            (Some(open), Some(close)) => open > close,
            (Some(_), None) => true,
            _ => false,
        }
    }

    fn mask_wikidot_comment_include_markers(wikitext: &mut String) {
        let source = wikitext.clone();
        let mut replacements = Vec::new();

        for captures in WIKIDOT_INCLUDE_OPEN_LINE_REGEX.captures_iter(&source) {
            let keyword = captures
                .name("keyword")
                .expect("include keyword capture exists");
            if Self::is_inside_wikidot_comment(&source, keyword.start()) {
                replacements.push(keyword.range());
            }
        }

        for range in replacements.into_iter().rev() {
            wikitext.replace_range(range, WIKIDOT_COMMENT_INCLUDE_SENTINEL);
        }
    }

    fn unmask_wikidot_comment_include_markers(wikitext: &mut String) {
        if wikitext.contains(WIKIDOT_COMMENT_INCLUDE_SENTINEL) {
            *wikitext = wikitext.replace(WIKIDOT_COMMENT_INCLUDE_SENTINEL, "include");
        }
    }

    fn wikidot_image_block_source(name: &str, page_info: &PageInfo<'_>) -> String {
        if name.starts_with("http://")
            || name.starts_with("https://")
            || name.starts_with('/')
        {
            return name.to_owned();
        }

        let page_slug = match page_info.category.as_deref() {
            Some(category) => format!("{category}:{}", page_info.page),
            None => page_info.page.to_string(),
        };

        format!(
            "http://{}.wikidot.com/local--files/{}/{}",
            page_info.site, page_slug, name
        )
    }

    fn parse_wikidot_include_arguments(args: &str) -> BTreeMap<String, String> {
        Self::split_wikidot_include_argument_segments(args)
            .into_iter()
            .filter_map(|segment| {
                let (key, value) = segment.trim().split_once('=')?;
                let key = key.trim().to_ascii_lowercase();
                if key.is_empty() {
                    return None;
                }
                Some((key, value.trim().to_owned()))
            })
            .collect()
    }

    fn split_wikidot_include_argument_segments(args: &str) -> Vec<&str> {
        let mut segments = Vec::new();
        let mut segment_start = 0;
        let mut offset = 0;

        while offset < args.len() {
            if args[offset..].starts_with("[[[") {
                if let Some(close_offset) = args[offset + 3..].find("]]]") {
                    offset += 3 + close_offset + 3;
                    continue;
                }
            } else if args[offset..].starts_with("[[") {
                if let Some(close_offset) = args[offset + 2..].find("]]") {
                    offset += 2 + close_offset + 2;
                    continue;
                }
            } else if args[offset..].starts_with('|') {
                segments.push(&args[segment_start..offset]);
                offset += 1;
                segment_start = offset;
                continue;
            }

            let ch = args[offset..]
                .chars()
                .next()
                .expect("offset is inside argument string");
            offset += ch.len_utf8();
        }

        segments.push(&args[segment_start..]);
        segments
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

    fn remove_collapsed_empty_negative_iftags_blocks(wikitext: &mut String) {
        const ACTIVE_OPEN_MARKER: &str = "[[iftags -]]";
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
            let Some(inner_body_start) = outer_body
                .find(INNER_OPEN_MARKER)
                .map(|offset| outer_body_start + offset + INNER_OPEN_MARKER.len())
            else {
                break;
            };
            let replacement = wikitext[inner_body_start..first_body_end].to_owned();

            wikitext.replace_range(block_start..block_end, &replacement);
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

    fn protect_wikidot_wikipedia_links(
        wikitext: &mut String,
        settings: &WikitextSettings,
    ) -> Vec<ProtectedWikidotWikipediaLink> {
        if !settings.enable_page_syntax {
            return Vec::new();
        }

        let source = wikitext.clone();
        let mut links = Vec::new();
        let mut output = String::with_capacity(source.len());
        let mut last = 0;

        for captures in WIKIDOT_WIKIPEDIA_LINK_REGEX.captures_iter(&source) {
            let Some(link_match) = captures.get(0) else {
                continue;
            };

            output.push_str(&source[last..link_match.start()]);
            last = link_match.end();

            let Some(target) = captures.name("target").map(|matched| matched.as_str())
            else {
                output.push_str(link_match.as_str());
                continue;
            };

            if Self::is_inside_wikidot_literal_region(&source, link_match.start()) {
                output.push_str(link_match.as_str());
                continue;
            }

            let label = captures.name("label").map(|matched| matched.as_str());
            let link = build_wikidot_wikipedia_link(target, label);
            let marker =
                format!("{WIKIDOT_WIKIPEDIA_LINK_SENTINEL_PREFIX}{}X", links.len());
            links.push(link);
            output.push_str(&marker);
        }

        if links.is_empty() {
            return links;
        }

        output.push_str(&source[last..]);
        *wikitext = output;
        links
    }

    fn protect_wikidot_compat_links(
        wikitext: &mut String,
        settings: &WikitextSettings,
    ) -> Vec<ProtectedWikidotCompatLink> {
        if !settings.enable_page_syntax {
            return Vec::new();
        }

        let mut links = Vec::new();
        Self::protect_wikidot_anchor_markers(wikitext, &mut links);
        Self::protect_wikidot_current_page_links(wikitext, &mut links);
        Self::protect_wikidot_star_local_links(wikitext, &mut links);
        links
    }

    fn protect_wikidot_anchor_markers(
        wikitext: &mut String,
        links: &mut Vec<ProtectedWikidotCompatLink>,
    ) {
        let source = wikitext.clone();
        let mut output = String::with_capacity(source.len());
        let mut last = 0;

        for captures in WIKIDOT_ANCHOR_MARKER_REGEX.captures_iter(&source) {
            let Some(marker_match) = captures.get(0) else {
                continue;
            };

            output.push_str(&source[last..marker_match.start()]);
            last = marker_match.end();

            if Self::is_inside_wikidot_literal_region(&source, marker_match.start()) {
                output.push_str(marker_match.as_str());
                continue;
            }

            let Some(name) = captures.name("name").map(|matched| matched.as_str().trim())
            else {
                output.push_str(marker_match.as_str());
                continue;
            };
            if name.is_empty() {
                output.push_str(marker_match.as_str());
                continue;
            }

            let marker = wikidot_compat_link_marker();
            links.push(ProtectedWikidotCompatLink {
                anchor: wikidot_named_anchor(name),
                marker: marker.clone(),
            });
            output.push_str(&marker);
        }

        if last == 0 {
            return;
        }

        output.push_str(&source[last..]);
        *wikitext = output;
    }

    fn protect_wikidot_current_page_links(
        wikitext: &mut String,
        links: &mut Vec<ProtectedWikidotCompatLink>,
    ) {
        let source = wikitext.clone();
        let mut output = String::with_capacity(source.len());
        let mut last = 0;

        for captures in WIKIDOT_CURRENT_PAGE_LINK_REGEX.captures_iter(&source) {
            let Some(link_match) = captures.get(0) else {
                continue;
            };

            output.push_str(&source[last..link_match.start()]);
            last = link_match.end();

            if source[..link_match.start()].ends_with('[')
                || source[link_match.end()..].starts_with(']')
            {
                output.push_str(link_match.as_str());
                continue;
            }

            if Self::is_inside_wikidot_literal_region(&source, link_match.start()) {
                output.push_str(link_match.as_str());
                continue;
            }

            let Some(label) = captures
                .name("label")
                .map(|matched| matched.as_str().trim())
            else {
                output.push_str(link_match.as_str());
                continue;
            };
            if label.is_empty() {
                output.push_str(link_match.as_str());
                continue;
            }

            let marker = wikidot_compat_link_marker();
            links.push(ProtectedWikidotCompatLink {
                anchor: wikidot_current_page_anchor(label),
                marker: marker.clone(),
            });
            output.push_str(&marker);
        }

        if last == 0 {
            return;
        }

        output.push_str(&source[last..]);
        *wikitext = output;
    }

    fn protect_wikidot_star_local_links(
        wikitext: &mut String,
        links: &mut Vec<ProtectedWikidotCompatLink>,
    ) {
        let source = wikitext.clone();
        let mut output = String::with_capacity(source.len());
        let mut last = 0;

        for captures in WIKIDOT_STAR_LOCAL_LINK_REGEX.captures_iter(&source) {
            let Some(link_match) = captures.get(0) else {
                continue;
            };

            output.push_str(&source[last..link_match.start()]);
            last = link_match.end();

            if Self::is_inside_wikidot_literal_region(&source, link_match.start()) {
                output.push_str(link_match.as_str());
                continue;
            }

            let Some(target) = captures.name("target").map(|matched| matched.as_str())
            else {
                output.push_str(link_match.as_str());
                continue;
            };
            let Some(label) = captures
                .name("label")
                .map(|matched| matched.as_str().trim())
            else {
                output.push_str(link_match.as_str());
                continue;
            };
            if label.is_empty() {
                output.push_str(link_match.as_str());
                continue;
            }

            let marker = wikidot_compat_link_marker();
            links.push(ProtectedWikidotCompatLink {
                anchor: wikidot_star_local_anchor(target, label),
                marker: marker.clone(),
            });
            output.push_str(&marker);
        }

        if last == 0 {
            return;
        }

        output.push_str(&source[last..]);
        *wikitext = output;
    }

    fn restore_protected_wikidot_compat_links(
        mut html: String,
        links: &[ProtectedWikidotCompatLink],
    ) -> String {
        for link in links {
            html = Self::replace_html_text_marker(&html, &link.marker, &link.anchor);
        }
        html
    }

    fn replace_html_text_marker(html: &str, marker: &str, replacement: &str) -> String {
        if marker.is_empty() || !html.contains(marker) {
            return html.to_owned();
        }

        let mut output = String::with_capacity(html.len());
        let mut last = 0;
        let mut index = 0;
        let mut in_tag = false;
        let bytes = html.as_bytes();
        let marker_bytes = marker.as_bytes();

        while index < bytes.len() {
            match bytes[index] {
                b'<' => in_tag = true,
                b'>' if in_tag => in_tag = false,
                _ if !in_tag && bytes[index..].starts_with(marker_bytes) => {
                    output.push_str(&html[last..index]);
                    output.push_str(replacement);
                    index += marker.len();
                    last = index;
                    continue;
                }
                _ => {}
            }
            index += 1;
        }

        if last == 0 {
            return html.to_owned();
        }

        output.push_str(&html[last..]);
        output
    }

    fn restore_protected_wikidot_wikipedia_links(
        mut html: String,
        links: &[ProtectedWikidotWikipediaLink],
    ) -> String {
        for (index, link) in links.iter().enumerate() {
            let marker = format!("{WIKIDOT_WIKIPEDIA_LINK_SENTINEL_PREFIX}{index}X");
            html = html.replace(&marker, &link.anchor);
        }
        html
    }

    fn record_protected_wikidot_wikipedia_backlinks(
        backlinks: &mut ftml::data::Backlinks<'_>,
        links: &[ProtectedWikidotWikipediaLink],
    ) {
        backlinks
            .external_links
            .extend(links.iter().map(|link| Cow::Owned(link.href.clone())));
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
                let href_email = decode_wikidot_email_html_entities(href_email);
                let visible_email =
                    decode_wikidot_email_html_entities(visible_email);

                if href_email != visible_email {
                    return captures.get(0).map_or("", |m| m.as_str()).to_string();
                }

                let (email, trailing) =
                    Self::split_trailing_email_punctuation(&visible_email);
                if Self::wikidot_obfuscated_email(email).is_none() {
                    return captures.get(0).map_or("", |m| m.as_str()).to_string();
                }

                format!(
                    r#"<span class="wiki-email" style="visibility: visible;"><a href="mailto:{email_attr}">{email_text}</a></span>{trailing}"#,
                    email_attr = escape_list_pages_html_attr(email),
                    email_text = escape_list_pages_html_text(email),
                )
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
        if !site_accepts_wikidot_local_asset_slug(current_site, &site_slug)
            && !site_accepts_cross_site_wdfiles_local_file(current_site, host, path)
        {
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
        page_info: &PageInfo<'_>,
        current_site_slug: &str,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        options: IncludeExpansionOptions,
    ) -> Result<IncludeExpansion> {
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

        let mut expansion = Self::expand_includes_for_site(
            ctx,
            wikitext,
            IncludeExpansionContext {
                current_site_id,
                current_site_slug: current_site_slug.to_owned(),
                page_info,
                settings,
                expand_wikidot_image_blocks: options.expand_wikidot_image_blocks,
                max_total_includes: options.max_total_includes,
            },
            0,
            options.max_total_includes,
        )
        .await?;
        unprotect_include_variables(&mut expansion.wikitext);

        Ok(expansion)
    }

    fn expand_includes_for_site<'a>(
        ctx: &'a ServiceContext<'_>,
        wikitext: String,
        expansion_context: IncludeExpansionContext<'a>,
        depth: usize,
        mut remaining_includes: usize,
    ) -> Pin<Box<dyn Future<Output = Result<IncludeExpansion>> + Send + 'a>> {
        Box::pin(async move {
            let mut wikitext = wikitext;
            Self::normalize_wikidot_ta_badge_multiline_includes(&mut wikitext);
            Self::prepare_wikidot_conditionals_for_include_expansion(
                &mut wikitext,
                expansion_context.page_info,
            );
            Self::mask_wikidot_comment_include_markers(&mut wikitext);
            let image_block_included_pages = if expansion_context
                .expand_wikidot_image_blocks
                && expansion_context.current_site_slug
                    == expansion_context.page_info.site.as_ref()
            {
                Self::expand_wikidot_image_block_includes(
                    &mut wikitext,
                    expansion_context.page_info,
                )
            } else {
                Vec::new()
            };

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
                Self::unmask_wikidot_comment_include_markers(&mut wikitext);
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

                let source = Self::fetch_include_source(
                    ctx,
                    expansion_context.current_site_id,
                    &expansion_context.current_site_slug,
                    include.page_ref(),
                )
                .await?;

                let Some(mut source) = source else {
                    fetched_pages.push(None);
                    nested_included_pages.push(Vec::new());
                    nested_include_counts.push(0);
                    continue;
                };

                apply_include_variables(&mut source.wikitext, include);
                Self::remove_wikidot_component_iftags_documentation(&mut source.wikitext);
                Self::remove_unresolved_variable_iftags_blocks(&mut source.wikitext);

                let expansion = Self::expand_includes_for_site(
                    ctx,
                    source.wikitext,
                    IncludeExpansionContext {
                        current_site_id: source.site_id,
                        current_site_slug: source.site_slug,
                        page_info: expansion_context.page_info,
                        settings: expansion_context.settings,
                        expand_wikidot_image_blocks: expansion_context
                            .expand_wikidot_image_blocks,
                        max_total_includes: expansion_context.max_total_includes,
                    },
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

                fetched_pages.push(Some(expansion.wikitext));
                nested_included_pages.push(expansion.included_pages);
            }

            let (mut expanded, direct_included_pages) = ftml::include(
                &wikitext,
                expansion_context.settings,
                PreparedIncluder {
                    pages: fetched_pages,
                },
                include_error,
            )?;

            Self::unmask_wikidot_comment_include_markers(&mut expanded);
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

                if current_site_matches
                    && let Some(source) = Self::fetch_include_source_from_site(
                        ctx,
                        current_site_id,
                        current_site_slug,
                        page_ref.page(),
                    )
                    .await?
                {
                    return Ok(Some(source));
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
                    // The include names a distinct site that is not available
                    // locally. Falling back to the current site can select the
                    // including page itself when both slugs match and recurse
                    // until the depth limit. Preserve missing-include behavior.
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
        let page_ref = Reference::from(page_slug);
        let Some(page) =
            PageService::get_optional(ctx, site_id, page_ref.clone()).await?
        else {
            return Ok(None);
        };

        let can_view = PermissionService::check_user_can(
            ctx,
            &CheckPermissionContext {
                user_id: None,
                site_id,
                page_reference: Some(page_ref),
            },
            Permission {
                resource_type: Resource::Page,
                resource_category: Some(Reference::Id(page.page_category_id)),
                action: Action::View,
            },
        )
        .await?;
        if !can_view {
            return Ok(None);
        }

        if let Some(wikitext) = PageRevisionService::get_wikitext_optional(
            ctx,
            site_id,
            Reference::Id(page.page_id),
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
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        current_page_id: Option<i64>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<IncludeExpansion> {
        let (Some(current_site_id), Some(current_page_id)) =
            (current_site_id, current_page_id)
        else {
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

        let mut expanded = String::with_capacity(wikitext.len());
        let mut included_pages = Vec::new();
        let mut cursor = 0;

        for captures in LISTPAGES_MODULE_REGEX.captures_iter(&wikitext) {
            let mtch = captures.get(0).unwrap();
            expanded.push_str(&wikitext[cursor..mtch.start()]);
            let head = captures.name("head").unwrap().as_str();
            let body = captures.name("body").unwrap().as_str();

            if list_pages_has_unsupported_parent_selector(head)
                || list_pages_has_unsupported_page_type_selector(head)
            {
                expanded.push_str(mtch.as_str());
                cursor = mtch.end();
                continue;
            }

            let Some(arguments) = parse_list_pages_arguments(head) else {
                expanded
                    .push_str(&unsupported_list_pages_replacement(mtch.as_str(), body));
                cursor = mtch.end();
                continue;
            };

            if !list_pages_body_variables_supported(body) {
                expanded
                    .push_str(&unsupported_list_pages_replacement(mtch.as_str(), body));
                cursor = mtch.end();
                continue;
            }

            let IncludeExpansion {
                wikitext: replacement,
                included_pages: replacement_included_pages,
                ..
            } = Self::render_list_pages_block(
                ctx,
                current_site_id,
                current_page_id,
                page_info,
                settings,
                arguments,
                body,
            )
            .await?;
            expanded.push_str(&register_generated_list_pages_html(
                replacement,
                compat_html,
            ));
            included_pages.extend(replacement_included_pages);
            cursor = mtch.end();
        }

        expanded.push_str(&wikitext[cursor..]);
        Ok(IncludeExpansion {
            wikitext: expanded,
            included_pages,
            expanded_include_count: 0,
        })
    }

    async fn expand_count_pages(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
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

        for captures in COUNTPAGES_MODULE_REGEX.captures_iter(&wikitext) {
            let mtch = captures.get(0).unwrap();
            expanded.push_str(&wikitext[cursor..mtch.start()]);
            if Self::is_inside_wikidot_literal_region(&wikitext, mtch.start()) {
                expanded.push_str(mtch.as_str());
                cursor = mtch.end();
                continue;
            }
            let head = captures.name("head").unwrap().as_str();
            let body = captures.name("body").unwrap().as_str();

            if list_pages_has_unsupported_parent_selector(head)
                || list_pages_has_unsupported_page_type_selector(head)
            {
                expanded.push_str(mtch.as_str());
                cursor = mtch.end();
                continue;
            }

            let Some(arguments) = parse_list_pages_arguments(head) else {
                expanded.push_str(mtch.as_str());
                cursor = mtch.end();
                continue;
            };
            if count_pages_should_remain_literal(&arguments) {
                expanded.push_str(mtch.as_str());
                cursor = mtch.end();
                continue;
            }

            let replacement = Self::render_count_pages_block(
                ctx,
                current_site_id,
                current_page_id,
                page_info,
                arguments,
                body,
                mtch.as_str(),
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

        let replacement =
            render_read_only_rate_module(page_info.score, &page_info.language);
        RATE_MODULE_REGEX
            .replace_all(&wikitext, replacement.as_str())
            .into_owned()
    }

    fn expand_members_modules_with_registry(
        wikitext: String,
        settings: &WikitextSettings,
        compat_html: &mut CompatHtmlFragments,
    ) -> String {
        if !settings.enable_page_syntax {
            return wikitext;
        }

        MEMBERS_MODULE_REGEX
            .replace_all(&wikitext, |captures: &regex::Captures<'_>| {
                let head = captures.name("head").map_or("", |mtch| mtch.as_str());
                let group = wikidot_module_argument(head, "group")
                    .unwrap_or("members")
                    .trim();
                compat_html.push(render_members_module_placeholder(group))
            })
            .into_owned()
    }

    fn expand_new_page_modules_with_registry(
        wikitext: String,
        settings: &WikitextSettings,
        compat_html: &mut CompatHtmlFragments,
    ) -> String {
        if !settings.enable_page_syntax {
            return wikitext;
        }

        NEWPAGE_MODULE_REGEX
            .replace_all(&wikitext, |captures: &regex::Captures<'_>| {
                let head = captures.name("head").map_or("", |mtch| mtch.as_str());
                compat_html.push(render_new_page_module(head))
            })
            .into_owned()
    }

    fn expand_clone_modules_with_registry(
        wikitext: String,
        settings: &WikitextSettings,
        compat_html: &mut CompatHtmlFragments,
    ) -> String {
        if !settings.enable_page_syntax {
            return wikitext;
        }

        CLONE_MODULE_REGEX
            .replace_all(&wikitext, |captures: &regex::Captures<'_>| {
                let head = captures.name("head").map_or("", |mtch| mtch.as_str());
                compat_html.push(render_clone_module(head))
            })
            .into_owned()
    }

    #[cfg(test)]
    fn expand_members_modules(wikitext: String, settings: &WikitextSettings) -> String {
        let mut fragments = CompatHtmlFragments::new(&wikitext);
        let protected = Self::expand_members_modules_with_registry(
            wikitext,
            settings,
            &mut fragments,
        );
        fragments.restore(&protected)
    }

    #[cfg(test)]
    fn expand_new_page_modules(wikitext: String, settings: &WikitextSettings) -> String {
        let mut fragments = CompatHtmlFragments::new(&wikitext);
        let protected = Self::expand_new_page_modules_with_registry(
            wikitext,
            settings,
            &mut fragments,
        );
        fragments.restore(&protected)
    }

    #[cfg(test)]
    fn expand_clone_modules(wikitext: String, settings: &WikitextSettings) -> String {
        let mut fragments = CompatHtmlFragments::new(&wikitext);
        let protected =
            Self::expand_clone_modules_with_registry(wikitext, settings, &mut fragments);
        fragments.restore(&protected)
    }

    async fn expand_tag_cloud_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        current_site_id: Option<i64>,
        current_page_id: Option<i64>,
    ) -> Result<String> {
        if !TAGCLOUD_MODULE_REGEX.is_match(&wikitext) {
            return Ok(wikitext);
        }

        let (Some(current_site_id), Some(current_page_id)) =
            (current_site_id, current_page_id)
        else {
            return Ok(wikitext);
        };

        let current_branch_tag = page_info
            .tags
            .iter()
            .find(|tag| tag.starts_with("branch-"))
            .map(Cow::as_ref);
        let tags = Self::load_tag_cloud_counts(
            ctx,
            current_site_id,
            current_page_id,
            current_branch_tag,
        )
        .await?;
        let replacement = render_tag_cloud_box(&tags);
        Ok(TAGCLOUD_MODULE_REGEX
            .replace_all(&wikitext, replacement.as_str())
            .into_owned())
    }

    async fn expand_backlinks_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        current_page_id: Option<i64>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax || !BACKLINKS_MODULE_REGEX.is_match(&wikitext) {
            return Ok(wikitext);
        }

        let (Some(current_site_id), Some(current_page_id)) =
            (current_site_id, current_page_id)
        else {
            return Ok(wikitext);
        };

        let mut expanded = String::with_capacity(wikitext.len());
        let mut cursor = 0;

        for captures in BACKLINKS_MODULE_REGEX.captures_iter(&wikitext) {
            let mtch = captures.get(0).unwrap();
            expanded.push_str(&wikitext[cursor..mtch.start()]);

            if Self::is_inside_wikidot_literal_region(&wikitext, mtch.start()) {
                expanded.push_str(mtch.as_str());
                cursor = mtch.end();
                continue;
            }

            let head = captures.name("head").map_or("", |mtch| mtch.as_str());
            if !head.trim().is_empty() {
                expanded.push_str(mtch.as_str());
                cursor = mtch.end();
                continue;
            }

            let pages =
                Self::load_backlinks_module_pages(ctx, current_site_id, current_page_id)
                    .await?;
            expanded.push_str(&compat_html.push(render_backlinks_module_box(&pages)));
            cursor = mtch.end();
        }

        expanded.push_str(&wikitext[cursor..]);
        Ok(expanded)
    }

    async fn load_backlinks_module_pages(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        current_page_id: i64,
    ) -> Result<Vec<BacklinksModulePage>> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to load Backlinks module rows for page ID {} in site ID {}",
                    current_page_id, current_site_id,
                ),
                ErrorType::Render,
            )
        };
        let txn = ctx.transaction();
        let statement = Statement::from_string(
            txn.get_database_backend(),
            format!(
                "SELECT p.page_id, p.page_category_id, p.slug, pr.title \
                 FROM page_connection pc \
                 JOIN page p ON p.page_id = pc.from_page_id \
                 JOIN page_revision pr ON pr.revision_id = p.latest_revision_id \
                 WHERE pc.to_page_id = {current_page_id} \
                   AND pc.connection_type = 'link' \
                   AND p.site_id = {current_site_id} \
                   AND p.deleted_at IS NULL \
                 ORDER BY lower(pr.title), p.slug \
                 LIMIT {MAX_BACKLINKS_MODULE_ROWS}",
            ),
        );

        let rows = BacklinksModulePage::find_by_statement(statement)
            .all(txn)
            .await
            .or_raise(make_error)?;

        let mut viewable = Vec::with_capacity(rows.len());
        for row in rows {
            let anonymously_viewable = PermissionService::check_user_can(
                ctx,
                &CheckPermissionContext {
                    user_id: None,
                    site_id: current_site_id,
                    page_reference: Some(Reference::Id(row.page_id)),
                },
                Permission {
                    resource_type: Resource::Page,
                    resource_category: Some(Reference::Id(row.page_category_id)),
                    action: Action::View,
                },
            )
            .await
            .or_raise(make_error)?;

            if anonymously_viewable {
                viewable.push(row);
            }
        }

        Ok(viewable)
    }

    async fn load_tag_cloud_counts(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        _current_page_id: i64,
        current_branch_tag: Option<&str>,
    ) -> Result<Vec<(String, usize)>> {
        let make_error =
            || Error::new("failed to render TagCloud module", ErrorType::Render);
        let txn = ctx.transaction();
        let pages = Page::find()
            .filter(page::Column::SiteId.eq(current_site_id))
            .filter(page::Column::DeletedAt.is_null())
            .all(txn)
            .await
            .or_raise(make_error)?;
        let revision_ids = pages
            .iter()
            .filter_map(|page| page.latest_revision_id)
            .collect::<Vec<_>>();
        if revision_ids.is_empty() {
            return Ok(Vec::new());
        }

        let revisions = page_revision::Entity::find()
            .filter(page_revision::Column::RevisionId.is_in(revision_ids))
            .all(txn)
            .await
            .or_raise(make_error)?;
        let mut counts = BTreeMap::<String, usize>::new();
        for revision in revisions {
            if let Some(branch_tag) = current_branch_tag
                && !revision.tags.iter().any(|tag| tag == branch_tag)
            {
                continue;
            }
            for tag in revision.tags {
                if !is_tag_cloud_visible_tag(&tag) {
                    continue;
                }
                *counts.entry(tag).or_default() += 1;
            }
        }

        Ok(counts.into_iter().collect())
    }

    fn protect_wikidot_css_modules(
        wikitext: &mut String,
        settings: &WikitextSettings,
        compat_html: &mut CompatHtmlFragments,
    ) {
        if !settings.enable_page_syntax {
            return;
        }

        let source = wikitext.as_str();
        let literal_regions = LiteralRegionIndex::new(source);
        let syntax_literal_regions = LiteralRegionIndex::new_wikidot_syntax(source);
        let mut output = String::with_capacity(source.len());
        let mut cursor = 0;

        while let Some(open) = CSS_MODULE_OPEN_REGEX.find_at(source, cursor) {
            if literal_regions.contains(open.start()) {
                output.push_str(&source[cursor..open.end()]);
                cursor = open.end();
                continue;
            }
            let mut close_cursor = open.end();
            let close = loop {
                let Some(candidate) = MODULE_CLOSE_REGEX.find_at(source, close_cursor)
                else {
                    output.push_str(&source[cursor..]);
                    *wikitext = output;
                    return;
                };
                if !syntax_literal_regions.contains(candidate.start()) {
                    break candidate;
                }
                close_cursor = candidate.end();
            };
            let body = source[open.end()..close.start()].trim_matches('\n');
            let body = Self::escape_wikidot_css_module_body(body);
            output.push_str(&source[cursor..open.start()]);
            output.push_str(&compat_html.push(format!("<style>\n{body}\n</style>")));
            cursor = close.end();
        }
        output.push_str(&source[cursor..]);
        *wikitext = output;
    }

    fn escape_wikidot_css_module_body(body: &str) -> String {
        body.replace('<', r"\3C ")
    }

    fn neutralize_authored_wikidot_compat_markers(wikitext: &mut String) {
        let source = wikitext.clone();
        let literal_regions = LiteralRegionIndex::new(&source);
        let mut replacements: Vec<(Range<usize>, String)> = Vec::new();

        for candidate in AUTHORED_WIKIDOT_COMPAT_OPEN_TAG_REGEX.find_iter(&source) {
            if literal_regions.contains(candidate.start()) {
                continue;
            }
            for captures in
                AUTHORED_WIKIDOT_COMPAT_MARKER_REGEX.captures_iter(candidate.as_str())
            {
                let full_match = captures.get(0).expect("compat marker capture exists");
                let start = candidate.start() + full_match.start();
                replacements.push((
                    start..candidate.start() + full_match.end(),
                    format!(
                        "data-wikijump-authored-compat-{}",
                        captures["kind"].to_ascii_lowercase(),
                    ),
                ));
            }
        }

        for (range, replacement) in replacements.into_iter().rev() {
            wikitext.replace_range(range, &replacement);
        }
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
    ) -> Vec<ProtectedWikidotColorSpan> {
        if !settings.enable_page_syntax {
            return Vec::new();
        }

        let mut spans = Vec::new();
        let protected = WIKIDOT_COLOR_SPAN_REGEX
            .replace_all(wikitext, |captures: &regex::Captures<'_>| {
                let Some(color) = parse_wikidot_compat_color_descriptor(
                    &captures["hashes"],
                    &captures["color"],
                ) else {
                    return captures[0].to_owned();
                };
                let marker = wikidot_color_span_marker();
                spans.push(ProtectedWikidotColorSpan {
                    marker: marker.clone(),
                    html: render_wikidot_color_span_html(&color, &captures["body"]),
                });
                marker
            })
            .into_owned();
        *wikitext = protected;
        spans
    }

    fn protect_wikidot_inline_html_spans(
        wikitext: &mut String,
        settings: &WikitextSettings,
    ) -> Vec<ProtectedWikidotInlineHtml> {
        if !settings.enable_page_syntax {
            return Vec::new();
        }

        let mut spans = Vec::new();
        let protected = WIKIDOT_ESCAPED_NBSP_REGEX
            .replace_all(wikitext, |captures: &regex::Captures<'_>| {
                let marker = wikidot_inline_html_marker();
                spans.push(ProtectedWikidotInlineHtml {
                    marker: marker.clone(),
                    html: captures["html"].to_owned(),
                });
                marker
            })
            .into_owned();
        let protected = WIKIDOT_BOLD_OUTER_COLOR_SPAN_REGEX
            .replace_all(&protected, |captures: &regex::Captures<'_>| {
                if captures["body"].contains("##") {
                    return captures[0].to_owned();
                }
                let Some(color) = parse_wikidot_compat_color_descriptor(
                    &captures["hashes"],
                    &captures["color"],
                ) else {
                    return captures[0].to_owned();
                };
                let marker = wikidot_inline_html_marker();
                spans.push(ProtectedWikidotInlineHtml {
                    marker: marker.clone(),
                    html: format!(
                        "<strong>{}</strong>",
                        render_wikidot_color_span_html(&color, &captures["body"]),
                    ),
                });
                marker
            })
            .into_owned();
        let protected = WIKIDOT_BOLD_COLOR_SPAN_REGEX
            .replace_all(&protected, |captures: &regex::Captures<'_>| {
                if captures["body"].contains("##") {
                    return captures[0].to_owned();
                }
                let Some(color) = parse_wikidot_compat_color_descriptor(
                    &captures["hashes"],
                    &captures["color"],
                ) else {
                    return captures[0].to_owned();
                };
                let marker = wikidot_inline_html_marker();
                spans.push(ProtectedWikidotInlineHtml {
                    marker: marker.clone(),
                    html: format!(
                        "<strong>{}</strong>",
                        render_wikidot_color_span_html(&color, &captures["body"]),
                    ),
                });
                marker
            })
            .into_owned();
        let protected = WIKIDOT_BOLD_UNDERLINE_SPAN_REGEX
            .replace_all(&protected, |captures: &regex::Captures<'_>| {
                let marker = wikidot_inline_html_marker();
                spans.push(ProtectedWikidotInlineHtml {
                    marker: marker.clone(),
                    html: format!(
                        "<strong><u>{}</u></strong>",
                        render_wikidot_protected_inline_body_html(&captures["body"]),
                    ),
                });
                marker
            })
            .into_owned();
        *wikitext = protected;
        spans
    }

    fn restore_protected_wikidot_color_spans(
        mut html: String,
        spans: &[ProtectedWikidotColorSpan],
    ) -> String {
        for span in spans {
            html = html.replace(&span.marker, &span.html);
        }
        html
    }

    fn restore_protected_wikidot_inline_html(
        mut html: String,
        spans: &[ProtectedWikidotInlineHtml],
    ) -> String {
        for span in spans {
            html = html.replace(&span.marker, &span.html);
        }
        html
    }

    fn escape_unrendered_wikidot_color_markers(
        wikitext: String,
        settings: &WikitextSettings,
    ) -> String {
        if !settings.enable_page_syntax {
            return wikitext;
        }

        wikitext.replace("##", "&#35;&#35;")
    }

    fn render_long_native_list_runs_with_registry(
        wikitext: String,
        compat_html: &mut CompatHtmlFragments,
    ) -> String {
        let lines = wikitext.split_inclusive('\n').collect::<Vec<_>>();
        let mut output = String::with_capacity(wikitext.len());
        let mut index = 0;

        while index < lines.len() {
            let mut end = index;
            while end < lines.len() && native_bullet_list_item(lines[end]).is_some() {
                end += 1;
            }

            if end - index >= LONG_NATIVE_LIST_RENDER_MIN_ITEMS {
                output.push_str(
                    &compat_html.push(render_native_bullet_list(&lines[index..end])),
                );
                index = end;
            } else {
                output.push_str(lines[index]);
                index += 1;
            }
        }

        output
    }

    #[cfg(test)]
    fn render_long_native_list_runs(wikitext: String) -> String {
        let mut fragments = CompatHtmlFragments::new(&wikitext);
        let protected =
            Self::render_long_native_list_runs_with_registry(wikitext, &mut fragments);
        fragments.restore(&protected)
    }

    fn should_use_wikidot_compatibility_fallback(
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

    fn ftml_compat_render_timeout(config: &Config, wikitext: &str) -> Duration {
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

    fn wikidot_compat_parse_complexity_score(wikitext: &str) -> usize {
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

    async fn load_wikidot_compat_fallback_link_titles(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        wikitext: &str,
    ) -> Result<WikidotCompatLinkTitleMap> {
        let slugs = collect_wikidot_compat_empty_label_link_slugs(wikitext);
        if slugs.is_empty() {
            return Ok(WikidotCompatLinkTitleMap::new());
        }

        let mut titles = WikidotCompatLinkTitleMap::new();
        for slug in slugs {
            let Some(page) =
                PageService::get_optional(ctx, site_id, Reference::from(slug.as_str()))
                    .await?
            else {
                continue;
            };

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
            if !can_view {
                continue;
            }

            let Some(revision_id) = page.latest_revision_id else {
                continue;
            };
            let revision = PageRevisionService::get_direct(ctx, revision_id).await?;
            let title = revision.title.trim();
            if !title.is_empty() {
                titles.insert(slug, title.to_owned());
            }
        }

        Ok(titles)
    }

    fn render_oversized_wikidot_compatibility_fallback(
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
    fn render_wikidot_compatibility_fallback_with_code_blocks(wikitext: &str) -> String {
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

    fn render_wikidot_compatibility_fallback_with_code_blocks_for_context(
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

    fn render_wikidot_compatibility_fallback_output_for_context(
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
        let mut code_chunk = String::new();
        let mut in_code = false;
        let mut collapsible_depth = 0usize;
        let mut code_blocks = 0;
        let mut collapsible_blocks = 0;

        for line in wikitext.lines() {
            let trimmed = line.trim_start();
            let marker = trimmed.to_ascii_lowercase();
            if !in_code && marker.starts_with("[[code") {
                Self::push_wikidot_compat_fallback_text_chunk_for_page(
                    &mut body,
                    &mut text_chunk,
                    current_page,
                    local_file_site_slug,
                    link_titles,
                    &mut html_block_texts,
                );
                in_code = true;
                code_chunk.clear();
                code_blocks += 1;
                continue;
            }

            if in_code && marker.starts_with("[[/code]]") {
                body.push_str(r#"<div class="code"><pre><code>"#);
                push_escaped_html(&mut body, code_chunk.trim_end_matches('\n'));
                body.push_str("</code></pre></div>");
                in_code = false;
                code_chunk.clear();
                continue;
            }

            if !in_code && marker.starts_with("[[collapsible") {
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

            if !in_code && marker.starts_with("[[/collapsible]]") {
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

            if in_code {
                code_chunk.push_str(line);
                code_chunk.push('\n');
            } else {
                text_chunk.push_str(line);
                text_chunk.push('\n');
            }
        }

        if in_code {
            text_chunk.push_str("[[code]]\n");
            text_chunk.push_str(&code_chunk);
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

        if code_blocks == 0 && collapsible_blocks == 0 {
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

    fn wikidot_residual_div_attributes(marker: &str) -> Option<String> {
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

    fn render_wikidot_compat_fallback_inline_markup(
        value: &str,
        link_titles: Option<&WikidotCompatLinkTitleMap>,
    ) -> String {
        let mut output = String::with_capacity(value.len());
        let mut rest = value;

        while let Some(marker) = Self::next_wikidot_compat_inline_marker(rest) {
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

    fn next_wikidot_compat_inline_marker(
        value: &str,
    ) -> Option<WikidotCompatInlineMarker> {
        let color = Self::find_wikidot_compat_color_marker(value);
        let italic = Self::find_wikidot_compat_delimited_marker(
            value,
            "//",
            WikidotCompatInlineMarkerKind::Italic,
        );
        let underline = Self::find_wikidot_compat_delimited_marker(
            value,
            "__",
            WikidotCompatInlineMarkerKind::Underline,
        );

        [color, italic, underline]
            .into_iter()
            .flatten()
            .min_by_key(|marker| marker.start)
    }

    fn find_wikidot_compat_color_marker(
        value: &str,
    ) -> Option<WikidotCompatInlineMarker> {
        let mut offset = 0;
        while let Some(relative_start) = value[offset..].find("##") {
            let start = offset + relative_start;
            let marker_start = &value[start + 2..];
            let Some(pipe_relative) = marker_start.find('|') else {
                offset = start + 2;
                continue;
            };
            let color = marker_start[..pipe_relative].trim();
            if !Self::wikidot_compat_valid_color_value(color) {
                offset = start + 2;
                continue;
            }
            let content_start = start + 2 + pipe_relative + 1;
            let Some(end_relative) = value[content_start..].find("##") else {
                offset = start + 2;
                continue;
            };
            return Some(WikidotCompatInlineMarker {
                start,
                end: content_start + end_relative + 2,
                kind: WikidotCompatInlineMarkerKind::Color,
            });
        }

        None
    }

    fn find_wikidot_compat_delimited_marker(
        value: &str,
        delimiter: &str,
        kind: WikidotCompatInlineMarkerKind,
    ) -> Option<WikidotCompatInlineMarker> {
        let mut offset = 0;
        while let Some(relative_start) = value[offset..].find(delimiter) {
            let start = offset + relative_start;
            if delimiter == "//" && value[..start].ends_with(':') {
                offset = start + delimiter.len();
                continue;
            }
            let content_start = start + delimiter.len();
            let end_relative = value[content_start..].find(delimiter)?;
            if end_relative == 0 {
                offset = content_start + delimiter.len();
                continue;
            }
            return Some(WikidotCompatInlineMarker {
                start,
                end: content_start + end_relative + delimiter.len(),
                kind,
            });
        }

        None
    }

    fn wikidot_compat_valid_color_value(value: &str) -> bool {
        !value.is_empty()
            && value.len() <= 32
            && value
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '#')
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

    fn categories_with_current_page_category(
        mut categories: Vec<Cow<'static, str>>,
        page_info: &PageInfo<'_>,
    ) -> Vec<Cow<'static, str>> {
        let category = page_info
            .category
            .as_ref()
            .map(Cow::as_ref)
            .unwrap_or("_default");
        if !categories.iter().any(|slug| slug.as_ref() == category) {
            categories.push(Cow::Owned(category.to_owned()));
        }
        categories
    }

    fn page_info_category_slug<'a>(page_info: &'a PageInfo<'_>) -> Cow<'a, str> {
        page_info
            .category
            .as_ref()
            .map(|category| Cow::Borrowed(category.as_ref()))
            .unwrap_or(Cow::Borrowed("_default"))
    }

    fn page_info_full_slug(page_info: &PageInfo<'_>) -> String {
        let page = page_info.page.as_ref();
        match Self::page_info_category_slug(page_info).as_ref() {
            "_default" => page.to_owned(),
            category => format!("{category}:{page}"),
        }
    }

    async fn render_list_pages_block(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        current_page_id: i64,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        arguments: ListPagesArguments,
        body: &str,
    ) -> Result<IncludeExpansion> {
        let ListPagesArguments {
            current_page_only,
            category_selector_present,
            category_all,
            include_current_category,
            categories,
            excluded_categories,
            mut any_tags,
            all_tags,
            default_tags,
            no_tags,
            authors,
            order,
            limit,
            count_pages_explicit_limit: _,
            count_pages_per_page,
            offset,
            exclude_current_page,
            page_type,
            page_parent,
            slug,
            data_form_fields,
            prepend_line,
            unsupported_count_pages_filter: _,
        } = arguments;
        any_tags.extend(default_tags);
        let (category_all, include_current_category) = if category_selector_present {
            (category_all, include_current_category)
        } else {
            (false, true)
        };
        let categories = if include_current_category && !category_all {
            Self::categories_with_current_page_category(categories, page_info)
        } else {
            categories
        };
        let requested_limit = count_pages_per_page
            .or(limit)
            .unwrap_or(DEFAULT_LISTPAGES_RENDER_LIMIT)
            .min(MAX_LISTPAGES_RENDER_LIMIT)
            .min(limit.unwrap_or(u64::MAX));
        let query_limit = limit
            .unwrap_or(u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS))
            .saturating_add(u64::from(offset))
            .saturating_add(u64::from(exclude_current_page))
            .min(u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS));
        let included_categories = if category_all {
            IncludedCategories::All
        } else {
            IncludedCategories::List(&categories)
        };

        let wants_created_by = list_pages_body_uses_variable(body, "created_by")
            || list_pages_body_uses_variable(body, "createdby")
            || list_pages_body_uses_variable(body, "created_by_linked")
            || list_pages_body_uses_variable(body, "createdbylinked")
            || list_pages_body_uses_variable(body, "author");
        let wants_created_at = list_pages_body_uses_variable(body, "created_at")
            || list_pages_body_uses_variable(body, "createdat")
            || list_pages_body_uses_variable(body, "date");
        let wants_updated_by = list_pages_body_uses_variable(body, "updated_by")
            || list_pages_body_uses_variable(body, "updatedby");
        let wants_updated_at = list_pages_body_uses_variable(body, "updated_at")
            || list_pages_body_uses_variable(body, "updatedat");
        let wants_tags = list_pages_body_uses_variable(body, "tags")
            || list_pages_body_uses_variable(body, "tags_linked")
            || list_pages_body_uses_variable(body, "tagslinked");
        let wants_rating_votes = list_pages_body_uses_variable(body, "rating_votes")
            || list_pages_body_uses_variable(body, "ratingvotes");
        let author_ids = Self::resolve_list_pages_author_ids(
            ctx,
            current_site_id,
            current_page_id,
            &authors,
        )
        .await?;
        let query = PageQuery {
            current_page_id,
            current_site_id,
            queried_site_id: None,
            page_type,
            categories: CategoriesSelector {
                included_categories,
                excluded_categories: &excluded_categories,
            },
            tags: TagCondition {
                any_present: &any_tags,
                all_present: &all_tags,
                none_present: &no_tags,
            },
            page_parent,
            contains_outgoing_links: &[],
            creation_date: DateSelector::FromPresent {
                start: time::OffsetDateTime::UNIX_EPOCH,
            },
            update_date: DateSelector::FromPresent {
                start: time::OffsetDateTime::UNIX_EPOCH,
            },
            author: &author_ids,
            score: &[],
            votes: &[],
            offset: 0,
            range: RangeSelector::Current,
            name: None,
            slug,
            data_form_fields: &data_form_fields,
            order,
            candidate_limit: if data_form_fields.is_empty() {
                None
            } else {
                Some(u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS))
            },
            pagination: PaginationSelector {
                limit: Some(MAX_LISTPAGES_RENDER_LIMIT),
                per_page: PaginationSelector::default().per_page,
                reversed: false,
            },
            variables: &[],
            fields: FoundPageFields {
                title: true,
                slug: true,
                page_category_id: true,
                created_by: wants_created_by,
                created_at: wants_created_at,
                tags: wants_tags,
                updated_by: wants_updated_by,
                updated_at: wants_updated_at,
                score: list_pages_body_uses_variable(body, "rating")
                    || wants_rating_votes,
                ..Default::default()
            },
        };

        let mut list_pages_metadata = None;
        let pages = if current_page_only
            && should_render_current_page_list_pages_row(current_page_only, limit, offset)
        {
            let pages = Self::current_page_list_pages_row(
                ctx,
                current_site_id,
                current_page_id,
                page_info,
                &query.fields,
            )
            .await?;
            if data_form_fields.is_empty()
                || Self::current_page_matches_data_form_fields(
                    ctx,
                    current_site_id,
                    current_page_id,
                    &data_form_fields,
                )
                .await?
            {
                pages
            } else {
                FoundPages { pages: Vec::new() }
            }
        } else if current_page_only {
            FoundPages { pages: Vec::new() }
        } else {
            let found = Self::find_viewable_list_pages_rows(
                ctx,
                query,
                query_limit.min(usize::MAX as u64) as usize,
            )
            .await?;
            list_pages_metadata = Some((
                found.metadata.clone(),
                found.view_permission_filtering_applied,
            ));
            found.pages
        };
        if let Some((metadata, view_permission_filtering_applied)) = list_pages_metadata {
            let diagnostics =
                list_pages_render_diagnostics(ListPagesRenderDiagnosticsInput {
                    metadata,
                    view_permission_filtering_applied,
                    post_query_exclusion_applied: exclude_current_page,
                    post_query_offset_applied: offset > 0,
                    requested_limit,
                    query_limit,
                });
            debug!("ListPages render diagnostics: {diagnostics:?}");
        }
        let selected_pages = pages
            .pages
            .into_iter()
            .filter(|page| !exclude_current_page || page.page_id != current_page_id)
            .skip(offset as usize)
            .collect::<Vec<_>>();
        let total_selected = selected_pages.len();
        let pages = selected_pages
            .into_iter()
            .take(requested_limit as usize)
            .collect::<Vec<_>>();
        let total = pages.len();
        let user_displays = if wants_created_by || wants_updated_by {
            Self::load_wikidot_user_displays(ctx, &pages).await?
        } else {
            BTreeMap::new()
        };
        let wants_comments = list_pages_body_uses_variable(body, "comments");
        let wants_commented_by = list_pages_body_uses_variable(body, "commented_by")
            || list_pages_body_uses_variable(body, "commentedby");
        let wants_commented_at = list_pages_body_uses_variable(body, "commented_at")
            || list_pages_body_uses_variable(body, "commentedat");
        let snapshot_displays = if wants_created_by
            || wants_updated_by
            || wants_created_at
            || wants_updated_at
            || wants_comments
            || wants_commented_by
            || wants_commented_at
            || wants_rating_votes
        {
            Self::load_list_pages_snapshot_displays(ctx, &pages).await?
        } else {
            BTreeMap::new()
        };
        let mut output = String::from("[[div class=\"list-pages-box\"]]\n");
        let mut included_pages = Vec::new();
        if let Some(prepend_line) = prepend_line {
            output.push_str(&prepend_line);
            output.push('\n');
        }

        let wants_content = list_pages_body_uses_content_variable(body);
        let wants_data_form_values = list_pages_body_uses_variable(body, "form_data")
            || list_pages_body_uses_variable(body, "form_raw");
        for (index, page) in pages.iter().enumerate() {
            output.push_str("[[div class=\"list-pages-item\"]]\n");
            let page_wikitext = if wants_content || wants_data_form_values {
                PageRevisionService::get_wikitext_optional(
                    ctx,
                    page.site_id,
                    Reference::Id(page.page_id),
                )
                .await?
            } else {
                None
            };
            let data_form_values = if wants_data_form_values {
                page_wikitext
                    .as_deref()
                    .map(parse_static_wikidot_data_form_values)
                    .unwrap_or_default()
            } else {
                BTreeMap::new()
            };
            let expanded_page_wikitext = if wants_content {
                match page_wikitext.as_deref() {
                    Some(wikitext) => {
                        let expansion = Self::expand_includes(
                            ctx,
                            wikitext.to_owned(),
                            page_info,
                            page_info.site.as_ref(),
                            settings,
                            Some(page.site_id),
                            IncludeExpansionOptions {
                                expand_wikidot_image_blocks: false,
                                max_total_includes: MAX_INCLUDE_EXPANSION_TOTAL,
                            },
                        )
                        .await?;
                        included_pages.extend(expansion.included_pages);
                        Some(expansion.wikitext)
                    }
                    None => None,
                }
            } else {
                None
            };
            let substitution_context = ListPagesSubstitutionContext {
                rendered_limit: requested_limit as usize,
                user_displays: &user_displays,
                snapshot_displays: &snapshot_displays,
                page_wikitext: expanded_page_wikitext
                    .as_deref()
                    .or(page_wikitext.as_deref()),
                data_form_values: &data_form_values,
                render_generated_html: list_pages_body_has_table_rows(body),
            };
            let mut body = substitute_list_pages_variables(
                body,
                page,
                index + 1,
                total,
                &substitution_context,
            );
            Self::neutralize_authored_wikidot_compat_markers(&mut body);
            if let Some(table) = render_list_pages_table_rows(&body) {
                output.push_str(&table);
            } else {
                output.push_str(&render_list_pages_numbered_rows(&body));
            }
            output.push_str("\n[[/div]]\n");
        }

        if let Some(per_page) = count_pages_per_page {
            push_list_pages_pager(
                &mut output,
                page_info,
                offset,
                per_page,
                total_selected,
            );
        }

        output.push_str("[[/div]]");
        Ok(IncludeExpansion {
            wikitext: output,
            included_pages,
            expanded_include_count: 0,
        })
    }

    async fn render_count_pages_block(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        current_page_id: i64,
        page_info: &PageInfo<'_>,
        arguments: ListPagesArguments,
        body: &str,
        original_module: &str,
    ) -> Result<String> {
        let ListPagesArguments {
            current_page_only,
            category_selector_present,
            category_all,
            include_current_category,
            categories,
            excluded_categories,
            mut any_tags,
            all_tags,
            default_tags,
            no_tags,
            authors,
            order,
            limit,
            count_pages_explicit_limit,
            count_pages_per_page: _,
            offset,
            exclude_current_page,
            page_type,
            page_parent,
            slug,
            prepend_line: _,
            data_form_fields,
            unsupported_count_pages_filter: _,
        } = arguments;
        let count_pages_query_limit = count_pages_explicit_limit
            .map(|limit| {
                limit
                    .saturating_add(u64::from(offset))
                    .saturating_add(u64::from(exclude_current_page))
            })
            .unwrap_or(u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS))
            .min(u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS));
        any_tags.extend(default_tags);
        let (category_all, include_current_category) = if category_selector_present {
            (category_all, include_current_category)
        } else {
            (false, true)
        };
        let categories = if include_current_category && !category_all {
            Self::categories_with_current_page_category(categories, page_info)
        } else {
            categories
        };
        let included_categories = if category_all {
            IncludedCategories::All
        } else {
            IncludedCategories::List(&categories)
        };
        let author_ids = Self::resolve_list_pages_author_ids(
            ctx,
            current_site_id,
            current_page_id,
            &authors,
        )
        .await?;
        let query = PageQuery {
            current_page_id,
            current_site_id,
            queried_site_id: None,
            page_type,
            categories: CategoriesSelector {
                included_categories,
                excluded_categories: &excluded_categories,
            },
            tags: TagCondition {
                any_present: &any_tags,
                all_present: &all_tags,
                none_present: &no_tags,
            },
            page_parent,
            contains_outgoing_links: &[],
            creation_date: DateSelector::FromPresent {
                start: time::OffsetDateTime::UNIX_EPOCH,
            },
            update_date: DateSelector::FromPresent {
                start: time::OffsetDateTime::UNIX_EPOCH,
            },
            author: &author_ids,
            score: &[],
            votes: &[],
            offset: 0,
            range: RangeSelector::Current,
            name: None,
            slug,
            data_form_fields: &data_form_fields,
            order,
            candidate_limit: if data_form_fields.is_empty() {
                None
            } else {
                Some(u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS))
            },
            pagination: PaginationSelector {
                limit: Some(count_pages_query_limit),
                per_page: PaginationSelector::default().per_page,
                reversed: false,
            },
            variables: &[],
            fields: FoundPageFields {
                page_category_id: true,
                ..Default::default()
            },
        };

        let mut count_pages_metadata = None;
        let pages = if current_page_only
            && should_render_current_page_list_pages_row(current_page_only, limit, offset)
        {
            Self::current_page_list_pages_row(
                ctx,
                current_site_id,
                current_page_id,
                page_info,
                &query.fields,
            )
            .await?
        } else if current_page_only {
            FoundPages { pages: Vec::new() }
        } else {
            let target_count = count_pages_query_limit.min(usize::MAX as u64) as usize;
            let found =
                Self::find_viewable_count_pages_rows(ctx, query, target_count).await?;
            count_pages_metadata = Some((
                found.metadata.clone(),
                found.view_permission_filtering_applied,
            ));
            found.pages
        };
        if let Some((metadata, view_permission_filtering_applied)) = count_pages_metadata
        {
            let diagnostics = count_pages_exact_count_render_diagnostics(
                metadata,
                view_permission_filtering_applied,
                exclude_current_page,
                offset > 0,
                count_pages_explicit_limit,
                count_pages_query_limit,
            );
            debug!("CountPages exact count eligibility diagnostics: {diagnostics:?}");
        }
        let pages = pages
            .pages
            .into_iter()
            .filter(|page| !exclude_current_page || page.page_id != current_page_id)
            .skip(offset as usize);
        let total = match count_pages_explicit_limit {
            Some(limit) => pages.take(limit.min(usize::MAX as u64) as usize).count(),
            None => {
                let total = pages.count();
                if count_pages_explicit_limit.is_none()
                    && total >= MAX_LISTPAGES_RENDER_SCAN_ROWS as usize
                {
                    return Ok(original_module.to_owned());
                }
                total
            }
        };

        Ok(substitute_count_pages_variables(body, total))
    }

    async fn filter_viewable_list_pages_rows(
        ctx: &ServiceContext<'_>,
        pages: Vec<FoundPageRow>,
    ) -> Result<Vec<FoundPageRow>> {
        let mut viewable = Vec::with_capacity(pages.len());
        for page in pages {
            let can_view = PermissionService::check_user_can(
                ctx,
                &CheckPermissionContext {
                    user_id: None,
                    site_id: page.site_id,
                    page_reference: Some(Reference::Id(page.page_id)),
                },
                Permission {
                    resource_type: Resource::Page,
                    resource_category: page.page_category_id.map(Reference::Id),
                    action: Action::View,
                },
            )
            .await?;
            if can_view {
                viewable.push(page);
            }
        }

        Ok(viewable)
    }

    async fn find_viewable_list_pages_rows(
        ctx: &ServiceContext<'_>,
        query: PageQuery<'_>,
        target_count: usize,
    ) -> Result<ViewableListPagesRows> {
        let mut pages = Vec::new();
        let mut raw_offset = 0;
        let mut metadata = None;
        let mut view_permission_filtering_applied = false;

        while pages.len() < target_count && raw_offset < MAX_LISTPAGES_RENDER_SCAN_ROWS {
            let mut query = query.clone();
            query.offset = raw_offset;
            query.pagination.limit = Some(
                MAX_LISTPAGES_RENDER_LIMIT
                    .min(u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS - raw_offset)),
            );

            let found = PageQueryService::find_with_metadata(ctx, query).await?;
            merge_render_page_query_metadata(&mut metadata, found.metadata);
            let raw_count = found.pages.pages.len();
            if raw_count == 0 {
                break;
            }
            let viewable =
                Self::filter_viewable_list_pages_rows(ctx, found.pages.pages).await?;
            view_permission_filtering_applied |= viewable.len() != raw_count;
            pages.extend(viewable);
            if raw_count < MAX_LISTPAGES_RENDER_LIMIT as usize {
                break;
            }
            raw_offset = raw_offset.saturating_add(MAX_LISTPAGES_RENDER_LIMIT as u32);
        }

        Ok(ViewableListPagesRows {
            pages: FoundPages { pages },
            metadata: metadata.unwrap_or_default(),
            view_permission_filtering_applied,
        })
    }

    async fn find_viewable_count_pages_rows(
        ctx: &ServiceContext<'_>,
        query: PageQuery<'_>,
        target_count: usize,
    ) -> Result<ViewableCountPagesRows> {
        let mut pages = Vec::new();
        let mut raw_offset = 0;
        let mut metadata = None;
        let mut view_permission_filtering_applied = false;

        while pages.len() < target_count && raw_offset < MAX_LISTPAGES_RENDER_SCAN_ROWS {
            let mut query = query.clone();
            query.offset = raw_offset;
            query.pagination.limit = Some(
                MAX_LISTPAGES_RENDER_LIMIT
                    .min(u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS - raw_offset)),
            );

            let found = PageQueryService::find_with_metadata(ctx, query).await?;
            merge_render_page_query_metadata(&mut metadata, found.metadata);
            let raw_count = found.pages.pages.len();
            if raw_count == 0 {
                break;
            }
            let viewable =
                Self::filter_viewable_list_pages_rows(ctx, found.pages.pages).await?;
            view_permission_filtering_applied |= viewable.len() != raw_count;
            pages.extend(viewable);
            if raw_count < MAX_LISTPAGES_RENDER_LIMIT as usize {
                break;
            }
            raw_offset = raw_offset.saturating_add(MAX_LISTPAGES_RENDER_LIMIT as u32);
        }

        Ok(ViewableCountPagesRows {
            pages: FoundPages { pages },
            metadata: metadata.unwrap_or_default(),
            view_permission_filtering_applied,
        })
    }

    async fn current_page_list_pages_row(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        current_page_id: i64,
        page_info: &PageInfo<'_>,
        fields: &FoundPageFields,
    ) -> Result<FoundPages> {
        let make_error = || {
            Error::new(
                "failed to load current page for ListPages render",
                ErrorType::Render,
            )
        };

        let page = PageService::get_direct(ctx, current_page_id, true)
            .await
            .or_raise(make_error)?;
        if page.site_id != current_site_id {
            bail!(Error::new(
                format!(
                    "current page ID {} is not in site ID {}",
                    current_page_id, current_site_id,
                ),
                ErrorType::Render,
            ));
        }
        let page_category_id = if fields.page_category_id {
            let category_slug = Self::page_info_category_slug(page_info);
            let category = CategoryService::get(
                ctx,
                current_site_id,
                Reference::Slug(Cow::Borrowed(category_slug.as_ref())),
            )
            .await
            .or_raise(make_error)?;
            Some(category.category_id)
        } else {
            None
        };
        let slug = if fields.slug {
            Some(Self::page_info_full_slug(page_info))
        } else {
            None
        };
        let latest_revision =
            if fields.title || fields.alt_title || fields.tags || fields.updated_by {
                match page.latest_revision_id {
                    Some(_) => Some(
                        PageRevisionService::get_latest(
                            ctx,
                            current_site_id,
                            current_page_id,
                        )
                        .await
                        .or_raise(make_error)?,
                    ),
                    None => None,
                }
            } else {
                None
            };
        let creation_revision = if fields.created_by {
            match page.latest_revision_id {
                Some(_) => Some(
                    PageRevisionService::get_optional(
                        ctx,
                        current_site_id,
                        current_page_id,
                        0,
                    )
                    .await
                    .or_raise(make_error)?,
                ),
                None => None,
            }
        } else {
            None
        }
        .flatten();
        let latest_revision = latest_revision.as_ref();
        let creation_revision = creation_revision.as_ref();

        Ok(FoundPages {
            pages: vec![FoundPageRow {
                page_id: page.page_id,
                site_id: page.site_id,
                slug,
                page_category_id,
                page_revision_id: if fields.page_revision_id {
                    page.latest_revision_id
                } else {
                    None
                },
                tags: if fields.tags {
                    Some(
                        latest_revision
                            .map(|revision| revision.tags.clone())
                            .unwrap_or_else(|| {
                                page_info.tags.iter().map(|tag| tag.to_string()).collect()
                            }),
                    )
                } else {
                    None
                },
                created_at: if fields.created_at {
                    Some(page.created_at)
                } else {
                    None
                },
                created_by: if fields.created_by {
                    creation_revision.map(|revision| revision.user_id)
                } else {
                    None
                },
                updated_at: if fields.updated_at {
                    page.updated_at
                } else {
                    None
                },
                updated_by: if fields.updated_by {
                    latest_revision.map(|revision| revision.user_id)
                } else {
                    None
                },
                title: if fields.title {
                    Some(
                        latest_revision
                            .map(|revision| revision.title.clone())
                            .unwrap_or_else(|| page_info.title.to_string()),
                    )
                } else {
                    None
                },
                alt_title: if fields.alt_title {
                    latest_revision
                        .and_then(|revision| revision.alt_title.clone())
                        .or_else(|| {
                            page_info.alt_title.as_ref().map(|title| title.to_string())
                        })
                } else {
                    None
                },
                score: requested_page_info_score(fields, page_info),
            }],
        })
    }

    async fn current_page_matches_data_form_fields(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        current_page_id: i64,
        data_form_fields: &[DataFormSelector<'_>],
    ) -> Result<bool> {
        let Some(wikitext) = PageRevisionService::get_wikitext_optional(
            ctx,
            current_site_id,
            Reference::Id(current_page_id),
        )
        .await?
        else {
            return Ok(false);
        };

        let values = parse_static_wikidot_data_form_values(&wikitext);
        Ok(static_wikidot_data_form_matches(&values, data_form_fields))
    }

    async fn load_wikidot_user_displays(
        ctx: &ServiceContext<'_>,
        pages: &[FoundPageRow],
    ) -> Result<BTreeMap<i64, WikidotUserDisplay>> {
        let make_error = || {
            Error::new(
                "failed to load Wikidot user names for ListPages render",
                ErrorType::Render,
            )
        };

        let user_ids = pages
            .iter()
            .flat_map(|page| [page.created_by, page.updated_by])
            .flatten()
            .collect::<BTreeSet<_>>();

        let wikidot_user_ids = user_ids
            .iter()
            .copied()
            .filter_map(|user_id| match i32::try_from(user_id) {
                Ok(user_id) => Some(user_id),
                Err(error) => {
                    warn!("Skipping Wikidot user ID {user_id} while rendering ListPages: {error}");
                    None
                }
            })
            .collect::<BTreeSet<_>>();

        if user_ids.is_empty() {
            return Ok(BTreeMap::new());
        }

        let mut displays = BTreeMap::new();
        if !wikidot_user_ids.is_empty() {
            let users = WikidotUser::find()
                .filter(wikidot_user::Column::UserId.is_in(wikidot_user_ids.clone()))
                .all(ctx.transaction())
                .await
                .or_raise(make_error)?;

            displays.extend(users.into_iter().filter_map(|user| {
                let name = user.name.or_else(|| user.slug.clone())?;
                Some((
                    i64::from(user.user_id),
                    WikidotUserDisplay {
                        user_id: i64::from(user.user_id),
                        name,
                        slug: user.slug,
                        wikidot_profile: true,
                    },
                ))
            }));
        }

        let missing_user_ids = user_ids
            .into_iter()
            .filter(|user_id| !displays.contains_key(user_id))
            .collect::<Vec<_>>();
        if !missing_user_ids.is_empty() {
            let users = UserTable::find()
                .filter(user::Column::UserId.is_in(missing_user_ids))
                .all(ctx.transaction())
                .await
                .or_raise(make_error)?;

            displays.extend(users.into_iter().map(|user| {
                (
                    user.user_id,
                    WikidotUserDisplay {
                        user_id: user.user_id,
                        name: user.name,
                        slug: Some(user.slug),
                        wikidot_profile: false,
                    },
                )
            }));
        }

        Ok(displays)
    }

    async fn load_list_pages_snapshot_displays(
        ctx: &ServiceContext<'_>,
        pages: &[FoundPageRow],
    ) -> Result<BTreeMap<i64, ListPagesSnapshotDisplay>> {
        #[derive(FromQueryResult, Debug)]
        struct SnapshotDisplayRow {
            page_id: i64,
            source_created_at: time::OffsetDateTime,
            source_updated_at: time::OffsetDateTime,
            created_by_name: Option<String>,
            updated_by_name: Option<String>,
            comments: i32,
            commented_at: Option<time::OffsetDateTime>,
            commented_by_name: Option<String>,
            rating_votes: Option<i64>,
        }

        let page_ids = pages
            .iter()
            .map(|page| page.page_id)
            .collect::<BTreeSet<_>>();
        if page_ids.is_empty() {
            return Ok(BTreeMap::new());
        }

        let make_error = || {
            Error::new(
                "failed to load imported Wikidot snapshot metadata for ListPages render",
                ErrorType::Render,
            )
        };
        let values = page_ids
            .iter()
            .map(|page_id| format!("({page_id})"))
            .collect::<Vec<_>>()
            .join(", ");
        let txn = ctx.transaction();
        let statement = Statement::from_string(
            txn.get_database_backend(),
            format!(
                "WITH input(page_id) AS (VALUES {values}) \
                 SELECT snapshot.page_id, snapshot.source_created_at, snapshot.source_updated_at, \
                        snapshot.created_by_name, snapshot.updated_by_name, snapshot.comments, \
                        snapshot.commented_at, snapshot.commented_by_name, \
                        CASE \
                            WHEN snapshot.meta_json ->> 'votes_count' ~ '^[0-9]{{1,19}}$' \
                                 AND (length(snapshot.meta_json ->> 'votes_count') < 19 \
                                      OR snapshot.meta_json ->> 'votes_count' <= '9223372036854775807') \
                            THEN (snapshot.meta_json ->> 'votes_count')::bigint \
                            ELSE NULL \
                        END AS rating_votes \
                 FROM input \
                 JOIN wikidot_page_snapshot snapshot ON snapshot.page_id = input.page_id",
            ),
        );

        SnapshotDisplayRow::find_by_statement(statement)
            .all(txn)
            .await
            .or_raise(make_error)
            .map(|rows| {
                rows.into_iter()
                    .map(
                        |SnapshotDisplayRow {
                             page_id,
                             source_created_at,
                             source_updated_at,
                             created_by_name,
                             updated_by_name,
                             comments,
                             commented_at,
                             commented_by_name,
                             rating_votes,
                         }| {
                            (
                                page_id,
                                ListPagesSnapshotDisplay {
                                    created_at: source_created_at,
                                    updated_at: source_updated_at,
                                    created_by_name,
                                    updated_by_name,
                                    comments,
                                    commented_at,
                                    commented_by_name,
                                    rating_votes,
                                },
                            )
                        },
                    )
                    .collect()
            })
    }

    async fn resolve_list_pages_author_ids(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        current_page_id: i64,
        author_names: &[Cow<'static, str>],
    ) -> Result<Vec<Cow<'static, str>>> {
        if author_names.is_empty() {
            return Ok(Vec::new());
        }

        let mut literal_authors = Vec::new();
        let mut include_current_page_author = false;
        for author in author_names {
            if author.as_ref() == "=" {
                include_current_page_author = true;
            } else {
                literal_authors.push(author.clone());
            }
        }

        let mut author_ids = if literal_authors.is_empty() {
            Vec::new()
        } else {
            Self::load_wikidot_author_ids(ctx, &literal_authors).await?
        };

        if include_current_page_author {
            let make_error = || {
                Error::new(
                    "failed to load current page creation author for ListPages render",
                    ErrorType::Render,
                )
            };
            let creation_revision = PageRevisionService::get_optional(
                ctx,
                current_site_id,
                current_page_id,
                0,
            )
            .await
            .or_raise(make_error)?;
            if let Some(revision) = creation_revision {
                author_ids.push(Cow::Owned(revision.user_id.to_string()));
            } else if literal_authors.is_empty() {
                author_ids.push(Cow::Borrowed(LISTPAGES_NO_MATCH_AUTHOR_ID));
            }
        }

        author_ids.sort();
        author_ids.dedup();
        Ok(author_ids)
    }

    async fn load_wikidot_author_ids(
        ctx: &ServiceContext<'_>,
        author_names: &[Cow<'static, str>],
    ) -> Result<Vec<Cow<'static, str>>> {
        let make_error = || {
            Error::new(
                "failed to load Wikidot author IDs for ListPages render",
                ErrorType::Render,
            )
        };
        let wanted = author_names
            .iter()
            .map(|name| normalize_list_pages_user_selector(name))
            .collect::<BTreeSet<_>>();
        if wanted.is_empty() {
            return Ok(Vec::new());
        }

        let users = WikidotUser::find()
            .all(ctx.transaction())
            .await
            .or_raise(make_error)?;

        let author_ids = users
            .into_iter()
            .filter(|user| {
                user.name.as_ref().is_some_and(|name| {
                    wanted.contains(&normalize_list_pages_user_selector(name))
                }) || user.slug.as_ref().is_some_and(|slug| {
                    wanted.contains(&normalize_list_pages_user_selector(slug))
                })
            })
            .map(|user| Cow::Owned(user.user_id.to_string()))
            .collect::<Vec<_>>();

        if author_ids.is_empty() {
            Ok(vec![Cow::Borrowed(LISTPAGES_NO_MATCH_AUTHOR_ID)])
        } else {
            Ok(author_ids)
        }
    }
}

#[derive(Debug, Clone)]
struct WikidotUserDisplay {
    user_id: i64,
    name: String,
    slug: Option<String>,
    wikidot_profile: bool,
}

#[derive(Debug, Clone)]
struct ListPagesSnapshotDisplay {
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
    created_by_name: Option<String>,
    updated_by_name: Option<String>,
    comments: i32,
    commented_at: Option<time::OffsetDateTime>,
    commented_by_name: Option<String>,
    rating_votes: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
struct BacklinksModulePage {
    page_id: i64,
    page_category_id: i64,
    slug: String,
    title: String,
}

#[derive(Debug)]
struct ListPagesArguments {
    current_page_only: bool,
    category_selector_present: bool,
    category_all: bool,
    include_current_category: bool,
    categories: Vec<Cow<'static, str>>,
    excluded_categories: Vec<Cow<'static, str>>,
    any_tags: Vec<Cow<'static, str>>,
    default_tags: Vec<Cow<'static, str>>,
    all_tags: Vec<Cow<'static, str>>,
    no_tags: Vec<Cow<'static, str>>,
    authors: Vec<Cow<'static, str>>,
    order: Option<OrderBySelector>,
    limit: Option<u64>,
    count_pages_explicit_limit: Option<u64>,
    count_pages_per_page: Option<u64>,
    offset: u32,
    exclude_current_page: bool,
    page_type: PageTypeSelector,
    page_parent: PageParentSelector<'static>,
    slug: Option<Cow<'static, str>>,
    data_form_fields: Vec<DataFormSelector<'static>>,
    prepend_line: Option<String>,
    unsupported_count_pages_filter: bool,
}

fn parse_list_pages_arguments(head: &str) -> Option<ListPagesArguments> {
    let unparsed = LISTPAGES_ARGUMENT_REGEX.replace_all(head, "");
    if !unparsed.trim().is_empty() {
        return None;
    }

    let mut category_all = true;
    let mut category_selector_present = false;
    let mut current_page_only = false;
    let mut include_current_category = false;
    let mut categories = Vec::new();
    let mut excluded_categories = Vec::new();
    let any_tags = Vec::new();
    let mut default_tags = Vec::new();
    let mut all_tags = Vec::new();
    let mut no_tags = Vec::new();
    let mut authors = Vec::new();
    let mut order = None;
    let mut limit = None;
    let mut count_pages_explicit_limit = None;
    let mut count_pages_per_page = None;
    let mut offset = 0;
    let mut exclude_current_page = false;
    let mut page_type = PageTypeSelector::Normal;
    let mut page_parent = PageParentSelector::All;
    let mut slug = None;
    let mut data_form_fields = Vec::new();
    let mut prepend_line = None;
    let mut unsupported_count_pages_filter = false;

    for captures in LISTPAGES_ARGUMENT_REGEX.captures_iter(head) {
        let raw_key = &captures["key"];
        let key = raw_key.to_ascii_lowercase();
        let value = captures
            .name("double")
            .or_else(|| captures.name("single"))
            .or_else(|| captures.name("bare"))
            .unwrap()
            .as_str()
            .trim();
        if captures.name("op").map_or("=", |matched| matched.as_str()) != "="
            && !key.starts_with('_')
        {
            return None;
        }

        match key.as_str() {
            "tags" => {
                let Some(value) = static_list_pages_selector(
                    value,
                    &mut unsupported_count_pages_filter,
                ) else {
                    continue;
                };
                for tag in split_list_pages_values(value) {
                    if is_no_tags_selector(&tag) {
                        unsupported_count_pages_filter = true;
                        continue;
                    }
                    if is_current_page_tag_selector(&tag) {
                        unsupported_count_pages_filter = true;
                    }
                    if let Some(tag) = tag.strip_prefix('-') {
                        no_tags.push(Cow::Owned(tag.to_owned()));
                    } else if let Some(tag) = tag.strip_prefix('+') {
                        all_tags.push(Cow::Owned(tag.to_owned()));
                    } else {
                        default_tags.push(Cow::Owned(tag));
                    }
                }
            }
            "tag" => {
                let Some(value) = static_list_pages_selector(
                    value,
                    &mut unsupported_count_pages_filter,
                ) else {
                    continue;
                };
                for tag in split_list_pages_values(value) {
                    if is_no_tags_selector(&tag) {
                        unsupported_count_pages_filter = true;
                        continue;
                    }
                    if is_current_page_tag_selector(&tag) {
                        unsupported_count_pages_filter = true;
                    }
                    if let Some(tag) = tag.strip_prefix('-') {
                        no_tags.push(Cow::Owned(tag.to_owned()));
                    } else if let Some(tag) = tag.strip_prefix('+') {
                        all_tags.push(Cow::Owned(tag.to_owned()));
                    } else {
                        default_tags.push(Cow::Owned(tag));
                    }
                }
            }
            "category" => {
                category_selector_present = true;
                let mut saw_included_category = false;
                let Some(value) = static_list_pages_selector(
                    value,
                    &mut unsupported_count_pages_filter,
                ) else {
                    continue;
                };
                for category in split_list_pages_values(value) {
                    if category == "*" {
                        category_all = true;
                    } else if category == "." {
                        include_current_category = true;
                        saw_included_category = true;
                    } else if let Some(category) = category.strip_prefix('+') {
                        categories.push(Cow::Owned(category.to_owned()));
                        saw_included_category = true;
                    } else if let Some(category) = category.strip_prefix('-') {
                        excluded_categories.push(Cow::Owned(category.to_owned()));
                    } else {
                        categories.push(Cow::Owned(category));
                        saw_included_category = true;
                    }
                }
                if saw_included_category {
                    category_all = false;
                }
            }
            "limit" => {
                let parsed = parse_list_pages_numeric_argument(value)?;
                limit = Some(parsed);
                count_pages_explicit_limit = Some(parsed);
            }
            "perpage" | "per_page" => {
                let parsed = parse_list_pages_numeric_argument(value)?;
                count_pages_per_page = Some(parsed);
            }
            "offset" => {
                let parsed = parse_list_pages_numeric_argument(value)?;
                if parsed > u64::from(MAX_LISTPAGES_RENDER_OFFSET) {
                    return None;
                }
                offset = parsed as u32;
            }
            "pagetype" | "page_type" | "page-type" => {
                page_type = parse_list_pages_page_type(value)?;
            }
            "parent" => {
                let value = list_pages_url_fallback(value).unwrap_or(value);
                match value {
                    "." => page_parent = PageParentSelector::ChildOf,
                    "*" | "" => page_parent = PageParentSelector::All,
                    _ if is_dynamic_list_pages_value(value) => return None,
                    _ => return None,
                }
            }
            "prependline" | "prepend_line" => {
                prepend_line = Some(value.to_owned());
            }
            "order" => {
                if value.is_empty() {
                    continue;
                }
                let value = list_pages_url_fallback(value).unwrap_or(value);
                order = Some(parse_list_pages_order(value)?);
            }
            "name" | "fullname" | "full_slug" | "fullslug" => {
                let Some(value) = static_list_pages_selector(
                    value,
                    &mut unsupported_count_pages_filter,
                ) else {
                    continue;
                };
                if value == "=" {
                    current_page_only = true;
                    limit = Some(1);
                } else if !is_dynamic_list_pages_value(value) {
                    slug = Some(Cow::Owned(wikidot_list_pages_name_slug(value)));
                }
            }
            // These inputs need additional data or Wikidot semantics that are not
            // implemented by PageQueryService yet. Leaving the module untouched is
            // safer than silently returning a wrong list.
            "separate" => {
                if !matches!(
                    value.to_ascii_lowercase().as_str(),
                    "yes" | "no" | "true" | "false"
                ) {
                    return None;
                }
            }
            "created_by" | "createdby" => {
                let Some(value) = static_list_pages_selector(
                    value,
                    &mut unsupported_count_pages_filter,
                ) else {
                    continue;
                };
                let author = value
                    .trim()
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .trim();
                if author == "-=" {
                    unsupported_count_pages_filter = true;
                    continue;
                }
                if !author.is_empty() {
                    authors.push(Cow::Owned(author.to_owned()));
                }
            }
            "range" => match value {
                "." => {
                    current_page_only = true;
                    limit = Some(1);
                }
                "others" | "other" => {
                    exclude_current_page = true;
                }
                "before" | "after" => {
                    unsupported_count_pages_filter = true;
                }
                _ => {}
            },
            "wrapper" => {}
            "rating" | "score" | "votes" | "form" | "link_to" | "linkto"
            | "urlattrprefix" | "created_at" | "createdat" | "updated_at"
            | "updatedat" => {
                unsupported_count_pages_filter = true;
                // These filters need Wikidot-specific query semantics that are not
                // fully implemented here. Parsing them keeps real corpus modules
                // out of FTML's generic module path, which otherwise panics on
                // ListPages bodies that start with numbered-list markers.
            }
            _ if raw_key.starts_with('_') => {
                let value = static_list_pages_selector(
                    value,
                    &mut unsupported_count_pages_filter,
                )?;
                let field = raw_key
                    .strip_prefix('_')
                    .expect("data form selector should start with an underscore");
                if field.is_empty() || is_dynamic_list_pages_value(value) {
                    return None;
                }
                data_form_fields.push(DataFormSelector {
                    field: Cow::Owned(field.to_owned()),
                    value: Cow::Owned(value.to_owned()),
                    negated: &captures["op"] == "!=",
                });
            }
            _ => return None,
        }
    }

    Some(ListPagesArguments {
        current_page_only,
        category_selector_present,
        category_all,
        include_current_category,
        categories,
        excluded_categories,
        any_tags,
        default_tags,
        all_tags,
        no_tags,
        authors,
        order,
        limit,
        count_pages_explicit_limit,
        count_pages_per_page,
        offset,
        exclude_current_page,
        page_type,
        page_parent,
        slug,
        data_form_fields,
        prepend_line,
        unsupported_count_pages_filter,
    })
}

fn count_pages_should_remain_literal(arguments: &ListPagesArguments) -> bool {
    let count_pages_bound = arguments
        .count_pages_explicit_limit
        .or(arguments.count_pages_per_page);
    arguments.unsupported_count_pages_filter
        || (count_pages_bound.is_none()
            && !arguments.current_page_only
            && !count_pages_has_static_filter(arguments))
        || count_pages_bound.is_some_and(|limit| {
            limit
                .saturating_add(u64::from(arguments.offset))
                .saturating_add(u64::from(arguments.exclude_current_page))
                > u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS)
        })
        || (arguments.category_selector_present
            && arguments.category_all
            && arguments.count_pages_explicit_limit.is_none()
            && !count_pages_has_static_filter(arguments))
        || (arguments.current_page_only
            && (arguments.category_selector_present
                || arguments.page_type != PageTypeSelector::Normal
                || arguments.page_parent != PageParentSelector::All
                || !arguments.default_tags.is_empty()
                || !arguments.any_tags.is_empty()
                || !arguments.all_tags.is_empty()
                || !arguments.no_tags.is_empty()
                || !arguments.authors.is_empty()
                || !arguments.excluded_categories.is_empty()
                || !arguments.data_form_fields.is_empty()
                || arguments.slug.is_some()))
}

fn count_pages_has_static_filter(arguments: &ListPagesArguments) -> bool {
    !arguments.categories.is_empty()
        || !arguments.default_tags.is_empty()
        || !arguments.any_tags.is_empty()
        || !arguments.all_tags.is_empty()
        || !arguments.authors.is_empty()
        || arguments.page_type != PageTypeSelector::Normal
        || arguments.page_parent != PageParentSelector::All
        || arguments.slug.is_some()
        || !arguments.data_form_fields.is_empty()
}

fn count_pages_exact_count_render_diagnostics(
    metadata: PageQueryResultMetadata,
    view_permission_filtering_applied: bool,
    post_query_exclusion_applied: bool,
    post_query_offset_applied: bool,
    count_pages_explicit_limit: Option<u64>,
    count_pages_query_limit: u64,
) -> CountPagesExactCountEligibilityDiagnostics {
    let explicit_count_pages_bound_matches_sql_window =
        count_pages_explicit_limit.is_some_and(|limit| limit == count_pages_query_limit);

    count_pages_exact_count_eligibility_diagnostics(
        CountPagesExactCountEligibilityInput {
            metadata,
            view_permission_filtering_applied,
            post_query_filtering_applied: false,
            post_query_exclusion_applied,
            post_query_offset_applied,
            explicit_count_pages_bound_matches_sql_window,
        },
    )
}

fn merge_render_page_query_metadata(
    metadata: &mut Option<PageQueryResultMetadata>,
    next: PageQueryResultMetadata,
) {
    let Some(current) = metadata.as_mut() else {
        *metadata = Some(next);
        return;
    };

    current.candidate_count = match (current.candidate_count, next.candidate_count) {
        (Some(left), Some(right)) => Some(left.saturating_add(right)),
        _ => None,
    };
    current.cap_exceeded |= next.cap_exceeded;
    current.sql_limit_offset_applied |= next.sql_limit_offset_applied;
    current.filtering_deferred_to_rust |= next.filtering_deferred_to_rust;
    current.ordering_deferred_to_rust |= next.ordering_deferred_to_rust;
    current.exact_count_safe &= next.exact_count_safe;
    if current.unsupported_reason.is_none() {
        current.unsupported_reason = next.unsupported_reason;
    }
}

fn should_render_current_page_list_pages_row(
    current_page_only: bool,
    limit: Option<u64>,
    offset: u32,
) -> bool {
    current_page_only && limit.unwrap_or(1) > 0 && offset == 0
}

fn requested_page_info_score(
    fields: &FoundPageFields,
    page_info: &PageInfo<'_>,
) -> Option<f32> {
    fields.score.then(|| page_info.score.to_f64() as f32)
}

fn parse_list_pages_numeric_argument(value: &str) -> Option<u64> {
    if let Some(fallback) = list_pages_url_fallback(value) {
        return fallback.parse().ok();
    }

    value.parse().ok()
}

fn is_dynamic_list_pages_value(value: &str) -> bool {
    value.eq_ignore_ascii_case("@url")
        || value
            .split_once('|')
            .is_some_and(|(selector, _)| selector.eq_ignore_ascii_case("@url"))
}

fn list_pages_url_fallback(value: &str) -> Option<&str> {
    value.split_once('|').and_then(|(selector, fallback)| {
        selector.eq_ignore_ascii_case("@url").then_some(fallback)
    })
}

fn static_list_pages_selector<'a>(
    value: &'a str,
    unsupported_count_pages_filter: &mut bool,
) -> Option<&'a str> {
    if let Some(fallback) = list_pages_url_fallback(value) {
        Some(fallback)
    } else if is_dynamic_list_pages_value(value) {
        *unsupported_count_pages_filter = true;
        None
    } else {
        Some(value)
    }
}

fn list_pages_has_unsupported_parent_selector(head: &str) -> bool {
    LISTPAGES_ARGUMENT_REGEX
        .captures_iter(head)
        .any(|captures| {
            if !captures["key"].eq_ignore_ascii_case("parent") {
                return false;
            }

            let value = captures
                .name("double")
                .or_else(|| captures.name("single"))
                .or_else(|| captures.name("bare"))
                .map(|matched| matched.as_str().trim())
                .unwrap_or_default();
            let value = list_pages_url_fallback(value).unwrap_or(value);
            !matches!(value, "." | "*" | "")
        })
}

fn list_pages_has_unsupported_page_type_selector(head: &str) -> bool {
    LISTPAGES_ARGUMENT_REGEX
        .captures_iter(head)
        .any(|captures| {
            if !matches!(
                captures["key"].to_ascii_lowercase().as_str(),
                "pagetype" | "page_type" | "page-type"
            ) {
                return false;
            }

            let value = captures
                .name("double")
                .or_else(|| captures.name("single"))
                .or_else(|| captures.name("bare"))
                .map(|matched| matched.as_str().trim())
                .unwrap_or_default();
            let value = list_pages_url_fallback(value).unwrap_or(value);
            !matches!(
                value.to_ascii_lowercase().as_str(),
                "all" | "*" | "hidden" | "normal" | ""
            )
        })
}

fn wikidot_list_pages_name_slug(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace(' ', "-")
}

fn split_list_pages_values(value: &str) -> Vec<String> {
    value
        .split(|ch: char| ch.is_whitespace() || ch == ',')
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect()
}

fn is_current_page_tag_selector(value: &str) -> bool {
    matches!(value.trim().trim_start_matches(['+', '-']), "=" | "==")
}

fn is_no_tags_selector(value: &str) -> bool {
    value.trim() == "-"
}

fn normalize_list_pages_user_selector(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace(['_', ' '], "-")
}

fn parse_list_pages_order(value: &str) -> Option<OrderBySelector> {
    let (value, ascending) = match value.split_once(char::is_whitespace) {
        Some((property, direction)) => {
            let ascending = match direction.trim().to_ascii_lowercase().as_str() {
                "asc" | "ascending" => true,
                "desc" | "descending" => false,
                _ => return None,
            };
            (property, ascending)
        }
        None => match value.strip_prefix('-') {
            Some(value) => (value, false),
            None => parse_wikidot_camel_case_order(value).unwrap_or((value, true)),
        },
    };

    let property = match value.to_ascii_lowercase().as_str() {
        "name" | "slug" => OrderProperty::PageSlug,
        "fullname" | "fullslug" | "full_slug" => OrderProperty::FullSlug,
        "title" => OrderProperty::Title,
        "alt_title" | "alttitle" => OrderProperty::AltTitle,
        "created_at" | "createdat" | "created" | "date" => OrderProperty::CreatedAt,
        "updated_at" | "updatedat" | "updated" => OrderProperty::UpdatedAt,
        "size" => OrderProperty::Size,
        "random" => OrderProperty::Random,
        _ => return None,
    };

    Some(OrderBySelector {
        property,
        ascending,
    })
}

fn parse_wikidot_camel_case_order(value: &str) -> Option<(&str, bool)> {
    let lower = value.to_ascii_lowercase();
    for (suffix, ascending) in [
        ("ascending", true),
        ("descending", false),
        ("asc", true),
        ("desc", false),
    ] {
        if lower.ends_with(suffix) && value.len() > suffix.len() {
            return Some((&value[..value.len() - suffix.len()], ascending));
        }
    }

    None
}

fn parse_list_pages_page_type(value: &str) -> Option<PageTypeSelector> {
    match value.to_ascii_lowercase().as_str() {
        "all" | "*" => Some(PageTypeSelector::All),
        "hidden" => Some(PageTypeSelector::Hidden),
        "normal" | "" => Some(PageTypeSelector::Normal),
        _ => None,
    }
}

fn list_pages_body_variables_supported(body: &str) -> bool {
    LISTPAGES_VARIABLE_REGEX
        .captures_iter(body)
        .all(
            |captures| match captures["name"].to_ascii_lowercase().as_str() {
                "title_linked" | "linked_title" | "title" | "name" | "slug"
                | "page_unix_name" | "fullname" | "full_slug" | "link" | "created_by"
                | "createdby" | "created_by_linked" | "createdbylinked" | "author"
                | "created_at" | "createdat" | "date" | "updated_by" | "updatedby"
                | "updated_at" | "updatedat" | "commented_at" | "commentedat"
                | "commented_by" | "commentedby" | "rating" | "rating_votes"
                | "ratingvotes" | "comments" | "tags" | "tags_linked" | "tagslinked"
                | "content" | "index" | "total" | "limit" => true,
                "form_data" | "form_raw" => captures.name("argument").is_some(),
                _ => false,
            },
        )
}

fn unsupported_list_pages_replacement(module_source: &str, body: &str) -> String {
    if list_pages_body_has_numbered_rows(body)
        || list_pages_body_is_no_visible_tracking_markup(body)
    {
        "[[div class=\"list-pages-box\"]][[/div]]".to_owned()
    } else {
        module_source.to_owned()
    }
}

fn list_pages_body_has_numbered_rows(body: &str) -> bool {
    body.lines()
        .any(|line| native_numbered_list_content(line).is_some())
}

fn list_pages_body_is_no_visible_tracking_markup(body: &str) -> bool {
    let mut saw_tracking_markup = false;

    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let lower = line.to_ascii_lowercase();
        let allowed = lower.starts_with("[[image ")
            || lower.starts_with("[[embed]]")
            || lower.starts_with("[[/embed]]")
            || lower.starts_with("<iframe ") && lower.contains("display: none")
            || lower.starts_with("[[module listusers ")
            || lower.starts_with("[[/module]]")
            || lower.starts_with("[[%%content{0}%%module listusers ")
            || lower.starts_with("[[%%content{0}%%/module]]");
        if !allowed {
            return false;
        }
        saw_tracking_markup = true;
    }

    saw_tracking_markup
}

fn list_pages_body_uses_variable(body: &str, variable: &str) -> bool {
    LISTPAGES_VARIABLE_REGEX
        .captures_iter(body)
        .any(|captures| captures["name"].eq_ignore_ascii_case(variable))
}

fn list_pages_body_uses_content_variable(body: &str) -> bool {
    list_pages_body_uses_variable(body, "content")
}

fn push_list_pages_pager(
    output: &mut String,
    page_info: &PageInfo<'_>,
    offset: u32,
    per_page: u64,
    total_selected: usize,
) {
    let per_page = per_page
        .min(MAX_LISTPAGES_RENDER_LIMIT)
        .min(usize::MAX as u64) as usize;
    if per_page == 0 || total_selected <= per_page {
        return;
    }

    let page_count = total_selected.div_ceil(per_page);
    let current_page = (offset as usize / per_page).saturating_add(1);
    if current_page > page_count {
        return;
    }

    output.push_str(r#"[[div class="pager"]]"#);
    output.push_str(&format!(
        r#"[[span class="pager-no"]]page {current_page} of {page_count}[[/span]]"#
    ));

    let mut pages = BTreeSet::from([1, current_page, page_count]);
    if current_page > 1 {
        pages.insert(current_page - 1);
    }
    if current_page < page_count {
        pages.insert(current_page + 1);
    }
    if current_page <= 2 && page_count >= 3 {
        pages.insert(3);
    }
    if current_page + 1 >= page_count && page_count > 2 {
        pages.insert(page_count - 2);
    }
    if page_count > 1 {
        pages.insert(page_count - 1);
    }

    let mut previous = 0;
    for page in pages {
        if previous != 0 && page > previous + 1 {
            output.push_str(r#"[[span class="dots"]]...[[/span]]"#);
        }
        if page == current_page {
            output.push_str(&format!(r#"[[span class="current"]]{page}[[/span]]"#));
        } else {
            push_list_pages_pager_target(output, page_info, page, &page.to_string());
        }
        previous = page;
    }

    if current_page < page_count {
        push_list_pages_pager_target(output, page_info, current_page + 1, "next »");
    }

    output.push_str("[[/div]]\n");
}

fn push_list_pages_pager_target(
    output: &mut String,
    page_info: &PageInfo<'_>,
    target_page: usize,
    label: &str,
) {
    output.push_str(r#"[[span class="target"]][/"#);
    output.push_str(page_info.page.as_ref());
    output.push_str("/p/");
    output.push_str(&target_page.to_string());
    output.push(' ');
    output.push_str(label);
    output.push_str("][[/span]]");
}

fn is_wikidot_content_separator_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.len() >= 4 && trimmed.chars().all(|character| character == '=')
}

fn wikidot_content_section(wikitext: &str, section: Option<usize>) -> String {
    let Some(section) = section else {
        return wikitext.to_owned();
    };
    if section == 0 {
        return String::new();
    }

    let mut sections = Vec::new();
    let mut current = String::new();
    for line in wikitext.split_inclusive('\n') {
        if is_wikidot_content_separator_line(line) {
            sections.push(current);
            current = String::new();
        } else {
            current.push_str(line);
        }
    }
    sections.push(current);

    sections
        .get(section - 1)
        .map(|section| section.trim_matches('\n').to_owned())
        .unwrap_or_default()
}

struct ListPagesSubstitutionContext<'a> {
    rendered_limit: usize,
    user_displays: &'a BTreeMap<i64, WikidotUserDisplay>,
    snapshot_displays: &'a BTreeMap<i64, ListPagesSnapshotDisplay>,
    page_wikitext: Option<&'a str>,
    data_form_values: &'a BTreeMap<String, String>,
    render_generated_html: bool,
}

fn substitute_list_pages_variables(
    template: &str,
    page: &FoundPageRow,
    index: usize,
    total: usize,
    context: &ListPagesSubstitutionContext<'_>,
) -> String {
    let slug = page.slug.as_deref().unwrap_or("");
    let title = page.title.as_deref().unwrap_or(slug);
    let generated_wikitext_title = preserve_list_pages_generated_text_typography(title);
    let title_linked = if slug.is_empty() {
        generated_wikitext_title.clone()
    } else {
        format!("[/{slug} {generated_wikitext_title}]")
    };
    let snapshot = context.snapshot_displays.get(&page.page_id);
    let created_by_snapshot =
        snapshot.and_then(|snapshot| snapshot.created_by_name.as_deref());
    let updated_by_snapshot =
        snapshot.and_then(|snapshot| snapshot.updated_by_name.as_deref());
    let commented_by_snapshot =
        snapshot.and_then(|snapshot| snapshot.commented_by_name.as_deref());
    let created_by = created_by_snapshot
        .map(str::to_owned)
        .or_else(|| {
            page.created_by.map(|user_id| {
                context
                    .user_displays
                    .get(&user_id)
                    .map(|user| user.name.clone())
                    .unwrap_or_else(|| user_id.to_string())
            })
        })
        .unwrap_or_default();
    let created_by_linked = created_by_snapshot
        .map(render_list_pages_snapshot_user)
        .or_else(|| {
            page.created_by.map(|user_id| {
                render_list_pages_wikidot_user(
                    user_id,
                    context.user_displays.get(&user_id),
                )
            })
        })
        .unwrap_or_default();
    let updated_by = updated_by_snapshot
        .map(str::to_owned)
        .or_else(|| {
            page.updated_by.map(|user_id| {
                context
                    .user_displays
                    .get(&user_id)
                    .map(|user| user.name.clone())
                    .unwrap_or_else(|| user_id.to_string())
            })
        })
        .unwrap_or_default();
    let commented_by = commented_by_snapshot.map(str::to_owned).unwrap_or_default();
    let created_at = snapshot
        .map(|snapshot| snapshot.created_at)
        .or(page.created_at);
    let updated_at = snapshot
        .map(|snapshot| snapshot.updated_at)
        .or(page.updated_at);
    let commented_at = snapshot.and_then(|snapshot| snapshot.commented_at);
    let comments = snapshot
        .map(|snapshot| snapshot.comments.to_string())
        .unwrap_or_default();
    let tags = page.tags.as_deref().unwrap_or(&[]);
    let visible_tags = tags
        .iter()
        .filter(|tag| is_list_pages_visible_tag(tag))
        .cloned()
        .collect::<Vec<_>>();
    let tags_text = visible_tags.join(" ");
    let rating = format_list_pages_rating(page.score);
    // The frozen corpus predates vote-count capture. Keep this value typed as
    // optional provenance and select the component's explicit zero-vote state
    // when it is absent; inventing a count from the net rating would create a
    // visibly plausible but false upvote/downvote ratio.
    let rating_votes = snapshot
        .and_then(|snapshot| snapshot.rating_votes)
        .unwrap_or(0)
        .to_string();
    let index = index.to_string();
    let total = total.to_string();
    let rendered_limit = context.rendered_limit.to_string();

    let substituted = LISTPAGES_VARIABLE_REGEX
        .replace_all(template, |captures: &regex::Captures<'_>| {
            match captures["name"].to_ascii_lowercase().as_str() {
                "title_linked" => title_linked.clone(),
                "linked_title" => title_linked.clone(),
                "title" => generated_wikitext_title.clone(),
                "name" | "slug" | "page_unix_name" | "fullname" | "full_slug"
                | "link" => slug.to_owned(),
                "created_by" | "createdby" => created_by.clone(),
                "created_by_linked" | "createdbylinked" | "author" => {
                    created_by_linked.clone()
                }
                "created_at" | "createdat" | "date" => format_list_pages_created_at(
                    created_at,
                    captures.name("format").map(|matched| matched.as_str()),
                    context.render_generated_html,
                ),
                "updated_by" | "updatedby" => updated_by.clone(),
                "updated_at" | "updatedat" => format_list_pages_created_at(
                    updated_at,
                    captures.name("format").map(|matched| matched.as_str()),
                    context.render_generated_html,
                ),
                "commented_by" | "commentedby" => commented_by.clone(),
                "commented_at" | "commentedat" => format_list_pages_created_at(
                    commented_at,
                    captures.name("format").map(|matched| matched.as_str()),
                    context.render_generated_html,
                ),
                "rating" => rating.clone(),
                "rating_votes" | "ratingvotes" => rating_votes.clone(),
                "comments" => comments.clone(),
                "tags" => tags_text.clone(),
                "tags_linked" | "tagslinked" => render_list_pages_tags(
                    &visible_tags,
                    captures.name("format").map(|matched| matched.as_str()),
                    context.render_generated_html,
                ),
                "form_data" | "form_raw" => captures
                    .name("argument")
                    .and_then(|matched| context.data_form_values.get(matched.as_str()))
                    .cloned()
                    .unwrap_or_default(),
                "content" => context
                    .page_wikitext
                    .map(|wikitext| {
                        wikidot_content_section(
                            wikitext,
                            captures
                                .name("argument")
                                .and_then(|matched| matched.as_str().parse().ok()),
                        )
                    })
                    .unwrap_or_default(),
                "index" => index.clone(),
                "total" => total.clone(),
                "limit" => rendered_limit.clone(),
                _ => captures
                    .get(0)
                    .map_or("", |matched| matched.as_str())
                    .to_owned(),
            }
        })
        .into_owned();

    RenderService::resolve_wikidot_parser_functions(
        &resolve_list_pages_signed_abs_expressions(&substituted),
    )
}

fn substitute_count_pages_variables(template: &str, total: usize) -> String {
    let total = total.to_string();
    let substituted = LISTPAGES_VARIABLE_REGEX
        .replace_all(template, |captures: &regex::Captures<'_>| {
            match captures["name"].to_ascii_lowercase().as_str() {
                "total" | "count" => total.clone(),
                _ => captures
                    .get(0)
                    .map_or("", |matched| matched.as_str())
                    .to_owned(),
            }
        })
        .into_owned();
    let mut substituted = RenderService::resolve_wikidot_parser_functions(
        &resolve_wikidot_numeric_ifexpr(&substituted),
    );
    RenderService::neutralize_authored_wikidot_compat_markers(&mut substituted);
    substituted
}

fn resolve_wikidot_numeric_ifexpr(value: &str) -> String {
    let literal_regions = LiteralRegionIndex::new(value);
    WIKIDOT_NUMERIC_IFEXPR_REGEX
        .replace_all(value, |captures: &regex::Captures<'_>| {
            let full_match = captures.get(0).expect("numeric ifexpr capture exists");
            if literal_regions.contains(full_match.start()) {
                return full_match.as_str().to_owned();
            }
            let left = captures["left"].parse::<f64>().ok();
            let right = captures["right"].parse::<f64>().ok();
            let Some(left) = left else {
                return captures
                    .get(0)
                    .map_or("", |matched| matched.as_str())
                    .to_owned();
            };
            let Some(right) = right else {
                return captures
                    .get(0)
                    .map_or("", |matched| matched.as_str())
                    .to_owned();
            };
            let matched = match &captures["op"] {
                ">" => left > right,
                ">=" => left >= right,
                "<" => left < right,
                "<=" => left <= right,
                "=" | "==" => (left - right).abs() <= f64::EPSILON,
                "!=" => (left - right).abs() > f64::EPSILON,
                _ => {
                    return captures
                        .get(0)
                        .map_or("", |matched| matched.as_str())
                        .to_owned();
                }
            };
            if matched {
                captures["when_true"].trim().to_owned()
            } else {
                captures["when_false"].trim().to_owned()
            }
        })
        .into_owned()
}

fn render_list_pages_tags(
    tags: &[String],
    path_prefix: Option<&str>,
    render_as_html: bool,
) -> String {
    let path_prefix = path_prefix
        .filter(|prefix| !prefix.trim().is_empty())
        .unwrap_or("/system:page-tags/tag/");
    tags.iter()
        .map(|tag| {
            let href = list_pages_tag_link_href(path_prefix, tag);
            if render_as_html {
                format!(
                    r#"<a href="{href}">{tag}</a>"#,
                    href = escape_list_pages_html_attr(&href),
                    tag = escape_list_pages_html_text(tag),
                )
            } else {
                format!("[{href} {tag}]", tag = escape_wikidot_link_text(tag))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn list_pages_tag_link_href(path_prefix: &str, tag: &str) -> String {
    let path_prefix = path_prefix.trim();
    let tag = tag.trim();
    if path_prefix.starts_with("http://")
        || path_prefix.starts_with("https://")
        || path_prefix.starts_with('/')
    {
        format!("{path_prefix}{tag}")
    } else {
        format!("/{path_prefix}{tag}")
    }
}

fn render_tag_cloud_box(tags: &[(String, usize)]) -> String {
    let max_count = tags.iter().map(|(_, count)| *count).max().unwrap_or(1);
    let mut output = String::from("[[div class=\"pages-tag-cloud-box\"]]\n");

    for (tag, count) in tags {
        let weight = if max_count <= 1 {
            1.0
        } else {
            0.5 + ((*count as f32 / max_count as f32) * 2.5)
        };
        let tag_path =
            format!("/system:page-tags/tag/{}", escape_list_pages_html_attr(tag));
        output.push_str(&format!(
            r#"[[span class="tag" style="font-size: {weight:.2}em;"]][{tag_path} {tag_text}][[/span]] "#,
            tag_text = escape_list_pages_html_text(tag),
        ));
    }

    output.push_str("\n[[/div]]");
    output
}

fn is_tag_cloud_visible_tag(tag: &str) -> bool {
    let tag = tag.trim();
    !tag.is_empty()
        && !tag.starts_with('_')
        && !tag.starts_with("codex-")
        && !tag.starts_with("branch-")
        && !tag.starts_with("feature-")
        && !matches!(
            tag,
            "declared-universe"
                | "declared-universe-include-support"
                | "verification"
                | "preview"
                | "ui-authoring"
                | "edited"
                | "fragment"
        )
}

fn is_list_pages_visible_tag(tag: &str) -> bool {
    let tag = tag.trim();
    !tag.is_empty() && !tag.starts_with('_')
}

fn render_list_pages_wikidot_user(
    user_id: i64,
    user: Option<&WikidotUserDisplay>,
) -> String {
    let Some(user) = user else {
        return user_id.to_string();
    };
    if !user.wikidot_profile {
        return escape_list_pages_html_text(&user.name);
    }
    let slug = user.slug.as_deref().unwrap_or(&user.name);
    format!(
        concat!(
            r#"<span class="printuser avatarhover">"#,
            r#"<a href="http://www.wikidot.com/user:info/{slug}" onclick="WIKIDOT.page.listeners.userInfo({user_id}); return false;">"#,
            r#"<img alt="{name}" class="small" src="http://www.wikidot.com/avatar.php?userid={user_id}&amp;size=small"/>"#,
            r#"</a><a href="http://www.wikidot.com/user:info/{slug}" onclick="WIKIDOT.page.listeners.userInfo({user_id}); return false;">{name}</a>"#,
            r#"</span>"#
        ),
        slug = escape_list_pages_html_attr(slug),
        user_id = user.user_id,
        name = escape_list_pages_html_text(&user.name),
    )
}

fn render_list_pages_snapshot_user(name: &str) -> String {
    escape_list_pages_html_text(name)
}

fn preserve_list_pages_generated_text_typography(value: &str) -> String {
    if !value.contains("...") {
        return value.to_owned();
    }
    let marker = list_pages_literal_ellipsis_marker();
    value.replace("...", &marker)
}

fn list_pages_literal_ellipsis_marker() -> String {
    format!(
        "{WIKIDOT_LISTPAGES_LITERAL_ELLIPSIS_SENTINEL_PREFIX}{}X",
        Uuid::new_v4().as_simple(),
    )
}

fn restore_list_pages_literal_ellipsis_markers(html: &str) -> String {
    WIKIDOT_LISTPAGES_LITERAL_ELLIPSIS_SENTINEL_REGEX
        .replace_all(html, "...")
        .into_owned()
}

fn format_list_pages_created_at(
    created_at: Option<time::OffsetDateTime>,
    format: Option<&str>,
    render_as_html: bool,
) -> String {
    let Some(created_at) = created_at else {
        return String::new();
    };
    let created_at = created_at
        .to_offset(time::UtcOffset::from_hms(9, 0, 0).expect("valid JST offset"));
    let format = format.unwrap_or("%e %b %Y, %H:%M");
    let display_format = format.split('|').next().unwrap_or(format);
    let text = format_wikidot_list_pages_date(created_at, display_format);
    let encoded_format = percent_encode_list_pages_class(format);
    if render_as_html {
        format!(
            r#"<span class="odate time_{} format_{}" style="cursor: help; display: inline;">{}</span>"#,
            created_at.unix_timestamp(),
            encoded_format,
            escape_list_pages_html_text(&text),
        )
    } else {
        format!(
            r#"<span class="odate time_{} format_{}" data-wikijump-compat-date="1" style="cursor: help; display: inline;">{}</span>"#,
            created_at.unix_timestamp(),
            encoded_format,
            escape_list_pages_html_text(&text),
        )
    }
}

fn register_generated_list_pages_html(
    value: String,
    compat_html: &mut CompatHtmlFragments,
) -> String {
    GENERATED_LISTPAGES_HTML_REGEX
        .replace_all(&value, |captures: &regex::Captures<'_>| {
            compat_html.push(captures[0].to_owned())
        })
        .into_owned()
}

fn resolve_list_pages_signed_abs_expressions(value: &str) -> String {
    let literal_regions = LiteralRegionIndex::new(value);
    WIKIDOT_LISTPAGES_SIGNED_ABS_EXPR_REGEX
        .replace_all(value, |captures: &regex::Captures<'_>| {
            let original = captures.get(0).map_or("", |matched| matched.as_str());
            let start = captures.get(0).map_or(0, |matched| matched.start());
            if literal_regions.contains(start) {
                return original.to_owned();
            }
            let Some(test_value) = captures
                .name("test")
                .and_then(|matched| matched.as_str().parse::<f64>().ok())
            else {
                return original.to_owned();
            };
            let Some(abs_value) = captures
                .name("abs")
                .and_then(|matched| matched.as_str().parse::<f64>().ok())
            else {
                return original.to_owned();
            };
            if (test_value - abs_value).abs() > f64::EPSILON
                && (test_value.abs() - abs_value).abs() > f64::EPSILON
            {
                return original.to_owned();
            }

            let sign = if test_value > -1.0 { "+" } else { "-" };
            let magnitude = format_list_pages_rating(Some(test_value.abs() as f32));
            format!("{sign}{magnitude}")
        })
        .into_owned()
}

fn escape_wikidot_link_text(value: &str) -> String {
    value.replace(']', r"\]")
}

fn wikidot_compat_link_marker() -> String {
    format!(
        "{WIKIDOT_COMPAT_LINK_SENTINEL_PREFIX}{}X",
        Uuid::new_v4().as_simple(),
    )
}

#[cfg(test)]
fn wikidot_compat_html_marker() -> String {
    format!(
        "{WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX}{}X",
        Uuid::new_v4().as_simple(),
    )
}

fn wikidot_color_span_marker() -> String {
    format!(
        "{WIKIDOT_COLOR_SPAN_SENTINEL_PREFIX}{}X",
        Uuid::new_v4().as_simple(),
    )
}

fn wikidot_inline_html_marker() -> String {
    format!(
        "{WIKIDOT_INLINE_HTML_SENTINEL_PREFIX}{}X",
        Uuid::new_v4().as_simple(),
    )
}

fn parse_wikidot_compat_color_descriptor<'a>(
    hashes: &str,
    descriptor: &'a str,
) -> Option<Cow<'a, str>> {
    match hashes.len() {
        2 => Some(Cow::Borrowed(descriptor)),
        3 if matches!(descriptor.len(), 3 | 6)
            && descriptor.bytes().all(|byte| byte.is_ascii_hexdigit()) =>
        {
            Some(Cow::Owned(format!("#{descriptor}")))
        }
        _ => None,
    }
}

fn render_wikidot_color_span_html(color: &str, body: &str) -> String {
    format!(
        r#"<span style="color: {color}">{body}</span>"#,
        color = escape_list_pages_html_attr(color),
        body = render_wikidot_protected_inline_body_html(body),
    )
}

fn render_wikidot_protected_inline_body_html(body: &str) -> String {
    let rendered = render_native_list_inline_wikidot_underlines(
        &render_native_list_inline_wikidot_strong(&render_native_list_inline_html(body)),
    );

    substitute_wikidot_protected_inline_typography(&rendered)
}

fn substitute_wikidot_protected_inline_typography(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(tag_start) = rest.find('<') {
        let (before, after_start) = rest.split_at(tag_start);
        output.push_str(&substitute_wikidot_protected_inline_text_typography(before));

        let Some(tag_end) = after_start.find('>') else {
            output.push_str(&substitute_wikidot_protected_inline_text_typography(
                after_start,
            ));
            return output;
        };
        let (tag, after_tag) = after_start.split_at(tag_end + 1);
        output.push_str(tag);
        rest = after_tag;
    }

    output.push_str(&substitute_wikidot_protected_inline_text_typography(rest));
    output
}

fn substitute_wikidot_protected_inline_text_typography(value: &str) -> String {
    let mut text = value.to_owned();
    ftml::preprocess(&mut text);
    substitute_wikidot_protected_inline_dashes(&text)
}

fn substitute_wikidot_protected_inline_dashes(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;

    while !rest.is_empty() {
        if rest.starts_with("[!--")
            && let Some(comment_end) = rest.find("--]")
        {
            let comment_end = comment_end + "--]".len();
            output.push_str(&rest[..comment_end]);
            rest = &rest[comment_end..];
            continue;
        }

        if rest.starts_with("--") {
            output.push('\u{2014}');
            rest = &rest["--".len()..];
            continue;
        }

        let character = rest
            .chars()
            .next()
            .expect("non-empty string has next character");
        output.push(character);
        rest = &rest[character.len_utf8()..];
    }

    output
}

fn wikidot_named_anchor(name: &str) -> String {
    format!(
        r#"<a name="{name}"></a>"#,
        name = escape_list_pages_html_attr(name),
    )
}

fn wikidot_current_page_anchor(label: &str) -> String {
    format!(
        r#"<a href="javascript:;">{label}</a>"#,
        label = escape_list_pages_html_text(label),
    )
}

fn wikidot_star_local_anchor(target: &str, label: &str) -> String {
    let target = target.trim();
    let href = if target.starts_with('/') {
        target.to_owned()
    } else {
        format!("/{target}")
    };

    format!(
        r#"<a href="{href}" target="_blank">{label}</a>"#,
        href = escape_list_pages_html_attr(&href),
        label = escape_list_pages_html_text(label),
    )
}

fn render_wikidot_wikipedia_link(target: &str, label: Option<&str>) -> String {
    build_wikidot_wikipedia_link(target, label).anchor
}

fn build_wikidot_wikipedia_link(
    target: &str,
    label: Option<&str>,
) -> ProtectedWikidotWikipediaLink {
    let (language, page) = wikidot_wikipedia_target(target);
    let href = wikidot_wikipedia_href(language, page);
    let label = label
        .filter(|value| !value.is_empty())
        .map(Cow::Borrowed)
        .unwrap_or_else(|| Cow::Owned(page.replace('_', " ")));
    let anchor = format!(
        r#"<a href="{href}" onclick="window.open(this.href, '_blank'); return false;">{label}</a>"#,
        href = escape_list_pages_html_attr(&href),
        label = escape_list_pages_html_text(&label),
    );
    ProtectedWikidotWikipediaLink { anchor, href }
}

fn wikidot_wikipedia_href(language: &str, page: &str) -> String {
    format!("http://{language}.wikipedia.org/wiki/{page}")
}

fn wikidot_wikipedia_target(target: &str) -> (&str, &str) {
    if let Some((language, page)) = target.split_once(':')
        && !page.is_empty()
        && (2..=3).contains(&language.len())
        && language
            .chars()
            .all(|character| character.is_ascii_alphabetic())
    {
        return (language, page);
    }

    ("en", target)
}

fn format_wikidot_list_pages_date(
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
            Some('Y') => output.push_str(&created_at.year().to_string()),
            Some('H') => output.push_str(&format!("{:02}", created_at.hour())),
            Some('M') => output.push_str(&format!("{:02}", created_at.minute())),
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

fn percent_encode_list_pages_class(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

fn render_native_bullet_list(lines: &[&str]) -> String {
    let items: Vec<_> = lines
        .iter()
        .filter_map(|line| native_bullet_list_item(line))
        .collect();
    let base_depth = items.iter().map(|(depth, _)| *depth).min().unwrap_or(0);
    let mut output = String::new();
    let mut current_depth = 0usize;
    let mut open_li = false;

    output.push_str(r#"<ul data-wikijump-compat-list="1">"#);
    output.push('\n');

    for (index, &(raw_depth, content)) in items.iter().enumerate() {
        let depth = raw_depth.saturating_sub(base_depth);
        let has_children = items
            .get(index + 1)
            .is_some_and(|(next_depth, _)| next_depth.saturating_sub(base_depth) > depth);

        if depth > current_depth {
            while current_depth < depth {
                output.push_str("<ul>\n");
                current_depth += 1;
            }
        } else if depth < current_depth {
            if open_li {
                output.push_str("</li>\n");
            }

            while current_depth > depth {
                output.push_str("</ul>\n</li>\n");
                current_depth -= 1;
            }
        } else if open_li {
            output.push_str("</li>\n");
        }

        output.push_str("<li>");
        output.push_str(&render_native_list_item_content(content, has_children));
        open_li = true;
    }

    if open_li {
        output.push_str("</li>\n");
    }

    while current_depth > 0 {
        output.push_str("</ul>\n</li>\n");
        current_depth -= 1;
    }

    output.push_str("</ul>\n");
    output
}

fn render_native_list_item_content(content: &str, has_children: bool) -> String {
    let rendered = render_native_list_inline_html(content);
    if has_children && !rendered.contains("<a ") {
        format!(
            r#"<a href="javascript:;">{rendered}
</a>"#
        )
    } else {
        rendered
    }
}

#[cfg(test)]
fn find_balanced_ul_end(html: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut cursor = 0usize;

    loop {
        let next_open = html[cursor..].find("<ul").map(|offset| cursor + offset);
        let next_close = html[cursor..].find("</ul>").map(|offset| cursor + offset);

        match (next_open, next_close) {
            (Some(open), Some(close)) if open < close => {
                depth += 1;
                cursor = open + 3;
            }
            (Some(open), None) => {
                depth += 1;
                cursor = open + 3;
            }
            (_, Some(close)) => {
                if depth == 0 {
                    return None;
                }

                depth -= 1;
                cursor = close + "</ul>".len();
                if depth == 0 {
                    return Some(cursor);
                }
            }
            (None, None) => return None,
        }
    }
}

fn native_bullet_list_item(line: &str) -> Option<(usize, &str)> {
    let trimmed_end = line.trim_end_matches(['\r', '\n']);
    let depth = trimmed_end
        .as_bytes()
        .iter()
        .take_while(|&&byte| byte == b' ')
        .count();
    trimmed_end[depth..]
        .strip_prefix("* ")
        .map(|content| (depth, content))
}

fn native_numbered_list_content(line: &str) -> Option<&str> {
    let trimmed = line.trim_start_matches(' ');
    trimmed
        .strip_prefix("# ")
        .map(|content| content.trim_end_matches(['\r', '\n']))
}

fn render_list_pages_numbered_rows(value: &str) -> String {
    let lines = value.split_inclusive('\n').collect::<Vec<_>>();
    let mut output = String::with_capacity(value.len());
    let mut index = 0;

    while index < lines.len() {
        let mut end = index;
        while end < lines.len() && native_numbered_list_content(lines[end]).is_some() {
            end += 1;
        }

        if end > index {
            output.push_str("<ol>\n");
            for line in &lines[index..end] {
                if let Some(content) = native_numbered_list_content(line) {
                    output.push_str("<li>");
                    output.push_str(&render_native_list_inline_html(content));
                    output.push_str("</li>\n");
                }
            }
            output.push_str("</ol>\n");
            index = end;
        } else {
            output.push_str(lines[index]);
            index += 1;
        }
    }

    output
}

fn render_list_pages_table_rows(value: &str) -> Option<String> {
    if !list_pages_body_has_table_rows(value) {
        return None;
    }

    let mut rows = Vec::new();
    for line in value.lines().filter(|line| !line.trim().is_empty()) {
        let trimmed = line.trim();
        let center = trimmed.starts_with("||=");
        let header = trimmed.starts_with("||~");
        let cell = trimmed
            .trim_start_matches("||=")
            .trim_start_matches("||~")
            .trim_end_matches("||")
            .trim();
        rows.push((header, center, render_list_pages_table_inline_html(cell)));
    }

    if rows.is_empty() {
        return None;
    }

    let mut output = String::from(
        "<table class=\"wiki-content-table\" data-wikijump-compat-listpages=\"1\">",
    );
    for (header, center, cell) in rows {
        output.push_str("<tr>");
        let tag = if header { "th" } else { "td" };
        output.push('<');
        output.push_str(tag);
        if center {
            output.push_str(" style=\"text-align: center;\"");
        }
        output.push('>');
        output.push_str(&cell);
        output.push_str("</");
        output.push_str(tag);
        output.push_str("></tr>");
    }
    output.push_str("</table>");
    Some(output)
}

fn list_pages_body_has_table_rows(value: &str) -> bool {
    let mut any = false;
    for line in value.lines().filter(|line| !line.trim().is_empty()) {
        any = true;
        let trimmed = line.trim();
        if !trimmed.starts_with("||") || !trimmed.ends_with("||") {
            return false;
        }
        if !trimmed.starts_with("||=") && !trimmed.starts_with("||~") {
            return false;
        }
    }
    any
}

fn render_list_pages_table_inline_html(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut strong = false;
    for segment in value.split("**") {
        if strong {
            output.push_str("<strong>");
        }
        push_list_pages_table_inline_segment(&mut output, segment);
        if strong {
            output.push_str("</strong>");
        }
        strong = !strong;
    }
    output
}

fn push_list_pages_table_inline_segment(output: &mut String, value: &str) {
    let mut rest = value;
    while let Some(start) = rest.find('<') {
        let (before, after_start) = rest.split_at(start);
        output.push_str(&escape_list_pages_html_text(before));
        if let Some(end) = after_start.find('>') {
            let (tag, after_tag) = after_start.split_at(end + 1);
            if let Some(tag) = sanitize_wikidot_compat_inline_tag(tag) {
                output.push_str(&tag);
            } else {
                output.push_str(&escape_list_pages_html_text(tag));
            }
            rest = after_tag;
        } else {
            output.push_str(&escape_list_pages_html_text(after_start));
            return;
        }
    }
    output.push_str(&escape_list_pages_html_text(rest));
}

fn render_native_list_inline_html(value: &str) -> String {
    render_native_list_inline_html_with_titles(value, None)
}

fn render_native_list_inline_html_with_titles(
    value: &str,
    link_titles: Option<&WikidotCompatLinkTitleMap>,
) -> String {
    let escaped = render_native_list_inline_wikidot_spans(value);
    let with_quadruple_links = WIKIDOT_QUADRUPLE_LINK_REGEX
        .replace_all(&escaped, |captures: &regex::Captures<'_>| {
            render_native_list_page_link(&captures["target"], None, link_titles)
        })
        .into_owned();
    let with_labeled_links = WIKIDOT_LABELED_LINK_REGEX
        .replace_all(&with_quadruple_links, |captures: &regex::Captures<'_>| {
            render_native_list_page_link(
                &captures["target"],
                Some(&captures["label"]),
                link_titles,
            )
        })
        .into_owned();
    let with_unlabeled_links = WIKIDOT_UNLABELED_LINK_REGEX
        .replace_all(&with_labeled_links, |captures: &regex::Captures<'_>| {
            render_native_list_page_link(&captures["target"], None, link_titles)
        })
        .into_owned();
    let with_local_links = WIKIDOT_LOCAL_LINK_REGEX
        .replace_all(&with_unlabeled_links, |captures: &regex::Captures<'_>| {
            render_native_list_page_link(
                &captures["target"],
                Some(&captures["label"]),
                link_titles,
            )
        })
        .into_owned();
    let with_user_links = WIKIDOT_USER_INLINE_REGEX
        .replace_all(&with_local_links, |captures: &regex::Captures<'_>| {
            render_native_list_wikidot_user(&captures["name"])
        })
        .into_owned();
    let with_wikipedia_links = WIKIDOT_WIKIPEDIA_LINK_REGEX
        .replace_all(&with_user_links, |captures: &regex::Captures<'_>| {
            render_wikidot_wikipedia_link(
                &captures["target"],
                captures.name("label").map(|matched| matched.as_str()),
            )
        })
        .into_owned();

    let with_external_links = WIKIDOT_EXTERNAL_LINK_REGEX
        .replace_all(&with_wikipedia_links, |captures: &regex::Captures<'_>| {
            format!(
                r#"<a href="{url}">{label}</a>"#,
                url = escape_list_pages_html_attr(&captures["url"]),
                label = captures["label"].to_owned(),
            )
        })
        .into_owned();

    render_native_list_inline_wikidot_italics(&with_external_links)
}

fn render_native_list_inline_wikidot_italics(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(tag_start) = rest.find('<') {
        let (before, after_start) = rest.split_at(tag_start);
        output.push_str(&render_native_list_text_italics(before));

        let Some(tag_end) = after_start.find('>') else {
            output.push_str(&render_native_list_text_italics(after_start));
            return output;
        };
        let (tag, after_tag) = after_start.split_at(tag_end + 1);
        output.push_str(tag);
        rest = after_tag;
    }

    output.push_str(&render_native_list_text_italics(rest));
    output
}

fn render_native_list_text_italics(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(open) = find_wikidot_italic_open(rest) {
        output.push_str(&rest[..open]);
        let after_open = &rest[open + "//".len()..];
        let Some(close) = find_wikidot_italic_close(after_open) else {
            output.push_str(&rest[open..]);
            return output;
        };

        output.push_str("<em>");
        output.push_str(&after_open[..close]);
        output.push_str("</em>");
        rest = &after_open[close + "//".len()..];
    }

    output.push_str(rest);
    output
}

fn render_native_list_inline_wikidot_strong(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(tag_start) = rest.find('<') {
        let (before, after_start) = rest.split_at(tag_start);
        output.push_str(&render_native_list_text_strong(before));

        let Some(tag_end) = after_start.find('>') else {
            output.push_str(&render_native_list_text_strong(after_start));
            return output;
        };
        let (tag, after_tag) = after_start.split_at(tag_end + 1);
        output.push_str(tag);
        rest = after_tag;
    }

    output.push_str(&render_native_list_text_strong(rest));
    output
}

fn render_native_list_text_strong(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(open) = rest.find("**") {
        output.push_str(&rest[..open]);
        let after_open = &rest[open + "**".len()..];
        let Some(close) = after_open.find("**") else {
            output.push_str(&rest[open..]);
            return output;
        };

        output.push_str("<strong>");
        output.push_str(&after_open[..close]);
        output.push_str("</strong>");
        rest = &after_open[close + "**".len()..];
    }

    output.push_str(rest);
    output
}

fn render_native_list_inline_wikidot_underlines(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(tag_start) = rest.find('<') {
        let (before, after_start) = rest.split_at(tag_start);
        output.push_str(&render_native_list_text_underlines(before));

        let Some(tag_end) = after_start.find('>') else {
            output.push_str(&render_native_list_text_underlines(after_start));
            return output;
        };
        let (tag, after_tag) = after_start.split_at(tag_end + 1);
        output.push_str(tag);
        rest = after_tag;
    }

    output.push_str(&render_native_list_text_underlines(rest));
    output
}

fn render_native_list_text_underlines(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(open) = rest.find("__") {
        output.push_str(&rest[..open]);
        let after_open = &rest[open + "__".len()..];
        let Some(close) = after_open.find("__") else {
            output.push_str(&rest[open..]);
            return output;
        };

        output.push_str("<u>");
        output.push_str(&after_open[..close]);
        output.push_str("</u>");
        rest = &after_open[close + "__".len()..];
    }

    output.push_str(rest);
    output
}

fn find_wikidot_italic_open(value: &str) -> Option<usize> {
    let mut cursor = 0usize;
    while let Some(offset) = value[cursor..].find("//") {
        let marker = cursor + offset;
        let previous = value[..marker].chars().next_back();
        let next = value[marker + "//".len()..].chars().next();
        if previous == Some(':')
            || next.is_none_or(|character| character.is_whitespace() || character == '/')
        {
            cursor = marker + "//".len();
            continue;
        }
        return Some(marker);
    }
    None
}

fn find_wikidot_italic_close(value: &str) -> Option<usize> {
    let mut cursor = 0usize;
    while let Some(offset) = value[cursor..].find("//") {
        let marker = cursor + offset;
        let previous = value[..marker].chars().next_back();
        let next = value[marker + "//".len()..].chars().next();
        if previous.is_none_or(char::is_whitespace) || next == Some('/') {
            cursor = marker + "//".len();
            continue;
        }
        return Some(marker);
    }
    None
}

fn render_native_list_inline_wikidot_spans(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(start) = rest.find("[[span") {
        let (before, marker_start) = rest.split_at(start);
        output.push_str(&escape_list_pages_html_text(before));

        let Some(marker_end) = marker_start.find("]]") else {
            output.push_str(&escape_list_pages_html_text(marker_start));
            return output;
        };
        let marker = &marker_start[..marker_end + 2];
        let after_marker = &marker_start[marker_end + 2..];

        let Some(close_start) = find_matching_wikidot_span_close(after_marker) else {
            output.push_str(&escape_list_pages_html_text(marker));
            rest = after_marker;
            continue;
        };

        if let Some(open_tag) = wikidot_inline_span_marker_open(marker) {
            output.push_str(&open_tag);
            output.push_str(&render_native_list_inline_wikidot_spans(
                &after_marker[..close_start],
            ));
            output.push_str("</span>");
            rest = &after_marker[close_start + "[[/span]]".len()..];
        } else {
            output.push_str(&escape_list_pages_html_text(marker));
            rest = after_marker;
        }
    }

    output.push_str(&escape_list_pages_html_text(rest));
    output
}

fn find_matching_wikidot_span_close(value: &str) -> Option<usize> {
    let mut depth = 1_usize;
    let mut offset = 0_usize;

    while offset < value.len() {
        let next_open = value[offset..].find("[[span").map(|index| offset + index);
        let next_close = value[offset..]
            .find("[[/span]]")
            .map(|index| offset + index);

        match (next_open, next_close) {
            (None, Some(close)) => {
                depth -= 1;
                if depth == 0 {
                    return Some(close);
                }
                offset = close + "[[/span]]".len();
            }
            (Some(open), Some(close)) if close <= open => {
                depth -= 1;
                if depth == 0 {
                    return Some(close);
                }
                offset = close + "[[/span]]".len();
            }
            (Some(open), _) => {
                let marker_end = value[open..].find("]]")?;
                let marker_end = open + marker_end + 2;
                if next_close.is_some_and(|close| marker_end > close) {
                    offset = open + "[[span".len();
                    continue;
                }

                let marker = &value[open..marker_end];
                if wikidot_inline_span_marker_open(marker).is_some() {
                    depth += 1;
                }
                offset = marker_end;
            }
            (None, None) => return None,
        }
    }

    None
}

fn wikidot_inline_span_marker_open(marker: &str) -> Option<String> {
    let marker = marker.trim();
    if !marker.ends_with("]]") {
        return None;
    }

    let inner = marker.strip_prefix("[[")?.strip_suffix("]]")?.trim();
    if inner.len() < "span".len() || !inner[.."span".len()].eq_ignore_ascii_case("span") {
        return None;
    }
    if inner.len() > "span".len()
        && !inner
            .as_bytes()
            .get("span".len())
            .is_some_and(u8::is_ascii_whitespace)
    {
        return None;
    }
    if inner.contains(['<', '>']) {
        return None;
    }

    sanitize_wikidot_compat_inline_tag(&format!("<{inner}>"))
}

fn render_native_list_page_link(
    target: &str,
    label: Option<&str>,
    link_titles: Option<&WikidotCompatLinkTitleMap>,
) -> String {
    let target = target.trim();
    let label = label
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| {
            native_list_page_link_title_label(target, link_titles)
                .unwrap_or_else(|| native_list_page_link_default_label(target))
        });
    let href = native_list_page_link_href(target);
    format!(
        r#"<a href="{href}">{label}</a>"#,
        href = escape_list_pages_html_attr(&href),
        label = label,
    )
}

fn native_list_page_link_title_label(
    target: &str,
    link_titles: Option<&WikidotCompatLinkTitleMap>,
) -> Option<String> {
    let slug = native_list_page_link_slug(target)?;
    link_titles?
        .get(&slug)
        .map(|title| escape_list_pages_html_text(title))
}

fn native_list_page_link_href(target: &str) -> String {
    if target.starts_with("http://") || target.starts_with("https://") {
        return target.to_owned();
    }

    let mut slug = String::with_capacity(target.len());
    let mut previous_dash = false;
    for character in target.trim().chars() {
        if character.is_whitespace() || character == '_' {
            if !previous_dash {
                slug.push('-');
                previous_dash = true;
            }
        } else {
            for lowercase in character.to_lowercase() {
                slug.push(lowercase);
            }
            previous_dash = character == '-';
        }
    }

    format!("/{}", slug.trim_matches('-'))
}

fn native_list_page_link_slug(target: &str) -> Option<String> {
    let target = target.trim();
    if target.is_empty()
        || target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with('#')
        || target.starts_with(':')
        || target.contains(['?', '&', '=', '#', '<', '>', '"', '\''])
    {
        return None;
    }

    let href = native_list_page_link_href(target);
    let slug = href.strip_prefix('/')?.trim_matches('-');
    if slug.is_empty()
        || slug.len() > 256
        || slug.contains('/')
        || !slug.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | ':')
        })
    {
        return None;
    }

    Some(slug.to_owned())
}

fn collect_wikidot_compat_empty_label_link_slugs(wikitext: &str) -> BTreeSet<String> {
    let mut slugs = BTreeSet::new();
    for captures in WIKIDOT_QUADRUPLE_LINK_REGEX.captures_iter(wikitext) {
        if let Some(slug) = native_list_page_link_slug(&captures["target"]) {
            slugs.insert(slug);
        }
        if slugs.len() >= MAX_WIKIDOT_COMPAT_FALLBACK_TITLE_LINKS {
            return slugs;
        }
    }
    for captures in WIKIDOT_UNLABELED_LINK_REGEX.captures_iter(wikitext) {
        if let Some(slug) = native_list_page_link_slug(&captures["target"]) {
            slugs.insert(slug);
        }
        if slugs.len() >= MAX_WIKIDOT_COMPAT_FALLBACK_TITLE_LINKS {
            return slugs;
        }
    }
    for captures in WIKIDOT_LABELED_LINK_REGEX.captures_iter(wikitext) {
        if !captures["label"].trim().is_empty() {
            continue;
        }
        if let Some(slug) = native_list_page_link_slug(&captures["target"]) {
            slugs.insert(slug);
        }
        if slugs.len() >= MAX_WIKIDOT_COMPAT_FALLBACK_TITLE_LINKS {
            return slugs;
        }
    }

    slugs
}

fn native_list_page_link_default_label(target: &str) -> String {
    if target.starts_with("http://") || target.starts_with("https://") {
        return target.to_owned();
    }
    if target.contains(char::is_whitespace) {
        return target.to_owned();
    }
    if let Some(label) = native_list_scp_style_page_link_default_label(target) {
        return label;
    }

    target
        .split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut word = String::new();
                    for uppercase in first.to_uppercase() {
                        word.push(uppercase);
                    }
                    word.push_str(chars.as_str());
                    word
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn native_list_scp_style_page_link_default_label(target: &str) -> Option<String> {
    let mut parts = target.trim().split('-');
    let prefix = parts.next()?;
    if !prefix.eq_ignore_ascii_case("scp") {
        return None;
    }

    let number = parts.next()?;
    if number.is_empty() || !number.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }

    let mut label = format!("SCP-{number}");
    for part in parts {
        if part.is_empty()
            || !part
                .chars()
                .all(|character| character.is_ascii_alphanumeric())
        {
            return None;
        }
        label.push('-');
        label.push_str(&part.to_ascii_uppercase());
    }

    Some(label)
}

fn render_native_list_wikidot_user(name: &str) -> String {
    let name = name.trim();
    format!(
        concat!(
            r#"<span class="printuser">"#,
            r#"<a href="http://www.wikidot.com/user:info/{slug}">{name}</a>"#,
            r#"</span>"#
        ),
        slug = escape_list_pages_html_attr(name),
        name = escape_list_pages_html_text(name),
    )
}

fn escape_list_pages_html_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_list_pages_html_attr(value: &str) -> String {
    escape_list_pages_html_text(value).replace('"', "&quot;")
}

fn decode_wikidot_email_html_entities(value: &str) -> String {
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

fn decode_numeric_html_entity(entity: &str) -> Option<char> {
    if let Some(hex) = entity
        .strip_prefix("#x")
        .or_else(|| entity.strip_prefix("#X"))
    {
        return u32::from_str_radix(hex, 16).ok().and_then(char::from_u32);
    }

    let decimal = entity.strip_prefix('#')?;
    decimal.parse::<u32>().ok().and_then(char::from_u32)
}

fn sanitize_wikidot_compat_inline_tag(tag: &str) -> Option<String> {
    match tag {
        "</span>" | "</a>" => return Some(tag.to_owned()),
        "<br>" | "<br/>" | "<br />" => return Some("<br>".to_owned()),
        _ => {}
    }

    let inner = tag.strip_prefix('<')?.strip_suffix('>')?.trim();
    let inner = inner.strip_suffix('/').map_or(inner, str::trim_end);
    let name_end = inner
        .find(|character: char| character.is_ascii_whitespace())
        .unwrap_or(inner.len());
    let name = inner[..name_end].to_ascii_lowercase();
    if !matches!(name.as_str(), "span" | "a" | "img") {
        return None;
    }

    let mut output = String::new();
    output.push('<');
    output.push_str(&name);

    let mut rest = &inner[name_end..];
    while let Some((attr_name, attr_value, after_attr)) =
        parse_wikidot_compat_html_attr(rest)
    {
        rest = after_attr;
        let Some(value) = sanitize_wikidot_compat_inline_attr(
            name.as_str(),
            attr_name.as_str(),
            attr_value.as_str(),
        ) else {
            continue;
        };
        output.push(' ');
        output.push_str(&attr_name.to_ascii_lowercase());
        output.push_str(r#"=""#);
        output.push_str(&escape_list_pages_html_attr(&value));
        output.push('"');
    }

    output.push('>');
    Some(output)
}

fn parse_wikidot_compat_html_attr(input: &str) -> Option<(String, String, &str)> {
    let rest = input.trim_start();
    if rest.is_empty() || rest.starts_with('/') {
        return None;
    }

    let name_end = rest.find(|character: char| {
        character.is_ascii_whitespace() || matches!(character, '=' | '/' | '>')
    })?;
    if name_end == 0 {
        return None;
    }
    let name = &rest[..name_end];
    if !name.chars().all(|character| {
        character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | ':')
    }) {
        return None;
    }

    let rest = rest[name_end..].trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let mut chars = rest.chars();
    let quote = chars.next()?;
    if matches!(quote, '"' | '\'') {
        let value_start = quote.len_utf8();
        let value_rest = &rest[value_start..];
        let value_end = value_rest.find(quote)?;
        let value = &value_rest[..value_end];
        let after = &value_rest[value_end + quote.len_utf8()..];
        return Some((name.to_owned(), value.to_owned(), after));
    }

    let value_end = rest
        .find(|character: char| {
            character.is_ascii_whitespace() || matches!(character, '/' | '>')
        })
        .unwrap_or(rest.len());
    if value_end == 0 {
        return None;
    }
    Some((
        name.to_owned(),
        rest[..value_end].to_owned(),
        &rest[value_end..],
    ))
}

fn sanitize_wikidot_compat_inline_attr(
    tag_name: &str,
    attr_name: &str,
    value: &str,
) -> Option<String> {
    let attr_name = attr_name.to_ascii_lowercase();
    if attr_name.starts_with("on") {
        return None;
    }

    match (tag_name, attr_name.as_str()) {
        ("span", "class") | ("a", "class") | ("img", "class") => Some(value.to_owned()),
        ("span", "title") | ("a", "title") | ("img", "title") | ("img", "alt") => {
            Some(value.to_owned())
        }
        ("span", "style") | ("img", "style") => {
            wikidot_compat_safe_inline_style(value).then(|| value.to_owned())
        }
        ("a", "href") => {
            wikidot_compat_safe_inline_url(value, true).then(|| value.to_owned())
        }
        ("a", "rel") => Some(value.to_owned()),
        ("a", "target") if matches!(value, "_blank" | "_self" | "_parent" | "_top") => {
            Some(value.to_owned())
        }
        ("img", "src") => {
            wikidot_compat_safe_inline_url(value, false).then(|| value.to_owned())
        }
        ("img", "width") | ("img", "height") => value
            .chars()
            .all(|character| character.is_ascii_digit() || character == '%')
            .then(|| value.to_owned()),
        _ => None,
    }
}

fn wikidot_compat_safe_inline_url(value: &str, allow_mailto: bool) -> bool {
    let value =
        value.trim_start_matches(|character: char| character.is_ascii_whitespace());
    if value.is_empty()
        || value
            .chars()
            .any(|character| matches!(character, '\0'..='\u{1f}' | '\u{7f}'))
    {
        return false;
    }

    let lower = value.to_ascii_lowercase();
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with('/')
        || lower.starts_with('#')
        || (allow_mailto && lower.starts_with("mailto:"))
    {
        return true;
    }

    !lower.contains(':')
}

fn wikidot_compat_safe_inline_style(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    !lower.contains("javascript:")
        && !lower.contains("expression")
        && !lower.contains("url(")
        && !lower.contains("behavior")
        && !lower.contains("-moz-binding")
        && !value.chars().any(|character| {
            matches!(
                character,
                '<' | '>' | '"' | '\'' | '\0'..='\u{1f}' | '\u{7f}'
            )
        })
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

fn render_read_only_rate_module(score: ftml::data::ScoreValue, language: &str) -> String {
    let score = format_score_value(score);
    let labels = wikidot_rate_module_labels(language);

    format!(
        concat!(
            "[[div class=\"page-rate-widget-box\"]]",
            "[[span class=\"rate-points\"]]{}",
            "[[span class=\"number prw54353\"]]{}[[/span]]",
            "[[/span]]",
            "[[span class=\"rateup btn btn-default\"]]",
            "[[a href=\"javascript:;\" onclick=\"WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, 1)\" title=\"{}\"]]+[[/a]]",
            "[[/span]]",
            "[[span class=\"ratedown btn btn-default\"]]",
            "[[a href=\"javascript:;\" onclick=\"WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, -1)\" title=\"{}\"]]–[[/a]]",
            "[[/span]]",
            "[[span class=\"cancel btn btn-default\"]]",
            "[[a href=\"javascript:;\" onclick=\"WIKIDOT.modules.PageRateWidgetModule.listeners.cancelVote(event)\" title=\"{}\"]]x[[/a]]",
            "[[/span]]",
            "[[/div]]"
        ),
        labels.rating_prefix,
        score,
        labels.up_title,
        labels.down_title,
        labels.cancel_title,
    )
}

#[derive(Debug)]
struct WikidotRateModuleLabels {
    rating_prefix: &'static str,
    up_title: &'static str,
    down_title: &'static str,
    cancel_title: &'static str,
}

fn wikidot_rate_module_labels(language: &str) -> WikidotRateModuleLabels {
    if is_japanese_wikidot_locale(language) {
        WikidotRateModuleLabels {
            rating_prefix: "評価:\u{00a0}",
            up_title: "好き",
            down_title: "好きじゃない",
            cancel_title: "投票を取り消す",
        }
    } else {
        WikidotRateModuleLabels {
            rating_prefix: "rating: ",
            up_title: "I like it",
            down_title: "I don't like it",
            cancel_title: "Cancel my vote",
        }
    }
}

fn is_japanese_wikidot_locale(language: &str) -> bool {
    let language = language.replace('_', "-").to_ascii_lowercase();
    matches!(language.as_str(), "ja" | "jp") || language.starts_with("ja-")
}

fn wikidot_module_argument<'a>(head: &'a str, name: &str) -> Option<&'a str> {
    for captures in LISTPAGES_ARGUMENT_REGEX.captures_iter(head) {
        let key = captures.name("key")?.as_str();
        if !key.eq_ignore_ascii_case(name) {
            continue;
        }

        return captures
            .name("double")
            .or_else(|| captures.name("single"))
            .or_else(|| captures.name("bare"))
            .map(|mtch| mtch.as_str());
    }

    None
}

fn render_backlinks_module_box(pages: &[BacklinksModulePage]) -> String {
    let mut output = String::from(
        "\n<div class=\"backlinks-module-box\" data-wikijump-compat-backlinks=\"1\"><ul>",
    );

    for page in pages {
        output.push_str(r#"<li><a href="/"#);
        output.push_str(&escape_list_pages_html_attr(&page.slug));
        output.push_str(r#"">"#);
        output.push_str(&escape_list_pages_html_text(&page.title));
        output.push_str("</a></li>");
    }

    output.push_str("</ul></div>\n");
    output
}

fn render_members_module_placeholder(group: &str) -> String {
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

fn render_new_page_module(head: &str) -> String {
    let size = wikidot_module_argument(head, "size")
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|value| (1..=128).contains(value))
        .unwrap_or(30);
    let button = wikidot_module_argument(head, "button")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("new page");

    format!(
        concat!(
            r#"<form class="new-page-box" data-wikijump-compat-new-page="1" action="javascript:;" method="post">"#,
            r#"<input class="text" type="text" name="page" size="{size}">"#,
            r#"<input class="button" type="button" value="{button}">"#,
            r#"</form>"#,
        ),
        size = size,
        button = escape_list_pages_html_attr(button),
    )
}

fn render_clone_module(head: &str) -> String {
    let button = wikidot_module_argument(head, "button")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Clone this site");

    format!(
        r#"<a class="button" data-wikijump-compat-clone="1" href="javascript:;">{button}</a>"#,
        button = escape_list_pages_html_text(button),
    )
}

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

fn push_escaped_html(output: &mut String, value: &str) {
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
struct RenderInnerOutput {
    html_output: HtmlOutput,
    errors: Vec<ParseError>,
    compiled_hash: TextHash,
}

#[derive(Debug)]
struct FtmlRenderOutput {
    html_output: HtmlOutput,
    errors: Vec<ParseError>,
    html_block_texts: Vec<String>,
    code_blocks: Vec<CodeBlock<'static>>,
}

#[derive(Debug)]
struct WikidotCompatibilityFallbackOutput {
    body: String,
    html_block_texts: Vec<String>,
}

impl WikidotCompatibilityFallbackOutput {
    fn body(body: String) -> Self {
        Self {
            body,
            html_block_texts: Vec::new(),
        }
    }
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
    expanded_include_count: usize,
}

#[derive(Clone, Copy, Debug)]
struct IncludeExpansionOptions {
    expand_wikidot_image_blocks: bool,
    max_total_includes: usize,
}

#[derive(Debug)]
struct IncludeExpansionContext<'a> {
    current_site_id: i64,
    current_site_slug: String,
    page_info: &'a PageInfo<'a>,
    settings: &'a WikitextSettings,
    expand_wikidot_image_blocks: bool,
    max_total_includes: usize,
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
        Ok(wikidot_no_such_include_replacement(page_ref))
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
        Ok(wikidot_no_such_include_replacement(page_ref))
    }
}

fn wikidot_no_such_include_replacement(page_ref: &PageRef) -> Cow<'static, str> {
    if is_optional_no_visible_wikidot_include(page_ref) {
        Cow::Borrowed("")
    } else {
        Cow::Owned(format!("No such page: {page_ref}"))
    }
}

fn is_optional_no_visible_wikidot_include(page_ref: &PageRef) -> bool {
    let Some(site) = page_ref.site() else {
        return false;
    };
    let page = page_ref.page();
    (site.eq_ignore_ascii_case("drizzles") && page.eq_ignore_ascii_case("raven"))
        || (site.eq_ignore_ascii_case("crom") && page.eq_ignore_ascii_case("pixel"))
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
    for _ in 0..MAX_INCLUDE_EXPANSION_DEPTH {
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
                let changed = value != mtch.as_str();
                matches.push((value, mtch.range(), changed));
            }
        }

        if matches.is_empty() {
            break;
        }

        let changed = matches.iter().any(|(_, _, changed)| *changed);
        matches.reverse();
        for (value, range, _) in matches {
            content.replace_range(range, &value);
        }
        if !changed {
            break;
        }
    }
}

fn trim_include_variable_value(value: &str) -> &str {
    value.trim_end_matches([' ', '\t', '\r', '\n'])
}

fn default_include_variable_value(name: &str) -> Option<String> {
    match name.to_ascii_lowercase().as_str() {
        "author" => Some("%%created_by%%".to_owned()),
        "shadow" => Some("no".to_owned()),
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

fn wikidot_tag_conditions_match(spec: &str, tags: &[Cow<'_, str>]) -> bool {
    let mut required = true;
    let mut prohibited = true;
    let mut present = false;
    let mut had_present = false;

    for raw_condition in spec.split_whitespace() {
        let (operator, tag) = raw_condition.split_at(usize::from(
            raw_condition.starts_with('+') || raw_condition.starts_with('-'),
        ));
        if tag.is_empty() {
            continue;
        }

        let has_tag = tags.iter().any(|value| value.as_ref() == tag);
        match operator {
            "+" => required &= has_tag,
            "-" => prohibited &= !has_tag,
            _ => {
                had_present = true;
                present |= has_tag;
            }
        }
    }

    if !had_present {
        present = true;
    }

    required && prohibited && present
}

fn apply_basalt_shell_compatibility(html: &mut String) {
    if !html.contains("theme%3Abasalt") && !html.contains("basalt-bedrock-min.css") {
        return;
    }

    html.push_str(
        r#"<style>
#top-bar {
    display: contents;
}
#top-bar ul ul {
    display: flex !important;
    visibility: visible !important;
    position: static !important;
}
#top-bar > div > ul > li > a,
.mobile-top-bar > p > ul > li > a {
    text-transform: uppercase;
}
#header h2 {
    display: none !important;
}
#side-bar {
    display: block !important;
    visibility: visible !important;
    left: -272px !important;
}
#side-bar .heading p {
    text-transform: uppercase;
}
#main-content {
    margin-left: auto !important;
    margin-right: auto !important;
    margin-top: -12rem !important;
}
#page-info {
    text-transform: uppercase;
}
#page-options-bottom.page-options-bottom {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
}
#page-options-bottom.page-options-bottom > a {
    display: flex;
}
.admo-rate_splash .page-rate-widget-box .rate-points {
    text-transform: uppercase;
}
.admo-rate_splash .page-rate-widget-box .cancel,
.admo-rate_splash .page-rate-widget-box .cancel a {
    text-transform: none;
}
</style>"#,
    );
}

fn apply_blankstyle_shell_compatibility(html: &mut String) {
    if !html.contains("theme%3Ablankstyle")
        && !html.contains("theme:blankstyle")
        && !html.contains("43Head.png")
    {
        return;
    }

    html.push_str(
        r#"<style>
#top-bar .mobile-top-bar {
    display: block !important;
}
#top-bar .mobile-top-bar > ul,
#top-bar .mobile-top-bar > p {
    display: none !important;
}
#top-bar div.open-menu a {
    display: block !important;
    position: fixed !important;
    top: 15px !important;
    left: 15px !important;
    width: 32px !important;
    height: 32px !important;
    line-height: 32px !important;
    text-align: center !important;
    z-index: 30 !important;
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

#[allow(dead_code)]
fn public_url_port_suffix(port: Option<u16>) -> String {
    port.map(|port| format!(":{port}")).unwrap_or_default()
}

fn rendered_wikidot_mailform_attribute(head: &str, name: &str) -> Option<String> {
    let prefix = format!("{name}=&quot;");
    let start = head.find(&prefix)? + prefix.len();
    let rest = &head[start..];
    let end = rest.find("&quot;")?;
    Some(rest[..end].to_owned())
}

#[cfg(test)]
mod tests {
    use super::{
        CollectingIncluder, CompatHtmlFragments, CorpusReplayExpandedWikitext,
        CorpusReplayPreparationStage, LISTPAGES_MODULE_REGEX, ListPagesSnapshotDisplay,
        ListPagesSubstitutionContext, MAX_FTML_COMPAT_COLLAPSIBLE_BLOCKS,
        MAX_FTML_COMPAT_DENSE_PARSE_SCORE, MAX_FTML_COMPAT_PARSE_BYTES,
        MAX_LISTPAGES_RENDER_SCAN_ROWS, MAX_WIKIDOT_SIMPLE_IF_PASSES,
        MIN_DENSE_FTML_COMPAT_RENDER_TIMEOUT_SECS, MIN_FTML_COMPAT_TABBED_FALLBACK_BYTES,
        MIN_FTML_COMPAT_TABBED_FALLBACK_MARKERS, OrderBySelector, OrderProperty,
        PreparedIncluder, RenderContext, RenderService,
        WIKIDOT_COLOR_SPAN_SENTINEL_PREFIX, WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX,
        WIKIDOT_COMPAT_LINK_SENTINEL_PREFIX, WIKIDOT_INLINE_HTML_SENTINEL_PREFIX,
        WIKIDOT_LISTPAGES_LITERAL_ELLIPSIS_SENTINEL_PREFIX,
        WIKIDOT_WIKIPEDIA_LINK_SENTINEL_PREFIX, WikidotCompatLinkTitleMap,
        WikidotUserDisplay, count_pages_exact_count_render_diagnostics,
        count_pages_should_remain_literal, format_list_pages_created_at, include_error,
        list_pages_body_is_no_visible_tracking_markup,
        list_pages_body_uses_content_variable, list_pages_body_variables_supported,
        list_pages_has_unsupported_page_type_selector,
        list_pages_has_unsupported_parent_selector, native_list_page_link_default_label,
        parse_list_pages_arguments, parse_wikidot_compat_color_descriptor,
        push_list_pages_pager, register_generated_list_pages_html, render_clone_module,
        render_list_pages_numbered_rows, render_list_pages_table_rows,
        render_members_module_placeholder, render_native_list_page_link,
        render_new_page_module, render_read_only_rate_module, render_tag_cloud_box,
        requested_page_info_score, resolve_list_pages_signed_abs_expressions,
        resolve_wikidot_numeric_ifexpr, restore_list_pages_literal_ellipsis_markers,
        should_render_current_page_list_pages_row, substitute_count_pages_variables,
        substitute_list_pages_variables, unsupported_list_pages_replacement,
        wikidot_content_section, wikidot_module_argument,
        wikidot_no_such_include_replacement,
    };
    use crate::config::Config;
    use crate::constants::ADMIN_USER_ID;
    use crate::models::site::Model as SiteModel;
    use crate::services::page_query::{
        DataFormSelector, FoundPageRow, PageQueryResultMetadata,
        parse_static_wikidot_data_form_values, static_wikidot_data_form_matches,
    };
    use crate::types::{License, PageId};
    use crate::utils::now;
    use ftml::data::PageRef;
    use ftml::includes::IncludeRef;
    use ftml::layout::Layout;
    use ftml::render::{Render, html::HtmlRender};
    use ftml::settings::{WikitextMode, WikitextSettings};
    use ftml::tree::VariableMap;
    use std::borrow::Cow;
    use std::collections::BTreeMap;
    use std::time::{Duration, Instant};

    fn list_pages_substitution_context<'a>(
        rendered_limit: usize,
        user_displays: &'a BTreeMap<i64, WikidotUserDisplay>,
        page_wikitext: Option<&'a str>,
        data_form_values: &'a BTreeMap<String, String>,
    ) -> ListPagesSubstitutionContext<'a> {
        list_pages_substitution_context_with_mode(
            rendered_limit,
            user_displays,
            empty_list_pages_snapshot_displays(),
            page_wikitext,
            data_form_values,
            false,
        )
    }

    fn list_pages_substitution_context_with_mode<'a>(
        rendered_limit: usize,
        user_displays: &'a BTreeMap<i64, WikidotUserDisplay>,
        snapshot_displays: &'a BTreeMap<i64, ListPagesSnapshotDisplay>,
        page_wikitext: Option<&'a str>,
        data_form_values: &'a BTreeMap<String, String>,
        render_generated_html: bool,
    ) -> ListPagesSubstitutionContext<'a> {
        ListPagesSubstitutionContext {
            rendered_limit,
            user_displays,
            snapshot_displays,
            page_wikitext,
            data_form_values,
            render_generated_html,
        }
    }

    fn empty_list_pages_snapshot_displays()
    -> &'static BTreeMap<i64, ListPagesSnapshotDisplay> {
        static EMPTY: std::sync::LazyLock<BTreeMap<i64, ListPagesSnapshotDisplay>> =
            std::sync::LazyLock::new(BTreeMap::new);
        &EMPTY
    }

    fn fallback_test_page_info(
        page: &'static str,
        title: &'static str,
    ) -> ftml::data::PageInfo<'static> {
        ftml::data::PageInfo {
            page: Cow::Borrowed(page),
            category: None,
            site: Cow::Borrowed("scp-wiki"),
            title: Cow::Borrowed(title),
            alt_title: None,
            score: ftml::data::ScoreValue::Integer(0),
            tags: Vec::new(),
            language: Cow::Borrowed("en"),
        }
    }

    #[test]
    fn page_info_full_slug_uses_render_target_category() {
        let default = fallback_test_page_info("restored", "Restored");
        assert_eq!(RenderService::page_info_full_slug(&default), "restored");

        let mut categorized = fallback_test_page_info("restored", "Restored");
        categorized.category = Some(Cow::Borrowed("archive"));
        assert_eq!(
            RenderService::page_info_full_slug(&categorized),
            "archive:restored",
        );

        let mut explicit_default = fallback_test_page_info("restored", "Restored");
        explicit_default.category = Some(Cow::Borrowed("_default"));
        assert_eq!(
            RenderService::page_info_full_slug(&explicit_default),
            "restored",
        );
    }

    fn render_wikidot_page_body_after_compat_restore(wikitext: &str) -> String {
        let page_info = fallback_test_page_info("scp-7243", "SCP-7243");
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut wikitext = wikitext.to_owned();
        let fragments = RenderService::protect_generated_wikidot_compat_html(
            &mut wikitext,
            &settings,
        );
        ftml::preprocess(&mut wikitext);
        let tokens = ftml::tokenize(&wikitext);
        let result = ftml::parse(&tokens, &page_info, &settings);
        let (tree, _) = result.into();
        let rendered = HtmlRender.render(&tree, &page_info, &settings).body;
        RenderService::restore_protected_generated_wikidot_compat_html(
            rendered, &fragments,
        )
    }

    fn render_wikidot_fallback_after_generated_compat_restore(wikitext: &str) -> String {
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut wikitext = wikitext.to_owned();
        let fragments = RenderService::protect_generated_wikidot_compat_html(
            &mut wikitext,
            &settings,
        );
        let rendered =
            RenderService::render_wikidot_compatibility_fallback_with_code_blocks(
                &wikitext,
            );

        RenderService::restore_protected_generated_wikidot_compat_html(
            rendered, &fragments,
        )
    }

    fn render_wikidot_css_after_registry_restore(
        wikitext: &str,
        fallback: bool,
    ) -> String {
        let page_info = fallback_test_page_info("css", "CSS");
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut protected = wikitext.to_owned();
        let mut fragments = CompatHtmlFragments::new(wikitext);
        RenderService::protect_wikidot_css_modules(
            &mut protected,
            &settings,
            &mut fragments,
        );
        let rendered = if fallback {
            RenderService::render_wikidot_compatibility_fallback_with_code_blocks(
                &protected,
            )
        } else {
            ftml::preprocess(&mut protected);
            let tokens = ftml::tokenize(&protected);
            let (tree, _) = ftml::parse(&tokens, &page_info, &settings).into();
            HtmlRender.render(&tree, &page_info, &settings).body
        };
        fragments.restore(&rendered)
    }

    #[test]
    fn renders_nested_plain_parentheses_directly_through_ftml() {
        let rendered =
            render_wikidot_page_body_after_compat_restore("before (a (b)) after");

        assert!(rendered.contains("before (a (b)) after"));
    }

    #[test]
    fn renders_maximum_dense_stray_bibcite_input_without_amplification() {
        let source = "))".repeat(MAX_FTML_COMPAT_PARSE_BYTES / 2);
        let rendered = render_wikidot_page_body_after_compat_restore(&source);

        assert_eq!(source.len(), MAX_FTML_COMPAT_PARSE_BYTES);
        assert!(rendered.contains(&source));
        assert!(rendered.len() <= source.len() + 64);
    }

    #[test]
    fn preserves_valid_bibcite_after_removing_stray_closer_protection() {
        let rendered = render_wikidot_page_body_after_compat_restore(
            "[[bibliography]]\n: alpha : Entry\n[[/bibliography]]\n\
             ((bibcite alpha)) before (a (b)) after",
        );

        assert!(rendered.contains("wj-bibliography-ref"));
        assert!(rendered.contains("before (a (b)) after"));
    }

    #[test]
    fn restores_wikidot_email_visibility() {
        let html = concat!(
            r#"<p><strong>Email:</strong> "#,
            r#"<span class="wiki-email" style="visibility: visible;">"#,
            r#"<a href="mailto:info@nfsi.gov">info@nfsi.gov</a></span><br /></p>"#,
        );

        assert_eq!(
            RenderService::restore_wikidot_email_obfuscation(html),
            concat!(
                r#"<p><strong>Email:</strong> "#,
                r#"<span class="wiki-email" style="visibility: visible;">"#,
                r#"<a href="mailto:info@nfsi.gov">info@nfsi.gov</a></span>"#,
                r#"<br /></p>"#,
            ),
        );
    }

    #[test]
    fn parses_current_page_list_pages_name_selector() {
        let arguments = parse_list_pages_arguments(r#" name="=" limit="20""#)
            .expect("current page selector should parse");

        assert!(arguments.current_page_only);
    }

    #[test]
    fn parses_current_page_list_pages_range_selector() {
        let arguments = parse_list_pages_arguments(r#" range=".""#)
            .expect("current page range selector should parse");

        assert!(arguments.current_page_only);
        assert_eq!(arguments.limit, Some(1));
    }

    #[test]
    fn parses_other_pages_list_pages_range_selector() {
        let arguments = parse_list_pages_arguments(
            r#" category="*" created_by="=" tags="scp" perPage="15" range="others""#,
        )
        .expect("other pages range selector should parse");

        assert!(!arguments.current_page_only);
        assert!(arguments.exclude_current_page);
        assert_eq!(arguments.count_pages_per_page, Some(15));
    }

    #[test]
    fn current_page_list_pages_selection_respects_limit_and_offset() {
        assert!(should_render_current_page_list_pages_row(true, Some(1), 0));
        assert!(!should_render_current_page_list_pages_row(true, Some(0), 0));
        assert!(!should_render_current_page_list_pages_row(true, Some(1), 1));
        assert!(!should_render_current_page_list_pages_row(
            false,
            Some(1),
            0
        ));
    }

    #[test]
    fn current_page_list_pages_score_uses_render_page_score_when_requested() {
        let mut page_info = fallback_test_page_info("rated-page", "Rated page");
        page_info.score = ftml::data::ScoreValue::Integer(49);

        assert_eq!(
            requested_page_info_score(
                &crate::services::page_query::FoundPageFields {
                    score: true,
                    ..Default::default()
                },
                &page_info,
            ),
            Some(49.0),
        );
        assert_eq!(
            requested_page_info_score(
                &crate::services::page_query::FoundPageFields::default(),
                &page_info,
            ),
            None,
        );
    }

    #[test]
    fn parses_corpus_list_pages_category_and_separate_arguments() {
        let arguments = parse_list_pages_arguments(
            r#" tags="1998" category=". +theme" separate="no" order="created_at desc" limit="5""#,
        )
        .expect("corpus ListPages selector should parse");

        assert!(!arguments.category_all);
        assert!(arguments.include_current_category);
        assert_eq!(arguments.categories, vec![Cow::Borrowed("theme")]);
        assert_eq!(arguments.default_tags, vec![Cow::Borrowed("1998")]);
        assert_eq!(arguments.limit, Some(5));
    }

    #[test]
    fn parses_corpus_list_pages_excluded_category_and_tag() {
        let arguments = parse_list_pages_arguments(
            r#" tags="地下東京奇譚 -ハブ" order="created_at" separate="no" category="* -deleted""#,
        )
        .expect("corpus ListPages selector with exclusions should parse");

        assert!(arguments.category_all);
        assert_eq!(
            arguments.excluded_categories,
            vec![Cow::Borrowed("deleted")]
        );
        assert_eq!(arguments.default_tags, vec![Cow::Borrowed("地下東京奇譚")]);
        assert_eq!(arguments.no_tags, vec![Cow::Borrowed("ハブ")]);
    }

    #[test]
    fn parses_singular_list_pages_tag_argument_with_exclusions() {
        let arguments = parse_list_pages_arguments(
            r#" tag="+scp -tale -goi-format -co-authored" category="-fragment" perPage="250""#,
        )
        .expect("Wikidot singular tag selector with exclusions should parse");

        assert_eq!(arguments.all_tags, vec![Cow::Borrowed("scp")]);
        assert_eq!(
            arguments.no_tags,
            vec![
                Cow::Borrowed("tale"),
                Cow::Borrowed("goi-format"),
                Cow::Borrowed("co-authored")
            ]
        );
        assert_eq!(
            arguments.excluded_categories,
            vec![Cow::Borrowed("fragment")]
        );
        assert_eq!(arguments.count_pages_per_page, Some(250));
    }

    #[test]
    fn parses_corpus_list_pages_table_arguments() {
        let arguments = parse_list_pages_arguments(
            r#" tags="1998" separate="no" category="* -deleted" perPage="100" prependLine="||~ ページ ||~ 投稿者 ||~ 投稿日 ||~ 評価 ||""#,
        )
        .expect("corpus ListPages table selector should parse");

        assert!(arguments.category_all);
        assert_eq!(
            arguments.excluded_categories,
            vec![Cow::Borrowed("deleted")]
        );
        assert_eq!(arguments.default_tags, vec![Cow::Borrowed("1998")]);
        assert_eq!(arguments.count_pages_per_page, Some(100));
        assert_eq!(
            arguments.prepend_line.as_deref(),
            Some("||~ ページ ||~ 投稿者 ||~ 投稿日 ||~ 評価 ||"),
        );
    }

    #[test]
    fn parses_corpus_list_pages_url_and_filter_arguments() {
        let arguments = parse_list_pages_arguments(
            r#" range="." limit="@URL|0" offset="@URL|5" urlAttrPrefix="list1""#,
        )
        .expect("URL-driven ListPages arguments should parse");

        assert_eq!(arguments.limit, Some(0));
        assert_eq!(arguments.offset, 5);

        let arguments = parse_list_pages_arguments(
            r#" separate="no" category="@URL|*" tags="@URL" created_at="@URL" updated_at="@URL" created_by="@URL" rating="@URL" votes="@URL" link_to="@URL" offset="@URL|0" name="@URL" limit="@URL|0" perPage="@URL|20" parent="*" order="@URL|created_at desc" wrapper="no""#,
        )
        .expect("tag-search ListPages arguments should parse");

        assert!(arguments.category_all);
        assert_eq!(arguments.limit, Some(0));
        assert_eq!(arguments.count_pages_per_page, Some(20));
        assert_eq!(arguments.offset, 0);
        assert!(arguments.slug.is_none());

        assert!(
            parse_list_pages_arguments(r#" parent="@URL""#).is_none(),
            "dynamic parent selectors should remain unsupported rather than widening to all parents"
        );
        assert!(list_pages_has_unsupported_parent_selector(
            r#" parent="@URL""#
        ));
        assert!(list_pages_has_unsupported_parent_selector(
            r#" parent="other-page""#
        ));
        assert!(!list_pages_has_unsupported_parent_selector(
            r#" parent="@URL|.""#
        ));
        assert!(
            parse_list_pages_arguments(r#" offset="1001""#).is_none(),
            "large offsets should remain unsupported during render"
        );
        assert!(list_pages_has_unsupported_page_type_selector(
            r#" pagetype="draft""#
        ));
        assert!(!list_pages_has_unsupported_page_type_selector(
            r#" pagetype="@URL|normal""#
        ));
    }

    #[test]
    fn parses_corpus_list_pages_literal_name_as_slug() {
        let arguments =
            parse_list_pages_arguments(r#" category="_default" name="SCP-655-JP""#)
                .expect("literal ListPages name selector should parse");

        assert_eq!(arguments.slug.as_deref(), Some("scp-655-jp"));
        assert!(!arguments.current_page_only);
    }

    #[test]
    fn parses_static_data_form_selectors() {
        let arguments = parse_list_pages_arguments(
            r#" category="codexdftdft01" _codexkind="alpha" _codexflag!="missing" _Status="open" __private="yes" order="titleAsc" limit="20" separate="false""#,
        )
        .expect("static data-form selectors should parse");

        assert_eq!(
            arguments.data_form_fields,
            vec![
                DataFormSelector {
                    field: Cow::Borrowed("codexkind"),
                    value: Cow::Borrowed("alpha"),
                    negated: false,
                },
                DataFormSelector {
                    field: Cow::Borrowed("codexflag"),
                    value: Cow::Borrowed("missing"),
                    negated: true,
                },
                DataFormSelector {
                    field: Cow::Borrowed("Status"),
                    value: Cow::Borrowed("open"),
                    negated: false,
                },
                DataFormSelector {
                    field: Cow::Borrowed("_private"),
                    value: Cow::Borrowed("yes"),
                    negated: false,
                },
            ],
        );
        assert!(
            parse_list_pages_arguments(r#" _status="@URL""#).is_none(),
            "dynamic data-form selectors should remain unsupported instead of being dropped"
        );
    }

    #[test]
    fn parses_corpus_list_pages_created_by_argument() {
        let source = r#"[[module ListPages created_by="[Congy]" separate="no"]]%%title_linked%%[[/module]]"#;
        let captures = LISTPAGES_MODULE_REGEX
            .captures(source)
            .expect("ListPages module with bracketed quoted author should match");
        assert_eq!(
            captures.name("head").unwrap().as_str().trim(),
            r#"created_by="[Congy]" separate="no""#,
        );

        let arguments = parse_list_pages_arguments(
            r#" created_by="[Congy]" separate="no" tags="+jp" order="created" category="-deleted""#,
        )
        .expect("bracketed Wikidot author selector should parse");

        assert_eq!(arguments.authors, vec![Cow::Borrowed("Congy")]);
        assert_eq!(
            arguments.excluded_categories,
            vec![Cow::Borrowed("deleted")]
        );
        assert_eq!(arguments.all_tags, vec![Cow::Borrowed("jp")]);
    }

    #[test]
    fn ignores_blank_wikidot_list_pages_order_argument() {
        let arguments = parse_list_pages_arguments(
            r#" created_by="=" order="" category="-fragment" tag="+scp -co-authored" perPage="250""#,
        )
        .expect("blank ListPages order should fall back to default order");

        assert_eq!(arguments.authors, vec![Cow::Borrowed("=")]);
        assert_eq!(arguments.order, None);
        assert_eq!(arguments.count_pages_per_page, Some(250));
        assert_eq!(arguments.all_tags, vec![Cow::Borrowed("scp")]);
        assert_eq!(arguments.no_tags, vec![Cow::Borrowed("co-authored")]);
        assert_eq!(
            arguments.excluded_categories,
            vec![Cow::Borrowed("fragment")]
        );
    }

    #[test]
    fn parses_wikidot_camel_case_list_pages_order_argument() {
        let ascending = parse_list_pages_arguments(
            r#" category="*" tags="codex" order="titleAsc" limit="20" wrapper="no""#,
        )
        .expect("Wikidot camel-case order should parse");
        assert_eq!(
            ascending.order,
            Some(OrderBySelector {
                property: OrderProperty::Title,
                ascending: true,
            }),
        );

        let descending = parse_list_pages_arguments(
            r#" category="*" tags="codex" order="createdAtDesc" limit="20""#,
        )
        .expect("Wikidot camel-case descending order should parse");
        assert_eq!(
            descending.order,
            Some(OrderBySelector {
                property: OrderProperty::CreatedAt,
                ascending: false,
            }),
        );
    }

    #[test]
    fn keeps_broad_unbounded_count_pages_literal() {
        let no_filter = parse_list_pages_arguments(r#""#)
            .expect("broad CountPages selector should parse");
        assert!(count_pages_should_remain_literal(&no_filter));

        let all_categories = parse_list_pages_arguments(r#" category="*" "#)
            .expect("all-category CountPages selector should parse");
        assert!(count_pages_should_remain_literal(&all_categories));

        let exclusion_only = parse_list_pages_arguments(r#" category="* -deleted" "#)
            .expect("exclusion-only CountPages selector should parse");
        assert!(count_pages_should_remain_literal(&exclusion_only));
    }

    #[test]
    fn allows_unbounded_count_pages_with_static_filter() {
        let tagged = parse_list_pages_arguments(r#" category="*" tags="codex" "#)
            .expect("static tag CountPages selector should parse");
        assert!(!count_pages_should_remain_literal(&tagged));

        let named = parse_list_pages_arguments(r#" name="example" "#)
            .expect("static name CountPages selector should parse");
        assert!(!count_pages_should_remain_literal(&named));
    }

    #[test]
    fn count_pages_substitution_resolves_numeric_ifexpr() {
        let output = substitute_count_pages_variables(
            r#"[[div class="activity-container [[#ifexpr %%total%% >= 60 | large-c | not-large-c ]]" data-number="%%total%%"]]x[[/div]]"#,
            0,
        );

        assert!(output.contains(r#"activity-container not-large-c"#));
        assert!(output.contains(r#"data-number="0""#));
        assert!(!output.contains("[[#ifexpr"));
    }

    #[test]
    fn count_pages_substitution_cannot_construct_a_trusted_compat_marker() {
        let output = substitute_count_pages_variables(
            concat!(
                "<div id=\"ml-1\" data-wikijump-compat-members=\"%%total%%\">",
                "<img src=x onerror=\"alert(1)\"></div>",
            ),
            1,
        );
        assert!(output.contains("data-wikijump-authored-compat-members=\"1\""));

        let mut protected = output.clone();
        let fragments = RenderService::protect_generated_wikidot_compat_html(
            &mut protected,
            &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
        );
        assert!(fragments.is_empty());
        let rendered = render_wikidot_page_body_after_compat_restore(&output);
        assert!(!rendered.contains(r#"<img src=x onerror="alert(1)">"#));
    }

    #[test]
    fn count_pages_exact_count_render_diagnostics_allow_matching_explicit_sql_window() {
        let diagnostics = count_pages_exact_count_render_diagnostics(
            PageQueryResultMetadata {
                candidate_count: Some(10),
                sql_limit_offset_applied: true,
                exact_count_safe: true,
                ..PageQueryResultMetadata::default()
            },
            false,
            false,
            false,
            Some(10),
            10,
        );

        assert!(diagnostics.allowed);
        assert_eq!(diagnostics.denied_reason_code, None);
        assert_eq!(diagnostics.denied_reason_detail, None);
    }

    #[test]
    fn count_pages_exact_count_render_diagnostics_denies_unbounded_sql_window() {
        let diagnostics = count_pages_exact_count_render_diagnostics(
            PageQueryResultMetadata {
                candidate_count: Some(MAX_LISTPAGES_RENDER_SCAN_ROWS as usize),
                sql_limit_offset_applied: true,
                exact_count_safe: true,
                ..PageQueryResultMetadata::default()
            },
            false,
            false,
            false,
            None,
            u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS),
        );

        assert!(!diagnostics.allowed);
        assert_eq!(diagnostics.denied_reason_code, Some("unsafe_sql_window"));
        assert_eq!(diagnostics.denied_reason_detail, None);
    }

    #[test]
    fn count_pages_exact_count_render_diagnostics_denies_post_query_exclusion_before_offset()
     {
        let diagnostics = count_pages_exact_count_render_diagnostics(
            PageQueryResultMetadata {
                candidate_count: Some(10),
                exact_count_safe: true,
                ..PageQueryResultMetadata::default()
            },
            false,
            true,
            true,
            Some(10),
            11,
        );

        assert!(!diagnostics.allowed);
        assert_eq!(diagnostics.denied_reason_code, Some("post_query_exclusion"));
        assert_eq!(diagnostics.denied_reason_detail, None);
    }

    #[test]
    fn renders_wikidot_tag_cloud_box_links() {
        let html = render_tag_cloud_box(&[
            ("scp".to_owned(), 10),
            ("needs<escape".to_owned(), 1),
        ]);

        assert!(html.contains(r#"[[div class="pages-tag-cloud-box"]]"#));
        assert!(html.contains(r#"class="tag""#));
        assert!(html.contains(r#"[/system:page-tags/tag/scp scp]"#));
        assert!(html.contains("needs&lt;escape"));
        assert!(!html.contains("[[module TagCloud"));
        assert!(!html.contains("<a class="));
    }

    #[test]
    fn renders_wikidot_read_only_rate_module_with_downvote() {
        let rendered =
            render_read_only_rate_module(ftml::data::ScoreValue::Integer(19), "en");

        assert!(rendered.contains(r#"[[span class="rate-points"]]rating: "#));
        assert!(rendered.contains(r#"[[span class="number prw54353"]]+19[[/span]]"#));
        assert!(rendered.contains(r#"[[span class="rateup btn btn-default"]]"#));
        assert!(rendered.contains(r#"listeners.rate(event, 1)"#));
        assert!(
            rendered.contains(r#"]][[/span]][[span class="rateup btn btn-default"]]"#)
        );
        assert!(rendered.contains(r#"[[span class="ratedown btn btn-default"]]"#));
        assert!(rendered.contains(r#"listeners.rate(event, -1)"#));
        assert!(
            rendered.contains(r#"]][[/span]][[span class="ratedown btn btn-default"]]"#)
        );
        assert!(rendered.contains(r#"title="I don't like it"]]–[[/a]]"#));
        assert!(rendered.contains(r#"[[span class="cancel btn btn-default"]]"#));
        assert!(rendered.contains(r#"listeners.cancelVote(event)"#));
        assert!(
            rendered.contains(r#"]][[/span]][[span class="cancel btn btn-default"]]"#)
        );
    }

    #[test]
    fn renders_japanese_wikidot_read_only_rate_module_labels() {
        let rendered =
            render_read_only_rate_module(ftml::data::ScoreValue::Integer(35), "ja");

        assert!(rendered.contains("[[span class=\"rate-points\"]]評価:\u{00a0}"));
        assert!(rendered.contains(r#"[[span class="number prw54353"]]+35[[/span]]"#));
        assert!(rendered.contains(r#"title="好き""#));
        assert!(rendered.contains(r#"title="好きじゃない""#));
        assert!(rendered.contains(r#"title="投票を取り消す""#));
    }

    #[test]
    fn renders_wikidot_members_module_placeholder() {
        let head = r#" group="moderators" order="joined""#;
        assert_eq!(wikidot_module_argument(head, "group"), Some("moderators"));

        let rendered = RenderService::expand_members_modules(
            "[[module Members group=\"moderators\"]]".to_owned(),
            &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
        );

        assert!(rendered.contains(r#"<div id="ml-607935" data-wikijump-compat-members="1" data-group="moderators">"#));
        assert!(rendered.contains(r#"<span class="printuser avatarhover">"#));
        assert!(rendered.contains("membership/MembersListModule"));
        assert!(!rendered.contains("[[module Members"));
    }

    #[test]
    fn protects_wikidot_members_module_html_before_parsing() {
        let mut wikitext = render_members_module_placeholder("moderators");
        let fragments = RenderService::protect_generated_wikidot_compat_html(
            &mut wikitext,
            &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
        );

        assert_eq!(fragments.len(), 1);
        assert!(wikitext.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
        let restored = RenderService::restore_protected_generated_wikidot_compat_html(
            wikitext, &fragments,
        );
        assert!(restored.contains(r#"<div id="ml-607935" data-group="moderators">"#));
        assert!(!restored.contains("data-wikijump-compat-members"));
    }

    #[test]
    fn wikidot_compatibility_fallback_restores_generated_members_html_as_block() {
        let source = format!(
            concat!(
                "before\n",
                "[[div_ class=\"a-randomizer\"]]\n",
                "{}\n",
                "[[div class=\"dude\"]]\n",
                "[[/div]]\n",
                "[[/div]]\n",
                "after\n",
            ),
            render_members_module_placeholder("moderators"),
        );

        let rendered = render_wikidot_fallback_after_generated_compat_restore(&source);

        assert!(rendered.contains(
            r#"<div class="a-randomizer"><div id="ml-607935" data-group="moderators">"#
        ));
        assert!(rendered.contains("membership/MembersListModule"));
        assert!(rendered.contains(r#"<div class="dude"></div>"#));
        assert!(!rendered.contains(r#"data-wikijump-compat-members"#));
        assert!(!rendered.contains(r#"&lt;div id="ml-607935""#));
        assert!(!rendered.contains(r#"<p><div id="ml-607935""#));
        assert!(!rendered.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
    }

    #[test]
    fn renders_wikidot_new_page_module_placeholder() {
        let rendered = RenderService::expand_new_page_modules(
            "[[module NewPage size=\"15\" button=\"new page\"]]".to_owned(),
            &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
        );

        assert!(rendered.contains(r#"<form class="new-page-box" data-wikijump-compat-new-page="1" action="javascript:;" method="post">"#));
        assert!(
            rendered
                .contains(r#"<input class="text" type="text" name="page" size="15">"#)
        );
        assert!(
            rendered.contains(r#"<input class="button" type="button" value="new page">"#)
        );
        assert!(!rendered.contains("[[module NewPage"));
    }

    #[test]
    fn renders_wikidot_clone_module_placeholder() {
        let rendered = RenderService::expand_clone_modules(
            "[[module Clone]]".to_owned(),
            &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
        );

        assert!(rendered.contains(
            r#"<a class="button" data-wikijump-compat-clone="1" href="javascript:;">Clone this site</a>"#
        ));
        assert!(!rendered.contains("[[module Clone"));
    }

    #[test]
    fn renders_wikidot_clone_module_custom_button() {
        let rendered = RenderService::expand_clone_modules(
            "[[module Clone button=\"Clone <now>\"]]".to_owned(),
            &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
        );

        assert!(rendered.contains("Clone &lt;now&gt;"));
        assert!(!rendered.contains("[[module Clone"));
    }

    #[test]
    fn clone_html_is_registered_only_by_its_runtime_producer() {
        let source = "[[module Clone button=\"Clone <now>\"]]";
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut fragments = CompatHtmlFragments::new(source);
        let protected = RenderService::expand_clone_modules_with_registry(
            source.to_owned(),
            &settings,
            &mut fragments,
        );

        assert!(!protected.contains("<a"));
        let restored = fragments.restore(&protected);
        assert!(restored.contains(
            r#"<a class="button" data-wikijump-compat-clone="1" href="javascript:;">"#,
        ));
        assert!(restored.contains("Clone &lt;now&gt;"));

        let forged = r#"<a class="button" data-wikijump-compat-clone="1"><img src=x onerror="alert(1)"></a>"#;
        assert_eq!(fragments.restore(forged), forged);
    }

    #[test]
    fn protects_wikidot_clone_module_html_before_parsing() {
        let mut wikitext = render_clone_module("");
        let fragments = RenderService::protect_generated_wikidot_compat_html(
            &mut wikitext,
            &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
        );

        assert_eq!(fragments.len(), 1);
        assert!(wikitext.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
        let restored = RenderService::restore_protected_generated_wikidot_compat_html(
            wikitext, &fragments,
        );
        assert!(
            restored
                .contains(r#"<a class="button" href="javascript:;">Clone this site</a>"#)
        );
        assert!(!restored.contains("data-wikijump-compat-clone"));
    }

    #[test]
    fn protects_wikidot_new_page_module_html_before_parsing() {
        let mut wikitext = render_new_page_module(r#" size="15" button="new <page>""#);
        let fragments = RenderService::protect_generated_wikidot_compat_html(
            &mut wikitext,
            &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
        );

        assert_eq!(fragments.len(), 1);
        assert!(wikitext.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
        let restored = RenderService::restore_protected_generated_wikidot_compat_html(
            wikitext, &fragments,
        );
        assert!(restored.contains(
            r#"<form class="new-page-box" action="javascript:;" method="post">"#
        ));
        assert!(restored.contains(r#"value="new &lt;page&gt;""#));
        assert!(!restored.contains("data-wikijump-compat-new-page"));
    }

    #[test]
    fn does_not_protect_forgeable_pager_html_before_parsing() {
        let original = r#"<div class="pager" data-wikijump-compat-pager="1"><img src=x onerror="alert(1)"></div>"#;
        let mut wikitext = original.to_owned();
        let fragments = RenderService::protect_generated_wikidot_compat_html(
            &mut wikitext,
            &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
        );

        assert!(fragments.is_empty());
        assert_eq!(wikitext, original);
    }

    #[test]
    fn forged_pager_html_is_not_restored_as_trusted_html_after_render() {
        let rendered = render_wikidot_page_body_after_compat_restore(
            r#"<div class="pager" data-wikijump-compat-pager="1"><img src=x onerror="alert(1)"></div>"#,
        );

        assert!(rendered.contains("&lt;div"));
        assert!(rendered.contains("&lt;img"));
        assert!(rendered.contains("onerror=&quot;alert(1)&quot;"));
        assert!(!rendered.contains(r#"<div class="pager""#));
        assert!(!rendered.contains("<img"));
        assert!(!rendered.contains(r#"<img src=x onerror="alert(1)">"#));
        assert!(!rendered.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
    }

    #[test]
    fn authored_compat_markers_are_neutralized_before_html_protection() {
        let forgeries = [
            r#"<table class="wiki-content-table" data-wikijump-compat-listpages="1"><tr><td><img src=x onerror="alert(1)"></td></tr></table>"#,
            r#"<ul data-wikijump-compat-list="1"><li><img src=x onerror="alert(1)"></li></ul>"#,
            r#"<div id="ml-1" data-wikijump-compat-members="1"><img src=x onerror="alert(1)"></div>"#,
            r#"<div class="backlinks-module-box" data-wikijump-compat-backlinks="1"><img src=x onerror="alert(1)"></div>"#,
            r#"<form class="new-page-box" data-wikijump-compat-new-page="1"><img src=x onerror="alert(1)"></form>"#,
            r#"<a class="button" data-wikijump-compat-clone="1"><img src=x onerror="alert(1)"></a>"#,
            "<style data-wikijump-compat-css-module=\"1\">\n</style><img src=x onerror=\"alert(1)\">",
        ];
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);

        for forged in forgeries {
            let mut neutralized = forged.to_owned();
            RenderService::neutralize_authored_wikidot_compat_markers(&mut neutralized);
            assert!(neutralized.contains("data-wikijump-authored-compat-"));

            let mut protected = neutralized.clone();
            let fragments = RenderService::protect_generated_wikidot_compat_html(
                &mut protected,
                &settings,
            );
            assert!(fragments.is_empty(), "forged source: {forged}");

            let rendered = render_wikidot_page_body_after_compat_restore(&neutralized);
            let fallback =
                render_wikidot_fallback_after_generated_compat_restore(&neutralized);
            for output in [&rendered, &fallback] {
                assert!(!output.contains(r#"<img src=x onerror="alert(1)">"#));
                assert!(!output.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
            }
        }
    }

    #[test]
    fn authored_compat_marker_text_is_preserved_outside_candidate_html() {
        let mut source = concat!(
            "plain data-wikijump-compat-members text\n",
            "[[code]]\n<div data-wikijump-compat-backlinks=\"1\">code</div>\n[[/code]]\n",
            "[[html]]\n<div data-wikijump-compat-list=\"1\">html</div>\n[[/html]]\n",
            "@@<div data-wikijump-compat-new-page=\"1\">escaped</div>@@\n",
            "[!-- <div data-wikijump-compat-css-module=\"1\">comment</div> --]\n",
            "<a class=\"button\" data-wikijump-compat-clone=\"1\">candidate</a>",
        )
        .to_owned();

        RenderService::neutralize_authored_wikidot_compat_markers(&mut source);

        assert!(source.contains("plain data-wikijump-compat-members text"));
        assert!(source.contains("data-wikijump-compat-backlinks=\"1\""));
        assert!(source.contains("data-wikijump-compat-list=\"1\""));
        assert!(source.contains("data-wikijump-compat-new-page=\"1\""));
        assert!(source.contains("data-wikijump-compat-css-module=\"1\""));
        assert!(source.contains("data-wikijump-authored-compat-clone=\"1\""));
    }

    #[test]
    fn malformed_outer_html_cannot_hide_a_forgeable_compat_fragment() {
        let forged = concat!(
            "<x a='<div id=\"ml-1\" data-wikijump-compat-members=\"1\">",
            "<img src=x onerror=\"alert(1)\"></div>",
        );
        let mut neutralized = forged.to_owned();
        RenderService::neutralize_authored_wikidot_compat_markers(&mut neutralized);

        assert!(neutralized.contains("data-wikijump-authored-compat-members"));
        let mut protected = neutralized.clone();
        let fragments = RenderService::protect_generated_wikidot_compat_html(
            &mut protected,
            &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
        );
        assert!(fragments.is_empty());

        let rendered = render_wikidot_page_body_after_compat_restore(&neutralized);
        assert!(!rendered.contains(r#"<img src=x onerror="alert(1)">"#));
        assert!(!rendered.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
    }

    #[test]
    fn generated_looking_compat_fragments_inside_code_remain_literal() {
        let source = concat!(
            "[[code]]\n",
            "<div id=\"ml-1\" data-wikijump-compat-members=\"1\">",
            "<img src=x onerror=\"alert(1)\"></div>\n",
            "<ul data-wikijump-compat-list=\"1\"><li>",
            "<img src=x onerror=\"alert(2)\"></li></ul>\n",
            "[[/code]]",
        );
        let mut neutralized = source.to_owned();
        RenderService::neutralize_authored_wikidot_compat_markers(&mut neutralized);
        assert_eq!(neutralized, source);

        let mut protected = neutralized.clone();
        let fragments = RenderService::protect_generated_wikidot_compat_html(
            &mut protected,
            &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
        );
        assert!(fragments.is_empty());
        assert_eq!(protected, source);

        let rendered = render_wikidot_page_body_after_compat_restore(&neutralized);
        assert!(!rendered.contains(r#"<img src=x onerror="alert(1)">"#));
        assert!(!rendered.contains(r#"<img src=x onerror="alert(2)">"#));
    }

    #[test]
    fn neutralizes_compat_markers_composed_by_list_pages_substitution() {
        let page = FoundPageRow {
            page_id: 1,
            site_id: 1,
            title: Some(
                r#"compat-members="1"><img src=x onerror="alert(1)"></div>"#.to_owned(),
            ),
            alt_title: None,
            slug: Some("forged-title".to_owned()),
            page_category_id: None,
            page_revision_id: None,
            tags: None,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
            score: None,
        };
        let mut substituted = substitute_list_pages_variables(
            r#"<div id="ml-1" data-wikijump-%%title%%"#,
            &page,
            1,
            1,
            &list_pages_substitution_context(
                20,
                &BTreeMap::new(),
                None,
                &BTreeMap::new(),
            ),
        );
        assert!(substituted.contains(r#"data-wikijump-compat-members="1""#));

        RenderService::neutralize_authored_wikidot_compat_markers(&mut substituted);
        let fragments = RenderService::protect_generated_wikidot_compat_html(
            &mut substituted,
            &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
        );

        assert!(fragments.is_empty());
        assert!(substituted.contains("data-wikijump-authored-compat-members"));
    }

    #[test]
    fn compat_html_restoration_ignores_authored_legacy_sentinel_text() {
        let authored_marker = format!("{WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX}0X");
        let mut wikitext = format!("{authored_marker}{}", render_clone_module(""));
        let fragments = RenderService::protect_generated_wikidot_compat_html(
            &mut wikitext,
            &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
        );

        assert_eq!(fragments.len(), 1);
        assert!(wikitext.contains(&authored_marker));
        assert_ne!(fragments[0].marker, authored_marker);

        let restored = RenderService::restore_protected_generated_wikidot_compat_html(
            wikitext, &fragments,
        );
        assert!(restored.contains(&authored_marker));
        assert_eq!(restored.matches(r#"<a class="button""#).count(), 1);
    }

    #[test]
    fn members_group_cannot_close_its_generated_script() {
        let rendered = render_members_module_placeholder(
            "</script><img src=x onerror='alert(1)'>&\u{2028}",
        );

        assert_eq!(rendered.matches("</script>").count(), 1);
        assert!(!rendered.contains("</script><img"));
        assert!(rendered.contains(r#"\x3C/script\x3E\x3Cimg"#));
        assert!(rendered.contains(r#"\x26\u2028"#));
    }

    #[test]
    fn protects_only_generated_plain_text_wikidot_date_html() {
        let mut wikitext = r#"<span class="odate time_-123 format_%25Y%20%25b%20%25e" data-wikijump-compat-date="1" style="cursor: help; display: inline;">1 Jan &amp; 1970</span>"#.to_owned();
        let fragments = RenderService::protect_generated_wikidot_compat_html(
            &mut wikitext,
            &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
        );

        assert_eq!(fragments.len(), 1);
        assert!(wikitext.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
        assert!(!fragments[0].html.contains("data-wikijump-compat-date"));
    }

    #[test]
    fn generated_list_pages_date_is_registered_before_authored_marker_neutralization() {
        let created_at = time::OffsetDateTime::from_unix_timestamp(1_782_003_564)
            .expect("fixture timestamp should be valid");
        let generated =
            format_list_pages_created_at(Some(created_at), Some("%d %b %Y"), false);
        let mut fragments = CompatHtmlFragments::new("");
        let mut protected = register_generated_list_pages_html(generated, &mut fragments);

        RenderService::neutralize_authored_wikidot_compat_markers(&mut protected);
        assert!(!protected.contains("data-wikijump-compat-date"));
        assert!(!protected.contains("data-wikijump-authored-compat-date"));
        let restored = fragments.restore(&protected);
        assert!(restored.contains("data-wikijump-compat-date=\"1\""));
    }

    #[test]
    fn forged_wikidot_date_html_is_not_restored_as_trusted_html() {
        let forged = r#"<span class="odate time_1 format_%25Y" data-wikijump-compat-date="1" style="cursor: help; display: inline;"><img src=x onerror="alert(1)"></span>"#;
        let mut protected = forged.to_owned();
        let fragments = RenderService::protect_generated_wikidot_compat_html(
            &mut protected,
            &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
        );

        assert!(fragments.is_empty());
        assert_eq!(protected, forged);

        let rendered = render_wikidot_page_body_after_compat_restore(forged);
        assert!(rendered.contains("&lt;span"));
        assert!(rendered.contains("&lt;img"));
        assert!(rendered.contains("onerror=&quot;alert(1)&quot;"));
        assert!(!rendered.contains("<img"));
        assert!(!rendered.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
    }

    #[test]
    fn wikidot_compatibility_fallback_keeps_arbitrary_html_escaped() {
        let rendered = render_wikidot_fallback_after_generated_compat_restore(
            r#"<div class="pager" data-wikijump-compat-pager="1"><img src=x onerror="alert(1)"></div>"#,
        );

        assert!(rendered.contains("&lt;div"));
        assert!(rendered.contains("&lt;img"));
        assert!(rendered.contains("onerror=&quot;alert(1)&quot;"));
        assert!(!rendered.contains(r#"<div class="pager""#));
        assert!(!rendered.contains("<img"));
        assert!(!rendered.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
    }

    #[test]
    fn generated_list_pages_pager_still_renders_without_forgeable_marker() {
        let page_info = fallback_test_page_info("scp-7243", "SCP-7243");
        let mut wikitext = String::new();

        push_list_pages_pager(&mut wikitext, &page_info, 0, 2, 5);

        assert!(wikitext.contains(r#"[[div class="pager"]]"#));
        assert!(!wikitext.contains("data-wikijump-compat-pager"));

        let rendered = render_wikidot_page_body_after_compat_restore(&wikitext);

        assert!(rendered.contains(r#"<div class="pager">"#));
        assert!(rendered.contains(r#"<span class="pager-no">page 1 of 3</span>"#));
        assert!(rendered.contains(r#"<a href="/scp-7243/p/2">2</a>"#));
        assert!(!rendered.contains("data-wikijump-compat-pager"));
    }

    #[test]
    fn accepts_corpus_list_pages_comments_placeholder() {
        let body = "# (%%created_at|%y/%m/%d%%) %%title_linked%% (評価: %%rating%% コメント: %%comments%% 最終コメント: %%commented_at%%)";

        assert!(list_pages_body_variables_supported(body));
        let substituted = substitute_list_pages_variables(
            body,
            &FoundPageRow {
                page_id: 1,
                site_id: 1,
                title: Some("SCP-655-JP".to_owned()),
                alt_title: None,
                slug: Some("scp-655-jp".to_owned()),
                page_category_id: None,
                page_revision_id: None,
                tags: None,
                created_at: None,
                created_by: None,
                updated_at: None,
                updated_by: None,
                score: Some(12.0),
            },
            1,
            1,
            &list_pages_substitution_context(
                20,
                &BTreeMap::new(),
                None,
                &BTreeMap::new(),
            ),
        );

        assert_eq!(
            substituted,
            "# () [/scp-655-jp SCP-655-JP] (評価: 12 コメント:  最終コメント: )",
        );
        assert_eq!(
            render_list_pages_numbered_rows(&substituted),
            "<ol>\n<li>() <a href=\"/scp-655-jp\">SCP-655-JP</a> (評価: 12 コメント:  最終コメント: )</li>\n</ol>\n",
        );
    }

    #[test]
    fn substitutes_wikidot_list_pages_content_sections() {
        let page = FoundPageRow {
            page_id: 1,
            site_id: 1,
            title: Some("SCP-2693".to_owned()),
            alt_title: None,
            slug: Some("scp-2693".to_owned()),
            page_category_id: None,
            page_revision_id: None,
            tags: None,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
            score: None,
        };
        let wikitext = concat!(
            "=====\n",
            "[[include component:preview text=Wherein an adorable kitten gets up to no good.]]\n",
            "=====\n",
            "Main page body\n",
            "=====\n",
            "Hidden title text\n",
            "=====\n",
            "License notes\n",
        );

        assert!(list_pages_body_variables_supported(
            "%%title%% -- %%content{4}%%"
        ));
        assert!(list_pages_body_uses_content_variable(
            "%%title%% -- %%content{4}%%"
        ));
        assert_eq!(
            wikidot_content_section(wikitext, Some(4)),
            "Hidden title text",
        );

        let rendered = substitute_list_pages_variables(
            "**%%title%% -- %%content{4}%%**",
            &page,
            1,
            1,
            &list_pages_substitution_context(
                20,
                &BTreeMap::new(),
                Some(wikitext),
                &BTreeMap::new(),
            ),
        );

        assert_eq!(rendered, "**SCP-2693 -- Hidden title text**");
    }

    #[test]
    fn substitutes_static_wikidot_data_form_variables() {
        let page = FoundPageRow {
            page_id: 1,
            site_id: 1,
            title: Some("Codex data form fixture".to_owned()),
            alt_title: None,
            slug: Some("codex-data-form-fixture".to_owned()),
            page_category_id: None,
            page_revision_id: None,
            tags: None,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
            score: None,
        };
        let values = parse_static_wikidot_data_form_values(
            "codexkind: alpha\ncodexflag: 'df-red'\ncodex-hyphen: ok\n",
        );

        assert!(list_pages_body_variables_supported(
            "%%title%% %%form_raw{codexkind}%% %%form_data{codexflag}%% %%form_raw{codex-hyphen}%%"
        ));
        assert_eq!(
            substitute_list_pages_variables(
                "%%title%% %%form_raw{codexkind}%% %%form_data{codexflag}%% %%form_raw{codex-hyphen}%%",
                &page,
                1,
                1,
                &list_pages_substitution_context(20, &BTreeMap::new(), None, &values),
            ),
            "Codex data form fixture alpha df-red ok",
        );
    }

    #[test]
    fn static_wikidot_data_form_values_do_not_scan_ordinary_body_text() {
        let values = parse_static_wikidot_data_form_values(
            "This is regular page text.\nstatus: published\nowner: codex\n",
        );

        assert!(values.is_empty());

        let values = parse_static_wikidot_data_form_values(
            "\nstatus: published\nowner: codex\nBody text starts here.\nignored: yes\n",
        );

        assert_eq!(values.get("status").map(String::as_str), Some("published"));
        assert_eq!(values.get("owner").map(String::as_str), Some("codex"));
        assert!(!values.contains_key("ignored"));

        assert!(!static_wikidot_data_form_matches(
            &values,
            &[DataFormSelector {
                field: Cow::Borrowed("missing"),
                value: Cow::Borrowed(""),
                negated: false,
            }],
        ));
        assert!(!static_wikidot_data_form_matches(
            &values,
            &[DataFormSelector {
                field: Cow::Borrowed("missing"),
                value: Cow::Borrowed("closed"),
                negated: true,
            }],
        ));
    }

    #[test]
    fn form_list_pages_variables_require_field_arguments() {
        assert!(list_pages_body_variables_supported(
            "%%form_raw{codexkind}%% %%form_data{codexflag}%%"
        ));
        assert!(!list_pages_body_variables_supported("%%form_raw%%"));
        assert!(!list_pages_body_variables_supported("%%form_data%%"));
    }

    #[test]
    fn renders_wikidot_list_pages_table_rows_as_raw_html() {
        let source = concat!(
            "||~ [en] Flopstyle: LITE ||\n",
            "||= **By:** <span class=\"printuser\"><a href=\"http://www.wikidot.com/user:info/stormbreath\">stormbreath</a></span> ||\n",
            "||~ Published on <span class=\"odate time_1782003564 format_%25d%20%25b%20%25Y\">21 Jun 2026</span> ||",
        );
        let rendered = render_list_pages_table_rows(source)
            .expect("authorbox ListPages body should render as raw table HTML");

        assert!(rendered.contains(
            "<table class=\"wiki-content-table\" data-wikijump-compat-listpages=\"1\">"
        ));
        assert!(rendered.contains("<strong>By:</strong>"));
        assert!(rendered.contains("<span class=\"printuser\">"));
        assert!(rendered.contains("<span class=\"odate time_1782003564"));
        assert!(!rendered.contains("&lt;span"));

        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut protected = rendered.clone();
        let fragments = RenderService::protect_generated_wikidot_compat_html(
            &mut protected,
            &settings,
        );
        assert_eq!(fragments.len(), 1);
        assert!(!protected.contains("<table"));
        let restored = RenderService::restore_protected_generated_wikidot_compat_html(
            protected, &fragments,
        );
        assert!(restored.contains("<table class=\"wiki-content-table\">"));
        assert!(!restored.contains("data-wikijump-compat-listpages"));
    }

    #[test]
    fn unsupported_numbered_list_pages_body_does_not_leak_to_ftml() {
        let module_source = concat!(
            "[[module ListPages unsupported=\"yes\"]]\n",
            "# %%unsupported_variable%%\n",
            "[[/module]]",
        );

        assert_eq!(
            unsupported_list_pages_replacement(
                module_source,
                "# %%unsupported_variable%%\n"
            ),
            "[[div class=\"list-pages-box\"]][[/div]]",
        );
    }

    #[test]
    fn unsupported_tracking_list_pages_body_does_not_leak_to_ftml() {
        let body = concat!(
            "[[%%content{0}%%module listusers users=\".\"]]\n",
            "[[image https://manage.scp-jp.com/api/public/assets/layoutSupporter.png?s_id=578002&s_name=scp-jp&m_id=%%number%%&m_name=%%ti%%content{0}%%tle%%&fn=%%fullname%%]]\n",
            "[[%%content{0}%%/module]]\n",
            "[[image https://manage.scp-jp.com/api/public/assets/analytics.png?s_id=578002&fn=%%fullname%%]]\n",
        );
        let module_source = format!(
            "[[module ListPages category=\"*\" pagetype=\"*\" range=\".\" wrapper=\"no\" separate=\"no\"]]\n{body}[[/module]]"
        );

        assert!(list_pages_body_is_no_visible_tracking_markup(body));
        assert_eq!(
            unsupported_list_pages_replacement(&module_source, body),
            "[[div class=\"list-pages-box\"]][[/div]]",
        );
    }

    #[test]
    fn optional_no_visible_wikidot_includes_do_not_render_missing_page_text() {
        assert_eq!(
            wikidot_no_such_include_replacement(&PageRef::page_and_site(
                "drizzles", "raven"
            )),
            "",
        );
        assert_eq!(
            wikidot_no_such_include_replacement(&PageRef::page_and_site("crom", "pixel")),
            "",
        );
        assert_eq!(
            wikidot_no_such_include_replacement(&PageRef::page_and_site(
                "scp-jp", "missing"
            )),
            "No such page: :scp-jp:missing",
        );
    }

    #[test]
    fn renders_long_native_list_runs_before_ftml_parsing() {
        let source = [
            "* [[[tokyo-incidents|東京事変]]] -- by [[*user Ryu JP]]\n",
            "* [[[scp-2408-jp|SCP-2408-JP]]] -- by [[*user O-92_Mallet]]\n",
            "* [*http://scp-jp.wikidot.com/example Example] -- by [[*user Example]]\n",
            "* [[[empty-label|]]] -- by [[*user seafield13]]\n",
            "* [[[qingtan-what-is-odse|第一級異災特区]]]\n",
            "* [[[meltrose002|特等席より]]]\n",
            "* [[[confessio-natorum|告解する子供たち]]]\n",
            "* [[[souyamisaki014-12|メシがうまくて何が悪い]]]\n",
        ]
        .join("");

        let rendered = RenderService::render_long_native_list_runs(source);

        assert!(rendered.starts_with(r#"<ul data-wikijump-compat-list="1">"#));
        assert!(
            rendered.contains(r#"<li><a href="/tokyo-incidents">東京事変</a> -- by "#)
        );
        assert!(rendered.contains(r#"<span class="printuser"><a href="http://www.wikidot.com/user:info/Ryu JP">Ryu JP</a></span>"#));
        assert!(
            rendered
                .contains(r#"<a href="http://scp-jp.wikidot.com/example">Example</a>"#)
        );
        assert!(rendered.contains(r#"<a href="/empty-label">Empty Label</a>"#));
        assert!(!rendered.contains("[[*user"));

        let mut protected = rendered.clone();
        let fragments = RenderService::protect_generated_wikidot_compat_html(
            &mut protected,
            &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
        );
        assert_eq!(fragments.len(), 1);
        assert!(protected.starts_with(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
        let restored = RenderService::restore_protected_generated_wikidot_compat_html(
            protected, &fragments,
        );
        assert!(restored.starts_with("<ul>"));
        assert!(!restored.contains("data-wikijump-compat-list"));
    }

    #[test]
    fn defaults_empty_scp_style_page_link_labels_to_canonical_slug_text() {
        assert_eq!(native_list_page_link_default_label("scp-8066"), "SCP-8066");
        assert_eq!(native_list_page_link_default_label("SCP-8091"), "SCP-8091");
        assert_eq!(
            native_list_page_link_default_label("scp-2408-jp"),
            "SCP-2408-JP"
        );
        assert_eq!(
            native_list_page_link_default_label("ordinary-page-name"),
            "Ordinary Page Name"
        );
        assert_eq!(
            native_list_page_link_default_label("scp-foundation"),
            "Scp Foundation"
        );

        assert_eq!(
            render_native_list_page_link("scp-8066", None, None),
            r#"<a href="/scp-8066">SCP-8066</a>"#
        );
        assert_eq!(
            render_native_list_page_link("scp-8066", Some("the article"), None),
            r#"<a href="/scp-8066">the article</a>"#
        );
        assert_eq!(
            render_native_list_page_link("scp-8596", Some(""), None),
            r#"<a href="/scp-8596">SCP-8596</a>"#
        );
    }

    #[test]
    fn defaults_empty_page_link_labels_to_known_target_titles() {
        let mut titles = WikidotCompatLinkTitleMap::new();
        titles.insert(
            "dr-frueh-s-proposal".to_owned(),
            "DarkStuff's Proposal".to_owned(),
        );

        assert_eq!(
            render_native_list_page_link("dr-frueh-s-proposal", None, Some(&titles)),
            r#"<a href="/dr-frueh-s-proposal">DarkStuff's Proposal</a>"#
        );
        assert_eq!(
            render_native_list_page_link(
                "dr-frueh-s-proposal",
                Some("Family Life"),
                Some(&titles),
            ),
            r#"<a href="/dr-frueh-s-proposal">Family Life</a>"#
        );
        assert_eq!(
            render_native_list_page_link("missing-target", None, Some(&titles)),
            r#"<a href="/missing-target">Missing Target</a>"#
        );
        assert_eq!(
            render_native_list_page_link("scp-8066", None, Some(&titles)),
            r#"<a href="/scp-8066">SCP-8066</a>"#
        );
    }

    #[test]
    fn escapes_known_target_titles_used_for_empty_page_link_labels() {
        let mut titles = WikidotCompatLinkTitleMap::new();
        titles.insert(
            "target-page".to_owned(),
            r#"<img src=x onerror=alert(1)>"#.to_owned(),
        );

        assert_eq!(
            render_native_list_page_link("target-page", None, Some(&titles)),
            r#"<a href="/target-page">&lt;img src=x onerror=alert(1)&gt;</a>"#
        );
    }

    #[test]
    fn wikidot_compatibility_fallback_uses_known_target_titles_for_empty_links() {
        let mut titles = WikidotCompatLinkTitleMap::new();
        titles.insert(
            "dr-frueh-s-proposal".to_owned(),
            "DarkStuff's Proposal".to_owned(),
        );

        let output =
            RenderService::render_wikidot_compatibility_fallback_output_for_context(
                "[[[dr-frueh-s-proposal|]]]\n[[[scp-8066|]]]\n[[[missing-target|]]]",
                Some("scp-anthology-2024"),
                Some("scp-wiki"),
                Some(&titles),
            );

        assert!(
            output
                .body
                .contains(r#"<a href="/dr-frueh-s-proposal">DarkStuff's Proposal</a>"#)
        );
        assert!(output.body.contains(r#"<a href="/scp-8066">SCP-8066</a>"#));
        assert!(
            output
                .body
                .contains(r#"<a href="/missing-target">Missing Target</a>"#)
        );
    }

    #[test]
    fn renders_nested_native_list_runs_before_ftml_parsing() {
        let source = [
            "* About\n",
            " * [[[about-the-scp-foundation|About Us]]]\n",
            " * [[[Site Rules]]]\n",
            " * [[[FAQ]]]\n",
            "* Community\n",
            " * [[[news|Site News]]]\n",
            " * [[[chat-guide|IRC Chat]]]\n",
            " * [[[artist-directory|]]]\n",
            " * [[[http://05command.wikidot.com/staff-list | Staff List]]]\n",
            "* [[[contact-staff | Contact Us]]]\n",
        ]
        .join("");

        let rendered = RenderService::render_long_native_list_runs(source);

        assert!(rendered.contains(
            "<li><a href=\"javascript:;\">About\n</a><ul>\n<li><a href=\"/about-the-scp-foundation\">About Us</a></li>"
        ));
        assert!(rendered.contains(
            "</ul>\n</li>\n<li><a href=\"javascript:;\">Community\n</a><ul>\n<li><a href=\"/news\">Site News</a></li>"
        ));
        assert!(rendered.contains(
            "</ul>\n</li>\n<li><a href=\"/contact-staff\">Contact Us</a></li>"
        ));
        assert!(rendered.contains(r#"<a href="/site-rules">Site Rules</a>"#));
        assert!(rendered.contains(r#"<a href="/faq">FAQ</a>"#));
        assert!(rendered.contains(r#"<a href="/artist-directory">Artist Directory</a>"#));
        assert!(rendered.contains(
            r#"<a href="http://05command.wikidot.com/staff-list">Staff List</a>"#
        ));

        let mut protected = rendered.clone();
        let fragments = RenderService::protect_generated_wikidot_compat_html(
            &mut protected,
            &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
        );
        assert_eq!(fragments.len(), 1);
        assert!(protected.starts_with(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
        assert!(!protected.contains("</ul>"));

        let restored = RenderService::restore_protected_generated_wikidot_compat_html(
            protected, &fragments,
        );
        assert!(restored.starts_with("<ul>"));
        assert!(restored.contains("<li><a href=\"javascript:;\">About\n</a><ul>"));
        assert!(!restored.contains("data-wikijump-compat-list"));
    }

    #[test]
    fn normalizes_leading_indentation_for_native_list_runs() {
        let source = [
            " * Parent\n",
            "  * Child\n",
            "  * Another child\n",
            " * Sibling\n",
            " * Item 3\n",
            " * Item 4\n",
            " * Item 5\n",
            " * Item 6\n",
        ]
        .join("");

        let rendered = RenderService::render_long_native_list_runs(source);

        assert!(rendered.starts_with(r#"<ul data-wikijump-compat-list="1">"#));
        assert!(rendered.contains("<li><a href=\"javascript:;\">Parent\n</a><ul>"));
        assert!(!rendered.contains("<ul data-wikijump-compat-list=\"1\">\n<ul>"));
        assert!(!rendered.contains("</ul>\n</li>\n</li>"));
    }

    #[test]
    fn renders_css_modules_before_ftml_parsing() {
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut source = concat!(
            "before\n",
            "[[module css]]\n",
            "#u-change{\n",
            "    display:none;\n",
            "}\n",
            "[[/module]]\n",
            "after\n",
        )
        .to_owned();

        let mut fragments = CompatHtmlFragments::new(&source);
        RenderService::protect_wikidot_css_modules(
            &mut source,
            &settings,
            &mut fragments,
        );

        assert!(source.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
        assert!(!source.contains("[[module css]]"));
        assert!(!source.contains("#u-change"));

        let restored = fragments.restore(&source);
        assert!(restored.contains("<style>\n#u-change{"));
        assert!(restored.contains("display:none;"));
        assert!(!restored.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
    }

    #[test]
    fn escapes_css_module_style_end_tags() {
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut source = concat!(
            "[[module css]]\n",
            "</style><img src=x onerror=alert(1)><style>\n",
            "[[/module]]\n",
        )
        .to_owned();

        let mut fragments = CompatHtmlFragments::new(&source);
        RenderService::protect_wikidot_css_modules(
            &mut source,
            &settings,
            &mut fragments,
        );
        let restored = fragments.restore(&source);

        assert!(
            restored.starts_with("<style>\n"),
            "unexpected restored CSS module: {restored:?}",
        );
        assert!(
            restored.trim_end().ends_with("\n</style>"),
            "unexpected restored CSS module suffix: {restored:?}",
        );
        assert!(!restored.contains("</style><img"));
        assert!(restored.contains(r"\3C /style>\3C img"));
    }

    #[test]
    fn css_registry_handles_multiple_literal_and_malformed_boundaries() {
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let source = concat!(
            "[[module css]]\n.first { color: red; }\n[[/module]]\n",
            "[!-- comment starts\n[[module css]]\n.comment { color: bad; }\n",
            "[[/module]]\n--]\n",
            "[[module css]]\n.second { color: blue; }\n[[/module]]\n",
            "[[module css]]\n.spanning { color: black; }\n",
            "[!-- [[/module]] --]\n.end { color: white; }\n[[/module]]\n",
            "[[html]]\n[[module css]]\n.html { color: green; }\n[[/module]]\n[[/html]]\n",
            "[[module css]]\n.unclosed { display: none; }\n",
        );
        let mut protected = source.to_owned();
        let mut fragments = CompatHtmlFragments::new(source);

        RenderService::protect_wikidot_css_modules(
            &mut protected,
            &settings,
            &mut fragments,
        );
        let restored = fragments.restore(&protected);

        assert_eq!(
            protected
                .matches(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX)
                .count(),
            3,
        );
        assert!(restored.contains("<style>\n.first { color: red; }\n</style>"));
        assert!(restored.contains("<style>\n.second { color: blue; }\n</style>"));
        assert!(restored.contains(".spanning { color: black; }"));
        assert!(restored.contains(".end { color: white; }"));
        assert!(restored.contains(".comment { color: bad; }"));
        assert!(restored.contains("[[html]]\n[[module css]]\n.html"));
        assert!(restored.contains("[[module css]]\n.unclosed"));
        assert!(!restored.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
    }

    #[test]
    fn css_registry_restores_normal_and_fallback_outputs_without_marker_leakage() {
        let source = concat!(
            "before\n",
            "[[module css]]\n.a { color: red; }\n[[/module]]\n",
            "[[module css]]\n.b::after { content: \"</style>\"; }\n[[/module]]\n",
            "after\n",
        );

        for fallback in [false, true] {
            let html = render_wikidot_css_after_registry_restore(source, fallback);
            assert!(html.contains("<style>\n.a { color: red; }\n</style>"));
            assert!(html.contains(r#".b::after { content: "\3C /style>"; }"#));
            assert!(!html.contains("</style>\"; }"));
            assert!(!html.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
            assert!(!html.contains("[[module css]]"));
        }
    }

    #[test]
    fn protects_wikidot_bold_underline_spans_before_ftml_parsing() {
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut source =
            "**##C5000B|That might be the reason.**##\n**__10 October 2022**__\n**In which the finale is foreshadowed**\n"
                .to_owned();

        let spans =
            RenderService::protect_wikidot_inline_html_spans(&mut source, &settings);

        assert_eq!(spans.len(), 2);
        assert!(source.contains(&spans[0].marker));
        assert!(!source.contains("**##C5000B"));
        assert!(!source.contains("**__10 October 2022**__"));
        assert_eq!(
            spans[0].html,
            r#"<strong><span style="color: C5000B">That might be the reason.</span></strong>"#
        );
        assert_eq!(spans[1].html, r#"<strong><u>10 October 2022</u></strong>"#);

        let restored =
            RenderService::restore_protected_wikidot_inline_html(source, &spans);
        assert!(restored.contains(
            r#"<strong><span style="color: C5000B">That might be the reason.</span></strong>"#
        ));
        assert!(restored.contains(r#"<strong><u>10 October 2022</u></strong>"#));
        assert!(restored.contains("**In which the finale is foreshadowed**"));
    }

    #[test]
    fn protects_nested_bold_underline_closers_without_crossing_table_cells() {
        const ROW_COUNT: usize = 128;

        let page_info =
            fallback_test_page_info("nested-inline-table", "Nested inline table");
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let row = concat!(
            "||= 1 || [[span style=\"font-family: 'Handlee' ;\"]]",
            "##000080|**__B-Roll:__** body##[[/span]] || ",
            "**__V.O.:__** narration ||\n",
        );
        let mut source = row.repeat(ROW_COUNT);

        let inline_spans =
            RenderService::protect_wikidot_inline_html_spans(&mut source, &settings);
        let color_spans =
            RenderService::protect_wikidot_color_spans(&mut source, &settings);
        source =
            RenderService::escape_unrendered_wikidot_color_markers(source, &settings);

        assert_eq!(inline_spans.len(), ROW_COUNT * 2);
        assert_eq!(color_spans.len(), ROW_COUNT);
        assert!(!source.contains("**__"), "{source}");
        assert!(!source.contains("__**"), "{source}");
        assert!(
            inline_spans
                .iter()
                .any(|span| span.html == "<strong><u>B-Roll:</u></strong>"),
        );
        assert!(
            inline_spans
                .iter()
                .any(|span| span.html == "<strong><u>V.O.:</u></strong>"),
        );

        let started = Instant::now();
        ftml::preprocess(&mut source);
        let tokens = ftml::tokenize(&source);
        let result = ftml::parse(&tokens, &page_info, &settings);
        let (_tree, errors) = result.into();

        assert!(started.elapsed() < Duration::from_secs(5));
        assert!(errors.is_empty(), "{errors:#?}");
    }

    #[test]
    fn protects_wikidot_escaped_nbsp_entities_before_ftml_parsing() {
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut source = r#"Icon @<&nbsp;>@ label"#.to_owned();

        let spans =
            RenderService::protect_wikidot_inline_html_spans(&mut source, &settings);

        assert_eq!(spans.len(), 1);
        assert!(source.contains(&spans[0].marker));
        assert!(!source.contains("@<&nbsp;>@"));
        assert_eq!(spans[0].html, "&nbsp;");

        let restored =
            RenderService::restore_protected_wikidot_inline_html(source, &spans);
        assert!(restored.contains("Icon &nbsp; label"));
    }

    #[test]
    fn protects_wikidot_bold_outer_color_spans_before_cross_line_matching() {
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut source = concat!(
            "//<Dr. Lillihammer and Thilo Zwist are walking through **##green|a dense and mysterious forest##**. ",
            "As they move down **##sienna|a gently arcing path##**, **##ce005c|enigmatic figures##** can be glimpsed ",
            "in **##green|the distant underbrush##**.>//\n",
            "**##orange|Giftschreiber/Guard##:** **##orange|You're/we're##** not **##red|[SAFEGUARDS ENGAGED]##**.\n",
            "**##ce005c|What## ##blue|the ghost of long-drowned sorrow## ##ce005c|says is true.##**\n",
            "protected from **##ce005c |the fae the fae the fae##** work\n",
            "**##C5000B|That might be the reason.**##\n",
        )
        .to_owned();

        let inline_spans =
            RenderService::protect_wikidot_inline_html_spans(&mut source, &settings);
        let color_spans =
            RenderService::protect_wikidot_color_spans(&mut source, &settings);

        assert_eq!(inline_spans.len() + color_spans.len(), 12);
        assert!(!source.contains("sienna|a gently"));
        assert!(!source.contains("forest##**"));
        assert!(!source.contains("figures##**"));
        assert!(!source.contains("Dr. Lillihammer##"));
        assert!(!source.contains("##orange|You're/we're"));
        assert!(!source.contains("##blue|the ghost"));
        assert!(!source.contains("##ce005c |the fae"));
        assert!(source.contains("//<Dr. Lillihammer"));
        assert!(inline_spans.iter().any(|span| span.html.contains(
            r#"<strong><span style="color: green">a dense and mysterious forest</span></strong>"#
        )));
        assert!(inline_spans.iter().any(|span| span.html.contains(
            r#"<strong><span style="color: C5000B">That might be the reason.</span></strong>"#
        )));
        assert!(color_spans.iter().any(|span| span.html.contains(
            r#"<span style="color: blue">the ghost of long-drowned sorrow</span>"#
        )));
    }

    #[test]
    fn protects_wikidot_color_spans_before_ftml_parsing() {
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut source = "##blue|**[[include :scp-wiki:component:coltop**##\n".to_owned();

        let spans = RenderService::protect_wikidot_color_spans(&mut source, &settings);

        assert_eq!(spans.len(), 1);
        assert!(source.contains(&spans[0].marker));
        assert!(source.ends_with('\n'));
        assert!(!source.contains("<span"));
        assert!(!source.contains("##blue"));
        assert_eq!(
            spans[0].html,
            r#"<span style="color: blue"><strong>[[include :scp-wiki:component:coltop</strong></span>"#
        );

        let restored =
            RenderService::restore_protected_wikidot_color_spans(source, &spans);
        assert_eq!(
            restored,
            r#"<span style="color: blue"><strong>[[include :scp-wiki:component:coltop</strong></span>"#
                .to_owned() + "\n",
        );

        let escaped = RenderService::escape_unrendered_wikidot_color_markers(
            "####blue|leftover##".to_owned(),
            &settings,
        );
        assert_eq!(escaped, "&#35;&#35;&#35;&#35;blue|leftover&#35;&#35;");
    }

    #[test]
    fn protects_wikidot_hash_prefixed_hex_colors_without_shifted_matches() {
        let page_info = fallback_test_page_info("scp-6670", "SCP-6670");
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut wikitext = concat!(
            "###880808|plain##\n",
            "**###880808|bold outer##**\n",
            "**###880808|bold inner**##\n",
            "###12345|bad five##\n",
            "###gggggg|bad nonhex##\n",
            "####880808|bad run##\n",
            "###880808;background:red|bad css##\n",
        )
        .to_owned();

        let inline_spans =
            RenderService::protect_wikidot_inline_html_spans(&mut wikitext, &settings);
        let color_spans =
            RenderService::protect_wikidot_color_spans(&mut wikitext, &settings);
        assert_eq!(inline_spans.len(), 2);
        assert_eq!(color_spans.len(), 1);

        wikitext =
            RenderService::escape_unrendered_wikidot_color_markers(wikitext, &settings);
        ftml::preprocess(&mut wikitext);
        let tokens = ftml::tokenize(&wikitext);
        let result = ftml::parse(&tokens, &page_info, &settings);
        let (tree, errors) = result.into();
        assert!(errors.is_empty(), "{errors:?}");

        let rendered = HtmlRender.render(&tree, &page_info, &settings).body;
        let rendered =
            RenderService::restore_protected_wikidot_color_spans(rendered, &color_spans);
        let rendered =
            RenderService::restore_protected_wikidot_inline_html(rendered, &inline_spans);

        assert!(rendered.contains(r#"<span style="color: #880808">plain</span>"#));
        assert!(rendered.contains(
            r#"<strong><span style="color: #880808">bold outer</span></strong>"#
        ));
        assert!(rendered.contains(
            r#"<strong><span style="color: #880808">bold inner</span></strong>"#
        ));
        assert!(!rendered.contains(r#"#<span style="color: 880808">"#));
        assert!(!rendered.contains(r#"style="color: 880808""#));
        assert!(!rendered.contains(r#"style="color: 12345""#));
        assert!(!rendered.contains(r#"style="color: gggggg""#));
        assert!(!rendered.contains(r#"style="color: 880808;background"#));
    }

    #[test]
    fn wikidot_hash_prefixed_color_descriptor_accepts_only_three_or_six_hex_digits() {
        assert_eq!(
            parse_wikidot_compat_color_descriptor("###", "abc").as_deref(),
            Some("#abc"),
        );
        assert_eq!(
            parse_wikidot_compat_color_descriptor("###", "880808").as_deref(),
            Some("#880808"),
        );
        assert!(parse_wikidot_compat_color_descriptor("###", "12345").is_none());
        assert!(parse_wikidot_compat_color_descriptor("###", "gggggg").is_none());
        assert!(parse_wikidot_compat_color_descriptor("####", "880808").is_none());
        assert_eq!(
            parse_wikidot_compat_color_descriptor("##", "blue").as_deref(),
            Some("blue"),
        );
    }

    #[test]
    fn renders_protected_wikidot_color_spans_as_html_after_ftml_parsing() {
        let page_info = fallback_test_page_info("scp-8382", "SCP-8382");
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut wikitext = concat!(
            "+ ##8E2C4D|Lillian S. Lillihammer##\n\n",
            "**##8E2C4D|Memetics and Countermemetics##**\n",
            "**##ce005c|I am... I //should// be...##**\n",
            "**##C5000B|PATH uses North -- heading for the arctic -- red.##**\n",
            "##C5000B|**That might be the reason.**##\n",
        )
        .to_owned();

        let spans = RenderService::protect_wikidot_color_spans(&mut wikitext, &settings);
        wikitext =
            RenderService::escape_unrendered_wikidot_color_markers(wikitext, &settings);
        ftml::preprocess(&mut wikitext);
        let tokens = ftml::tokenize(&wikitext);
        let result = ftml::parse(&tokens, &page_info, &settings);
        let (tree, errors) = result.into();
        assert!(errors.is_empty(), "{errors:?}");

        let rendered = HtmlRender.render(&tree, &page_info, &settings).body;
        let rendered =
            RenderService::restore_protected_wikidot_color_spans(rendered, &spans);

        assert!(rendered.contains(
            r#"<h1 id="toc0"><span style="color: 8E2C4D">Lillian S. Lillihammer</span></h1>"#
        ));
        assert!(rendered.contains(
            r#"<strong><span style="color: 8E2C4D">Memetics and Countermemetics</span></strong>"#
        ));
        assert!(rendered.contains(
            r#"<strong><span style="color: ce005c">I am… I <em>should</em> be…</span></strong>"#
        ));
        assert!(rendered.contains(
            r#"<strong><span style="color: C5000B">PATH uses North — heading for the arctic — red.</span></strong>"#
        ));
        assert!(rendered.contains(
            r#"<span style="color: C5000B"><strong>That might be the reason.</strong></span>"#
        ));
        assert!(!rendered.contains("&lt;span"));
        assert!(!rendered.contains(WIKIDOT_COLOR_SPAN_SENTINEL_PREFIX));
    }

    #[test]
    fn protected_wikidot_inline_typography_does_not_rewrite_tag_attributes() {
        let rendered = super::render_wikidot_protected_inline_body_html(
            "[https://example.com/a--b go -- now...]",
        );

        assert_eq!(
            rendered,
            r#"<a href="https://example.com/a--b">go — now…</a>"#
        );
    }

    #[test]
    fn substitutes_wikidot_list_pages_author_and_created_at_variables() {
        let created_at = time::OffsetDateTime::from_unix_timestamp(1_782_003_564)
            .expect("fixture timestamp should be valid");
        let page = FoundPageRow {
            page_id: 1,
            site_id: 1,
            title: Some("Codex virtual Wikidot DOM 001".to_owned()),
            alt_title: None,
            slug: Some("dom-001".to_owned()),
            page_category_id: None,
            page_revision_id: None,
            tags: None,
            created_at: Some(created_at),
            created_by: Some(8_955_132),
            updated_at: None,
            updated_by: None,
            score: None,
        };
        let mut users = BTreeMap::new();
        users.insert(
            8_955_132,
            WikidotUserDisplay {
                user_id: 8_955_132,
                name: "scpaiueouiuiuiui".to_owned(),
                slug: Some("scpaiueouiuiuiui".to_owned()),
                wikidot_profile: true,
            },
        );

        let rendered = substitute_list_pages_variables(
            "||~ %%title%% ||\n||= **By:** %%created_by_linked%% ||\n||~ Published on %%created_at|%d %b %Y%% ||",
            &page,
            1,
            1,
            &list_pages_substitution_context(20, &users, None, &BTreeMap::new()),
        );

        assert!(rendered.contains("Codex virtual Wikidot DOM 001"));
        assert!(rendered.contains("user:info/scpaiueouiuiuiui"));
        assert!(rendered.contains("WIKIDOT.page.listeners.userInfo(8955132)"));
        assert!(!rendered.contains("userkarma.php"));
        assert!(rendered.contains(
            r#"<span class="odate time_1782003564 format_%25d%20%25b%20%25Y" data-wikijump-compat-date="1" style="cursor: help; display: inline;">21 Jun 2026</span>"#
        ));

        let rendered = substitute_list_pages_variables(
            "%%created_at%%",
            &page,
            1,
            1,
            &list_pages_substitution_context(20, &users, None, &BTreeMap::new()),
        );
        assert_eq!(
            rendered,
            r#"<span class="odate time_1782003564 format_%25e%20%25b%20%25Y%2C%20%25H%3A%25M" data-wikijump-compat-date="1" style="cursor: help; display: inline;">21 Jun 2026, 09:59</span>"#
        );

        let rendered = substitute_list_pages_variables(
            "%%date|%r|agohover%%",
            &page,
            1,
            1,
            &list_pages_substitution_context(20, &users, None, &BTreeMap::new()),
        );
        assert!(
            rendered.contains(r#"class="odate time_1782003564 format_%25r%7Cagohover""#)
        );
        assert!(!rendered.contains("agohover[[/span]]"));

        let rendered = substitute_list_pages_variables(
            "%%linked_title%%",
            &page,
            1,
            1,
            &list_pages_substitution_context(20, &users, None, &BTreeMap::new()),
        );
        assert_eq!(rendered, "[/dom-001 Codex virtual Wikidot DOM 001]");

        let ellipsis_title_page = FoundPageRow {
            title: Some("Now watch and learn, here's the deal...".to_owned()),
            ..page.clone()
        };
        let rendered = substitute_list_pages_variables(
            "%%title_linked%%",
            &ellipsis_title_page,
            1,
            1,
            &list_pages_substitution_context(20, &users, None, &BTreeMap::new()),
        );
        assert!(rendered.starts_with("[/dom-001 Now watch and learn, here's the deal"));
        assert!(rendered.contains(WIKIDOT_LISTPAGES_LITERAL_ELLIPSIS_SENTINEL_PREFIX));
        assert_eq!(
            restore_list_pages_literal_ellipsis_markers(&rendered),
            "[/dom-001 Now watch and learn, here's the deal...]"
        );

        let rendered = substitute_list_pages_variables(
            "%%author%%",
            &page,
            1,
            1,
            &list_pages_substitution_context(20, &users, None, &BTreeMap::new()),
        );
        assert!(rendered.contains("printuser avatarhover"));
        assert!(rendered.contains("user:info/scpaiueouiuiuiui"));

        let local_author = FoundPageRow {
            created_by: Some(ADMIN_USER_ID),
            ..page
        };
        let rendered = substitute_list_pages_variables(
            "%%author%%",
            &local_author,
            1,
            1,
            &list_pages_substitution_context(
                20,
                &BTreeMap::new(),
                None,
                &BTreeMap::new(),
            ),
        );
        assert_eq!(rendered, ADMIN_USER_ID.to_string());
        assert!(!rendered.contains("wikidot.com/user:info"));

        let mut local_users = BTreeMap::new();
        local_users.insert(
            -20,
            WikidotUserDisplay {
                user_id: -20,
                name: "SeekGull".to_owned(),
                slug: Some("seekgull".to_owned()),
                wikidot_profile: false,
            },
        );
        let local_mirror_author = FoundPageRow {
            created_by: Some(-20),
            ..local_author
        };
        let rendered = substitute_list_pages_variables(
            "%%created_by%% / %%author%%",
            &local_mirror_author,
            1,
            1,
            &list_pages_substitution_context(20, &local_users, None, &BTreeMap::new()),
        );
        assert_eq!(rendered, "SeekGull / SeekGull");
        assert!(!rendered.contains("wikidot.com/user:info"));
    }

    #[test]
    fn substitutes_wikidot_list_pages_limit_variable() {
        let body = "%%index%%/%%total%% limit=%%limit%% %%title%%";
        let page = FoundPageRow {
            page_id: 1,
            site_id: 1,
            title: Some("Codex fixture".to_owned()),
            alt_title: None,
            slug: Some("codex-fixture".to_owned()),
            page_category_id: None,
            page_revision_id: None,
            tags: None,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
            score: None,
        };

        assert!(list_pages_body_variables_supported(body));
        assert_eq!(
            substitute_list_pages_variables(
                body,
                &page,
                2,
                7,
                &list_pages_substitution_context(
                    20,
                    &BTreeMap::new(),
                    None,
                    &BTreeMap::new()
                ),
            ),
            "2/7 limit=20 Codex fixture",
        );
    }

    #[test]
    fn substitutes_wikidot_list_pages_author_tool_variables() {
        let updated_at = time::OffsetDateTime::from_unix_timestamp(1_782_005_400)
            .expect("fixture timestamp should be valid");
        let page = FoundPageRow {
            page_id: 1,
            site_id: 1,
            title: Some("SCP-2693".to_owned()),
            alt_title: None,
            slug: Some("scp-2693".to_owned()),
            page_category_id: None,
            page_revision_id: None,
            tags: Some(vec![
                "_image".to_owned(),
                "scp".to_owned(),
                "safe".to_owned(),
            ]),
            created_at: None,
            created_by: None,
            updated_at: Some(updated_at),
            updated_by: Some(954_000_337),
            score: Some(42.0),
        };
        let mut users = BTreeMap::new();
        users.insert(
            954_000_337,
            WikidotUserDisplay {
                user_id: 954_000_337,
                name: "Calibold".to_owned(),
                slug: Some("calibold".to_owned()),
                wikidot_profile: true,
            },
        );

        let body = concat!(
            "**%%title_linked%%**\n",
            "**Rating:** +%%rating%%\n",
            "**Comments:** %%comments%%\n",
            "**Last Comment:** %%commented_by%% (//%%commented_at|%D %H:%M|agohover%%//)\n",
            "**Last Edit:** %%updated_by%% (//%%updated_at|%D %H:%M|agohover%%//)\n",
            "%%tags_linked%%\n",
            "%%link%%",
        );
        assert!(list_pages_body_variables_supported(body));

        let rendered = substitute_list_pages_variables(
            body,
            &page,
            1,
            1,
            &list_pages_substitution_context(20, &users, None, &BTreeMap::new()),
        );

        assert!(rendered.contains("[/scp-2693 SCP-2693]"));
        assert!(rendered.contains("**Rating:** +42"));
        assert!(rendered.contains("**Last Edit:** Calibold"));
        assert!(rendered.contains(r#"<span class="odate time_1782005400"#));
        assert!(rendered.contains(r#"data-wikijump-compat-date="1""#));
        assert!(rendered.contains("[/system:page-tags/tag/scp scp]"));
        assert!(rendered.contains("[/system:page-tags/tag/safe safe]"));
        assert!(rendered.ends_with("scp-2693"));
        assert!(!rendered.contains("%%updated_by%%"));
        assert!(!rendered.contains("%%tags_linked%%"));
    }

    #[test]
    fn substitutes_imported_wikidot_snapshot_metadata_for_list_pages_rows() {
        let local_created_at = time::OffsetDateTime::from_unix_timestamp(1_600_000_000)
            .expect("fixture timestamp should be valid");
        let source_created_at = time::OffsetDateTime::from_unix_timestamp(1_781_900_521)
            .expect("fixture timestamp should be valid");
        let source_commented_at =
            time::OffsetDateTime::from_unix_timestamp(1_781_934_132)
                .expect("fixture timestamp should be valid");
        let page = FoundPageRow {
            page_id: 101,
            site_id: 1,
            title: Some("Aspenq Pride Art 2026".to_owned()),
            alt_title: None,
            slug: Some("aspenq-pride-art-2026".to_owned()),
            page_category_id: None,
            page_revision_id: None,
            tags: None,
            created_at: Some(local_created_at),
            created_by: Some(ADMIN_USER_ID),
            updated_at: Some(local_created_at),
            updated_by: Some(ADMIN_USER_ID),
            score: Some(28.0),
        };
        let mut users = BTreeMap::new();
        users.insert(
            ADMIN_USER_ID,
            WikidotUserDisplay {
                user_id: ADMIN_USER_ID,
                name: "Administrator".to_owned(),
                slug: Some("admin".to_owned()),
                wikidot_profile: false,
            },
        );
        let mut snapshots = BTreeMap::new();
        snapshots.insert(
            101,
            ListPagesSnapshotDisplay {
                created_at: source_created_at,
                updated_at: source_created_at,
                created_by_name: Some("Aspenq".to_owned()),
                updated_by_name: Some("Aspenq".to_owned()),
                comments: 10,
                commented_at: Some(source_commented_at),
                commented_by_name: Some("Aspenq".to_owned()),
                rating_votes: Some(31),
            },
        );

        let rendered = substitute_list_pages_variables(
            "%%title%% by %%author%% on %%created_at|%Y %b %e|agohover%% -- %%comments%% Comments -- %%commented_by%% %%commented_at|%Y %b %e%% -- %%rating_votes%% votes",
            &page,
            1,
            1,
            &list_pages_substitution_context_with_mode(
                20,
                &users,
                &snapshots,
                None,
                &BTreeMap::new(),
                false,
            ),
        );

        assert!(rendered.contains("Aspenq Pride Art 2026 by "));
        assert!(rendered.contains("by Aspenq on "));
        assert!(rendered.contains("2026 Jun 20"));
        assert!(rendered.contains("10 Comments"));
        assert!(rendered.contains("-- Aspenq "));
        assert!(rendered.contains("-- 31 votes"));
        assert!(rendered.contains(r#"style="cursor: help; display: inline;""#));
        assert!(!rendered.contains("Administrator"));
        assert_eq!(
            rendered.matches("data-wikijump-compat-date=\"1\"").count(),
            2
        );
        assert!(!rendered.contains("user:info"));
        assert!(!rendered.contains("2020 Sep"));
        assert!(!rendered.contains("%%comments%%"));
    }

    #[test]
    fn missing_snapshot_vote_count_uses_zero_vote_ratio_state() {
        let page = FoundPageRow {
            page_id: 101,
            site_id: 1,
            title: Some("Ratio page".to_owned()),
            alt_title: None,
            slug: Some("ratio-page".to_owned()),
            page_category_id: None,
            page_revision_id: None,
            tags: None,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
            score: Some(49.0),
        };
        let body = concat!(
            "[[#ifexpr %%rating_votes%% == 0 | zero-vote | has-votes]] ",
            "[[#expr (%%rating%%+%%rating_votes%%)/2]] ",
            "[[#expr (%%rating_votes%%-%%rating%%)/2/%%rating_votes%%*(-180)]]",
        );

        let rendered = substitute_list_pages_variables(
            body,
            &page,
            1,
            1,
            &list_pages_substitution_context(
                20,
                &BTreeMap::new(),
                None,
                &BTreeMap::new(),
            ),
        );

        assert_eq!(rendered, "zero-vote 24.5 0");
        assert!(!rendered.contains("[[#"));
    }

    #[test]
    fn substitutes_wikidot_list_pages_table_body_generated_variables_as_html() {
        let created_at = time::OffsetDateTime::from_unix_timestamp(1_782_003_564)
            .expect("fixture timestamp should be valid");
        let page = FoundPageRow {
            page_id: 1,
            site_id: 1,
            title: Some("Codex virtual Wikidot DOM 001".to_owned()),
            alt_title: None,
            slug: Some("dom-001".to_owned()),
            page_category_id: None,
            page_revision_id: None,
            tags: Some(vec![
                "_image".to_owned(),
                "scp".to_owned(),
                "safe".to_owned(),
                "preview".to_owned(),
            ]),
            created_at: Some(created_at),
            created_by: None,
            updated_at: None,
            updated_by: None,
            score: None,
        };
        let body = concat!(
            "||~ Published on %%created_at|%d %b %Y%% ||\n",
            "||= %%tags_linked%% ||",
        );

        let substituted = substitute_list_pages_variables(
            body,
            &page,
            1,
            1,
            &list_pages_substitution_context_with_mode(
                20,
                &BTreeMap::new(),
                empty_list_pages_snapshot_displays(),
                None,
                &BTreeMap::new(),
                true,
            ),
        );

        assert!(substituted.contains(
            r#"<span class="odate time_1782003564 format_%25d%20%25b%20%25Y" style="cursor: help; display: inline;">21 Jun 2026</span>"#
        ));
        assert!(substituted.contains(r#"<a href="/system:page-tags/tag/scp">scp</a>"#));
        assert!(
            substituted
                .contains(r#"<a href="/system:page-tags/tag/preview">preview</a>"#)
        );
        assert!(!substituted.contains("_image"));
        assert!(!substituted.contains("[[span"));
        assert!(!substituted.contains("[/system:page-tags/tag/scp scp]"));

        let rendered = render_list_pages_table_rows(&substituted)
            .expect("table-shaped ListPages body should render as raw table HTML");

        assert!(rendered.contains("<table class=\"wiki-content-table\""));
        assert!(rendered.contains(r#"<span class="odate time_1782003564"#));
        assert!(rendered.contains(r#"<a href="/system:page-tags/tag/scp">scp</a>"#));
        assert!(
            rendered.contains(r#"<a href="/system:page-tags/tag/preview">preview</a>"#)
        );
        assert!(!rendered.contains("&lt;span"));
        assert!(!rendered.contains("&lt;a href"));
    }

    #[test]
    fn substitutes_artwork_hub_listpages_body_without_visible_html_or_parser_functions() {
        let created_at = time::OffsetDateTime::from_unix_timestamp(1_781_900_521)
            .expect("fixture timestamp should be valid");
        let page = FoundPageRow {
            page_id: 1,
            site_id: 1,
            title: Some("Aspenq Pride Art 2026".to_owned()),
            alt_title: None,
            slug: Some("aspenq-pride-art-2026".to_owned()),
            page_category_id: None,
            page_revision_id: None,
            tags: Some(vec![
                "_image".to_owned(),
                "_licensebox".to_owned(),
                "artwork".to_owned(),
                "preview".to_owned(),
                "colored-pencil".to_owned(),
                "pridefest2026".to_owned(),
            ]),
            created_at: Some(created_at),
            created_by: None,
            updated_at: None,
            updated_by: None,
            score: Some(28.0),
        };
        let body = concat!(
            "[[div class=\"tale-block %%tags%%\"]]\n",
            "[[div_ class=\"title\"]]%%linked_title%%[[/div]]\n",
            "[[div_ class=\"date\"]]%%created_at|%Y %b %e|agohover%%[[/div]]\n",
            "[[span class=\"tag-list\"]]%%tags_linked|artwork-hub/tag/-scp,-goi-format,-supplement,-tale,-hub,-site,-resource,-guide,-essay,-theme,%%[[/span]]\n",
            "[[span class=\"rating\"]][[#ifexpr %%rating%% > -1 | + | - ]][[#expr abs(%%rating%%)]][[/span]]\n",
            "[[/div]]",
        );

        let rendered = substitute_list_pages_variables(
            body,
            &page,
            1,
            1,
            &list_pages_substitution_context(
                20,
                &BTreeMap::new(),
                None,
                &BTreeMap::new(),
            ),
        );

        assert!(rendered.contains(
            "[[div class=\"tale-block artwork preview colored-pencil pridefest2026\"]]"
        ));
        assert!(!rendered.contains("_image"));
        assert!(!rendered.contains("_licensebox"));
        assert!(rendered.contains("[/aspenq-pride-art-2026 Aspenq Pride Art 2026]"));
        assert!(rendered.contains(r#"<span class="odate time_1781900521 format_%25Y%20%25b%20%25e%7Cagohover" data-wikijump-compat-date="1" style="cursor: help; display: inline;">2026 Jun 20</span>"#));
        assert!(rendered.contains("[/artwork-hub/tag/-scp,-goi-format,-supplement,-tale,-hub,-site,-resource,-guide,-essay,-theme,artwork artwork]"));
        assert!(rendered.contains("[/artwork-hub/tag/-scp,-goi-format,-supplement,-tale,-hub,-site,-resource,-guide,-essay,-theme,preview preview]"));
        assert!(rendered.contains("[/artwork-hub/tag/-scp,-goi-format,-supplement,-tale,-hub,-site,-resource,-guide,-essay,-theme,colored-pencil colored-pencil]"));
        assert!(rendered.contains(r#"[[span class="rating"]]+28[[/span]]"#));
        assert_eq!(
            rendered.matches("data-wikijump-compat-date=\"1\"").count(),
            1
        );
        assert!(!rendered.contains("<a href"));
        assert!(!rendered.contains("&lt;span"));
        assert!(!rendered.contains("[[#ifexpr"));
        assert!(!rendered.contains("[[#expr"));
        assert!(!rendered.contains("agohover[[/span]]"));
    }

    #[test]
    fn resolves_wikidot_signed_abs_rating_expressions() {
        assert_eq!(
            resolve_list_pages_signed_abs_expressions(
                "[[#ifexpr -3 > -1 | + | - ]][[#expr abs(-3)]]",
            ),
            "-3",
        );
        assert_eq!(
            resolve_list_pages_signed_abs_expressions(
                "[[#ifexpr 4.5 > -1 | + | - ]][[#expr abs(4.5)]]",
            ),
            "+4.5",
        );
        assert_eq!(
            resolve_list_pages_signed_abs_expressions(
                "[[#ifexpr -3 > -1 | + | - ]][[#expr abs(4)]]",
            ),
            "[[#ifexpr -3 > -1 | + | - ]][[#expr abs(4)]]",
        );
    }

    #[test]
    fn moves_sentence_punctuation_outside_wikidot_email_anchor() {
        let html = concat!(
            r#"<p>For more information, contact "#,
            r#"<span class="wiki-email" style="visibility: visible;">"#,
            r#"<a href="mailto:training@nfsi.gov.">training@nfsi.gov.</a></span></p>"#,
        );

        assert_eq!(
            RenderService::restore_wikidot_email_obfuscation(html),
            concat!(
                r#"<p>For more information, contact "#,
                r#"<span class="wiki-email" style="visibility: visible;">"#,
                r#"<a href="mailto:training@nfsi.gov">training@nfsi.gov</a></span>."#,
                r#"</p>"#,
            ),
        );
    }

    #[test]
    fn decodes_escaped_wikidot_email_before_rendering_visible_anchor() {
        let html = concat!(
            r#"<span class="wiki-email" style="visibility: visible;">"#,
            r#"<a href="mailto:o&#39;hara@example.com">o&#39;hara@example.com</a></span>"#,
        );

        assert_eq!(
            RenderService::restore_wikidot_email_obfuscation(html),
            concat!(
                r#"<span class="wiki-email" style="visibility: visible;">"#,
                r#"<a href="mailto:o'hara@example.com">o'hara@example.com</a></span>"#,
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
    fn removes_spurious_wikidot_email_classes_after_render() {
        let html = concat!(
            r#"<p><span class="wiki-email">]]etontoof/[[.}}@@]]@|{{#]]etontoof/[[.}}@@]]@|{{</span></p>"#,
            r#"<p><span class="wiki-email">vog.ibf|320sggirb.j#vog.ibf|320sggirb.j</span></p>"#,
        );

        assert_eq!(
            RenderService::remove_spurious_wikidot_email_classes(html),
            concat!(
                r#"<p><span>]]etontoof/[[.}}@@]]@|{{#]]etontoof/[[.}}@@]]@|{{</span></p>"#,
                r#"<p><span class="wiki-email">vog.ibf|320sggirb.j#vog.ibf|320sggirb.j</span></p>"#,
            ),
        );
    }

    #[test]
    fn preserves_recoverable_wikidot_email_classes_after_render() {
        let html = concat!(
            r#"<p>Jim Briggs <span class="wiki-email">"#,
            r#"]]naps/[[;tg&vog.ibf|320sggirb.j;tl&#]]naps/[[;tg&vog.ibf|320sggirb.j;tl&"#,
            r#"</span></p>"#,
        );

        assert_eq!(
            RenderService::remove_spurious_wikidot_email_classes(html),
            concat!(
                r#"<p>Jim Briggs <span class="wiki-email">"#,
                r#"vog.ibf|320sggirb.j#vog.ibf|320sggirb.j"#,
                r#"</span></p>"#,
            ),
        );
    }

    #[test]
    fn parses_negative_singular_list_pages_tag_as_exclusion() {
        let arguments =
            parse_list_pages_arguments(r#"tag="-excluded" limit="10" order="name""#)
                .expect("negative singular tag selector should parse as exclusion");

        assert_eq!(arguments.no_tags, vec![Cow::Borrowed("excluded")]);
        assert_eq!(arguments.limit, Some(10));
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
    fn localizes_wikidot_local_file_urls_for_corpus_site_slug() {
        let site = wikidot_site("scp-wiki-en-corpus-scp9506-slice-v2", None);
        let mut config = Config::integration_testing();
        config.files_domain = ".wjfiles.localhost".to_owned();
        config.files_domain_no_dot = "wjfiles.localhost".to_owned();
        let html =
            r#"<img src="http://scp-wiki.wikidot.com/local--files/scp-9506/NFSI.png">"#;

        assert_eq!(
            RenderService::localize_wikidot_local_file_urls(html, Some(&site), &config,),
            r#"<img src="https://scp-wiki-en-corpus-scp9506-slice-v2.wjfiles.localhost/local--files/scp-9506/NFSI.png">"#,
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
    fn localizes_cross_site_wdfiles_local_file_urls_for_imported_corpus_files() {
        let mut site = wikidot_site("scp-wiki", Some("scp-wiki.wikidot.com"));
        site.from_wikidot = false;
        let mut config = Config::integration_testing();
        config.files_domain = ".wjfiles.localhost".to_owned();
        config.files_domain_no_dot = "wjfiles.localhost".to_owned();
        let html = concat!(
            r#"<img src="https://scp-sandbox-3.wdfiles.com/local--files/harry-blank-9/Lillihammer_Preview.png">"#,
            r#"<style>.logo{background:url("http://scp-sandbox-3.wdfiles.com/local--files/harry-blank-4/deicidium-logo.svg")}</style>"#,
            r#"<style>@import "https://scp-sandbox-3.wdfiles.com/local--code/theme%3Aforeign/1";</style>"#,
        );

        assert_eq!(
            RenderService::localize_wikidot_local_file_urls(html, Some(&site), &config,),
            concat!(
                r#"<img src="https://scp-wiki.wjfiles.localhost/local--files/harry-blank-9/Lillihammer_Preview.png">"#,
                r#"<style>.logo{background:url("https://scp-wiki.wjfiles.localhost/local--files/harry-blank-4/deicidium-logo.svg")}</style>"#,
                r#"<style>@import "https://scp-sandbox-3.wdfiles.com/local--code/theme%3Aforeign/1";</style>"#,
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
    fn corpus_replay_worker_preparation_uses_production_protection_order() {
        let page_info = fallback_test_page_info("replay", "Replay");
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let input = CorpusReplayExpandedWikitext {
            wikitext: concat!(
                "Before\ttext\r\n",
                "**__Label:__** body\n",
                "[[module css]]\n.x { color: red; }\n[[/module]]\n",
            )
            .to_owned(),
            page_info,
            settings,
            id: PageId {
                site_id: 1,
                category_id: 2,
                page_id: 3,
            },
            included_pages: vec![PageRef::page_only("component:fixture")],
            wikidot_compat_html: CompatHtmlFragments::new(""),
        };
        let encoded = serde_json::to_string(&input).expect("serialize replay input");
        let decoded = serde_json::from_str(&encoded).expect("deserialize replay input");

        let prepared = RenderService::prepare_corpus_replay_wikitext(decoded);

        assert!(!prepared.compatibility_fallback);
        assert!(prepared.preprocessed);
        assert_eq!(prepared.included_pages.len(), 1);
        assert!(prepared.wikitext.contains("Before    text\n"));
        assert!(!prepared.wikitext.contains('\t'));
        assert!(!prepared.wikitext.contains('\r'));
        assert!(!prepared.wikitext.contains("**__Label:__**"));
        assert!(!prepared.wikitext.contains("[[module css]]"));
        assert!(
            prepared
                .wikitext
                .contains(WIKIDOT_INLINE_HTML_SENTINEL_PREFIX)
        );
        assert!(
            prepared
                .wikitext
                .contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX)
        );
        assert_eq!(prepared.features.bytes, prepared.wikitext.len());
        assert!(prepared.features.lines >= 3);

        let decoded = serde_json::from_str(&encoded).expect("deserialize replay input");
        let mut stages = Vec::new();
        let _ = RenderService::prepare_corpus_replay_wikitext_with_observer(
            decoded,
            |stage| stages.push(stage),
        );
        assert_eq!(
            stages,
            vec![
                CorpusReplayPreparationStage::Normalization,
                CorpusReplayPreparationStage::OuterProtection,
                CorpusReplayPreparationStage::FallbackCheck,
                CorpusReplayPreparationStage::InnerProtection,
                CorpusReplayPreparationStage::Preprocess,
            ],
        );
    }

    #[test]
    fn corpus_replay_worker_does_not_preprocess_fallback_pages() {
        let mut wikitext = String::new();
        for index in 0..=MAX_FTML_COMPAT_COLLAPSIBLE_BLOCKS {
            wikitext.push_str(&format!(
                "[[collapsible show=\"+ {index}\" hide=\"- {index}\"]]\nbody\n[[/collapsible]]\n"
            ));
        }
        let input = CorpusReplayExpandedWikitext {
            wikitext: wikitext.clone(),
            page_info: fallback_test_page_info("fallback-replay", "Fallback Replay"),
            settings: WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
            id: PageId {
                site_id: 1,
                category_id: 2,
                page_id: 3,
            },
            included_pages: Vec::new(),
            wikidot_compat_html: CompatHtmlFragments::new(""),
        };

        let prepared = RenderService::prepare_corpus_replay_wikitext(input);

        assert!(prepared.compatibility_fallback);
        assert!(!prepared.preprocessed);
        assert_eq!(prepared.wikitext, wikitext);
        assert_eq!(prepared.timings.inner_protection_us, 0);
        assert_eq!(prepared.timings.preprocess_us, 0);
    }

    #[test]
    fn component_heavy_corpus_page_stays_parse_eligible() {
        const DOM045_EXPANDED_WIKITEXT_BYTES: usize = 507_299;

        const {
            assert!(DOM045_EXPANDED_WIKITEXT_BYTES < MAX_FTML_COMPAT_PARSE_BYTES);
            assert!(1_500_000 > MAX_FTML_COMPAT_PARSE_BYTES);
        };
    }

    #[test]
    fn dense_style_resource_uses_compatibility_fallback_before_ftml() {
        let mut source = String::new();
        source.push_str("[[include component:anomaly-class-bar-source]]\n");
        for index in 0..88 {
            source.push_str(&format!("[[include component:example-{index} p=value]]\n"));
        }
        for index in 0..580 {
            source.push_str(&format!(
                ".scp-style-{index} {{ --accent-{index}: #b01; color: #111; }}\n"
            ));
        }
        for index in 0..350 {
            source.push_str(&format!(
                "* [[[style-resource-{index}|Style Resource {index}]]]\n"
            ));
        }
        while source.len() < 105_000 {
            source.push_str("ordinary corpus prose line\n");
        }

        assert!(
            RenderService::wikidot_compat_parse_complexity_score(&source)
                > MAX_FTML_COMPAT_DENSE_PARSE_SCORE
        );
        assert!(RenderService::should_use_wikidot_compatibility_fallback(
            &source,
            &fallback_test_page_info(
                "vg021-jp-scp-style-resource-769330c42a",
                "[jp] SCPスタイルリソース"
            )
        ));
    }

    #[test]
    fn wikidot_compatibility_fallback_preserves_code_blocks() {
        let source = concat!(
            "Before\n",
            "[[code type=\"css\"]]\n",
            ".x { color: red; }\n",
            "[[/code]]\n",
            "After\n",
        );

        let html =
            RenderService::render_wikidot_compatibility_fallback_with_code_blocks(source);

        assert!(html.contains(r#"<div class="wikidot-compat-fallback">"#));
        assert!(html.contains(
            r#"<div class="code"><pre><code>.x { color: red; }</code></pre></div>"#
        ));
        assert!(html.contains("<pre>Before</pre>"));
        assert!(html.contains("<pre>After</pre>"));
        assert!(!html.contains("[[code"));
        assert!(!html.contains("[[/code]]"));
    }

    #[test]
    fn wikidot_compatibility_fallback_keeps_unclosed_code_literal() {
        let source =
            concat!("Before\n", "[[code]]\n", ".x { color: red; }\n", "After\n",);

        let html =
            RenderService::render_wikidot_compatibility_fallback_with_code_blocks(source);

        assert!(html.contains("code"));
        assert!(html.contains("color: red"));
        assert!(!html.contains(r#"<div class="code">"#));
    }

    #[test]
    fn wikidot_compatibility_fallback_preserves_collapsible_code_blocks() {
        let source = concat!(
            "Before\n",
            "[[collapsible show=\"+ open\" hide=\"- close\" folded=\"no\"]]\n",
            "[[code]]\n",
            ".x { color: red; }\n",
            "[[/code]]\n",
            "[[/collapsible]]\n",
            "After\n",
        );

        let html =
            RenderService::render_wikidot_compatibility_fallback_with_code_blocks(source);

        assert!(html.contains(r#"<div class="collapsible-block">"#));
        assert!(
            html.contains(
                r#"<div class="collapsible-block-folded" style="display:none">"#
            )
        );
        assert!(html.contains(r#"<div class="collapsible-block-unfolded">"#));
        assert!(html.contains(r#"<div class="collapsible-block-content">"#));
        assert!(html.contains("+ open"));
        assert!(html.contains("- close"));
        assert!(html.contains(
            r#"<div class="code"><pre><code>.x { color: red; }</code></pre></div>"#
        ));
        assert!(!html.contains("[[collapsible"));
        assert!(!html.contains("[[/collapsible]]"));
    }

    #[test]
    fn wikidot_compatibility_fallback_leaves_quoted_collapsible_literal() {
        let source = concat!(
            "> @@[[collapsible]]@@\n",
            "> quoted body\n",
            "> @@[[/collapsible]]@@\n",
        );

        let html =
            RenderService::render_wikidot_compatibility_fallback_with_code_blocks(source);

        assert!(html.contains("collapsible"));
        assert!(!html.contains(r#"<div class="collapsible-block">"#));
    }

    #[test]
    fn wikidot_compatibility_fallback_renders_generated_listpages_divs() {
        let source = concat!(
            "[[div class=\"list-pages-box\"]]\n",
            "[[div_]]\n",
            "[[div class=\"list-pages-item\"]]\n",
            "**<span class=\"odate time_123 format_%25e%20%25b%20%25Y%20%25H%3A%25M\">9 Aug 2017 13:06</span> <span style=\"color: green\">+3034</span>**\n",
            "[[/div]]\n",
            "[[/div]]\n",
            "[[/div]]\n",
        );

        let html =
            RenderService::render_wikidot_compatibility_fallback_with_code_blocks(source);

        assert!(html.contains(r#"<div class="list-pages-box">"#));
        assert!(html.contains(r#"<div><div class="list-pages-item">"#));
        assert!(html.contains(r#"<div class="list-pages-item">"#));
        assert!(html.contains("<strong>"));
        assert!(html.contains(
            r#"<span class="odate time_123 format_%25e%20%25b%20%25Y%20%25H%3A%25M">"#
        ));
        assert!(html.contains(r#"<span style="color: green">+3034</span>"#));
        assert!(html.contains("9 Aug 2017 13:06"));
        assert!(html.contains("+3034"));
        assert!(!html.contains("[[div"));
        assert!(!html.contains("[[/div]]"));
    }

    #[test]
    fn wikidot_compatibility_fallback_strips_visible_comment_blocks() {
        let source = concat!(
            "[[module CSS]]\n",
            ".theme { display: block; }\n",
            "[[/module]]\n",
            "[[div_]]\n",
            "[!--\n",
            "Usage:\n",
            " [[include :scp-wiki:component:interwiki-style\n",
            "| priority=X\n",
            "]]\n",
            "--]\n",
            "Visible body\n",
            "[[/div]]\n",
        );
        let html = render_wikidot_css_after_registry_restore(source, true);

        assert!(html.contains(r#"<div class="wikidot-compat-fallback">"#));
        assert!(html.contains(r#"<div><p>Visible body</p></div>"#));
        assert!(html.contains(".theme { display: block; }"));
        assert!(!html.contains("Usage:"));
        assert!(!html.contains("[[include :scp-wiki:component:interwiki-style"));
        assert!(!html.contains("[!--"));
        assert!(!html.contains("[[div_]]"));
    }

    #[test]
    fn wikidot_compatibility_fallback_renders_page_body_block_markers() {
        let source = concat!(
            "[[=]]\n",
            "Centered body\n",
            "[[/=]]\n",
            "////\n",
            "[[div class=\"city-block\"]]\n",
            "-----\n",
            "[[/div]]\n",
        );

        let output =
            RenderService::render_wikidot_compatibility_fallback_output_for_context(
                source,
                Some("scp-anthology-2024"),
                Some("scp-wiki"),
                None,
            );

        assert!(output.body.contains(r#"<div style="text-align: center;">"#));
        assert!(output.body.contains("<p>Centered body</p></div>"));
        assert!(output.body.contains("<br>"));
        assert!(
            output
                .body
                .contains(r#"<div class="city-block"><hr></div>"#)
        );
        assert!(output.html_block_texts.is_empty());
        assert!(!output.body.contains("[[=]]"));
        assert!(!output.body.contains("[[/=]]"));
        assert!(!output.body.contains("////"));
        assert!(!output.body.contains("-----"));
    }

    #[test]
    fn wikidot_compatibility_fallback_centers_read_only_rate_module() {
        let source = format!(
            "[[=]]\n{}\n[[/=]]\n",
            render_read_only_rate_module(ftml::data::ScoreValue::Integer(396), "en",),
        );

        let output =
            RenderService::render_wikidot_compatibility_fallback_output_for_context(
                &source,
                Some("scp-anthology-2024"),
                Some("scp-wiki"),
                None,
            );

        assert!(output.body.contains(
            r#"<div style="text-align: center;"><div class="page-rate-widget-box">"#
        ));
        assert!(output.body.contains(r#"<span class="rate-points">rating: <span class="number prw54353">+396</span></span>"#));
        assert!(output.body.contains(r#"<span class="rateup btn btn-default"><a href="javascript:;" onclick="WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, 1)" title="I like it">+</a></span>"#));
        assert!(output.body.contains("</div></div>"));
        assert!(!output.body.contains("[[=]]"));
        assert!(!output.body.contains("[[/=]]"));
    }

    #[test]
    fn wikidot_compatibility_fallback_collects_html_blocks_as_iframes() {
        let source = concat!(
            "[[div_ class=\"audio_iframe INTRO\"]]\n",
            "[[html]]\n",
            "<html>\n",
            "<body><script src=\"https://example.test/audio.js\"></script></body>\n",
            "</html>\n",
            "[[/html]]\n",
            "[[/div]]\n",
        );

        let output =
            RenderService::render_wikidot_compatibility_fallback_output_for_context(
                source,
                Some("scp-anthology-2024"),
                Some("scp-wiki"),
                None,
            );

        assert_eq!(output.html_block_texts.len(), 1);
        assert!(output.html_block_texts[0].contains("<script src="));
        assert!(output.body.contains(r#"<div class="audio_iframe INTRO"><iframe src="/scp-anthology-2024/html/1" allowtransparency="true" frameborder="0" class="html-block-iframe"></iframe></div>"#));
        assert!(!output.body.contains("&lt;script"));
        assert!(!output.body.contains("[[html]]"));
        assert!(!output.body.contains("[[/html]]"));
    }

    #[test]
    fn wikidot_compatibility_fallback_keeps_raw_script_and_unclosed_html_literal() {
        let source = concat!(
            "<script>alert(1)</script>\n",
            "[[html]]\n",
            "<span>unfinished</span>\n",
        );

        let output =
            RenderService::render_wikidot_compatibility_fallback_output_for_context(
                source,
                Some("scp-anthology-2024"),
                Some("scp-wiki"),
                None,
            );

        assert!(output.html_block_texts.is_empty());
        assert!(
            output
                .body
                .contains("&lt;script&gt;alert(1)&lt;/script&gt;")
        );
        assert!(output.body.contains("[[html]]"));
        assert!(
            !output
                .body
                .contains(r#"<iframe src="/scp-anthology-2024/html/1""#)
        );
    }

    #[test]
    fn wikidot_compatibility_fallback_leaves_markers_inside_code_blocks_literal() {
        let source = concat!(
            "[[code]]\n",
            "[[=]]\n",
            "////\n",
            "[[html]]\n",
            "[[/code]]\n",
        );

        let output =
            RenderService::render_wikidot_compatibility_fallback_output_for_context(
                source,
                Some("scp-anthology-2024"),
                Some("scp-wiki"),
                None,
            );

        assert!(output.html_block_texts.is_empty());
        assert!(output.body.contains("[[=]]"));
        assert!(output.body.contains("////"));
        assert!(output.body.contains("[[html]]"));
        assert!(!output.body.contains(r#"<div style="text-align: center;">"#));
        assert!(
            !output
                .body
                .contains(r#"<iframe src="/scp-anthology-2024/html/1""#)
        );
    }

    #[test]
    fn wikidot_compatibility_fallback_preserves_comments_inside_code_blocks() {
        let source = concat!(
            "[[code]]\n",
            "[!-- kept as code --]\n",
            "[[/code]]\n",
            "[!-- hidden prose --]\n",
            "Visible prose\n",
        );

        let html =
            RenderService::render_wikidot_compatibility_fallback_with_code_blocks(source);

        assert!(html.contains("[!-- kept as code --]"));
        assert!(html.contains("Visible prose"));
        assert!(!html.contains("hidden prose"));
    }

    #[test]
    fn wikidot_compatibility_fallback_renders_inline_markers() {
        let source = concat!(
            "**##ce005c|I am... I //should// be...##**\n",
            "__10 October 2022__\n",
            "Plain URL http://example.com/a/b stays plain.\n",
        );

        let html =
            RenderService::render_wikidot_compatibility_fallback_with_code_blocks(source);

        assert!(html.contains(
            r#"<strong><span style="color: #ce005c;">I am... I <em>should</em> be...</span></strong>"#
        ));
        assert!(html.contains("<u>10 October 2022</u>"));
        assert!(html.contains("http://example.com/a/b"));
        assert!(!html.contains("##ce005c"));
        assert!(!html.contains("//should//"));
        assert!(!html.contains("__10 October"));
    }

    #[test]
    fn wikidot_compatibility_fallback_sanitizes_preserved_inline_tags() {
        let source = concat!(
            "[[collapsible show=\"+ open\" hide=\"- close\"]]\n",
            "**<span class=\"safe\" onclick=\"alert(1)\">safe</span>**\n",
            "**<a href=\"javascript:alert(1)\" title=\"kept\" onmouseover=\"alert(2)\">bad link</a>**\n",
            "**<img src=\"https://example.com/image.png\" alt=\"kept\" onerror=\"alert(3)\">**\n",
            "**<img src=\"data:text/html,<script>alert(4)</script>\">**\n",
            "[[/collapsible]]\n",
        );

        let html =
            RenderService::render_wikidot_compatibility_fallback_with_code_blocks(source);

        assert!(html.contains(r#"<span class="safe">safe</span>"#));
        assert!(html.contains(r#"<a title="kept">bad link</a>"#));
        assert!(html.contains(r#"<img src="https://example.com/image.png" alt="kept">"#));
        assert!(html.contains("<img>"));
        assert!(!html.contains("onclick"));
        assert!(!html.contains("onmouseover"));
        assert!(!html.contains("onerror"));
        assert!(!html.contains("javascript:alert"));
        assert!(!html.contains("data:text/html"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn list_pages_inline_tag_preservation_sanitizes_attributes() {
        let html = super::render_list_pages_table_inline_html(concat!(
            r#"<span class="safe" onclick="alert(1)">ok</span> "#,
            r#"<a href="/safe" onclick="alert(2)">link</a> "#,
            r#"<img src="javascript:alert(3)" onerror="alert(4)">"#,
        ));

        assert!(html.contains(r#"<span class="safe">ok</span>"#));
        assert!(html.contains(r#"<a href="/safe">link</a>"#));
        assert!(html.contains("<img>"));
        assert!(!html.contains("onclick"));
        assert!(!html.contains("onerror"));
        assert!(!html.contains("javascript:"));
    }

    #[test]
    fn wikidot_compatibility_fallback_renders_css_modules_and_style_divs() {
        let source = concat!(
            "[[module CSS]]\n",
            ".scp-pride { display: block; }\n",
            "[[/module]]\n",
            "[[div style=\"font-weight: bold; text-align: center;\"]]\n",
            "[https://example.com keep coming]\n",
            "[[/div]]\n",
        );
        let html = render_wikidot_css_after_registry_restore(source, true);

        assert!(
            html.contains("<style>"),
            "unexpected fallback HTML: {html:?}"
        );
        assert!(html.contains(".scp-pride { display: block; }"));
        assert!(html.contains(r#"<div style="font-weight: bold; text-align: center;">"#));
        assert!(!html.contains("[[module CSS"));
        assert!(!html.contains("[[/module]]"));
        assert!(!html.contains("[[div"));
    }

    #[test]
    fn wikidot_compatibility_fallback_escapes_css_module_style_end_tags() {
        let source = concat!(
            "[[module CSS]]\n",
            "</style><img src=x onerror=alert(1)><style>\n",
            "[[/module]]\n",
            "[[collapsible]]\n",
            "body\n",
            "[[/collapsible]]\n",
        );
        let html = render_wikidot_css_after_registry_restore(source, true);

        assert!(html.contains("<style>"));
        assert!(html.contains(r"\3C /style>\3C img"));
        assert!(!html.contains("</style><img"));
        assert!(!html.contains("<img src=x"));
    }

    #[test]
    fn wikidot_compatibility_fallback_renders_tabs_size_and_page_images() {
        let source = concat!(
            "[[tabview]]\n",
            "[[tab SCPs]]\n",
            "[[size 75%]]\n",
            "[[=image hippo2.jpg size=\"small\"]]\n",
            "caption [[/size]][[size 75%]] tail\n",
            "[[/size]]\n",
            "[[/tab]]\n",
            "[[/tabview]]\n",
        );

        let html =
            RenderService::render_wikidot_compatibility_fallback_with_code_blocks_for_context(
                source,
                Some("the-great-hippo"),
                Some("scp-wiki"),
            );

        assert!(html.contains(
            r#"<div class="yui-navset yui-navset-top wikidot-compat-tabview">"#
        ));
        assert!(html.contains(r#"<ul class="yui-nav">"#));
        assert!(html.contains(
            r#"<li class="selected" title="active"><a href="javascript:;"><em>SCPs</em></a></li>"#
        ));
        assert!(html.contains(r#"<div style="display: block;">"#));
        assert!(html.contains(r#"<span style="font-size: 75%;">"#));
        assert!(html.contains(r#"<div class="image-container aligncenter">"#));
        assert!(html.contains(r#"src="https://scp-wiki.wdfiles.com/local--files/the-great-hippo/hippo2.jpg""#));
        assert!(html.contains(r#"class="image image-size-small""#));
        assert!(!html.contains(r#"<div class="wikidot-compat-tab"><h3>"#));
        assert!(!html.contains("[[tabview"));
        assert!(!html.contains("[[tab"));
        assert!(!html.contains("[[size"));
        assert!(!html.contains("[[=image"));
    }

    #[test]
    fn wikidot_compatibility_fallback_renders_raw_space_markers_as_spacers() {
        let html = RenderService::render_wikidot_compatibility_fallback_with_code_blocks(
            "before\n@@ @@\nafter\ninline @@ @@ stays\n",
        );

        assert!(html.contains(r#"<span style="white-space: pre-wrap;"> </span><br>"#));
        assert!(!html.contains("<p>@@ @@</p>"));
        assert!(html.contains("inline @@ @@ stays"));
    }

    #[test]
    fn wikidot_compatibility_fallback_preserves_tabview_bodies_in_hidden_panels() {
        let source = concat!(
            "[[div class=\"closable-tab\"]]\n",
            "[[tabview]]\n",
            "[[tab X]]\n",
            "[[/tab]]\n",
            "[[tab One]]\n",
            "first body\n",
            "[[div id=\"newest\"]]\n",
            "[[/div]]\n",
            "[[/tab]]\n",
            "[[tab Two]]\n",
            "second body\n",
            "[[/tab]]\n",
            "[[/tabview]]\n",
            "[[/div]]\n",
        );

        let html =
            RenderService::render_wikidot_compatibility_fallback_with_code_blocks(source);

        assert!(html.contains(r#"<div class="closable-tab">"#));
        assert!(html.contains(
            r#"<div class="yui-navset yui-navset-top wikidot-compat-tabview">"#
        ));
        assert!(html.contains(r#"<ul class="yui-nav">"#));
        assert!(html.contains(
            r#"<li class="selected" title="active"><a href="javascript:;"><em>X</em></a></li>"#
        ));
        assert!(html.contains(r#"<li><a href="javascript:;"><em>One</em></a></li>"#));
        assert!(html.contains(r#"<li><a href="javascript:;"><em>Two</em></a></li>"#));
        assert!(html.contains(r#"<div style="display: block;"></div>"#));
        assert!(html.contains(r#"<div style="display:none"><p>first body</p><div id="u-newest"></div></div>"#));
        assert!(html.contains(r#"<div style="display:none"><p>second body</p></div>"#));
        assert!(!html.contains(r#"<div class="wikidot-compat-tab"><h3>"#));
        assert!(!html.contains("[[tabview"));
        assert!(!html.contains("[[tab "));
    }

    #[test]
    fn many_collapsible_corpus_page_uses_compatibility_fallback_before_ftml() {
        let mut source = String::new();
        for index in 0..=MAX_FTML_COMPAT_COLLAPSIBLE_BLOCKS {
            source.push_str(&format!(
                "[[collapsible show=\"+ Skip {index}\" hide=\"- Skip {index}\"]]\n"
            ));
            source.push_str("ordinary corpus prose line\n");
            source.push_str("[[/collapsible]]\n");
        }

        assert!(source.len() < MAX_FTML_COMPAT_PARSE_BYTES);
        assert!(RenderService::should_use_wikidot_compatibility_fallback(
            &source,
            &fallback_test_page_info("the-great-hippo", "Great Hippo's Great Skippos")
        ));
    }

    #[test]
    fn tabbed_corpus_page_uses_compatibility_fallback_before_ftml() {
        let mut source = String::new();
        source.push_str("[[tabview]]\n");
        for index in 0..MIN_FTML_COMPAT_TABBED_FALLBACK_MARKERS {
            source.push_str(&format!("[[tab Section {index}]]\n"));
            source.push_str("ordinary author page prose\n".repeat(220).as_str());
            source.push_str("[[/tab]]\n");
        }
        source.push_str("[[/tabview]]\n");
        while source.len() < MIN_FTML_COMPAT_TABBED_FALLBACK_BYTES {
            source.push_str("ordinary author page prose\n");
        }

        assert!(source.len() < MAX_FTML_COMPAT_PARSE_BYTES);
        assert!(RenderService::should_use_wikidot_compatibility_fallback(
            &source,
            &fallback_test_page_info(
                "a-plague-of-philosophical-zombies",
                "A Plague of Philosophical Zombies",
            )
        ));
    }

    #[test]
    fn large_component_page_under_byte_cap_stays_parse_eligible() {
        let source = "ordinary component prose line\n".repeat(20_000);

        assert!(source.len() < MAX_FTML_COMPAT_PARSE_BYTES);
        assert!(!RenderService::should_use_wikidot_compatibility_fallback(
            &source,
            &fallback_test_page_info(
                "vg021-jp-author-congy-2e28d21069",
                "[jp] author:congy"
            )
        ));
    }

    #[test]
    fn expanded_dense_style_resource_still_uses_compatibility_fallback() {
        let mut source = String::new();
        for index in 0..180 {
            source.push_str(&format!("[[include component:expanded-{index} p=value]]\n"));
        }
        for index in 0..760 {
            source.push_str(&format!(
                ".expanded-style-{index} {{ --accent-{index}: #b01; color: #111; }}\n"
            ));
        }
        for index in 0..520 {
            source.push_str(&format!(
                "* [[[expanded-style-resource-{index}|Style Resource {index}]]]\n"
            ));
        }
        while source.len() <= 200_000 {
            source.push_str("expanded corpus prose line with no extra parser stress\n");
        }

        assert!(source.len() < MAX_FTML_COMPAT_PARSE_BYTES);
        assert!(RenderService::should_use_wikidot_compatibility_fallback(
            &source,
            &fallback_test_page_info(
                "vg021-jp-scp-style-resource-769330c42a",
                "[jp] SCPスタイルリソース"
            )
        ));
    }

    #[test]
    fn expanded_dense_non_style_resource_stays_parse_eligible() {
        let mut source = String::new();
        for index in 0..180 {
            source.push_str(&format!("[[include component:expanded-{index} p=value]]\n"));
        }
        for index in 0..760 {
            source.push_str(&format!(
                ".expanded-style-{index} {{ --accent-{index}: #b01; color: #111; }}\n"
            ));
        }
        for index in 0..520 {
            source.push_str(&format!(
                "* [[[expanded-style-resource-{index}|Style Resource {index}]]]\n"
            ));
        }
        while source.len() <= 200_000 {
            source.push_str("expanded corpus prose line with no extra parser stress\n");
        }

        assert!(source.len() < MAX_FTML_COMPAT_PARSE_BYTES);
        assert!(!RenderService::should_use_wikidot_compatibility_fallback(
            &source,
            &fallback_test_page_info(
                "vg021-jp-author-congy-2e28d21069",
                "[jp] author:congy"
            )
        ));
    }

    #[test]
    fn dense_parse_eligible_wikidot_pages_get_extended_render_deadline() {
        let mut config = Config::integration_testing();
        config.preprocess_timeout = Duration::from_millis(500);
        config.render_timeout = Duration::from_millis(2_000);
        let mut source = String::new();
        for index in 0..180 {
            source.push_str(&format!("[[include component:expanded-{index} p=value]]\n"));
        }
        for index in 0..760 {
            source.push_str(&format!(
                ".expanded-style-{index} {{ --accent-{index}: #b01; color: #111; }}\n"
            ));
        }
        for index in 0..520 {
            source.push_str(&format!(
                "* [[[expanded-style-resource-{index}|Style Resource {index}]]]\n"
            ));
        }

        assert!(source.len() < MAX_FTML_COMPAT_PARSE_BYTES);
        assert_eq!(
            RenderService::ftml_compat_render_timeout(&config, &source),
            Duration::from_secs(MIN_DENSE_FTML_COMPAT_RENDER_TIMEOUT_SECS)
        );
    }

    #[test]
    fn large_tabbed_wikidot_pages_get_extended_render_deadline() {
        let mut config = Config::integration_testing();
        config.preprocess_timeout = Duration::from_millis(500);
        config.render_timeout = Duration::from_millis(2_000);
        let mut source = String::new();
        source.push_str("[[tabview]]\n");
        for index in 0..24 {
            source.push_str(&format!("[[tab Section {index}]]\n"));
            source.push_str("ordinary author page prose\n".repeat(220).as_str());
            source.push_str("[[/tab]]\n");
        }
        source.push_str("[[/tabview]]\n");
        while source.len() < 140_000 {
            source.push_str("ordinary author page prose\n");
        }

        assert!(source.len() < MAX_FTML_COMPAT_PARSE_BYTES);
        assert!(
            RenderService::wikidot_compat_parse_complexity_score(&source)
                < MAX_FTML_COMPAT_DENSE_PARSE_SCORE
        );
        assert_eq!(
            RenderService::ftml_compat_render_timeout(&config, &source),
            Duration::from_secs(MIN_DENSE_FTML_COMPAT_RENDER_TIMEOUT_SECS)
        );
    }

    #[test]
    fn ordinary_wikidot_pages_keep_configured_render_deadline() {
        let mut config = Config::integration_testing();
        config.preprocess_timeout = Duration::from_millis(500);
        config.render_timeout = Duration::from_millis(2_000);
        let source = "ordinary component prose line\n".repeat(20_000);

        assert_eq!(
            RenderService::ftml_compat_render_timeout(&config, &source),
            Duration::from_millis(2_500)
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
    fn protects_wikidot_jp_interwiki_embed_iframe_before_ftml() {
        let mut wikitext = concat!(
            "[[embed]]\n",
            r#"<iframe src="//interwiki.scp-jp.org/interwikiFrame.html?lang=jp&community=scp&pagename=scp-3000-jp" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#,
            "\n[[/embed]]",
        )
        .to_owned();

        let iframes = RenderService::protect_wikidot_embed_iframes(&mut wikitext);
        assert_eq!(wikitext, "WIKIJUMPWIKIDOTEMBEDIFRAME0X");
        assert_eq!(
            iframes,
            vec![
                r#"<iframe src="/-/wikidot-interwiki/interwikiFrame.html?lang=jp&community=scp&pagename=scp-3000-jp" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#
                    .to_owned()
            ],
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
    fn renders_wikidot_styleframe_embed_in_compat_fallback() {
        let rendered =
            RenderService::render_wikidot_compatibility_fallback_with_code_blocks(
                concat!(
                    "[[embed]]\n",
                    r#"<iframe src="//interwiki.scpwiki.com/styleFrame.html?priority=1&theme=https://scp-wiki.wdfiles.com/local--code/theme%3Asunside/1&css={$css}" style="display: none"></iframe>"#,
                    "\n[[/embed]]",
                ),
            );

        assert_eq!(
            rendered,
            concat!(
                r#"<div class="wikidot-compat-fallback"><div>"#,
                r#"<iframe src="/-/wikidot-interwiki/styleFrame.html?priority=1&theme=https://scp-wiki.wdfiles.com/local--code/theme%3Asunside/1&css={$css}" style="display: none"></iframe>"#,
                r#"</div></div>"#,
            ),
        );
    }

    #[test]
    fn renders_wikidot_interwiki_embed_in_compat_fallback() {
        let rendered =
            RenderService::render_wikidot_compatibility_fallback_with_code_blocks(
                concat!(
                    "[[embed]]\n",
                    r#"<iframe src="//interwiki.scpwiki.com/interwikiFrame.html?lang=en&community=scp&pagename=scp-anthology-2024" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#,
                    "\n[[/embed]]",
                ),
            );

        assert_eq!(
            rendered,
            concat!(
                r#"<div class="wikidot-compat-fallback"><div>"#,
                r#"<iframe src="/-/wikidot-interwiki/interwikiFrame.html?lang=en&community=scp&pagename=scp-anthology-2024" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>"#,
                r#"</div></div>"#,
            ),
        );
    }

    #[test]
    fn leaves_unsupported_embed_literal_in_compat_fallback() {
        let rendered =
            RenderService::render_wikidot_compatibility_fallback_with_code_blocks(
                concat!(
                    "[[embed]]\n",
                    r#"<iframe src="//example.com/widget" style="display: none"></iframe>"#,
                    "\n[[/embed]]",
                ),
            );

        assert!(rendered.contains("[[embed]]"));
        assert!(rendered.contains(
            r#"&lt;iframe src="//example.com/widget" style="display: none"&gt;&lt;/iframe&gt;"#
        ));
        assert!(!rendered.contains(r#"<iframe src="//example.com/widget""#));
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
    fn removes_included_component_documentation_before_nested_includes() {
        let mut wikitext = concat!(
            "[[iftags +component]]\n",
            "+ How To Use\n",
            "@@[[include :scp-wiki:component:license-box]]@@\n",
            "[[/iftags]]\n",
            "[[include :scp-wiki:component:license-box-backend\n",
            "|author={$author}\n",
            "|author=%%created_by%%]]\n",
        )
        .to_owned();

        RenderService::remove_wikidot_component_iftags_documentation(&mut wikitext);

        assert!(!wikitext.contains("How To Use"));
        assert!(!wikitext.contains("@@[[include :scp-wiki:component:license-box]]@@"));
        assert!(wikitext.contains("component:license-box-backend"));
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
    fn unwraps_active_collapsed_empty_negative_iftags_block() {
        let mut wikitext = concat!(
            "before\n",
            ">[[iftags -]]\n",
            ">[[iftags]]\n",
            ">================= end ========================\n",
            "[[include :scp-jp:user-component:ta-badge-smooth-base-base name=v-1|v-1={$v-1}|type=false]]\n",
            "[[/div]]\n",
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
                ">================= end ========================\n",
                "[[include :scp-jp:user-component:ta-badge-smooth-base-base name=v-1|v-1={$v-1}|type=false]]\n",
                "[[/div]]\n",
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
    fn expands_wikidot_image_block_includes_with_defaults_and_arguments() {
        let mut wikitext = concat!(
            "[[include component:image-block\n",
            "    name=theend.jpg|\n",
            "    caption=The end title card.\n",
            "]]\n",
            "[[include component:image-block name=steel.png|align=center|width=100%|caption=Steel frame.|alt=alt|alt-text=A steel frame.]]\n",
            "[[include component:image-block-base name=raw.jpg]]\n",
        )
        .to_owned();

        let page_info = fallback_test_page_info("scp-3922", "SCP-3922");
        let included_pages =
            RenderService::expand_wikidot_image_block_includes(&mut wikitext, &page_info);

        assert!(wikitext.contains(
            r#"[[div class="scp-image-block block-right" style="width:300px;"]]"#
        ));
        assert!(wikitext.contains(
            "[[image http://scp-wiki.wikidot.com/local--files/scp-3922/theend.jpg]]"
        ));
        assert!(wikitext.contains("The end title card."));
        assert!(wikitext.contains(
            r#"[[div class="scp-image-block block-center" style="width:100%;"]]"#
        ));
        assert!(wikitext.contains(
            r#"[[image http://scp-wiki.wikidot.com/local--files/scp-3922/steel.png alt="A steel frame."]]"#
        ));
        assert!(wikitext.contains("[[include component:image-block-base name=raw.jpg]]"));
        assert!(!wikitext.contains("[[include component:image-block\n"));
        assert_eq!(
            included_pages,
            vec![
                PageRef::page_only("component:image-block"),
                PageRef::page_only("component:image-block-base"),
                PageRef::page_only("component:image-block"),
                PageRef::page_only("component:image-block-base"),
            ],
        );

        let mut category_page_info = fallback_test_page_info("basalt", "Basalt Theme");
        category_page_info.category = Some(Cow::Borrowed("theme"));
        assert_eq!(
            RenderService::wikidot_image_block_source("logo.svg", &category_page_info),
            "http://scp-wiki.wikidot.com/local--files/theme:basalt/logo.svg"
        );
    }

    #[test]
    fn expands_wikidot_image_block_includes_with_nested_caption_markup() {
        let mut wikitext = concat!(
            "[[include :scp-wiki:component:image-block ",
            "name=linked.jpg|caption=See [[[SCP-173|the statue]]] for details.]]\n",
        )
        .to_owned();
        let page_info = fallback_test_page_info("scp-3922", "SCP-3922");

        let included_pages =
            RenderService::expand_wikidot_image_block_includes(&mut wikitext, &page_info);

        assert!(wikitext.contains("See [[[SCP-173|the statue]]] for details."));
        assert!(wikitext.contains(
            "[[image http://scp-wiki.wikidot.com/local--files/scp-3922/linked.jpg]]"
        ));
        assert_eq!(
            included_pages,
            vec![
                PageRef::page_and_site("scp-wiki", "component:image-block"),
                PageRef::page_and_site("scp-wiki", "component:image-block-base"),
            ],
        );
    }

    #[test]
    fn expands_included_fragment_wikidot_image_blocks_before_generic_includes() {
        let page_info = fallback_test_page_info("scp-8382", "SCP-8382");
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut fragment_wikitext = concat!(
            "Before the block.\n",
            "[[include component:image-block\n",
            "  name=Alis.jpg|\n",
            "  caption=Photograph of PoI-6721-1 at Secure Area-219, 2023. |\n",
            "  width=225px|\n",
            "  align=left|\n",
            "]]\n",
            "After the block.\n",
        )
        .to_owned();

        let image_block_includes = RenderService::expand_wikidot_image_block_includes(
            &mut fragment_wikitext,
            &page_info,
        );
        let top_wikitext = "[[include fragment:scp-8382-2]]\n";
        let (mut expanded, direct_included_pages) = ftml::include(
            top_wikitext,
            &settings,
            PreparedIncluder {
                pages: vec![Some(fragment_wikitext)],
            },
            include_error,
        )
        .expect("prepared include should expand");

        ftml::preprocess(&mut expanded);
        let tokens = ftml::tokenize(&expanded);
        let result = ftml::parse(&tokens, &page_info, &settings);
        let (tree, _) = result.into();
        let rendered = HtmlRender.render(&tree, &page_info, &settings).body;

        assert_eq!(
            direct_included_pages,
            vec![PageRef::page_only("fragment:scp-8382-2")]
        );
        assert_eq!(
            image_block_includes,
            vec![
                PageRef::page_only("component:image-block"),
                PageRef::page_only("component:image-block-base"),
            ]
        );
        assert!(rendered.contains(
            r#"<img src="http://scp-wiki.wikidot.com/local--files/scp-8382/Alis.jpg""#
        ));
        assert!(rendered.contains("Photograph of PoI-6721-1 at Secure Area-219, 2023."));
        assert!(!rendered.contains("[[image"));
        assert!(!rendered.contains("{$alt}"));
        assert!(!rendered.contains("link=#"));
    }

    #[test]
    fn leaves_image_block_includes_on_non_scp_wiki_sites_for_normal_expansion() {
        let mut wikitext = concat!(
            "[[include component:image-block name=custom.jpg|caption=Custom block.]]\n",
            "[[include :sandbox-for-codex:component:image-block name=custom.jpg]]\n",
        )
        .to_owned();
        let page_info = ftml::data::PageInfo {
            site: Cow::Borrowed("sandbox-for-codex"),
            page: Cow::Borrowed("start"),
            title: Cow::Borrowed("Sandbox"),
            alt_title: None,
            tags: Vec::new(),
            category: None,
            score: ftml::data::ScoreValue::Integer(0),
            language: Cow::Borrowed("en"),
        };

        let included_pages =
            RenderService::expand_wikidot_image_block_includes(&mut wikitext, &page_info);

        assert!(included_pages.is_empty());
        assert!(wikitext.contains("[[include component:image-block name=custom.jpg"));
        assert!(wikitext.contains("[[include :sandbox-for-codex:component:image-block"));
    }

    #[test]
    fn leaves_literal_wikidot_image_block_includes_unexpanded() {
        let mut wikitext = concat!(
            "[[code]]\n",
            "[[include component:image-block name=code.jpg]]\n",
            "[[/code]]\n",
            "@@[[include component:image-block name=escaped.jpg]]@@\n",
            "[!-- [[include component:image-block name=comment.jpg]] --]\n",
            "[[include component:image-block name=live.jpg]]\n",
        )
        .to_owned();
        let page_info = fallback_test_page_info("scp-3922", "SCP-3922");

        let included_pages =
            RenderService::expand_wikidot_image_block_includes(&mut wikitext, &page_info);

        assert!(wikitext.contains("[[include component:image-block name=code.jpg]]"));
        assert!(wikitext.contains("[[include component:image-block name=escaped.jpg]]"));
        assert!(wikitext.contains("[[include component:image-block name=comment.jpg]]"));
        assert!(wikitext.contains(
            "[[image http://scp-wiki.wikidot.com/local--files/scp-3922/live.jpg]]"
        ));
        assert_eq!(
            included_pages,
            vec![
                PageRef::page_only("component:image-block"),
                PageRef::page_only("component:image-block-base"),
            ],
        );
    }

    #[test]
    fn skips_generic_includes_inside_wikidot_comments() {
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut wikitext = concat!(
            "[!--\n",
            "Usage:\n",
            "[[include :scp-wiki:component:interwiki-style\n",
            "| priority=1\n",
            "]]\n",
            "--]\n",
            "[[include component:live]]\n",
        )
        .to_owned();

        RenderService::mask_wikidot_comment_include_markers(&mut wikitext);
        let mut includes = Vec::new();
        ftml::include(
            &wikitext,
            &settings,
            CollectingIncluder {
                includes: &mut includes,
            },
            include_error,
        )
        .expect("include collection should skip comment-hidden usage examples");
        RenderService::unmask_wikidot_comment_include_markers(&mut wikitext);

        assert_eq!(includes.len(), 1);
        assert_eq!(
            includes[0].page_ref(),
            &PageRef::page_only("component:live")
        );
        assert!(wikitext.contains("[[include :scp-wiki:component:interwiki-style"));
    }

    #[test]
    fn strips_included_comment_usage_examples_after_expansion() {
        let page_info =
            fallback_test_page_info("scp-anthology-2024", "SCP Anthology 2024");
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut component_source = concat!(
            "[!--\n",
            "Usage:\n",
            "[[include :scp-wiki:component:interwiki-style\n",
            "| priority=1\n",
            "]]\n",
            "--]\n",
            "[[embed]]\n",
            "<iframe src=\"/-/wikidot-interwiki/styleFrame.html?priority=1\"></iframe>\n",
            "[[/embed]]\n",
        )
        .to_owned();

        RenderService::mask_wikidot_comment_include_markers(&mut component_source);
        let mut nested_includes = Vec::new();
        ftml::include(
            &component_source,
            &settings,
            CollectingIncluder {
                includes: &mut nested_includes,
            },
            include_error,
        )
        .expect("comment usage examples should not request nested pages");
        RenderService::unmask_wikidot_comment_include_markers(&mut component_source);

        let (mut expanded, included_pages) = ftml::include(
            "[[include :scp-wiki:component:interwiki-style]]\n",
            &settings,
            PreparedIncluder {
                pages: vec![Some(component_source)],
            },
            include_error,
        )
        .expect("prepared include should expand the component source");
        ftml::preprocess(&mut expanded);
        let tokens = ftml::tokenize(&expanded);
        let result = ftml::parse(&tokens, &page_info, &settings);
        let (tree, _) = result.into();
        let rendered = HtmlRender.render(&tree, &page_info, &settings).body;

        assert!(nested_includes.is_empty());
        assert_eq!(
            included_pages,
            vec![PageRef::page_and_site(
                "scp-wiki",
                "component:interwiki-style"
            )]
        );
        assert!(rendered.contains("styleFrame.html?priority=1"));
        assert!(!rendered.contains("Usage:"));
        assert!(!rendered.contains("[[include :scp-wiki:component:interwiki-style"));
    }

    #[test]
    fn keeps_selected_comment_branch_includes_collectable() {
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut wikitext = concat!(
            "[!-- --]\n",
            "[[include component:selected]]\n",
            "[!----]\n",
            "[!-- {$inc-hidden}\n",
            "[[include component:hidden]]\n",
            "[!----]\n",
        )
        .to_owned();

        RenderService::mask_wikidot_comment_include_markers(&mut wikitext);
        let mut includes = Vec::new();
        ftml::include(
            &wikitext,
            &settings,
            CollectingIncluder {
                includes: &mut includes,
            },
            include_error,
        )
        .expect("include collection should keep selected branch includes");
        RenderService::unmask_wikidot_comment_include_markers(&mut wikitext);

        assert_eq!(includes.len(), 1);
        assert_eq!(
            includes[0].page_ref(),
            &PageRef::page_only("component:selected")
        );
        assert!(wikitext.contains("[[include component:hidden]]"));
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

        let mut multiline_includes = Vec::new();
        let mut multiline_include = concat!(
            "[[include :scp-jp:user-component:ta-badge\n",
            "|badge-top=gold-medal\n",
            "|badge-right=photographer\n",
            "|badge-left=high-calibre\n",
            "|frame=\n",
            "|bg-img=background-color: #fff\n",
            "|bg-shadow=false\n",
            "|plate=style2\n",
            "]]\n",
        )
        .to_owned();
        RenderService::normalize_wikidot_ta_badge_multiline_includes(
            &mut multiline_include,
        );

        ftml::include(
            &multiline_include,
            &settings,
            CollectingIncluder {
                includes: &mut multiline_includes,
            },
            include_error,
        )
        .expect("include collection should parse Wikidot multiline variables");

        assert_eq!(multiline_includes.len(), 1);
        assert_eq!(
            multiline_includes[0].page_ref(),
            &PageRef::page_and_site("scp-jp", "user-component:ta-badge"),
        );
        assert_eq!(
            multiline_includes[0]
                .variables()
                .get("bg-img")
                .map(Cow::as_ref),
            Some("background-color: #fff "),
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
    fn resolves_nested_wikidot_include_variables() {
        let include = IncludeRef::new(
            PageRef::page_and_site("scp-jp", "user-component:ta-badge-smooth-base-base"),
            VariableMap::from([
                (Cow::Borrowed("name"), Cow::Borrowed("action")),
                (Cow::Borrowed("action"), Cow::Borrowed("true")),
                (Cow::Borrowed("type"), Cow::Borrowed("false")),
            ]),
        );
        let mut source =
            r#"[[div_ class="badges badge-{$name} {$name} a{${$name}} b{$type}"]]"#
                .to_owned();

        super::apply_include_variables(&mut source, &include);

        assert!(source.contains(r#"class="badges badge-action action atrue bfalse""#));
        assert!(!source.contains("{$action}"));
    }

    #[test]
    fn normalizes_wikidot_div_style_url_quotes_for_acs_icon_markers() {
        let mut wikitext = concat!(
            "[[div_ class=\"icon-1\" style=\"background-image: url(\"",
            "https://scp-wiki.wdfiles.com/local--files/scp-7243/7243-godel-icon.svg",
            "\");\"]]\n",
            "[[/div]]\n",
        )
        .to_owned();

        RenderService::normalize_wikidot_div_style_url_quotes(&mut wikitext);

        assert!(wikitext.contains(
            "style=\"background-image: url('https://scp-wiki.wdfiles.com/local--files/scp-7243/7243-godel-icon.svg');\""
        ));

        let page_info = fallback_test_page_info("scp-7243", "SCP-7243");
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        ftml::preprocess(&mut wikitext);
        let tokens = ftml::tokenize(&wikitext);
        let result = ftml::parse(&tokens, &page_info, &settings);
        let (tree, _) = result.into();
        let rendered = HtmlRender.render(&tree, &page_info, &settings).body;

        assert!(rendered.contains(r#"<div class="icon-1""#));
        assert!(rendered.contains(
            "style=\"background-image: url(&#39;https://scp-wiki.wdfiles.com/local--files/scp-7243/7243-godel-icon.svg&#39;);\""
        ));
        assert!(!rendered.contains("[[div_"));
    }

    #[test]
    fn protects_wikidot_div_class_include_variables_before_ftml() {
        let mut wikitext = concat!(
            r#"[[div_ class="anom-bar-container item-SCP-001 {$american}"]]"#,
            "\n",
            r#"[[span class="item"]]Item#:[[/span]]"#,
            "\n[[/div]]\n",
        )
        .to_owned();

        RenderService::protect_wikidot_marker_class_include_variables(&mut wikitext);

        assert!(wikitext.contains("wikijump-include-var-american"));
        let page_info =
            fallback_test_page_info("001-blank-i", "Proposal Blank the First");
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        ftml::preprocess(&mut wikitext);
        let tokens = ftml::tokenize(&wikitext);
        let result = ftml::parse(&tokens, &page_info, &settings);
        let (tree, _) = result.into();
        let rendered = HtmlRender.render(&tree, &page_info, &settings).body;
        let restored =
            RenderService::restore_protected_wikidot_marker_class_include_variables(
                &rendered,
            );

        assert!(
            restored
                .contains(r#"<div class="anom-bar-container item-SCP-001 {$american}">"#)
        );
        assert!(restored.contains(r#"<span class="item">Item#:</span>"#));
        assert!(!restored.contains("[[span"));
        assert!(!restored.contains("wikijump-include-var-"));
    }

    #[test]
    fn normalizes_wikidot_multiline_page_links_before_ftml() {
        let mut wikitext = concat!(
            "[[[an-incredibly-importanterest-announcement|",
            "Creck Fection Contest 2 (TWO DAY EXTRAVAGANZA!)\n",
            "]]]\n",
            "[!-- [[[literal|Nope\n]]] --]\n",
        )
        .to_owned();

        RenderService::normalize_wikidot_multiline_page_links(&mut wikitext);

        assert!(wikitext.contains(
            "[[[an-incredibly-importanterest-announcement|Creck Fection Contest 2 (TWO DAY EXTRAVAGANZA!)]]]"
        ));
        assert!(wikitext.contains("[!-- [[[literal|Nope\n]]] --]"));

        let page_info = fallback_test_page_info("049-x-minion-x-reader", "Reader");
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        ftml::preprocess(&mut wikitext);
        let tokens = ftml::tokenize(&wikitext);
        let result = ftml::parse(&tokens, &page_info, &settings);
        let (tree, _) = result.into();
        let rendered = HtmlRender.render(&tree, &page_info, &settings).body;

        assert!(
            rendered.contains(r#"href="/an-incredibly-importanterest-announcement""#)
        );
        assert!(rendered.contains("Creck Fection Contest 2 (TWO DAY EXTRAVAGANZA!)"));
        assert!(!rendered.contains("[[[an-incredibly-importanterest-announcement"));
    }

    #[test]
    fn protects_wikidot_current_page_links_inside_inline_code() {
        let page_info = fallback_test_page_info("scp-7243", "SCP-7243");
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut wikitext = concat!(
            "{{[# phmd@scip.net]:// ./msg ",
            "[*/scp-6276 ext_server-6276] !undata recipient 0000}}",
        )
        .to_owned();

        let links = RenderService::protect_wikidot_compat_links(&mut wikitext, &settings);

        assert_eq!(links.len(), 2);
        assert!(wikitext.contains(WIKIDOT_COMPAT_LINK_SENTINEL_PREFIX));
        assert!(!wikitext.contains("[# phmd@scip.net]"));
        assert!(!wikitext.contains("[*/scp-6276 ext_server-6276]"));

        ftml::preprocess(&mut wikitext);
        let tokens = ftml::tokenize(&wikitext);
        let result = ftml::parse(&tokens, &page_info, &settings);
        let (tree, _) = result.into();
        let mut rendered = HtmlRender.render(&tree, &page_info, &settings).body;
        rendered =
            RenderService::restore_protected_wikidot_compat_links(rendered, &links);
        rendered = RenderService::restore_wikidot_email_obfuscation(&rendered);
        rendered = RenderService::remove_spurious_wikidot_email_classes(&rendered);

        assert!(rendered.contains(r#"<a href="javascript:;">phmd@scip.net</a>:// ./msg <a href="/scp-6276" target="_blank">ext_server-6276</a> !undata recipient 0000"#));
        assert!(!rendered.contains("//:]ten.pics|dmhp"));
        assert!(!rendered.contains("[# phmd@scip.net]"));
        assert!(!rendered.contains("[*/scp-6276 ext_server-6276]"));
    }

    #[test]
    fn protects_wikidot_named_anchor_markers_without_visible_brackets() {
        let page_info = fallback_test_page_info("scp-7243", "SCP-7243");
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut wikitext = "[[# tabanchor]]\nVisible text".to_owned();

        let links = RenderService::protect_wikidot_compat_links(&mut wikitext, &settings);

        assert_eq!(links.len(), 1);
        assert!(wikitext.contains(WIKIDOT_COMPAT_LINK_SENTINEL_PREFIX));
        assert!(!wikitext.contains("[# tabanchor]"));

        ftml::preprocess(&mut wikitext);
        let tokens = ftml::tokenize(&wikitext);
        let result = ftml::parse(&tokens, &page_info, &settings);
        let (tree, _) = result.into();
        let rendered = RenderService::restore_protected_wikidot_compat_links(
            HtmlRender.render(&tree, &page_info, &settings).body,
            &links,
        );

        assert!(rendered.contains(r#"<a name="tabanchor"></a>"#));
        assert!(rendered.contains("Visible text"));
        assert!(!rendered.contains("[tabanchor]"));
        assert!(!rendered.contains("[# tabanchor]"));
    }

    #[test]
    fn wikidot_named_anchor_markers_do_not_restore_predictable_literal_sentinels() {
        let page_info = fallback_test_page_info("scp-7243", "SCP-7243");
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut wikitext = concat!(
            "[[# x onmouseover=alert(1) y]]\n",
            "[[span class=\"WIKIJUMPWIKIDOTCOMPATLINK0X\"]]hover[[/span]]",
        )
        .to_owned();

        let links = RenderService::protect_wikidot_compat_links(&mut wikitext, &settings);

        assert_eq!(links.len(), 1);
        assert_ne!(links[0].marker, "WIKIJUMPWIKIDOTCOMPATLINK0X");
        assert!(wikitext.contains(&links[0].marker));
        assert!(wikitext.contains("WIKIJUMPWIKIDOTCOMPATLINK0X"));

        ftml::preprocess(&mut wikitext);
        let tokens = ftml::tokenize(&wikitext);
        let result = ftml::parse(&tokens, &page_info, &settings);
        let (tree, _) = result.into();
        let rendered = RenderService::restore_protected_wikidot_compat_links(
            HtmlRender.render(&tree, &page_info, &settings).body,
            &links,
        );

        assert!(rendered.contains(r#"<a name="x onmouseover=alert(1) y"></a>"#));
        assert!(
            rendered
                .contains(r#"<span class="WIKIJUMPWIKIDOTCOMPATLINK0X">hover</span>"#)
        );
        assert!(!rendered.contains(r#"<span class="<a name="#));
    }

    #[test]
    fn wikidot_named_anchor_markers_do_not_restore_markers_inside_attributes() {
        let page_info = fallback_test_page_info("scp-7243", "SCP-7243");
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut wikitext = concat!(
            "[[span class=\"[[# x onmouseover=alert(1) y]]\"]]",
            "hover",
            "[[/span]]",
        )
        .to_owned();

        let links = RenderService::protect_wikidot_compat_links(&mut wikitext, &settings);

        assert_eq!(links.len(), 1);
        assert!(wikitext.contains(&links[0].marker));

        ftml::preprocess(&mut wikitext);
        let tokens = ftml::tokenize(&wikitext);
        let result = ftml::parse(&tokens, &page_info, &settings);
        let (tree, _) = result.into();
        let rendered = RenderService::restore_protected_wikidot_compat_links(
            HtmlRender.render(&tree, &page_info, &settings).body,
            &links,
        );

        assert!(rendered.contains(&format!(
            r#"<span class="{}">hover</span>"#,
            links[0].marker,
        )));
        assert!(!rendered.contains("onmouseover"));
        assert!(!rendered.contains(r#"<span class="<a name="#));
    }

    #[test]
    fn protects_wikidot_wikipedia_links_before_ftml_parsing() {
        let page_info = fallback_test_page_info("scp-7243", "SCP-7243");
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut wikitext = concat!(
            "\"Our Foundation\" of ",
            "[wikipedia:Canonical_bundle Canonical Bundle]",
            " DW17 Timeline Delta-Blue",
        )
        .to_owned();

        let links =
            RenderService::protect_wikidot_wikipedia_links(&mut wikitext, &settings);

        assert_eq!(links.len(), 1);
        assert!(wikitext.contains(WIKIDOT_WIKIPEDIA_LINK_SENTINEL_PREFIX));
        assert!(!wikitext.contains("[wikipedia:Canonical_bundle"));

        ftml::preprocess(&mut wikitext);
        let tokens = ftml::tokenize(&wikitext);
        let result = ftml::parse(&tokens, &page_info, &settings);
        let (tree, _) = result.into();
        let mut html_output = HtmlRender.render(&tree, &page_info, &settings);
        html_output.body = RenderService::restore_protected_wikidot_wikipedia_links(
            html_output.body,
            &links,
        );
        RenderService::record_protected_wikidot_wikipedia_backlinks(
            &mut html_output.backlinks,
            &links,
        );

        assert!(html_output.body.contains(r#"<a href="http://en.wikipedia.org/wiki/Canonical_bundle" onclick="window.open(this.href, '_blank'); return false;">Canonical Bundle</a>"#));
        assert!(!html_output.body.contains("[wikipedia:Canonical_bundle"));
        assert_eq!(
            html_output.backlinks.external_links,
            vec![Cow::Borrowed(
                "http://en.wikipedia.org/wiki/Canonical_bundle"
            )],
        );
    }

    #[test]
    fn renders_wikidot_wikipedia_links_with_language_and_default_label() {
        assert_eq!(
            super::render_wikidot_wikipedia_link("it:Albert_Einstein", Some("Albert")),
            r#"<a href="http://it.wikipedia.org/wiki/Albert_Einstein" onclick="window.open(this.href, '_blank'); return false;">Albert</a>"#,
        );
        assert_eq!(
            super::render_wikidot_wikipedia_link("Canonical_bundle", None),
            r#"<a href="http://en.wikipedia.org/wiki/Canonical_bundle" onclick="window.open(this.href, '_blank'); return false;">Canonical bundle</a>"#,
        );
    }

    #[test]
    fn leaves_wikidot_wikipedia_links_inside_literal_regions_unchanged() {
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut escaped = "@@[wikipedia:Canonical_bundle Canonical Bundle]@@".to_owned();
        let links =
            RenderService::protect_wikidot_wikipedia_links(&mut escaped, &settings);

        assert!(links.is_empty());
        assert_eq!(escaped, "@@[wikipedia:Canonical_bundle Canonical Bundle]@@",);

        let mut code = concat!(
            "[[code]]\n",
            "[wikipedia:Canonical_bundle Canonical Bundle]\n",
            "[[/code]]\n",
        )
        .to_owned();
        let links = RenderService::protect_wikidot_wikipedia_links(&mut code, &settings);

        assert!(links.is_empty());
        assert!(code.contains("[wikipedia:Canonical_bundle Canonical Bundle]"));
    }

    #[test]
    fn renders_inline_wikidot_spans_inside_preprocessed_native_list_runs() {
        let wikitext = concat!(
            "* Item 1\n",
            "* Item 2\n",
            "* Item 3\n",
            "* Item 4\n",
            "* Item 5\n",
            "* Item 6\n",
            "* Safe [[span class=\"safe\" onclick=\"alert(1)\"]]span[[/span]]\n",
            "* The Logistics Branch must maintain supply lines for transport of non-existent",
            "[[span class=\"fnnum\"]].[[/span]]",
            "[[span class=\"fncon\"]]For clarity: payloads will be absent.[[/span]]",
            " effluence to Site-43;\n",
        )
        .to_owned();

        let rendered = RenderService::render_long_native_list_runs(wikitext);

        assert!(rendered.contains(r#"<li>The Logistics Branch"#));
        assert!(rendered.contains(r#"<span class="safe">span</span>"#));
        assert!(rendered.contains(r#"<span class="fnnum">.</span>"#));
        assert!(rendered.contains(
            r#"<span class="fncon">For clarity: payloads will be absent.</span>"#
        ));
        assert!(!rendered.contains("onclick"));
        assert!(!rendered.contains("[[span"));
    }

    #[test]
    fn renders_wikidot_wikipedia_links_inside_preprocessed_native_list_runs() {
        let wikitext = concat!(
            "* Item 1\n",
            "* Item 2\n",
            "* Item 3\n",
            "* Item 4\n",
            "* Item 5\n",
            "* Item 6\n",
            "* Item 7\n",
            "* Source [wikipedia:Canonical_bundle Canonical Bundle]\n",
        )
        .to_owned();

        let rendered = RenderService::render_long_native_list_runs(wikitext);

        assert!(rendered.contains(r#"<li>Source <a href="http://en.wikipedia.org/wiki/Canonical_bundle" onclick="window.open(this.href, '_blank'); return false;">Canonical Bundle</a></li>"#));
        assert!(!rendered.contains("[wikipedia:Canonical_bundle"));
    }

    #[test]
    fn renders_wikidot_italic_inside_preprocessed_native_list_runs() {
        let wikitext = concat!(
            "* Item 1\n",
            "* Item 2\n",
            "* Item 3\n",
            "* Item 4\n",
            "* Item 5\n",
            "* Item 6\n",
            "* Item 7\n",
            "* All acroamatic material //in absentia// must be voided.\n",
        )
        .to_owned();

        let rendered = RenderService::render_long_native_list_runs(wikitext);

        assert!(rendered.contains(
            r#"<li>All acroamatic material <em>in absentia</em> must be voided.</li>"#
        ));
        assert!(!rendered.contains("//in absentia//"));
    }

    #[test]
    fn leaves_double_slashes_inside_native_list_external_link_urls() {
        let wikitext = concat!(
            "* Item 1\n",
            "* Item 2\n",
            "* Item 3\n",
            "* Item 4\n",
            "* Item 5\n",
            "* Item 6\n",
            "* Item 7\n",
            "* Source [http://example.com/a//b//c label]\n",
        )
        .to_owned();

        let rendered = RenderService::render_long_native_list_runs(wikitext);

        assert!(rendered.contains(r#"<a href="http://example.com/a//b//c">label</a>"#));
        assert!(!rendered.contains("a<em>b</em>c"));
        assert!(!rendered.contains("a&lt;em&gt;b&lt;/em&gt;c"));
    }

    #[test]
    fn renders_nested_inline_wikidot_spans_inside_preprocessed_native_list_runs() {
        let wikitext = concat!(
            "* Item 1\n",
            "* Item 2\n",
            "* Item 3\n",
            "* Item 4\n",
            "* Item 5\n",
            "* Item 6\n",
            "* Item 7\n",
            "* Nested [[span class=\"outer\"]]a [[span class=\"inner\"]]b[[/span]] c[[/span]]\n",
        )
        .to_owned();

        let rendered = RenderService::render_long_native_list_runs(wikitext);

        assert!(
            rendered.contains(
                r#"<span class="outer">a <span class="inner">b</span> c</span>"#
            )
        );
        assert!(!rendered.contains("[[span"));
        assert!(!rendered.contains("[[/span]]"));
    }

    #[test]
    fn ignores_unterminated_span_like_text_before_native_list_span_close() {
        let wikitext = concat!(
            "* Item 1\n",
            "* Item 2\n",
            "* Item 3\n",
            "* Item 4\n",
            "* Item 5\n",
            "* Item 6\n",
            "* Item 7\n",
            "* Literal [[span class=\"outer\"]]a [[span text[[/span]]\n",
        )
        .to_owned();

        let rendered = RenderService::render_long_native_list_runs(wikitext);

        assert!(rendered.contains(r#"<span class="outer">a [[span text</span>"#));
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
    fn removes_wikidot_metacomponent_documentation_block() {
        let mut wikitext = concat!(
            "[[module CSS]]\n",
            "@import url(https://scp-wiki.wdfiles.com/local--code/component%3Acroqstyle/1);\n",
            "[[/module]]\n",
            "\n",
            "[!-- Begin metacomponent context detection --]\n",
            "[!-- -[[iftags +component]]-][[/iftags]]\n",
            "[[div class=\"croqstyle__documentation\"]]\n",
            "Documentation that live Wikidot hides when included on articles.\n",
            "[[/div]]\n",
            "[[iftags +component]][!-[[/iftags]]- --]\n",
            "[!-- End metacomponent context detection --]\n",
            "\n",
            "[[module CSS]]\n",
            ".usable { display: block; }\n",
            "[[/module]]\n",
        )
        .to_owned();

        RenderService::remove_wikidot_metacomponent_documentation(&mut wikitext);

        assert!(wikitext.contains("component%3Acroqstyle/1"));
        assert!(wikitext.contains(".usable { display: block; }"));
        assert!(!wikitext.contains("croqstyle__documentation"));
        assert!(!wikitext.contains("Documentation that live Wikidot hides"));
    }

    #[test]
    fn restores_wikidot_collapsible_legacy_classes() {
        let html = concat!(
            r#"<details class="wj-collapsible" data-show-top>"#,
            r#"<summary class="wj-collapsible-button wj-collapsible-button-top">"#,
            r#"<span class="wj-collapsible-show-text">show</span>"#,
            r#"<span class="wj-collapsible-hide-text">hide</span>"#,
            "</summary>",
            r#"<div class="wj-collapsible-content"><p>body</p></div>"#,
            "</details>",
        );

        let restored = RenderService::restore_wikidot_collapsible_compatibility(html);

        assert!(restored.contains("collapsible-block"));
        assert!(restored.contains("collapsible-block-folded"));
        assert!(restored.contains("collapsible-block-unfolded"));
        assert!(restored.contains("collapsible-block-content"));
        assert!(restored.contains("collapsible-block-link"));
        assert!(restored.contains("collapsible-block-unfolded-link"));
        assert!(restored.contains("<details"));
        assert!(restored.contains("<summary"));
        assert!(!restored.contains("onclick="));
        assert!(!restored.contains("style=\"display:none\""));
        assert!(!restored.contains("wj-collapsible"));
    }

    #[test]
    fn restores_wikidot_code_block_dom_classes() {
        let html = concat!(
            r#"<wj-code class="wj-code wj-language-css">"#,
            r#"<div class="wj-code-panel">"#,
            r#"<wj-code-copy type="button" class="wj-code-copy">copy</wj-code-copy>"#,
            r#"<span class="wj-code-language">css</span>"#,
            "</div>",
            "<pre><code>.x { color: red; }</code></pre>",
            "</wj-code>",
        );

        let restored = RenderService::restore_wikidot_code_block_dom_compatibility(html);

        assert!(restored.contains(r#"<div class="code">"#));
        assert!(restored.contains("<pre><code>.x { color: red; }</code></pre>"));
        assert!(!restored.contains("wj-code"));
        assert!(!restored.contains("wj-code-copy"));
        assert!(!restored.contains("wj-code-language"));
    }

    #[test]
    fn restores_wikidot_tabview_dom_classes() {
        let html = concat!(
            r#"<wj-tabs class="wj-tabs">"#,
            r#"<div class="wj-tabs-button-list" role="tablist">"#,
            r#"<wj-tabs-button class="wj-tabs-button" id="wj-id-a" role="tab" aria-label="One" aria-selected="true" aria-controls="wj-id-pa" tabindex="0">One</wj-tabs-button>"#,
            r#"<wj-tabs-button class="wj-tabs-button" id="wj-id-b" role="tab" aria-label="Two" aria-selected="false" aria-controls="wj-id-pb" tabindex="-1">Two</wj-tabs-button>"#,
            "</div>",
            r#"<div class="wj-tabs-panel-list">"#,
            r#"<div class="wj-tabs-panel" id="wj-id-pa" role="tabpanel" aria-labelledby="wj-id-a" tabindex="0">First</div>"#,
            r#"<div class="wj-tabs-panel" id="wj-id-pb" role="tabpanel" aria-labelledby="wj-id-b" tabindex="0" hidden>Second</div>"#,
            "</div>",
            "</wj-tabs>",
        );

        let restored = RenderService::restore_wikidot_tabview_dom_compatibility(html);

        assert!(!restored.contains("tabview-min.js"));
        assert!(restored.contains(r#"<div class="yui-navset">"#));
        assert!(restored.contains(r#"<ul class="yui-nav">"#));
        assert!(restored.contains(r#"<div class="yui-content">"#));
        assert!(restored.contains(r#"<div style="display: block;">First</div>"#));
        assert!(restored.contains(r#"<div style="display:none">Second</div>"#));
        assert!(
            restored
                .contains(r#"<li class="selected"><a href="javascript:;">One</a></li>"#)
        );
        assert!(restored.contains(r#"<li><a href="javascript:;">Two</a></li>"#));
        assert!(restored.contains("</a></li>\n<li>"));
        assert!(!restored.contains("wj-tabs"));
        assert!(!restored.contains("aria-selected"));
        assert!(!restored.contains("role=\"tab\""));
        assert!(!restored.contains(" hidden"));
    }

    #[test]
    fn restores_wikidot_tabview_panel_visibility_per_tabview() {
        let html = concat!(
            r#"<wj-tabs class="wj-tabs">"#,
            r#"<div class="wj-tabs-button-list">"#,
            r#"<wj-tabs-button class="wj-tabs-button" aria-selected="true">One</wj-tabs-button>"#,
            r#"<wj-tabs-button class="wj-tabs-button" aria-selected="false">Two</wj-tabs-button>"#,
            "</div>",
            r#"<div class="wj-tabs-panel-list">"#,
            r#"<div class="wj-tabs-panel">First A</div>"#,
            r#"<div class="wj-tabs-panel" hidden>First B</div>"#,
            "</div>",
            "</wj-tabs>",
            r#"<wj-tabs class="wj-tabs">"#,
            r#"<div class="wj-tabs-button-list">"#,
            r#"<wj-tabs-button class="wj-tabs-button" aria-selected="true">Three</wj-tabs-button>"#,
            r#"<wj-tabs-button class="wj-tabs-button" aria-selected="false">Four</wj-tabs-button>"#,
            "</div>",
            r#"<div class="wj-tabs-panel-list">"#,
            r#"<div class="wj-tabs-panel">Second A</div>"#,
            r#"<div class="wj-tabs-panel" hidden>Second B</div>"#,
            "</div>",
            "</wj-tabs>",
        );

        let restored = RenderService::restore_wikidot_tabview_dom_compatibility(html);

        assert!(restored.contains(r#"<div style="display: block;">First A</div>"#));
        assert!(restored.contains(r#"<div style="display:none">First B</div>"#));
        assert!(restored.contains(r#"<div style="display: block;">Second A</div>"#));
        assert!(restored.contains(r#"<div style="display:none">Second B</div>"#));
    }

    #[test]
    fn restores_residual_wikidot_div_markers_around_tabview() {
        let html = concat!(
            r#"<p>[[div class=&quot;m-wrapper standalone series&quot;]]</p>"#,
            r#"<div class="yui-navset"><div class="yui-content">"#,
            r#"<div><p>Order by Date of Creation</p></div>"#,
            r#"<div><p>[[/div]]</p></div>"#,
            r#"</div></div>"#,
        );

        let restored =
            RenderService::restore_residual_wikidot_div_paragraph_markers(html);

        assert!(restored.contains(r#"<div class="m-wrapper standalone series">"#));
        assert!(restored.contains(r#"<div class="yui-navset">"#));
        assert!(!restored.contains("[[div"));
        assert!(!restored.contains("[[/div]]"));
    }

    #[test]
    fn leaves_residual_wikidot_div_closer_without_restored_opener() {
        let html = concat!(
            r#"<p>[[div onclick=&quot;unsupported&quot;]]</p>"#,
            r#"<span>Body</span>"#,
            r#"<p>[[/div]]</p>"#,
        );

        let restored =
            RenderService::restore_residual_wikidot_div_paragraph_markers(html);

        assert_eq!(restored, html);
    }

    #[test]
    fn restores_standalone_residual_wikidot_div_lines() {
        let html = concat!(
            "====\n",
            r#"[[div class=&quot;preview&quot;]]"#,
            "\nPreview text\n",
            "[[/div]]\n",
            r#"[[div id=&quot;cromthumbnail&quot; style=&quot;display: none;&quot;]]"#,
            "\nHidden text\n",
            "[[/div]]\n",
            "[[div]]\n",
            "Bare body\n",
            "[[/div]]\n",
        );

        let restored =
            RenderService::restore_residual_wikidot_div_paragraph_markers(html);

        assert!(restored.contains(r#"<div class="preview">"#));
        assert!(restored.contains(r#"<div id="cromthumbnail" style="display: none;">"#));
        assert!(restored.contains("<div>\nBare body\n</div>"));
        assert!(!restored.contains("[[div"));
        assert!(!restored.contains("[[/div]]"));
    }

    #[test]
    fn leaves_standalone_residual_wikidot_div_lines_inside_pre() {
        let html = concat!(
            "<pre>\n",
            r#"[[div class=&quot;literal&quot;]]"#,
            "\nbody\n",
            "[[/div]]\n",
            "</pre>\n",
        );

        let restored =
            RenderService::restore_residual_wikidot_div_paragraph_markers(html);

        assert_eq!(restored, html);
    }

    #[test]
    fn leaves_standalone_residual_wikidot_div_closer_without_restored_opener() {
        let html = "Before\n[[/div]]\nAfter\n";

        let restored =
            RenderService::restore_residual_wikidot_div_paragraph_markers(html);

        assert_eq!(restored, html);
    }

    #[test]
    fn restores_residual_wikidot_span_markers() {
        let html = concat!(
            r#"<div class="top-left-box">"#,
            r#"[[span class=&quot;item&quot;]]Item#:[[/span]] "#,
            r#"[[span class=&quot;number&quot; onclick=&quot;bad()&quot;]]SCP-001[[/span]]"#,
            "</div>",
        );

        let restored = RenderService::restore_residual_wikidot_span_markers(html);

        assert!(restored.contains(r#"<span class="item">Item#:</span>"#));
        assert!(restored.contains(r#"<span class="number">SCP-001</span>"#));
        assert!(!restored.contains("[[span"));
        assert!(!restored.contains("onclick"));
    }

    #[test]
    fn restores_multiline_residual_wikidot_span_markers() {
        let html = concat!(
            "<div>\n",
            r#"[[span style=&quot;font-family: Courier New; font-size: 120%;&quot;]]William Shakespeare has a problem. "#,
            "\n\n",
            "None could articulate why.[[/span]]\n",
            "</div>\n",
        );

        let restored = RenderService::restore_residual_wikidot_span_markers(html);

        assert!(restored.contains(
            r#"<span style="font-family: Courier New; font-size: 120%;">William Shakespeare has a problem. "#
        ));
        assert!(restored.contains("None could articulate why.</span>"));
        assert!(!restored.contains("[[span"));
        assert!(!restored.contains("[[/span]]"));
    }

    #[test]
    fn leaves_residual_wikidot_span_markers_inside_pre() {
        let html = concat!(
            "<pre>\n",
            r#"[[span class=&quot;literal&quot;]]body"#,
            "\n",
            "still literal[[/span]]",
            "\n</pre>\n",
        );

        let restored = RenderService::restore_residual_wikidot_span_markers(html);

        assert_eq!(restored, html);
    }

    #[test]
    fn leaves_residual_span_markers_inside_quoted_tag_attributes() {
        let html = concat!(
            r#"<img src=x alt="safe > [[span class=&quot;x onerror=alert(1)//&quot;]]broken[[/span]]">"#,
            r#" [[span class=&quot;safe&quot;]]body[[/span]]"#,
        );

        let restored = RenderService::restore_residual_wikidot_span_markers(html);

        assert!(restored.contains(
            r#"<img src=x alt="safe > [[span class=&quot;x onerror=alert(1)//&quot;]]broken[[/span]]">"#,
        ));
        assert!(restored.contains(r#"<span class="safe">body</span>"#));
        assert!(!restored.contains(r#"<span class="x onerror=alert(1)//">"#));
    }

    #[test]
    fn restores_residual_spans_across_safe_html_elements_only() {
        let html = concat!(
            r#"[[span class=&quot;outer&quot;]]before <strong>bold</strong> "#,
            r#"[[span class=&quot;inner&quot;]]inside[[/span]] after[[/span]]"#,
        );

        assert_eq!(
            RenderService::restore_residual_wikidot_span_markers(html),
            r#"<span class="outer">before <strong>bold</strong> <span class="inner">inside</span> after</span>"#,
        );
    }

    #[test]
    fn leaves_residual_span_markers_in_comments_and_raw_or_foreign_elements() {
        let html = concat!(
            "<!-- [[span class=&quot;comment&quot;]]x[[/span]] -->",
            "<style>[[span class=&quot;style&quot;]]x[[/span]]</style>",
            "<ScRiPt>[[span class=&quot;script&quot;]]x[[/span]]</sCrIpT>",
            "<svg><text>[[span class=&quot;svg&quot;]]x[[/span]]</text></svg>",
        );

        assert_eq!(
            RenderService::restore_residual_wikidot_span_markers(html),
            html
        );
    }

    #[test]
    fn does_not_pair_residual_spans_across_opaque_or_comment_boundaries() {
        for boundary in [
            "<style>body { color: red; }</style>",
            "<script>void 0</script>",
            "<pre>literal</pre>",
            "<svg><text>foreign</text></svg>",
            "<!-- comment -->",
        ] {
            let html = format!(
                r#"[[span class=&quot;outer&quot;]]before{boundary}after[[/span]]"#,
            );

            assert_eq!(
                RenderService::restore_residual_wikidot_span_markers(&html),
                html,
                "boundary {boundary}",
            );
        }
    }

    #[test]
    fn restores_standalone_residual_wikidot_alignment_lines() {
        let html = concat!(
            "<div>\n",
            "[[=]]\n",
            "**Centered**\n",
            "[[/=]]\n",
            "[[&lt;]]\n",
            "Left\n",
            "[[/&lt;]]\n",
            "[[&gt;]]\n",
            "Right\n",
            "[[/&gt;]]\n",
            "</div>\n",
        );

        let restored = RenderService::restore_residual_wikidot_alignment_markers(html);

        assert!(restored.contains(r#"<div style="text-align: center;">"#,));
        assert!(restored.contains(r#"<div style="text-align: left;">"#));
        assert!(restored.contains(r#"<div style="text-align: right;">"#));
        assert!(!restored.contains("[[=]]"));
        assert!(!restored.contains("[[/=]]"));
        assert!(!restored.contains("[[&lt;]]"));
        assert!(!restored.contains("[[/&lt;]]"));
        assert!(!restored.contains("[[&gt;]]"));
        assert!(!restored.contains("[[/&gt;]]"));
    }

    #[test]
    fn restores_residual_wikidot_alignment_html_markers_around_collapsible() {
        let html = concat!(
            "<hr><p>[[=]]</p>",
            r#"<span style="font-size: 150%;">"#,
            r#"<details class="collapsible-block">"#,
            r#"<summary class="collapsible-block-link">"#,
            r#"<span class="collapsible-block-link">Show</span>"#,
            "</summary></details></span>",
            "<br>[[/=]]<br>",
            "<span>after</span>",
        );

        let restored = RenderService::restore_residual_wikidot_alignment_markers(html);

        assert!(restored.contains(
            r#"<hr><div style="text-align: center;"><span style="font-size: 150%;">"#
        ));
        assert!(restored.contains(r#"</details></span></div><br><span>after</span>"#));
        assert!(!restored.contains("[[=]]"));
        assert!(!restored.contains("[[/=]]"));
    }

    #[test]
    fn leaves_standalone_residual_wikidot_alignment_lines_inside_pre() {
        let html = concat!("<pre>\n", "[[=]]\n", "literal\n", "[[/=]]\n", "</pre>\n");

        let restored = RenderService::restore_residual_wikidot_alignment_markers(html);

        assert_eq!(restored, html);
    }

    #[test]
    fn leaves_standalone_residual_wikidot_alignment_closer_without_opener() {
        let html = "Before\n[[/=]]\nAfter\n";

        let restored = RenderService::restore_residual_wikidot_alignment_markers(html);

        assert_eq!(restored, html);
    }

    #[test]
    fn restores_standalone_residual_wikidot_separator_lines() {
        let html = concat!("Before\n", "------\n", "@@ @@\n", "~~~~\n", "After\n",);

        let restored = RenderService::restore_residual_wikidot_separator_markers(html);

        assert!(restored.contains("Before\n<hr>\n"));
        assert!(
            restored.contains(r#"<p><span style="white-space: pre-wrap;"> </span></p>"#)
        );
        assert!(
            restored.contains(
                r#"<div style="clear:both; height: 0px; font-size: 1px"></div>"#
            )
        );
        assert!(!restored.contains("------"));
        assert!(!restored.contains("@@ @@"));
        assert!(!restored.contains("~~~~"));
    }

    #[test]
    fn leaves_standalone_residual_wikidot_separator_lines_inside_raw_text() {
        let html = concat!(
            "<style>\n",
            "------\n",
            "@@ @@\n",
            "</style>\n",
            "<pre>\n",
            "~~~~\n",
            "</pre>\n",
        );

        let restored = RenderService::restore_residual_wikidot_separator_markers(html);

        assert_eq!(restored, html);
    }

    #[test]
    fn restores_standalone_residual_wikidot_heading_lines() {
        let html = concat!(
            "====\n",
            "++* Info\n",
            "++ 43NET DEVICE REPORT\n",
            "+++ **Chief, Security and Containment Section**\n",
            "=====\n",
        );

        let restored = RenderService::restore_residual_wikidot_heading_markers(html);

        assert!(restored.contains("<h2><span>Info</span></h2>"));
        assert!(restored.contains("<h2><span>43NET DEVICE REPORT</span></h2>"));
        assert!(restored.contains(
            "<h3><span>**Chief, Security and Containment Section**</span></h3>"
        ));
        assert!(!restored.contains("===="));
        assert!(!restored.contains("====="));
        assert!(!restored.contains("++*"));
        assert!(!restored.contains("++ 43NET"));
    }

    #[test]
    fn leaves_standalone_residual_wikidot_heading_lines_inside_raw_text() {
        let html = concat!(
            "<style>\n",
            "++ hidden\n",
            "====\n",
            "</style>\n",
            "<pre>\n",
            "+++ literal\n",
            "</pre>\n",
        );

        let restored = RenderService::restore_residual_wikidot_heading_markers(html);

        assert_eq!(restored, html);
    }

    #[test]
    fn removes_residual_wikidot_iftags_fragments_after_render() {
        let html = concat!(
            r#"<div class="modalbox-title [[iftags 殿堂入り]]heritage[[/iftags]]">"#,
            "title</div>",
            r#"<a href="/[[iftags +en]]target[[/iftags]]">link</a>"#,
        );

        let restored = RenderService::remove_residual_wikidot_iftags_fragments(html);

        assert_eq!(
            restored,
            r#"<div class="modalbox-title ">title</div><a href="/">link</a>"#,
        );
        assert!(!restored.contains("[[iftags"));
        assert!(!restored.contains("[[/iftags]]"));
    }

    #[test]
    fn resolves_residual_wikidot_simple_if_fragments_after_render() {
        let html = concat!(
            r#"<li class="[[#if 1 | folded | unfolded ]] [[#if 0 | colmod-collapsiblealt | active ]]">"#,
            r#"<a href="javascript:;">+ Open</a>"#,
            r#"[[#if 0 | | <a href="javascript:;">- Close</a> ]]"#,
            r#"[[#if false | hidden | <span>Visible</span> ]]"#,
            r#"[[#if true | <span>Shown</span> | hidden ]]"#,
            r#"[[#if 1 | one-branch ]]"#,
            r#"[[#if 0 | hidden-one-branch ]]"#,
            "</li>",
        );

        let restored = RenderService::resolve_wikidot_simple_if_fragments(html);

        assert_eq!(
            restored,
            r#"<li class="folded active"><a href="javascript:;">+ Open</a><a href="javascript:;">- Close</a><span>Visible</span><span>Shown</span>one-branch</li>"#,
        );
        assert!(!restored.contains("[[#if"));
    }

    #[test]
    fn preserves_simple_if_fragments_in_wikidot_literal_regions() {
        let source = concat!(
            "[[code]]\n[[#if 1 | code-example ]]\n[[/code]]\n",
            "@@[[#if 0 | escaped-example ]]@@\n",
            "[[html]]\n[[#if true | html-example ]]\n[[/html]]\n",
            "[!-- [[#if false | comment-example ]] --]\n",
            "[[#if 1 | resolved ]]",
        );

        let restored = RenderService::resolve_wikidot_simple_if_fragments(source);

        assert_eq!(
            restored,
            concat!(
                "[[code]]\n[[#if 1 | code-example ]]\n[[/code]]\n",
                "@@[[#if 0 | escaped-example ]]@@\n",
                "[[html]]\n[[#if true | html-example ]]\n[[/html]]\n",
                "[!-- [[#if false | comment-example ]] --]\n",
                "resolved",
            )
        );
    }

    #[test]
    fn preserves_simple_if_fragments_in_rendered_html_literal_regions() {
        let html = concat!(
            "<code>[[#if 1 | inline-code ]]</code>",
            "<pre>[[#if 0 | preformatted ]]</pre>",
            r#"<div class="code"><div>[[#if true | code-panel ]]</div></div>"#,
            "<script>[[#if 1 | script-example ]]</script>",
            "<style>[[#if 1 | style-example ]]</style>",
            "<textarea>[[#if 1 | textarea-example ]]</textarea>",
            "<p>[[#if 1 | resolved ]]</p>",
        );

        let restored = RenderService::resolve_wikidot_simple_if_fragments(html);

        assert_eq!(
            restored,
            concat!(
                "<code>[[#if 1 | inline-code ]]</code>",
                "<pre>[[#if 0 | preformatted ]]</pre>",
                r#"<div class="code"><div>[[#if true | code-panel ]]</div></div>"#,
                "<script>[[#if 1 | script-example ]]</script>",
                "<style>[[#if 1 | style-example ]]</style>",
                "<textarea>[[#if 1 | textarea-example ]]</textarea>",
                "<p>resolved</p>",
            )
        );
    }

    #[test]
    fn resolves_simple_if_branches_with_balanced_wikidot_markup() {
        let source = concat!(
            "[[#if 1 | [[div]]shown[[/div]] | hidden ]]",
            "[[#if 0 | hidden | [[span class=\"visible\"]]shown[[/span]] ]]",
            "[[#if true | [[div]]one-branch[[/div]] ]]",
            "[[#if 1 | [[#if 0 | no | nested ]] | outer-no ]]",
        );

        let restored = RenderService::resolve_wikidot_simple_if_fragments(source);

        assert_eq!(
            restored,
            concat!(
                "[[div]]shown[[/div]]",
                "[[span class=\"visible\"]]shown[[/span]]",
                "[[div]]one-branch[[/div]]",
                "nested",
            )
        );
    }

    #[test]
    fn resolves_many_sibling_simple_if_fragments_in_one_bounded_pass() {
        let source = "[[#if 1 | x | y ]]".repeat(5_000);

        let restored = RenderService::resolve_wikidot_simple_if_fragments(&source);

        assert_eq!(restored, "x".repeat(5_000));
    }

    #[test]
    fn deeply_nested_simple_if_resolution_stops_at_the_pass_limit() {
        let mut source = "leaf".to_owned();
        for _ in 0..(MAX_WIKIDOT_SIMPLE_IF_PASSES + 8) {
            source = format!("[[#if 1 | {source} | hidden ]]");
        }

        let restored = RenderService::resolve_wikidot_simple_if_fragments(&source);

        assert_eq!(restored.matches("[[#if").count(), 8);
        assert!(restored.contains("leaf"));
    }

    #[test]
    fn resolves_parser_functions_only_outside_literal_regions() {
        let source = concat!(
            "[[code]]\n[[#expr 1+1]]\n[[/code]]\n",
            "@@[[#ifexpr 1 | escaped | hidden]]@@\n",
            "[[html]]\n[[#expr 2+2]]\n[[/html]]\n",
            "[!-- [[#expr 3+3]] --]\n",
            "<code>[[#expr 4+4]]</code>\n",
            "[[#ifexpr 1 | resolved | hidden]] [[#expr 5+5]]",
        );

        let restored = RenderService::resolve_wikidot_parser_functions(source);

        assert_eq!(
            restored,
            concat!(
                "[[code]]\n[[#expr 1+1]]\n[[/code]]\n",
                "@@[[#ifexpr 1 | escaped | hidden]]@@\n",
                "[[html]]\n[[#expr 2+2]]\n[[/html]]\n",
                "[!-- [[#expr 3+3]] --]\n",
                "<code>[[#expr 4+4]]</code>\n",
                "resolved 10",
            )
        );
    }

    #[test]
    fn list_pages_expression_prepasses_preserve_literal_examples() {
        let signed = concat!(
            "@@[[#ifexpr -3 > -1 | + | - ]][[#expr abs(-3)]]@@ ",
            "[[#ifexpr -3 > -1 | + | - ]][[#expr abs(-3)]]",
        );
        let signed = resolve_list_pages_signed_abs_expressions(signed);
        let signed = RenderService::resolve_wikidot_parser_functions(&signed);
        assert_eq!(
            signed,
            "@@[[#ifexpr -3 > -1 | + | - ]][[#expr abs(-3)]]@@ -3"
        );

        let numeric = concat!(
            "[[code]]\n[[#ifexpr 2 > 1 | code | hidden]]\n[[/code]] ",
            "[[#ifexpr 2 > 1 | visible | hidden]]",
        );
        assert_eq!(
            resolve_wikidot_numeric_ifexpr(numeric),
            "[[code]]\n[[#ifexpr 2 > 1 | code | hidden]]\n[[/code]] visible"
        );
    }

    #[test]
    fn resolves_literal_wikidot_simple_if_before_ftml_parsing() {
        let mut source = r#"[[div class="[[#if 1 | folded | unfolded ]] [[#if 0 | inactive | active ]]"]]body[[/div]]"#.to_owned();
        let page_info = fallback_test_page_info("conditional", "Conditional");

        RenderService::prepare_wikidot_conditionals_for_include_expansion(
            &mut source,
            &page_info,
        );

        assert_eq!(source, r#"[[div class="folded active"]]body[[/div]]"#);
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let tokens = ftml::tokenize(&source);
        let (_, errors) = ftml::parse(&tokens, &page_info, &settings).into();
        assert!(errors.is_empty(), "{errors:#?}");
    }

    #[test]
    fn render_preparation_resolves_generated_simple_if_with_link_branch() {
        let page_info = fallback_test_page_info("conditional", "Conditional");
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let outer = RenderService::prepare_outer_render_wikitext(
            super::ExpandedRenderWikitext {
                wikitext: concat!(
                    "[[div class=\"colmod-block\"]]\n",
                    "[[div]]link[[#if 0 | | [# fallback] ]][[/div]]\n",
                    "[[/div]]",
                )
                .to_owned(),
                included_pages: Vec::new(),
                wikidot_compat_html: CompatHtmlFragments::new(""),
            },
            &page_info,
            &settings,
        );

        assert!(!outer.wikitext.contains("[[#if"));
        assert!(outer.wikitext.contains("[# fallback]"));
        let inner = RenderService::prepare_inner_render_wikitext(outer, &settings);
        assert!(!inner.wikitext.contains("[[#if"));
        let tokens = ftml::tokenize(&inner.wikitext);
        let (_, errors) = ftml::parse(&tokens, &page_info, &settings).into();
        assert!(errors.is_empty(), "{errors:#?}");
    }

    #[test]
    fn restores_rendered_wikidot_mailform_blocks_after_render() {
        let html = concat!(
            r#"<div class="fakeprot">"#,
            r#"<p>[[module MailForm to=&quot;dummy&quot; button=&quot;Go&quot;]]</p>"#,
            "<ol><li>name</li><ul>",
            "<li>title: ID</li>",
            "<li>default: Site:8192 Director Y.Gineri</li>",
            "<li>type: text</li>",
            "<li>rules:</li><ul><li>required: true</li><li>maxLength:10</li></ul>",
            "</ul></ol>",
            r#"<p>[[/module]]</p>"#,
            "</div>",
        );

        let restored = RenderService::restore_wikidot_mailform_compatibility(html);

        assert!(restored.contains(r#"<div class="mailform-box">"#));
        assert!(restored.contains(r#"<form class="form" action="javascript:;">"#));
        assert!(restored.contains(
            r#"<input class="text" type="text" name="name" value="Site:8192 Director Y.Gineri" maxlength="10" size="30">"#,
        ));
        assert!(restored.contains(r#"<div class="field-error-message"></div>"#));
        assert!(
            restored.contains(
                r#"<div class="buttons"><input type="submit" value="Go"></div>"#
            )
        );
        assert!(!restored.contains("[[module MailForm"));
    }

    #[test]
    fn removes_wikijump_table_body_wrappers_after_render() {
        let html = "<table><tbody><tr><td>cell</td></tr></tbody></table>";

        let restored = RenderService::remove_wikijump_table_body_wrappers(html);

        assert_eq!(restored, "<table><tr><td>cell</td></tr></table>");
    }

    #[test]
    fn removes_wikidot_compat_style_blocks_after_render() {
        let html = concat!(
            "<p>before</p>",
            r#"<style type="text/css">.x { color: red; }</style>"#,
            r#"<div style="color: blue">after</div>"#,
        );

        let restored = RenderService::remove_wikidot_compat_style_blocks(html);

        assert_eq!(
            restored,
            r#"<p>before</p><div style="color: blue">after</div>"#,
        );
    }

    #[test]
    fn preserves_wikidot_css_module_style_blocks_after_render() {
        let html = concat!(
            "<p>before</p>",
            r#"<style>.name { font-size: 10rem; }</style>"#,
            r#"<div style="color: blue">after</div>"#,
        );

        let restored = RenderService::remove_wikidot_compat_style_blocks(html);

        assert_eq!(
            restored,
            concat!(
                "<p>before</p>",
                r#"<style>.name { font-size: 10rem; }</style>"#,
                r#"<div style="color: blue">after</div>"#,
            ),
        );
    }

    #[test]
    fn preserves_basalt_shell_compatibility_style_after_render() {
        let mut html = r#"<p><iframe src="/-/wikidot-interwiki/styleFrame.html?theme=https://scp-wiki.wdfiles.com/local--code/theme%3Abasalt/1&css={$css}" style="display: none"></iframe></p>"#.to_owned();

        super::apply_basalt_shell_compatibility(&mut html);
        let restored = RenderService::remove_wikidot_compat_style_blocks(&html);

        assert!(restored.contains("#side-bar"));
        assert!(restored.contains("display: block !important"));
        assert!(restored.contains("left: -272px !important"));
        assert!(restored.contains("#top-bar"));
        assert!(restored.contains("display: contents"));
        assert!(restored.contains("#top-bar ul ul"));
        assert!(restored.contains("display: flex !important"));
        assert!(restored.contains("#header h2"));
        assert!(restored.contains("#side-bar .heading p"));
        assert!(restored.contains("margin-top: -12rem !important"));
        assert!(restored.contains("#page-info"));
        assert!(restored.contains("text-transform: uppercase"));
        assert!(restored.contains("#page-options-bottom.page-options-bottom"));
        assert!(restored.contains("display: flex"));
        assert!(
            restored.contains(".admo-rate_splash .page-rate-widget-box .rate-points")
        );
        assert!(restored.contains(".admo-rate_splash .page-rate-widget-box .cancel"));
    }

    #[test]
    fn preserves_blankstyle_open_menu_compatibility_style_after_render() {
        let mut html = concat!(
            r#"<p><style>div#extra-div-1{background:url("https://scp-wiki.wjfiles.localhost/local--files/theme%3Ablankstyle/43Head.png");}</style></p>"#,
            r#"<p>body</p>"#,
        )
        .to_owned();

        super::apply_blankstyle_shell_compatibility(&mut html);
        let restored = RenderService::remove_wikidot_compat_style_blocks(&html);

        assert!(restored.contains("#top-bar .mobile-top-bar"));
        assert!(restored.contains("display: block !important"));
        assert!(restored.contains("#top-bar .mobile-top-bar > ul"));
        assert!(restored.contains("display: none !important"));
        assert!(restored.contains("#top-bar div.open-menu a"));
        assert!(restored.contains("position: fixed !important"));
        assert!(restored.contains("line-height: 32px !important"));
    }

    #[test]
    fn blankstyle_open_menu_compatibility_ignores_unrelated_pages() {
        let mut html = r#"<p>ordinary page body</p>"#.to_owned();

        super::apply_blankstyle_shell_compatibility(&mut html);

        assert_eq!(html, r#"<p>ordinary page body</p>"#);
    }

    #[test]
    fn restores_wikidot_inline_math_compatibility_after_render() {
        let html = concat!(
            "This is ",
            r#"<span class="wj-math wj-math-inline">"#,
            r#"<code class="wj-math-source wj-hidden" aria-hidden="true">\frac{1}{2}</code>"#,
            r#"<wj-math-ml class="wj-math-ml"><math><mfrac><mn>1</mn><mn>2</mn></mfrac></math></wj-math-ml>"#,
            "</span>",
            ".",
        );

        let restored = RenderService::restore_wikidot_inline_math_compatibility(html);

        assert_eq!(
            restored,
            r#"This is <span class="math-inline">$\frac{1}{2}$</span>."#,
        );
        assert!(!restored.contains("wj-math"));
        assert!(!restored.contains("<math"));
    }

    #[test]
    fn restores_wikidot_footnote_dom_compatibility_after_render() {
        let html = concat!(
            r#"<p>text<span class="wj-footnote-ref">"#,
            r#"<wj-footnote-ref-marker class="wj-footnote-ref-marker" role="link" aria-label="Footnote 2." data-id="2">2</wj-footnote-ref-marker>"#,
            r#"</span><div class="wj-footnote-ref-tooltip" aria-hidden="true">"#,
            r#"<span class="wj-footnote-ref-tooltip-label">Footnote 2.</span>"#,
            r#"<div class="wj-footnote-ref-contents"><div>hidden note</div></div>"#,
            r#"</div> after</p>"#,
            r#"<div class="wj-footnote-list"><div class="wj-title">Footnotes</div></div>"#,
        );

        let restored = RenderService::restore_wikidot_footnote_dom_compatibility(html);

        assert!(restored.contains(
            r#"<sup class="footnoteref"><a id="footnoteref-2" href="javascript:;" class="footnoteref" onclick="WIKIDOT.page.utils.scrollToReference('footnote-2')">2</a></sup> after"#
        ));
        assert!(restored.contains(r#"<div class="wj-footnote-list footnotes-footer">"#));
        assert!(restored.contains(r#"<div class="wj-title title">Footnotes</div>"#));
        assert!(!restored.contains(r#"<span class="wj-footnote-ref">"#));
        assert!(!restored.contains("wj-footnote-ref-tooltip"));
        assert!(!restored.contains("hidden note"));
    }

    #[test]
    fn removes_phrasing_content_footnote_tooltips_after_render() {
        let html = concat!(
            r#"<p>text<span class="wj-footnote-ref">"#,
            r#"<wj-footnote-ref-marker class="wj-footnote-ref-marker" role="link" aria-label="Footnote 2." data-id="2">2</wj-footnote-ref-marker>"#,
            r#"<span class="wj-footnote-ref-tooltip" aria-hidden="true">"#,
            r#"<span class="wj-footnote-ref-tooltip-label">Footnote 2.</span>"#,
            r#"<span class="wj-footnote-ref-contents">hidden <em>note</em></span>"#,
            r#"</span></span> after</p>"#,
        );

        let restored = RenderService::restore_wikidot_footnote_dom_compatibility(html);

        assert!(restored.contains(
            r#"<sup class="footnoteref"><a id="footnoteref-2" href="javascript:;" class="footnoteref" onclick="WIKIDOT.page.utils.scrollToReference('footnote-2')">2</a></sup>"#
        ));
        assert!(!restored.contains("wj-footnote-ref-tooltip"));
        assert!(!restored.contains("hidden note"));
    }

    #[test]
    fn restores_wikidot_footnote_refs_without_source_spacing() {
        let html = concat!(
            r#"<p>behaviour. <span class="wj-footnote-ref">"#,
            r#"<wj-footnote-ref-marker class="wj-footnote-ref-marker" role="link" aria-label="Footnote 7." data-id="7">7</wj-footnote-ref-marker>"#,
            r#"</span></p>"#,
        );

        let restored = RenderService::restore_wikidot_footnote_dom_compatibility(html);

        assert!(restored.contains(r#"behaviour.<sup class="footnoteref">"#));
        assert!(!restored.contains(r#"behaviour. <sup"#));
    }

    #[test]
    fn restores_wikidot_text_ellipsis_only_in_text_nodes() {
        let html = concat!(
            r#"<p>"scp-049...." be.... witnessed...</p>"#,
            r#"<a title="keep....">label....</a>"#,
            r#"<code>keep....</code>"#,
            r#"<style>.x:after{content:"keep...."}</style>"#,
        );

        let restored = RenderService::restore_wikidot_text_ellipsis_compatibility(html);

        assert!(restored.contains(r#"<p>"scp-049…." be…. witnessed…</p>"#));
        assert!(restored.contains(r#"<a title="keep....">label….</a>"#));
        assert!(restored.contains(r#"<code>keep....</code>"#));
        assert!(restored.contains(r#"<style>.x:after{content:"keep...."}</style>"#));
    }

    #[test]
    fn restores_wikidot_ta_badge_default_classes_after_render() {
        let html = concat!(
            r#"<div class="bg-frame bg-shadow-{$bg-shadow} plate-shadow-{$plate-shadow}">"#,
            r#"<div class="item-mobile-mode-{$item-mobile-mode} item-align-{$item-align}">"#,
            r#"<a class="{$badge-top-link}" href="{$badge-top-link}"></a>"#,
            r#"<a class="{$badge-right-link}" href="{$badge-right-link}"></a>"#,
            r#"<a class="{$badge-left-link}" href="{$badge-left-link}"></a>"#,
            r#"<a class="{$item-lt-link}" href="{$item-lt-link}"></a>"#,
            r#"<a class="{$item-lc-link}" href="{$item-lc-link}"></a>"#,
            r#"<a class="{$item-lb-link}" href="{$item-lb-link}"></a>"#,
            r#"<a class="{$item-rt-link}" href="{$item-rt-link}"></a>"#,
            r#"<a class="{$item-rc-link}" href="{$item-rc-link}"></a>"#,
            r#"<a class="{$item-rb-link}" href="{$item-rb-link}"></a>"#,
            "</div></div>",
        );

        let restored =
            RenderService::restore_wikidot_ta_badge_default_compatibility(html);

        assert_eq!(
            restored,
            concat!(
                r#"<div class="bg-frame bg-shadow-true plate-shadow-true">"#,
                r#"<div class="item-mobile-mode-true item-align-true">"#,
                r#"<a class="empty" href="empty"></a>"#,
                r#"<a class="empty" href="empty"></a>"#,
                r#"<a class="empty" href="empty"></a>"#,
                r#"<a class="empty" href="empty"></a>"#,
                r#"<a class="empty" href="empty"></a>"#,
                r#"<a class="empty" href="empty"></a>"#,
                r#"<a class="empty" href="empty"></a>"#,
                r#"<a class="empty" href="empty"></a>"#,
                r#"<a class="empty" href="empty"></a>"#,
                "</div></div>",
            ),
        );
    }

    #[test]
    fn removes_wikijump_underline_wrappers_without_stripping_strikethrough() {
        let html = "<p><u>under</u> and <s>strike</s></p>";

        let restored = RenderService::remove_wikijump_underline_wrappers(html);

        assert_eq!(restored, "<p>under and <s>strike</s></p>");
    }

    #[test]
    fn preserves_compact_ftml_dash_strikethrough_after_compat_cleanup() {
        let rendered =
            render_wikidot_page_body_after_compat_restore("before --removed-- after");

        assert_eq!(rendered, "<p>before <s>removed</s> after</p>");
        assert_eq!(
            RenderService::remove_wikijump_underline_wrappers(&rendered),
            rendered,
        );
    }

    #[test]
    fn removes_wikidot_userkarma_background_styles_after_render() {
        let html = concat!(
            r#"<span class="printuser avatarhover">"#,
            r#"<img class="small" src="http://www.wikidot.com/avatar.php?userid=4598089&amp;size=small" style="background-image: url(https://www.wikidot.com/userkarma.php?u=4598089)">"#,
            r#"</span>"#,
        );

        let restored = RenderService::remove_wikidot_userkarma_background_styles(html);

        assert_eq!(
            restored,
            concat!(
                r#"<span class="printuser avatarhover">"#,
                r#"<img class="small" src="http://www.wikidot.com/avatar.php?userid=4598089&amp;size=small">"#,
                r#"</span>"#,
            ),
        );
        assert!(!restored.contains("userkarma.php"));
    }

    #[test]
    fn leaves_plain_iftags_block() {
        let mut wikitext = "[[iftags +theme]]\nbody\n[[/iftags]]\n".to_owned();

        RenderService::remove_unresolved_variable_iftags_blocks(&mut wikitext);

        assert_eq!(wikitext, "[[iftags +theme]]\nbody\n[[/iftags]]\n");
    }

    #[test]
    fn resolves_single_line_wikidot_iftags_fragments_in_source() {
        let page_info = ftml::data::PageInfo {
            page: Cow::Borrowed("some-page"),
            category: None,
            site: Cow::Borrowed("sandbox"),
            title: Cow::Borrowed("A page"),
            alt_title: None,
            score: ftml::data::ScoreValue::Float(0.0),
            tags: vec![Cow::Borrowed("active")],
            language: Cow::Borrowed("default"),
        };
        let mut wikitext = concat!(
            "[[div_ [[iftags +missing]]style=\"display: flex;\"[[/iftags]] class=\"Dendo\"]]\n",
            "[[/div]]\n",
            "[[span class=\"[[iftags +active]]visible[[/iftags]] [[iftags +missing]]hidden[[/iftags]]\"]]body[[/span]]\n",
            "[[iftags +missing]]\n",
            "multiline\n",
            "[[/iftags]]\n",
        )
        .to_owned();

        RenderService::resolve_single_line_wikidot_iftags_fragments(
            &mut wikitext,
            &page_info,
        );

        assert!(wikitext.contains("[[div_  class=\"Dendo\"]]"));
        assert!(wikitext.contains("[[span class=\"visible \"]]body[[/span]]"));
        assert!(wikitext.contains("[[iftags +missing]]\nmultiline\n[[/iftags]]"));
        assert!(!wikitext.contains("display: flex"));
        assert!(!wikitext.contains("hidden"));
    }

    #[test]
    fn resolves_simple_multiline_wikidot_iftags_blocks_in_source() {
        let page_info = fallback_test_page_info("black-queen-hub", "Black Queen Hub");
        let page_info = ftml::data::PageInfo {
            tags: vec![Cow::Borrowed("theme")],
            ..page_info
        };
        let mut wikitext = concat!(
            "before\n",
            "[[iftags -component]]\n",
            "[[module css]]\n.a { color: red; }\n[[/module]]\n",
            "[[/iftags]]\n",
            "[[iftags +component]]\n",
            "documentation\n",
            "[[/iftags]]\n",
            "after\n",
        )
        .to_owned();

        RenderService::resolve_simple_wikidot_iftags_blocks(&mut wikitext, &page_info);

        assert!(wikitext.contains("[[module css]]"));
        assert!(wikitext.contains(".a { color: red; }"));
        assert!(!wikitext.contains("documentation"));
        assert!(!wikitext.contains("[[iftags"));
        assert!(!wikitext.contains("[[/iftags]]"));
    }

    #[test]
    fn resolves_active_compound_multiline_wikidot_iftags_blocks_in_source() {
        let page_info = fallback_test_page_info("black-highlighter-theme", "BHL");
        let page_info = ftml::data::PageInfo {
            tags: vec![Cow::Borrowed("theme")],
            ..page_info
        };
        let mut wikitext = concat!(
            "before\n",
            "[[iftags +theme -nobhl]]\n",
            "[[module css]]\n.a { color: red; }\n[[/module]]\n",
            "[[/iftags]]\n",
            "after\n",
        )
        .to_owned();

        RenderService::resolve_simple_wikidot_iftags_blocks(&mut wikitext, &page_info);

        assert!(wikitext.contains("[[module css]]"));
        assert!(wikitext.contains(".a { color: red; }"));
        assert!(!wikitext.contains("[[iftags"));
        assert!(!wikitext.contains("[[/iftags]]"));
    }

    #[test]
    fn removes_inactive_compound_multiline_wikidot_iftags_blocks_in_source() {
        let page_info = fallback_test_page_info("scp-5516", "SCP-5516");
        let mut wikitext = concat!(
            "before\n",
            "[[iftags +theme -nobhl]]\n",
            "[[module css]]\n.a { color: red; }\n[[/module]]\n",
            "[[/iftags]]\n",
            "after\n",
        )
        .to_owned();

        RenderService::resolve_simple_wikidot_iftags_blocks(&mut wikitext, &page_info);

        assert!(wikitext.contains("before\n"));
        assert!(wikitext.contains("after\n"));
        assert!(!wikitext.contains("[[module css]]"));
        assert!(!wikitext.contains(".a { color: red; }"));
        assert!(!wikitext.contains("[[iftags"));
        assert!(!wikitext.contains("[[/iftags]]"));
    }

    #[test]
    fn include_expansion_cleanup_removes_includes_in_inactive_wikidot_iftags_blocks() {
        let page_info = fallback_test_page_info("scp-5516", "SCP-5516");
        let mut wikitext = concat!(
            "before\n",
            "[[iftags +theme -nobhl]]\n",
            "[[include component:hidden]]\n",
            "[[/iftags]]\n",
            "[[include component:visible]]\n",
        )
        .to_owned();

        RenderService::prepare_wikidot_conditionals_for_include_expansion(
            &mut wikitext,
            &page_info,
        );

        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let mut includes = Vec::new();
        ftml::include(
            &wikitext,
            &settings,
            CollectingIncluder {
                includes: &mut includes,
            },
            include_error,
        )
        .expect("include collection should parse visible includes only");

        assert_eq!(includes.len(), 1);
        assert_eq!(
            includes[0].page_ref(),
            &PageRef::page_only("component:visible")
        );
        assert!(!wikitext.contains("component:hidden"));
    }

    #[test]
    fn leaves_nested_multiline_wikidot_iftags_blocks_for_later_handling() {
        let page_info = fallback_test_page_info("black-queen-hub", "Black Queen Hub");
        let mut wikitext = concat!(
            "[[iftags -component]]\n",
            "[[iftags +theme]]nested[[/iftags]]\n",
            "[[/iftags]]\n",
        )
        .to_owned();

        RenderService::resolve_simple_wikidot_iftags_blocks(&mut wikitext, &page_info);

        assert!(wikitext.contains("[[iftags -component]]"));
        assert!(wikitext.contains("[[iftags +theme]]nested[[/iftags]]"));
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
