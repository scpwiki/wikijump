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

use super::compat::CompatHtmlFragments;
use super::compat::color_and_inline_protection::*;
use super::compat::footnote_dom::restore_wikidot_footnote_list_dom;
use super::compat::issued_markers::restore_issued_html_text_markers;
use super::compat::preparation::{
    extract_css_modules, neutralize_authored_markers,
    protect_css_modules_before_first_list_pages,
};
use super::compat::text_fragments::{COMPAT_TEXT_MARKER_PREFIX, CompatTextFragments};
use super::compat::wikidot_inline_markers::{
    WikidotCompatInlineMarkerKind, next_wikidot_compat_inline_marker,
};
use super::compat::wikidot_link_protection::{
    ProtectedWikidotWikipediaLink, WikidotWikipediaLink, build_wikidot_wikipedia_link,
};
use super::compat::{WikidotCompatibilityFallbackOutput, scan_compat_code_blocks};
use super::diagnostics::{
    CorpusRenderDimension, CorpusRenderScope, CorpusRenderStage, CorpusRenderTrace,
    StageGuard,
};
use super::generator::COMPILED_GENERATOR;
use super::html_text::html_data_segments;
use super::iftags::{
    resolve_outermost_wikidot_iftags,
    resolve_outermost_wikidot_iftags_before_include_expansion,
    wikidot_tag_conditions_match,
};
use super::include_attachment_owners::{
    AttachmentOwner, AttachmentProvenanceRegistry, AttachmentVariableOwners,
    find_wikidot_directive_end, owned_url, parse_wikidot_include_argument,
    protect_forwarded_attachment_variables, qualify_included_relative_image_attachments,
    qualify_relative_image_variable_attachments, relative, semantic_attachment_value,
    split_wikidot_include_argument_segments, wikidot_include_segment_is_space,
};
use super::include_comment_branches::{
    remove_unresolved_include_comment_branches,
    remove_unresolved_include_comment_branches_source_local,
};
use super::include_variable_iftags::{
    resolve_include_variable_iftags, resolve_unbound_include_variable_iftags,
};
use super::list_pages::content_sections::{
    isolate_wikidot_content_section, wikidot_content_section,
};
use super::list_pages::scanner::{
    CountPagesCloseReachabilityIndex, find_list_pages_module_matches,
    first_list_pages_module_opening_candidate, has_count_pages_module_opening_candidate,
    has_list_pages_module_opening_candidate, list_pages_runtime_head_is_safe,
};
use super::list_pages::template::{
    LISTPAGES_VARIABLE_REGEX, ListPagesOutputShape, ListPagesTemplatePlan,
};
use super::list_pages::*;
use super::literal_regions::{
    ListPagesSourceProjection, LiteralRegionCursor, LiteralRegionIndex,
    WikidotNativeQuoteIndex,
};
use super::metacomponent::{
    MetacomponentSourceContext, select_metacomponent_documentation,
};
use super::native_list_context::NativeListSourceContext;
use super::percent_encoding::percent_encode_path_segment;
use super::prelude::*;
use super::runtime::{IncludeSourceCache, RenderRuntime};
use super::runtime_page_queries::{
    CountPagesRawScanCompletion, count_pages_raw_scan_completion,
    random_page_query_scan_limit, render_page_query_batch_limit,
    render_page_query_uses_single_scan,
};
use crate::hash::{TextHash, k12_hash};
use crate::models::page::{self, Entity as Page};
use crate::models::page_category::{self, Entity as PageCategory};
use crate::models::page_revision;
use crate::models::site::Model as SiteModel;
use crate::models::user::{self, Entity as UserTable};
use crate::models::wikidot_user::{self, Entity as WikidotUser};
use crate::services::page_query::{
    AuthorSelector, CategoriesSelector, ComparisonOperation,
    CountPagesExactCountEligibilityDiagnostics, CountPagesExactCountEligibilityInput,
    DataFormSelector, DateSelector, DateTimeResolution, FoundPageFields, FoundPageRow,
    FoundPages, IncludedCategories, ListPagesRenderDiagnosticsInput,
    MAX_PAGE_QUERY_SCORE_SELECTORS, OrderBySelector, OrderProperty, PageParentSelector,
    PageQuery, PageQueryResultMetadata, PageQueryScoreFilterCache, PageTypeSelector,
    PaginationSelector, RangeSelector, ScoreSelector, TagCondition,
    count_pages_exact_count_eligibility_diagnostics, list_pages_render_diagnostics,
    normalize_wikidot_author_name, parse_static_wikidot_data_form_values,
    static_wikidot_data_form_matches,
};
use crate::services::page_revision::GetPageRevision;
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::settings::{NavigationPageWikitext, SettingsService};
use crate::services::text_block::{
    MIME_HTML, TextBlock, TextBlockService, mime_for_language,
};
use crate::services::{
    CategoryService, PageRevisionService, PageService, SiteService, TextService,
};
use crate::types::{Action, PageId, Permission, Resource, TextBlockType};
use crate::utils::locale_for_ftml;
use ftml::data::PageRef;
use ftml::includes::{FetchedPage, IncludeRef};
use ftml::prelude::*;
use ftml::tree::{CodeBlock, VariableMap};
use regex::Regex;
use sea_orm::{ColumnTrait, EntityTrait, FromQueryResult, QueryFilter, Statement, Value};
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

#[derive(Debug)]
enum ListPagesBlockRenderResult {
    Expanded(IncludeExpansion),
    PreserveOriginal,
}

#[derive(Debug, Clone)]
enum CountPagesBlockRenderResult {
    Expanded(String),
    PreserveOriginal,
}

#[derive(Debug, FromQueryResult)]
struct CountPagesRequiredTagTotal {
    tag: String,
    total: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CountPagesRequiredTagBatchResult {
    Exact(usize),
    PreserveLiteral,
}

struct CountPagesRequiredTagSource<'a> {
    literal_regions: &'a LiteralRegionIndex,
    close_reachability: &'a CountPagesCloseReachabilityIndex,
    source_projection: Option<&'a ListPagesSourceProjection>,
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
    included_pages: Vec<PageRef>,
    wikidot_compat_html: CompatHtmlFragments,
    wikidot_compat_text: CompatTextFragments,
}

#[derive(Debug)]
struct OuterPreparedRenderWikitext {
    wikitext: String,
    included_pages: Vec<PageRef>,
    wikidot_css_modules: Vec<String>,
    wikidot_inline_html: Vec<ProtectedWikidotInlineHtml>,
    wikidot_color_spans: ProtectedWikidotColorSpans,
    wikidot_compat_html: CompatHtmlFragments,
    wikidot_compat_text: CompatTextFragments,
    native_list_wikipedia_links: Vec<WikidotWikipediaLink>,
    compatibility_fallback: bool,
    timings: CorpusReplayStageTimings,
}

#[derive(Debug)]
struct InnerPreparedRenderWikitext {
    wikitext: String,
    included_pages: Vec<PageRef>,
    wikidot_css_modules: Vec<String>,
    wikidot_inline_html: Vec<ProtectedWikidotInlineHtml>,
    wikidot_color_spans: ProtectedWikidotColorSpans,
    wikidot_compat_links: Vec<ProtectedWikidotCompatLink>,
    wikidot_wikipedia_links: Vec<ProtectedWikidotWikipediaLink>,
    wikidot_compat_html: CompatHtmlFragments,
    wikidot_compat_text: CompatTextFragments,
    native_list_wikipedia_links: Vec<WikidotWikipediaLink>,
    wikidot_embed_iframes: Vec<String>,
    timings: CorpusReplayStageTimings,
}
const MAX_INCLUDE_EXPANSION_DEPTH: usize = 8;
const MAX_INCLUDE_EXPANSION_TOTAL: usize = 256;
// The frozen EN corpus contains a page with 1,266 direct includes. Only the
// trusted corpus finalizer receives this higher ceiling; user-controlled
// render paths retain the ordinary limit above.
const MAX_CORPUS_INCLUDE_EXPANSION_TOTAL: usize = 4096;
const DEFAULT_LISTPAGES_RENDER_LIMIT: u64 = 100;
pub(super) const MAX_LISTPAGES_RENDER_LIMIT: u64 = 250;
// Keep runtime-owned content expansion within the ordinary ListPages page size. Explicitly larger content modules remain literal before revision loading and nested include expansion.
const MAX_LISTPAGES_CONTENT_ROWS_PER_RENDER: usize =
    DEFAULT_LISTPAGES_RENDER_LIMIT as usize;
// Content-backed ListPages modules can trigger permission filtering, revision loading, and nested include expansion. Three modules cover the common corpus shape while stopping dense author-page compositions before they exhaust the render budget.
const MAX_LISTPAGES_CONTENT_MODULES_PER_RENDER: usize = 3;
pub(super) const MAX_LISTPAGES_RENDER_OFFSET: u32 = 1_000;
pub(super) const MAX_LISTPAGES_RENDER_SCAN_ROWS: u32 = 5_000;
pub(super) const MAX_WIKIDOT_AJAX_MODULE_BODY_BYTES: usize = 65_536;
pub(super) const MAX_WIKIDOT_AJAX_MODULE_PARAMETERS: usize = 64;
pub(super) const MAX_WIKIDOT_AJAX_MODULE_PARAMETER_BYTES: usize = 4_096;
pub(super) const MAX_BACKLINKS_MODULE_ROWS: usize = 500;
const LONG_NATIVE_LIST_RENDER_MIN_ITEMS: usize = 8;
const MAX_NATIVE_LIST_COMPAT_DEPTH: usize = 64;
const MAX_FTML_COMPAT_PARSE_BYTES: usize = 768_000;
const MAX_FTML_COMPAT_DENSE_PARSE_SCORE: usize = 180_000;
const MAX_FTML_COMPAT_COLLAPSIBLE_BLOCKS: usize = 48;
const MIN_FTML_COMPAT_TABBED_FALLBACK_BYTES: usize = 64_000;
const MIN_FTML_COMPAT_TABBED_FALLBACK_MARKERS: usize = 12;
const MIN_FTML_COMPAT_TABBED_RENDER_BYTES: usize = 100_000;
const MIN_FTML_COMPAT_TABBED_MARKERS: usize = 10;
const MIN_DENSE_FTML_COMPAT_RENDER_TIMEOUT_SECS: u64 = 150;
const INCLUDE_VARIABLE_OPEN_SENTINEL: &str = "__WIKIJUMP_INCLUDE_VAR_OPEN__";
const INCLUDE_VARIABLE_CLOSE_SENTINEL: &str = "__WIKIJUMP_INCLUDE_VAR_CLOSE__";
const WIKIDOT_COMMENT_INCLUDE_SENTINEL: &str = "__WIKIJUMP_COMMENT_INCLUDE__";
pub(super) const WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTCOMPATHTML";
pub(super) const WIKIDOT_COMPAT_LINK_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTCOMPATLINK";
pub(super) const WIKIDOT_WIKIPEDIA_LINK_SENTINEL_PREFIX: &str =
    "WIKIJUMPWIKIDOTWIKIPEDIALINK";
pub(super) const WIKIDOT_WIKIPEDIA_LINK_SENTINEL_NONCE_LEN: usize = 32;
#[cfg(test)]
const WIKIDOT_COLOR_SPAN_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTCOLORSPAN";
pub(super) const WIKIDOT_INLINE_HTML_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTINLINEHTML";
const WIKIDOT_RATE_ANCHOR_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTRATEANCHOR";
pub(super) const WIKIDOT_LISTPAGES_LITERAL_ELLIPSIS_SENTINEL_PREFIX: &str =
    "WIKIJUMPWIKIDOTLISTPAGESELLIPSIS";
pub(super) const WIKIDOT_TABVIEW_SCRIPT: &str = "";
pub(super) const WIKIDOT_TABVIEW_INIT_SCRIPT: &str =
    r#"<script type="text/javascript"></script>"#;
const MAX_WIKIDOT_COMPAT_FALLBACK_TITLE_LINKS: usize = 128;

type WikidotCompatLinkTitleMap = BTreeMap<String, String>;

static INCLUDE_VARIABLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\$(?P<name>[a-zA-Z0-9_\-]+)\}").unwrap());
pub(super) static WIKIDOT_LISTPAGES_LITERAL_ELLIPSIS_SENTINEL_REGEX: LazyLock<Regex> =
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
pub(super) static RATE_MODULE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)\[\[module\s+Rate(?P<head>[^\]]*)\]\]").unwrap());
static WIKIDOT_RATE_ANCHOR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"\[\[a href="javascript:;" onclick="(?P<onclick>WIKIDOT\.modules\.PageRateWidgetModule\.listeners\.(?:rate\(event, -?1\)|cancelVote\(event\)))" title="(?P<title>[^"]*)"\]\](?P<label>[^\[]*)\[\[/a\]\]"#,
    )
    .unwrap()
});
pub(super) static TAGCLOUD_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+TagCloud(?P<head>[^\]]*)\]\]").unwrap()
});
pub(super) static BACKLINKS_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+Backlinks(?P<head>[^\]]*)\]\]").unwrap()
});
pub(super) static REGISTRY_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+(?P<name>Members|NewPage|Clone)(?P<head>[^\]]*)\]\]")
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
pub(super) static LISTPAGES_ARGUMENT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)(?P<key>[A-Za-z_][A-Za-z0-9_\-]*)\s*(?P<op>!?=)\s*(?:"(?P<double>[^"]*)"|'(?P<single>[^']*)'|(?P<bare>[^\s\]]+))"#)
        .unwrap()
});

pub(super) fn list_pages_runtime_regex_recognizes_entire_head(head: &str) -> bool {
    LISTPAGES_ARGUMENT_REGEX
        .replace_all(head, "")
        .trim()
        .is_empty()
}
static WIKIDOT_USER_INLINE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[\*user\s+(?P<name>[^\]]+)\]\]").unwrap());
pub(super) static WIKIDOT_ANCHOR_MARKER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[#\s+(?P<name>[^\]\n]+)\]\]").unwrap());
pub(super) static WIKIDOT_CURRENT_PAGE_LINK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[#\s+(?P<label>[^\]\n]+)\]").unwrap());
pub(super) static WIKIDOT_STAR_LOCAL_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
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
static WIKIDOT_IMAGE_BLOCK_INCLUDE_START_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r#"(?is)\[\[include(?:[ \t\r\n]+|\[!--.*?--\])+"#,
        r#"(?::(?P<site>[A-Za-z0-9_-]+):)?component:image-block"#,
        r#"(?P<after>(?:[ \t\r\n]+|\[!--.*?--\])+|\||\]\])"#,
    ))
    .unwrap()
});
static WIKIDOT_INCLUDE_OPEN_LINE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?im)^\[\[\s*(?P<keyword>include)(?P<after>\s+)").unwrap()
});
pub(super) static WIKIDOT_COMPAT_STYLE_BLOCK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| {
        Regex::new(r#"(?is)<style\b[^>]*\btype\s*=\s*["']text/css["'][^>]*>.*?</style>"#)
            .unwrap()
    });
static WIKIDOT_USERKARMA_BACKGROUND_STYLE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\sstyle="background-image:\s*url\(https?://www\.wikidot\.com/userkarma\.php\?u=[0-9]+\)""#)
        .unwrap()
});
pub(super) static WIKIDOT_RENDERED_MAILFORM_REGEX: LazyLock<Regex> = LazyLock::new(
    || {
        Regex::new(
        r#"(?is)<p>\[\[module\s+MailForm(?P<head>[^\]]*)\]\]</p>(?P<body>.*?)<p>\[\[/module\]\]</p>"#,
    )
    .unwrap()
    },
);
pub(super) static WIKIDOT_RENDERED_MAILFORM_FIELD_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?is)<ol>\s*<li>(?P<name>[^<]+)</li>"#).unwrap());
pub(super) static WIKIDOT_RENDERED_MAILFORM_DEFAULT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| {
        Regex::new(r#"(?is)<li>default:\s*(?P<default>[^<]*)</li>"#).unwrap()
    });
pub(super) static WIKIDOT_RENDERED_MAILFORM_MAX_LENGTH_REGEX: LazyLock<Regex> =
    LazyLock::new(|| {
        Regex::new(r#"(?is)<li>maxLength:\s*(?P<max>[0-9]+)</li>"#).unwrap()
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
pub(super) static WIKIDOT_EMBED_PARAGRAPH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)<p>\[\[embed\]\]<br/?>(.*?)<br/?>\[\[/embed\]\]</p>"#).unwrap()
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
        } = Box::pin(Self::render_inner(
            ctx,
            wikitext,
            page_info,
            settings,
            RenderInnerOptions {
                render_context: RenderContext::none(),
                max_include_expansions: MAX_INCLUDE_EXPANSION_TOTAL,
                trace: None,
                persist_compiled_text: true,
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
        let wikitext = build_wikidot_list_pages_module_source(module_body, parameters)
            .ok_or_raise(make_error)?;
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
        } = Box::pin(Self::render_inner(
            ctx,
            wikitext,
            &page_info,
            &settings,
            RenderInnerOptions {
                render_context: RenderContext::ajax_module(site_id),
                max_include_expansions: MAX_INCLUDE_EXPANSION_TOTAL,
                trace: None,
                persist_compiled_text: false,
            },
        ))
        .await
        .or_raise(make_error)?;

        let normalized_body = html_output.body.to_ascii_lowercase();
        if normalized_body.contains("[[module listpages")
            || normalized_body.contains("[[/module]]")
        {
            return Err(Error::new(
                "unsupported Wikidot ListPages module query",
                ErrorType::Render,
            )
            .into());
        }

        Ok(RenderOutput {
            html_output,
            errors,
            compiled_hash,
            compiled_at: now(),
            compiled_generator: COMPILED_GENERATOR.clone(),
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
            MAX_INCLUDE_EXPANSION_TOTAL,
            None,
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
            MAX_CORPUS_INCLUDE_EXPANSION_TOTAL,
            None,
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
            MAX_CORPUS_INCLUDE_EXPANSION_TOTAL,
            Some(trace),
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
        max_include_expansions: usize,
        trace: Option<&CorpusRenderTrace>,
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
            mut html_output,
            errors,
            compiled_hash: compiled_body_html_hash,
        } = Self::render_inner(
            ctx,
            wikitext,
            page_info,
            &page_settings,
            RenderInnerOptions {
                render_context: RenderContext::page(site_id, page_id),
                max_include_expansions,
                trace: trace.map(|trace| (trace, CorpusRenderScope::Body)),
                persist_compiled_text: true,
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
        let render_nav_page = |wikitext, scope| async move {
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
                            render_context: RenderContext::page_nav(site_id, page_id),
                            max_include_expansions,
                            trace: trace.map(|trace| (trace, scope)),
                            persist_compiled_text: true,
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
            serde_json::to_string(&html_output.styles).or_raise(make_error)?;
        let compiled_body_styles_hash = TextService::create(ctx, styles_json)
            .await
            .or_raise(make_error)?;

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
                current_page_id: Some(id.page_id),
                max_include_expansions: MAX_CORPUS_INCLUDE_EXPANSION_TOTAL,
                trace: None,
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
            wikitext,
            included_pages,
            wikidot_compat_html,
            wikidot_compat_text,
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
                wikidot_css_modules: outer.wikidot_css_modules,
                wikidot_compat_html: outer.wikidot_compat_html,
                wikidot_compat_text: outer.wikidot_compat_text,
                native_list_wikipedia_links: outer.native_list_wikipedia_links,
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
            wikidot_css_modules: inner.wikidot_css_modules,
            wikidot_compat_html: inner.wikidot_compat_html,
            wikidot_compat_text: inner.wikidot_compat_text,
            native_list_wikipedia_links: inner.native_list_wikipedia_links,
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
            current_page_id,
            max_include_expansions,
            trace,
        } = options;
        let make_error =
            || Error::new("failed to perform render operation", ErrorType::Render);
        let mut include_budget = IncludeExpansionBudget::new(max_include_expansions);
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
        Self::remove_preview_component_separator_markers(&mut wikitext);
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
                    budget: include_budget,
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
            remove_unresolved_include_comment_branches(&mut wikitext);
            Self::prepare_wikidot_conditionals_for_include_expansion(
                &mut wikitext,
                page_info,
                &mut wikidot_compat_text,
            );
            neutralize_authored_markers(&mut wikitext);
        }
        let mut wikidot_compat_html = CompatHtmlFragments::new(&wikitext);
        let IncludeExpansion {
            wikitext: expanded_wikitext,
            included_pages: list_pages_included_pages,
            ..
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
                    include_budget,
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
        wikitext = {
            let _stage = StageGuard::new(trace, CorpusRenderStage::CountPages);
            Self::expand_count_pages(
                ctx,
                wikitext,
                page_info,
                settings,
                current_site_id,
                current_page_id,
                &mut wikidot_compat_text,
            )
            .await
            .or_raise(make_error)?
        };
        wikitext = {
            let _stage = StageGuard::new(trace, CorpusRenderStage::TagCloud);
            Self::expand_tag_cloud_modules(
                ctx,
                wikitext,
                page_info,
                current_site_id,
                current_page_id,
            )
            .await
            .or_raise(make_error)?
        };
        wikitext = {
            let _stage = StageGuard::new(trace, CorpusRenderStage::Backlinks);
            Self::expand_backlinks_modules(
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
        {
            let _stage = StageGuard::new(trace, CorpusRenderStage::RegistryModules);
            wikitext = Self::expand_registry_modules_with_registry(
                wikitext,
                settings,
                &mut wikidot_compat_html,
            );
            wikitext = Self::expand_rate_modules_with_registry(
                wikitext,
                page_info,
                settings,
                &mut wikidot_compat_html,
            );
        }

        if let Some((trace, CorpusRenderScope::Body)) = trace {
            trace.set_dimension(CorpusRenderDimension::ExpandedBytes, wikitext.len());
            trace.set_dimension(
                CorpusRenderDimension::IncludedPages,
                included_pages.len(),
            );
        }

        Ok(ExpandedRenderWikitext {
            wikitext,
            included_pages,
            wikidot_compat_html,
            wikidot_compat_text,
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
            Self::protect_wikidot_marker_class_include_variables(
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
        expanded.wikitext =
            Self::escape_unrendered_wikidot_color_markers(expanded.wikitext, settings);
        let (rendered, native_list_wikipedia_links) =
            Self::render_long_native_list_runs_with_registry(
                expanded.wikitext,
                &mut expanded.wikidot_compat_html,
            );
        expanded.wikitext = rendered;
        let wikidot_css_modules = extract_css_modules(&mut expanded.wikitext, settings);
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
            wikidot_css_modules,
            wikidot_inline_html,
            wikidot_color_spans,
            wikidot_compat_html: expanded.wikidot_compat_html,
            wikidot_compat_text,
            native_list_wikipedia_links,
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
        ftml::preprocess(&mut outer.wikitext);
        outer.timings.preprocess_us = elapsed_micros(started);

        InnerPreparedRenderWikitext {
            wikitext: outer.wikitext,
            included_pages: outer.included_pages,
            wikidot_css_modules: outer.wikidot_css_modules,
            wikidot_inline_html: outer.wikidot_inline_html,
            wikidot_color_spans: outer.wikidot_color_spans,
            wikidot_compat_html: outer.wikidot_compat_html,
            wikidot_compat_text: outer.wikidot_compat_text,
            native_list_wikipedia_links: outer.native_list_wikipedia_links,
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
        options: RenderInnerOptions<'_>,
    ) -> Result<RenderInnerOutput> {
        let config = ctx.config();
        let RenderInnerOptions {
            render_context,
            max_include_expansions,
            trace,
            persist_compiled_text,
        } = options;
        let RenderContext {
            current_site_id,
            current_page_id,
            text_block_page_id,
        } = render_context;

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
                current_page_id,
                max_include_expansions,
                trace,
            },
        )
        .await?;
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
                wikidot_css_modules,
                wikidot_inline_html,
                wikidot_color_spans,
                wikidot_compat_html,
                wikidot_compat_text,
                native_list_wikipedia_links,
                compatibility_fallback: _,
                timings: _,
            } = outer;
            let mut backlinks = ftml::data::Backlinks::new();
            backlinks.included_pages.extend(included_pages);
            Self::record_wikidot_wikipedia_backlinks(
                &mut backlinks,
                &native_list_wikipedia_links,
            );
            let fallback_link_titles = {
                let _stage = StageGuard::new(trace, CorpusRenderStage::FallbackTitles);
                if let Some(site_id) = current_site_id {
                    Self::load_wikidot_compat_fallback_link_titles(
                        ctx, site_id, &wikitext,
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
                page_info.page.as_ref(),
                Some(&fallback_link_titles),
            );
            let mut wikidot_css_modules = wikidot_css_modules;
            Self::localize_wikidot_generated_styles(
                &mut wikidot_css_modules,
                current_site.as_ref(),
                config,
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
                    let html = Self::localize_wikidot_local_file_urls(
                        &html,
                        current_site.as_ref(),
                        config,
                    );
                    wikidot_compat_text.restore(&html)
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
                        contents: Cow::Owned(wikidot_compat_text.restore(
                            &Self::restore_wikidot_code_block_compatibility(
                                &wikidot_compat_html.restore_plain(contents),
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
                    let body = Self::localize_wikidot_local_file_urls(
                        &body,
                        current_site.as_ref(),
                        config,
                    );
                    wikidot_compat_text.restore(&body)
                },
                meta: Vec::new(),
                styles: wikidot_css_modules,
                backlinks,
            };
            drop(fallback_render_stage);
            if let Some((trace, CorpusRenderScope::Body)) = trace {
                trace.set_dimension(
                    CorpusRenderDimension::OutputBytes,
                    html_output.body.len(),
                );
            }
            let compiled_hash = Self::compiled_text_hash(
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
        let worker_trace = trace.map(|(trace, scope)| (trace.clone(), scope));
        let queued_at = worker_trace.as_ref().map(|_| Instant::now());

        let render_task = task::spawn_blocking(move || {
            let trace = worker_trace.as_ref().map(|(trace, scope)| (trace, *scope));
            if let (Some((trace, scope)), Some(queued_at)) = (trace, queued_at) {
                trace.record_elapsed(scope, CorpusRenderStage::WorkerQueue, queued_at);
            }
            let InnerPreparedRenderWikitext {
                wikitext,
                included_pages,
                wikidot_css_modules,
                wikidot_inline_html,
                wikidot_color_spans,
                wikidot_compat_links,
                wikidot_wikipedia_links,
                wikidot_compat_html,
                wikidot_compat_text,
                native_list_wikipedia_links,
                wikidot_embed_iframes,
                timings,
            } = Self::prepare_inner_render_wikitext(outer, &render_settings);
            if let Some((trace, scope)) = trace {
                trace.add_us(
                    scope,
                    CorpusRenderStage::InnerProtect,
                    timings.inner_protection_us,
                );
                trace.add_us(scope, CorpusRenderStage::Preprocess, timings.preprocess_us);
            }
            let tokens = {
                let _stage = StageGuard::new(trace, CorpusRenderStage::Tokenize);
                ftml::tokenize(&wikitext)
            };
            let result = {
                let _stage = StageGuard::new(trace, CorpusRenderStage::Parse);
                ftml::parse(&tokens, &render_page_info, &render_settings)
            };
            let (tree, errors) = result.into();
            let mut html_output = {
                let _stage = StageGuard::new(trace, CorpusRenderStage::HtmlRender);
                HtmlRender.render(&tree, &render_page_info, &render_settings)
            };
            // Deepwell's Wikidot compatibility scanner identifies actual CSS module
            // syntax before raw HTML fragments are restored. Keeping that typed
            // provenance separate ensures authored <style> HTML stays in the body.
            if !wikidot_css_modules.is_empty() {
                let mut styles = wikidot_css_modules;
                styles.append(&mut html_output.styles);
                Self::localize_wikidot_generated_styles(
                    &mut styles,
                    render_current_site.as_ref(),
                    &render_config,
                );
                html_output.styles = styles;
            } else {
                Self::localize_wikidot_generated_styles(
                    &mut html_output.styles,
                    render_current_site.as_ref(),
                    &render_config,
                );
            }
            let (html_block_texts, code_blocks) = {
                let _stage = StageGuard::new(trace, CorpusRenderStage::HtmlCompat);
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
                Self::record_wikidot_wikipedia_backlinks(
                    &mut html_output.backlinks,
                    &native_list_wikipedia_links,
                );
                html_output.body = Self::restore_wikidot_render_compatibility(
                    &html_output.body,
                    render_current_site.as_ref(),
                    &render_config,
                );
                html_output.body = wikidot_compat_text.restore(&html_output.body);
                html_output.backlinks.included_pages.extend(included_pages);
                let html_block_texts = tree
                    .html_blocks
                    .iter()
                    .map(|html| {
                        let html = wikidot_compat_html.restore(html);
                        let html = Self::localize_wikidot_local_file_urls(
                            &html,
                            render_current_site.as_ref(),
                            &render_config,
                        );
                        wikidot_compat_text.restore(&html)
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
                            contents: Cow::Owned(wikidot_compat_text.restore(
                                &Self::restore_wikidot_code_block_compatibility(
                                    &wikidot_compat_html.restore_plain(contents),
                                    render_current_site.as_ref(),
                                    &render_config,
                                ),
                            )),
                            language: language
                                .as_ref()
                                .map(|language| Cow::Owned(language.to_string())),
                            name: name.as_ref().map(|name| Cow::Owned(name.to_string())),
                        },
                    )
                    .collect();
                (html_block_texts, code_blocks)
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

        let compiled_hash = Self::compiled_text_hash(
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
            html_output,
            errors,
            compiled_hash,
        })
    }

    async fn compiled_text_hash(
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

    pub(super) fn remove_spurious_wikidot_email_classes(html: &str) -> String {
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

    pub(super) fn wikijump_tab_panel_is_hidden(panel_open_tag: &str) -> bool {
        panel_open_tag
            .trim_end_matches('>')
            .split_ascii_whitespace()
            .any(|attribute| attribute == "hidden" || attribute.starts_with("hidden="))
    }

    pub(super) fn remove_wikijump_footnote_ref_tooltips(html: &str) -> String {
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

    pub(super) fn restore_standalone_residual_wikidot_div_markers(html: &str) -> String {
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

    pub(super) fn push_replaced_standalone_wikidot_marker_line(
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

    pub(super) fn update_residual_div_raw_text_depth(
        mut depth: usize,
        line: &str,
    ) -> usize {
        let lower = line.to_ascii_lowercase();
        for tag in ["pre", "code", "textarea", "style", "script"] {
            depth += lower.matches(&format!("<{tag}")).count();
            depth = depth.saturating_sub(lower.matches(&format!("</{tag}>")).count());
        }
        depth
    }

    pub(super) fn decode_residual_wikidot_marker_quotes(marker: &str) -> String {
        marker
            .replace("&quot;", "\"")
            .replace("&#34;", "\"")
            .replace("&#x22;", "\"")
            .replace("&#X22;", "\"")
    }

    pub(super) fn residual_wikidot_alignment_open_replacement(
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

    pub(super) fn residual_wikidot_alignment_close(marker: &str) -> Option<&'static str> {
        match marker.to_ascii_lowercase().as_str() {
            "[[/=]]" => Some("center"),
            "[[/<]]" | "[[/&lt;]]" => Some("left"),
            "[[/>]]" | "[[/&gt;]]" => Some("right"),
            _ => None,
        }
    }

    pub(super) fn residual_wikidot_horizontal_rule_line(line: &str) -> bool {
        line.len() >= 4 && line.chars().all(|character| character == '-')
    }

    pub(super) fn residual_wikidot_content_section_line(line: &str) -> bool {
        line.len() >= 4 && line.chars().all(|character| character == '=')
    }

    pub(super) fn residual_wikidot_heading_replacement(
        line: &str,
    ) -> Option<(usize, &str)> {
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

    fn prepare_wikidot_conditionals_for_include_expansion(
        wikitext: &mut String,
        page_info: &ftml::data::PageInfo<'_>,
        preserved: &mut CompatTextFragments,
    ) {
        resolve_unbound_include_variable_iftags(wikitext);
        if wikitext.contains("[[#") {
            *wikitext = ftml::preproc::resolve_wikidot_parser_functions(wikitext);
        }
        Self::resolve_wikidot_iftags(wikitext, page_info, preserved);
    }

    fn normalize_wikidot_cross_closed_div_collapsibles(wikitext: &mut String) {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Block {
            Div,
            Collapsible,
        }

        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Marker {
            OpenDiv,
            CloseDiv,
            OpenCollapsible,
            CloseCollapsible,
        }

        fn marker_kind(marker: &str) -> Option<Marker> {
            let marker = marker.to_ascii_lowercase();
            if marker == "[[/div]]" {
                return Some(Marker::CloseDiv);
            }
            if marker == "[[/collapsible]]" {
                return Some(Marker::CloseCollapsible);
            }
            if marker.ends_with("]]")
                && (marker == "[[div]]"
                    || marker.starts_with("[[div ")
                    || marker == "[[div_]]"
                    || marker.starts_with("[[div_ "))
            {
                return Some(Marker::OpenDiv);
            }
            if marker.ends_with("]]")
                && (marker == "[[collapsible]]" || marker.starts_with("[[collapsible "))
            {
                return Some(Marker::OpenCollapsible);
            }
            None
        }

        let literal_regions = LiteralRegionIndex::new_wikidot_syntax(wikitext);
        let markers = Self::wikitext_line_ranges(wikitext)
            .into_iter()
            .filter_map(|(start, _, line)| {
                let marker = Self::trim_wikitext_line(line);
                let relative_start = line.find(marker)?;
                let marker_start = start + relative_start;
                if literal_regions.contains(marker_start) {
                    return None;
                }
                marker_kind(marker)
                    .map(|kind| (kind, marker_start..marker_start + marker.len()))
            })
            .collect::<Vec<_>>();

        let mut stack = Vec::new();
        let mut replacements = Vec::new();
        let mut index = 0usize;
        while index < markers.len() {
            let (kind, range) = &markers[index];
            match kind {
                Marker::OpenDiv => stack.push(Block::Div),
                Marker::OpenCollapsible => stack.push(Block::Collapsible),
                Marker::CloseDiv
                    if stack.ends_with(&[Block::Div, Block::Collapsible])
                        && markers.get(index + 1).is_some_and(|(next, _)| {
                            *next == Marker::CloseCollapsible
                        }) =>
                {
                    replacements.push((range.clone(), "[[/collapsible]]"));
                    replacements.push((markers[index + 1].1.clone(), "[[/div]]"));
                    stack.truncate(stack.len() - 2);
                    index += 1;
                }
                Marker::CloseDiv if stack.last() == Some(&Block::Div) => {
                    stack.pop();
                }
                Marker::CloseCollapsible if stack.last() == Some(&Block::Collapsible) => {
                    stack.pop();
                }
                Marker::CloseDiv | Marker::CloseCollapsible => {}
            }
            index += 1;
        }

        for (range, replacement) in replacements.into_iter().rev() {
            wikitext.replace_range(range, replacement);
        }
    }

    fn prepare_wikidot_conditionals_before_include_expansion(
        wikitext: &mut String,
        page_info: &ftml::data::PageInfo<'_>,
        preserved: &mut CompatTextFragments,
        include_depth: usize,
    ) {
        // Keep incomplete boundaries available for adjacent caller/include
        // source while still pruning self-contained inactive gates early.
        if include_depth == 0 {
            // Nested sources were already resolved against their include callsite immediately before recursion.
            resolve_unbound_include_variable_iftags(wikitext);
        }
        if wikitext.contains("[[#") {
            *wikitext = ftml::preproc::resolve_wikidot_parser_functions(wikitext);
        }
        resolve_outermost_wikidot_iftags_before_include_expansion(
            wikitext,
            &page_info.tags,
            preserved,
        );
    }

    fn resolve_wikidot_iftags(
        wikitext: &mut String,
        page_info: &ftml::data::PageInfo<'_>,
        preserved: &mut CompatTextFragments,
    ) {
        resolve_outermost_wikidot_iftags(wikitext, &page_info.tags, preserved);
    }

    pub(super) fn resolve_wikidot_parser_functions(value: &str) -> String {
        if !value.contains("[[#") {
            return value.to_owned();
        }
        // Frozen ListPages rows use zero for a missing vote count. Preserve the
        // evidenced operator-level result without maintaining another parser.
        ftml::preproc::resolve_wikidot_parser_functions_with_options(
            value,
            ftml::preproc::WikidotParserFunctionOptions {
                zero_operator_policy:
                    ftml::preproc::WikidotZeroOperatorPolicy::ReplaceOperationWithZero,
            },
        )
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

    fn protect_wikidot_marker_class_include_variables(
        wikitext: &mut String,
        fragments: &mut CompatTextFragments,
    ) {
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
                        fragments.push(captures.get(0).expect("full match").as_str())
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

    fn normalize_wikidot_multiline_page_links(wikitext: &mut String) {
        let source = wikitext.clone();
        let literal_regions = LiteralRegionIndex::new(&source);
        let mut normalized = String::with_capacity(source.len());
        let mut last = 0usize;
        let mut changed = false;

        for captures in WIKIDOT_MULTILINE_LABELED_LINK_REGEX.captures_iter(&source) {
            let Some(link_match) = captures.get(0) else {
                continue;
            };

            normalized.push_str(&source[last..link_match.start()]);
            last = link_match.end();

            if literal_regions.contains(link_match.start()) {
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

    pub(super) fn remove_wikijump_table_body_wrappers(html: &str) -> String {
        html.replace("<tbody>", "").replace("</tbody>", "")
    }

    pub(super) fn remove_wikidot_compat_style_blocks(html: &str) -> String {
        WIKIDOT_COMPAT_STYLE_BLOCK_REGEX
            .replace_all(html, "")
            .into_owned()
    }

    pub(super) fn push_wikidot_text_ellipsis_segment(
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

    pub(super) fn update_wikidot_ellipsis_literal_depth(
        tag: &str,
        literal_depth: &mut usize,
    ) {
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

    pub(super) fn remove_wikijump_underline_wrappers(html: &str) -> String {
        // FTML uses semantic <s> elements for paired Wikidot --text--
        // strikethrough. Those are visible formatting, not plain wrappers.
        html.replace("<u>", "").replace("</u>", "")
    }

    pub(super) fn remove_wikidot_userkarma_background_styles(html: &str) -> String {
        WIKIDOT_USERKARMA_BACKGROUND_STYLE_REGEX
            .replace_all(html, "")
            .into_owned()
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

    fn expand_wikidot_image_block_includes(
        wikitext: &mut String,
        page_info: &PageInfo<'_>,
        attachment_owner: Option<(&str, &str)>,
    ) -> Vec<PageRef> {
        Self::expand_wikidot_image_block_includes_with_provenance(
            wikitext,
            page_info,
            attachment_owner,
            None,
        )
    }

    fn expand_wikidot_image_block_includes_with_provenance(
        wikitext: &mut String,
        page_info: &PageInfo<'_>,
        attachment_owner: Option<(&str, &str)>,
        attachment_provenance: Option<&AttachmentProvenanceRegistry>,
    ) -> Vec<PageRef> {
        let source = wikitext.clone();
        let literal_regions = LiteralRegionIndex::new_wikidot_syntax(&source);
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
                    find_wikidot_directive_end(&source, match_end, source.len())
                else {
                    search_start = match_end;
                    continue;
                };
                (args_start, include_end)
            };

            search_start = include_end;

            if literal_regions.contains(include_start)
                || !Self::should_expand_wikidot_image_block_include(
                    captures.name("site").map(|site| site.as_str()),
                    page_info,
                )
            {
                continue;
            }

            let Some(args) = Self::parse_wikidot_include_arguments(
                &source[args_start..include_end - 2],
                attachment_provenance,
            ) else {
                continue;
            };
            let Some(name) = args.get("name") else {
                continue;
            };

            let caption = args
                .get("caption")
                .map_or("", |argument| argument.value.as_str());
            let width = args
                .get("width")
                .map_or("300px", |argument| argument.value.as_str());
            let align = args
                .get("align")
                .map_or("right", |argument| argument.value.as_str());
            let raw_link = args
                .get("link")
                .map_or("#", |argument| argument.value.as_str());
            let Some(semantic_name) = semantic_attachment_value(&name.value) else {
                continue;
            };
            if semantic_name.is_empty() {
                continue;
            }
            let image_source = match &name.attachment_owner {
                Some(owner) => {
                    if relative(semantic_name) {
                        owned_url(owner, semantic_name)
                    } else {
                        name.value.clone()
                    }
                }
                None => Self::wikidot_image_block_source(
                    semantic_name,
                    page_info,
                    attachment_owner,
                ),
            };
            let link = match args
                .get("link")
                .and_then(|argument| argument.attachment_owner.as_ref())
            {
                Some(owner) => {
                    let Some(semantic) = semantic_attachment_value(raw_link) else {
                        continue;
                    };
                    if relative(semantic) {
                        owned_url(owner, semantic)
                    } else {
                        raw_link.to_owned()
                    }
                }
                None => {
                    let Some(semantic) = semantic_attachment_value(raw_link) else {
                        continue;
                    };
                    if relative(semantic) {
                        owned_url(
                            &Self::wikidot_image_block_attachment_owner(
                                page_info,
                                attachment_owner,
                            ),
                            semantic,
                        )
                    } else {
                        raw_link.to_owned()
                    }
                }
            };
            let image_attribute = args
                .get("alt")
                .map(|argument| argument.value.as_str())
                .filter(|attribute| is_include_variable_name(attribute))
                .zip(args.get("alt-text"))
                .map(|(attribute, value)| {
                    format!(r#" {attribute}="{}""#, value.value.replace('"', "&quot;"),)
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

    pub(super) fn is_inside_wikidot_literal_region(source: &str, start: usize) -> bool {
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
        if !wikitext.contains("[!--") {
            return;
        }
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

    fn wikidot_image_block_source(
        name: &str,
        page_info: &PageInfo<'_>,
        attachment_owner: Option<(&str, &str)>,
    ) -> String {
        if name.starts_with("http://")
            || name.starts_with("https://")
            || name.starts_with('/')
        {
            return name.to_owned();
        }

        let owner =
            Self::wikidot_image_block_attachment_owner(page_info, attachment_owner);

        format!(
            "http://{}.wikidot.com/local--files/{}/{}",
            owner.site_slug,
            owner.page_slug,
            percent_encode_path_segment(name),
        )
    }

    fn wikidot_image_block_attachment_owner(
        page_info: &PageInfo<'_>,
        attachment_owner: Option<(&str, &str)>,
    ) -> AttachmentOwner {
        attachment_owner
            .map(|(site, page)| AttachmentOwner {
                site_slug: site.to_owned(),
                page_slug: page.to_owned(),
            })
            .unwrap_or_else(|| {
                let page_slug = match page_info.category.as_deref() {
                    Some(category) => format!("{category}:{}", page_info.page),
                    None => page_info.page.to_string(),
                };
                AttachmentOwner {
                    site_slug: page_info.site.to_string(),
                    page_slug,
                }
            })
    }

    fn parse_wikidot_include_arguments(
        args: &str,
        attachment_provenance: Option<&AttachmentProvenanceRegistry>,
    ) -> Option<BTreeMap<String, WikidotImageBlockArgument>> {
        let segments = split_wikidot_include_argument_segments(args)?;
        let mut arguments = BTreeMap::new();

        for segment in segments {
            if wikidot_include_segment_is_space(segment) {
                continue;
            }
            let argument = parse_wikidot_include_argument(segment)?;
            let (value, attachment_owner) = attachment_provenance
                .and_then(|registry| registry.decode(argument.value))
                .map_or_else(
                    || (argument.value.to_owned(), None),
                    |(value, owner)| (value.clone(), Some(owner.clone())),
                );
            let self_reference = value
                .strip_prefix("{$")
                .and_then(|value| value.strip_suffix('}'))
                == Some(argument.raw_key);
            if !self_reference {
                arguments
                    .entry(argument.raw_key.to_ascii_lowercase())
                    .or_insert(WikidotImageBlockArgument {
                        value,
                        attachment_owner,
                    });
            }
        }

        Some(arguments)
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

    async fn expand_includes(
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
            budget,
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
                expand_wikidot_image_blocks,
                max_total_includes: budget.maximum,
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
            Self::normalize_wikidot_ta_badge_multiline_includes(&mut wikitext);
            Self::prepare_wikidot_conditionals_before_include_expansion(
                &mut wikitext,
                expansion_context.page_info,
                compat_text,
                depth,
            );
            Self::mask_wikidot_comment_include_markers(&mut wikitext);
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
                Self::unmask_wikidot_comment_include_markers(&mut wikitext);
                protect_include_variables(&mut wikitext);
                return Ok(IncludeExpansion {
                    wikitext,
                    included_pages: image_block_included_pages,
                    expanded_include_count: 0,
                });
            }

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
                );
                qualify_included_relative_image_attachments(
                    &mut source.wikitext,
                    &source.site_slug,
                    &source.page_slug,
                );
                select_metacomponent_documentation(
                    &mut source.wikitext,
                    MetacomponentSourceContext::Included,
                );

                let attachment_owner = AttachmentOwner {
                    site_slug: source.site_slug.clone(),
                    page_slug: source.page_slug.clone(),
                };
                let expansion = Self::expand_includes_for_site(
                    ctx,
                    source.wikitext,
                    IncludeExpansionContext {
                        current_site_id: source.site_id,
                        current_site_slug: source.site_slug,
                        attachment_owner: Some(attachment_owner),
                        page_info: expansion_context.page_info,
                        settings: expansion_context.settings,
                        expand_wikidot_image_blocks: expansion_context
                            .expand_wikidot_image_blocks,
                        max_total_includes: expansion_context.max_total_includes,
                    },
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

    #[allow(clippy::too_many_arguments)]
    async fn expand_list_pages(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        compat_html: &mut CompatHtmlFragments,
        include_source_cache: &mut IncludeSourceCache,
        compat_text: &mut CompatTextFragments,
        options: ListPagesExpansionOptions,
    ) -> Result<IncludeExpansion> {
        let ListPagesExpansionOptions {
            current_site_id,
            current_page_id,
            mut include_budget,
        } = options;
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

        if !has_list_pages_module_opening_candidate(&wikitext) {
            return Ok(IncludeExpansion {
                wikitext,
                included_pages: Vec::new(),
                expanded_include_count: 0,
            });
        }

        let initial_remaining_include_expansions = include_budget.remaining;
        enum ListPagesBlockPlan {
            Static(String),
            PreserveOriginal,
            Render {
                arguments: ListPagesArguments,
                template: ListPagesTemplatePlan,
                batch_key: Option<ExactNameListPagesBatchKey>,
            },
        }
        struct ListPagesBlock {
            start: usize,
            end: usize,
            plan: ListPagesBlockPlan,
        }

        let current_category = Self::page_info_category_slug(page_info);
        let unsupported_plan = |module_source: &str, body: &str| {
            let replacement = unsupported_list_pages_replacement(module_source, body);
            if replacement == module_source {
                ListPagesBlockPlan::PreserveOriginal
            } else {
                ListPagesBlockPlan::Static(replacement)
            }
        };
        let blocks = find_list_pages_module_matches(&wikitext)
            .into_iter()
            .map(|module| {
                let head = module.head;
                let body = module.body;
                let plan = if !module.runtime_safe
                    || list_pages_has_unsupported_parent_selector(head)
                    || list_pages_has_unsupported_page_type_selector(head)
                {
                    ListPagesBlockPlan::PreserveOriginal
                } else if let Some(arguments) = parse_list_pages_arguments(head) {
                    if arguments.unsupported_author_filter
                        || arguments.unsupported_score_filter
                    {
                        ListPagesBlockPlan::PreserveOriginal
                    } else if let Some(template) = ListPagesTemplatePlan::compile(body) {
                        let batch_key = exact_name_list_pages_batch_key(
                            head,
                            &template,
                            &arguments,
                            current_category.as_ref(),
                        );
                        ListPagesBlockPlan::Render {
                            arguments,
                            template,
                            batch_key,
                        }
                    } else {
                        unsupported_plan(module.original, body)
                    }
                } else {
                    unsupported_plan(module.original, body)
                };
                ListPagesBlock {
                    start: module.start,
                    end: module.end,
                    plan,
                }
            })
            .collect::<Vec<_>>();

        let mut expanded = String::with_capacity(wikitext.len());
        let mut included_pages = Vec::new();
        let mut content_cache = ListPagesContentCache::default();
        let mut expansion_budget = ListPagesExpansionBudget::new();
        let mut permission_cache = BTreeMap::new();
        let mut score_filter_cache = PageQueryScoreFilterCache::default();
        let mut author_resolution_cache = BTreeMap::new();
        let mut cursor = 0;
        let mut blocks = blocks.into_iter().peekable();

        while let Some(block) = blocks.next() {
            let batch_key = match &block.plan {
                ListPagesBlockPlan::Render {
                    batch_key: Some(key),
                    ..
                } => Some(key.clone()),
                _ => None,
            };
            if let Some(batch_key) = batch_key {
                let mut batch = vec![block];
                while batch.len() < MAX_LISTPAGES_RENDER_LIMIT as usize
                    && blocks.peek().is_some_and(|next| {
                        matches!(
                            &next.plan,
                            ListPagesBlockPlan::Render {
                                batch_key: Some(key),
                                ..
                            } if key == &batch_key
                        )
                    })
                {
                    batch.push(blocks.next().unwrap());
                }

                let mut unique_slugs = BTreeSet::new();
                let mut fields = FoundPageFields::default();
                let mut display_requirements =
                    ListPagesBatchDisplayRequirements::default();
                for block in &batch {
                    let ListPagesBlockPlan::Render {
                        arguments,
                        template,
                        ..
                    } = &block.plan
                    else {
                        unreachable!();
                    };
                    unique_slugs.insert(arguments.slug.as_ref().unwrap().to_string());
                    union_found_page_fields(&mut fields, &template.fields());
                    display_requirements.include(template);
                }
                let slugs = unique_slugs
                    .iter()
                    .map(|slug| Cow::Borrowed(slug.as_str()))
                    .collect::<Vec<_>>();
                let prefetched = Self::load_exact_name_list_pages_batch(
                    ctx,
                    current_site_id,
                    current_page_id,
                    &batch_key,
                    &slugs,
                    fields,
                    &mut permission_cache,
                )
                .await?;
                let prefetched_displays = if let Some(prefetched) = prefetched.as_ref() {
                    let prefetched_rows =
                        prefetched.values().flatten().cloned().collect::<Vec<_>>();
                    Some(
                        Self::load_list_pages_batch_displays(
                            ctx,
                            &prefetched_rows,
                            display_requirements,
                        )
                        .await?,
                    )
                } else {
                    None
                };

                for block in batch {
                    expanded.push_str(&wikitext[cursor..block.start]);
                    let ListPagesBlockPlan::Render {
                        arguments,
                        template,
                        ..
                    } = block.plan
                    else {
                        unreachable!();
                    };
                    let slug = arguments.slug.as_ref().unwrap().to_string();
                    let prefetched_pages =
                        prefetched.as_ref().map(|prefetched| FoundPages {
                            pages: prefetched.get(&slug).cloned().unwrap_or_default(),
                        });
                    let rendered = Self::render_list_pages_block(
                        ctx,
                        ListPagesPageContext {
                            site_id: current_site_id,
                            page_id: current_page_id,
                        },
                        page_info,
                        settings,
                        arguments,
                        &template,
                        include_budget,
                        prefetched_pages,
                        prefetched_displays.as_ref(),
                        include_source_cache,
                        &mut content_cache,
                        &mut expansion_budget,
                        &mut permission_cache,
                        &mut score_filter_cache,
                        &mut author_resolution_cache,
                        compat_text,
                    )
                    .await?;
                    match rendered {
                        ListPagesBlockRenderResult::Expanded(IncludeExpansion {
                            wikitext: replacement,
                            included_pages: replacement_included_pages,
                            expanded_include_count: replacement_expanded_include_count,
                        }) => {
                            include_budget.consume(replacement_expanded_include_count);
                            expanded.push_str(&register_generated_list_pages_html(
                                replacement,
                                compat_html,
                            ));
                            included_pages.extend(replacement_included_pages);
                        }
                        ListPagesBlockRenderResult::PreserveOriginal => {
                            expanded.push_str(&compat_text.push_escaped_html_text(
                                &wikitext[block.start..block.end],
                            ));
                        }
                    }
                    cursor = block.end;
                }
                continue;
            }

            expanded.push_str(&wikitext[cursor..block.start]);
            match block.plan {
                ListPagesBlockPlan::Static(replacement) => {
                    expanded.push_str(&replacement);
                }
                ListPagesBlockPlan::PreserveOriginal => {
                    expanded.push_str(
                        &compat_text
                            .push_escaped_html_text(&wikitext[block.start..block.end]),
                    );
                }
                ListPagesBlockPlan::Render {
                    arguments,
                    template,
                    ..
                } => {
                    let rendered = Self::render_list_pages_block(
                        ctx,
                        ListPagesPageContext {
                            site_id: current_site_id,
                            page_id: current_page_id,
                        },
                        page_info,
                        settings,
                        arguments,
                        &template,
                        include_budget,
                        None,
                        None,
                        include_source_cache,
                        &mut content_cache,
                        &mut expansion_budget,
                        &mut permission_cache,
                        &mut score_filter_cache,
                        &mut author_resolution_cache,
                        compat_text,
                    )
                    .await?;
                    match rendered {
                        ListPagesBlockRenderResult::Expanded(IncludeExpansion {
                            wikitext: replacement,
                            included_pages: replacement_included_pages,
                            expanded_include_count: replacement_expanded_include_count,
                        }) => {
                            include_budget.consume(replacement_expanded_include_count);
                            expanded.push_str(&register_generated_list_pages_html(
                                replacement,
                                compat_html,
                            ));
                            included_pages.extend(replacement_included_pages);
                        }
                        ListPagesBlockRenderResult::PreserveOriginal => {
                            expanded.push_str(&compat_text.push_escaped_html_text(
                                &wikitext[block.start..block.end],
                            ));
                        }
                    }
                }
            }
            cursor = block.end;
        }

        expanded.push_str(&wikitext[cursor..]);
        Ok(IncludeExpansion {
            wikitext: expanded,
            included_pages,
            expanded_include_count: initial_remaining_include_expansions
                .saturating_sub(include_budget.remaining),
        })
    }

    async fn load_exact_name_list_pages_batch(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        current_page_id: i64,
        key: &ExactNameListPagesBatchKey,
        slugs: &[Cow<'_, str>],
        fields: FoundPageFields,
        permission_cache: &mut BTreeMap<(i64, Option<i64>), bool>,
    ) -> Result<Option<BTreeMap<String, Vec<FoundPageRow>>>> {
        let categories = key
            .categories
            .iter()
            .map(|category| Cow::Borrowed(category.as_str()))
            .collect::<Vec<_>>();
        let excluded_categories = key
            .excluded_categories
            .iter()
            .map(|category| Cow::Borrowed(category.as_str()))
            .collect::<Vec<_>>();
        let included_categories = if key.category_all {
            IncludedCategories::All
        } else {
            IncludedCategories::List(&categories)
        };
        // A single exact-name block can normally consume up to 250 rows. Reserve that same allowance for every distinct slug in the batch, while keeping the combined prefetch within the existing 5,000-row render scan cap.
        let batch_scan_target = slugs
            .len()
            .saturating_mul(MAX_LISTPAGES_RENDER_LIMIT as usize)
            .min(MAX_LISTPAGES_RENDER_SCAN_ROWS as usize);
        let query = PageQuery {
            current_page_id,
            current_site_id,
            queried_site_id: None,
            page_type: PageTypeSelector::Normal,
            categories: CategoriesSelector {
                included_categories,
                excluded_categories: &excluded_categories,
            },
            tags: TagCondition {
                any_present: &[],
                all_present: &[],
                none_present: &[],
            },
            page_parent: PageParentSelector::All,
            contains_outgoing_links: &[],
            creation_date: DateSelector::FromPresent {
                start: time::OffsetDateTime::UNIX_EPOCH,
            },
            update_date: DateSelector::FromPresent {
                start: time::OffsetDateTime::UNIX_EPOCH,
            },
            author: AuthorSelector::All,
            score: &[],
            votes: &[],
            offset: 0,
            range: RangeSelector::Current,
            name: None,
            slug: None,
            slugs,
            data_form_fields: &[],
            order: None,
            candidate_limit: None,
            pagination: PaginationSelector {
                limit: Some(batch_scan_target as u64),
                per_page: PaginationSelector::default().per_page,
                reversed: false,
            },
            variables: &[],
            fields,
        };
        let found = RenderRuntime::new(ctx)
            .find_viewable_list_pages_rows(
                query,
                batch_scan_target,
                permission_cache,
                None,
            )
            .await?;
        // Permission filtering and duplicate live slugs can make one globally ordered batch consume its scan window before another slug is reached. Returning None reuses the existing per-slug query path for every block in this batch.
        if found.view_permission_filtering_applied {
            return Ok(None);
        }
        let mut pages_by_slug = BTreeMap::<String, Vec<FoundPageRow>>::new();
        for page in found.pages.pages {
            if let Some(slug) = page.slug.clone() {
                pages_by_slug.entry(slug).or_default().push(page);
            }
        }
        if pages_by_slug.values().any(|pages| pages.len() > 1) {
            return Ok(None);
        }
        Ok(Some(pages_by_slug))
    }

    async fn load_list_pages_batch_displays(
        ctx: &ServiceContext<'_>,
        pages: &[FoundPageRow],
        requirements: ListPagesBatchDisplayRequirements,
    ) -> Result<ListPagesBatchDisplays> {
        let user_displays = if requirements.users {
            Self::load_wikidot_user_displays(ctx, pages).await?
        } else {
            BTreeMap::new()
        };
        let snapshot_displays = if requirements.snapshots {
            Self::load_list_pages_snapshot_displays(ctx, pages).await?
        } else {
            BTreeMap::new()
        };
        Ok(ListPagesBatchDisplays {
            user_displays,
            snapshot_displays,
        })
    }

    async fn expand_count_pages(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        current_page_id: Option<i64>,
        compat_text: &mut CompatTextFragments,
    ) -> Result<String> {
        let (Some(current_site_id), Some(current_page_id)) =
            (current_site_id, current_page_id)
        else {
            return Ok(wikitext);
        };

        if !settings.enable_page_syntax {
            return Ok(wikitext);
        }

        if !has_count_pages_module_opening_candidate(&wikitext) {
            return Ok(wikitext);
        }

        let close_reachability = CountPagesCloseReachabilityIndex::new(&wikitext);
        let literal_regions = LiteralRegionIndex::new_count_pages_syntax(&wikitext);
        let source_projection = ListPagesSourceProjection::new(&wikitext);
        let mut expanded = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        let mut permission_cache = BTreeMap::new();
        let page_context = ListPagesPageContext {
            site_id: current_site_id,
            page_id: current_page_id,
        };
        let batched_required_tag_totals = Self::load_count_pages_required_tag_totals(
            ctx,
            &wikitext,
            CountPagesRequiredTagSource {
                literal_regions: &literal_regions,
                close_reachability: &close_reachability,
                source_projection: source_projection.as_ref(),
            },
            page_info,
            page_context,
            &mut permission_cache,
        )
        .await?;
        let mut close_reachability = close_reachability.monotone_cursor();
        let mut replacement_cache =
            BTreeMap::<(String, String), CountPagesBlockRenderResult>::new();
        let mut literal_regions = literal_regions.monotone_cursor();
        let mut source_projection_ranges = source_projection
            .as_ref()
            .map(ListPagesSourceProjection::original_range_cursor);

        for captures in COUNTPAGES_MODULE_REGEX.captures_iter(&wikitext) {
            let mtch = captures.get(0).unwrap();
            expanded.push_str(&wikitext[cursor..mtch.start()]);
            if count_pages_capture_is_literal(&mut literal_regions, mtch.start()) {
                expanded.push_str(mtch.as_str());
                cursor = mtch.end();
                continue;
            }
            if !close_reachability
                .regex_capture_close_is_reachable(mtch.start()..mtch.end())
            {
                expanded.push_str(&compat_text.push_escaped_html_text(mtch.as_str()));
                cursor = mtch.end();
                continue;
            }
            let head_match = captures.name("head").unwrap();
            let head = head_match.as_str();
            let body = captures.name("body").unwrap().as_str();

            if source_projection_ranges.as_mut().is_some_and(|ranges| {
                !ranges
                    .range_is_unchanged(&wikitext, head_match.start()..head_match.end())
            }) {
                expanded.push_str(&compat_text.push_escaped_html_text(mtch.as_str()));
                cursor = mtch.end();
                continue;
            }

            if list_pages_has_unsupported_parent_selector(head)
                || list_pages_has_unsupported_page_type_selector(head)
            {
                expanded.push_str(&compat_text.push_escaped_html_text(mtch.as_str()));
                cursor = mtch.end();
                continue;
            }

            let Some(arguments) = parse_list_pages_arguments(head) else {
                expanded.push_str(&compat_text.push_escaped_html_text(mtch.as_str()));
                cursor = mtch.end();
                continue;
            };
            if count_pages_should_remain_literal(&arguments) {
                expanded.push_str(&compat_text.push_escaped_html_text(mtch.as_str()));
                cursor = mtch.end();
                continue;
            }

            let cache_key = (head.to_owned(), body.to_owned());
            if let Some(rendered) = replacement_cache.get(&cache_key) {
                match rendered {
                    CountPagesBlockRenderResult::Expanded(replacement) => {
                        expanded.push_str(replacement);
                    }
                    CountPagesBlockRenderResult::PreserveOriginal => {
                        expanded
                            .push_str(&compat_text.push_escaped_html_text(mtch.as_str()));
                    }
                }
                cursor = mtch.end();
                continue;
            }

            if let Some(tag) = count_pages_required_tag_batch_selector(&arguments)
                && let Some(result) = batched_required_tag_totals.get(&(
                    arguments.no_tags.iter().map(ToString::to_string).collect(),
                    tag.to_owned(),
                ))
            {
                match result {
                    CountPagesRequiredTagBatchResult::Exact(total) => {
                        let replacement = substitute_count_pages_variables(body, *total);
                        expanded.push_str(&replacement);
                        replacement_cache.insert(
                            cache_key,
                            CountPagesBlockRenderResult::Expanded(replacement),
                        );
                    }
                    CountPagesRequiredTagBatchResult::PreserveLiteral => {
                        expanded
                            .push_str(&compat_text.push_escaped_html_text(mtch.as_str()));
                        replacement_cache.insert(
                            cache_key,
                            CountPagesBlockRenderResult::PreserveOriginal,
                        );
                    }
                }
                cursor = mtch.end();
                continue;
            }

            let rendered = Self::render_count_pages_block(
                ctx,
                page_context,
                page_info,
                arguments,
                body,
                &mut permission_cache,
            )
            .await?;
            match &rendered {
                CountPagesBlockRenderResult::Expanded(replacement) => {
                    expanded.push_str(replacement);
                }
                CountPagesBlockRenderResult::PreserveOriginal => {
                    expanded.push_str(&compat_text.push_escaped_html_text(mtch.as_str()));
                }
            }
            replacement_cache.insert(cache_key, rendered);
            cursor = mtch.end();
        }
        let _close_reachability_advances = close_reachability.advances();

        expanded.push_str(&wikitext[cursor..]);
        Ok(expanded)
    }

    async fn load_count_pages_required_tag_totals(
        ctx: &ServiceContext<'_>,
        wikitext: &str,
        source: CountPagesRequiredTagSource<'_>,
        page_info: &PageInfo<'_>,
        page_context: ListPagesPageContext,
        permission_cache: &mut BTreeMap<(i64, Option<i64>), bool>,
    ) -> Result<BTreeMap<(Vec<String>, String), CountPagesRequiredTagBatchResult>> {
        let ListPagesPageContext {
            site_id: current_site_id,
            page_id: current_page_id,
        } = page_context;
        let CountPagesRequiredTagSource {
            literal_regions,
            close_reachability,
            source_projection,
        } = source;
        let mut tags_by_exclusions = BTreeMap::<Vec<String>, BTreeSet<String>>::new();
        let mut literal_regions = literal_regions.monotone_cursor();
        let mut close_reachability = close_reachability.monotone_cursor();
        let mut source_projection_ranges =
            source_projection.map(ListPagesSourceProjection::original_range_cursor);
        for captures in COUNTPAGES_MODULE_REGEX.captures_iter(wikitext) {
            let mtch = captures.get(0).unwrap();
            if count_pages_capture_is_literal(&mut literal_regions, mtch.start()) {
                continue;
            }
            if !close_reachability
                .regex_capture_close_is_reachable(mtch.start()..mtch.end())
            {
                continue;
            }
            let head_match = captures.name("head").unwrap();
            if source_projection_ranges.as_mut().is_some_and(|ranges| {
                !ranges.range_is_unchanged(wikitext, head_match.start()..head_match.end())
            }) {
                continue;
            }
            let head = head_match.as_str();
            if list_pages_has_unsupported_parent_selector(head)
                || list_pages_has_unsupported_page_type_selector(head)
            {
                continue;
            }
            let Some(arguments) = parse_list_pages_arguments(head) else {
                continue;
            };
            if count_pages_should_remain_literal(&arguments) {
                continue;
            }
            let Some(tag) = count_pages_required_tag_batch_selector(&arguments) else {
                continue;
            };
            tags_by_exclusions
                .entry(arguments.no_tags.iter().map(ToString::to_string).collect())
                .or_default()
                .insert(tag.to_owned());
        }
        tags_by_exclusions.retain(|_, required_tags| required_tags.len() >= 2);
        if tags_by_exclusions.is_empty() {
            return Ok(BTreeMap::new());
        }

        let category_slug = Self::page_info_category_slug(page_info);
        let category = CategoryService::get(
            ctx,
            current_site_id,
            Reference::Slug(Cow::Borrowed(category_slug.as_ref())),
        )
        .await?;
        let permission_key = (current_site_id, Some(category.category_id));
        let can_view = if let Some(can_view) = permission_cache.get(&permission_key) {
            Some(*can_view)
        } else {
            match PermissionService::check_user_can(
                ctx,
                &CheckPermissionContext {
                    user_id: None,
                    site_id: current_site_id,
                    page_reference: Some(Reference::Id(current_page_id)),
                },
                Permission {
                    resource_type: Resource::Page,
                    resource_category: Some(Reference::Id(category.category_id)),
                    action: Action::View,
                },
            )
            .await
            {
                Ok(can_view) => {
                    permission_cache.insert(permission_key, can_view);
                    Some(can_view)
                }
                Err(error) => {
                    warn!(
                        "Preserving batched CountPages modules after an inconclusive view permission check: {error}"
                    );
                    None
                }
            }
        };

        let Some(can_view) = can_view else {
            return Ok(tags_by_exclusions
                .into_iter()
                .flat_map(|(excluded_tags, required_tags)| {
                    required_tags.into_iter().map(move |tag| {
                        (
                            (excluded_tags.clone(), tag),
                            CountPagesRequiredTagBatchResult::PreserveLiteral,
                        )
                    })
                })
                .collect());
        };
        if !can_view {
            return Ok(tags_by_exclusions
                .into_iter()
                .flat_map(|(excluded_tags, required_tags)| {
                    required_tags.into_iter().map(move |tag| {
                        (
                            (excluded_tags.clone(), tag),
                            CountPagesRequiredTagBatchResult::Exact(0),
                        )
                    })
                })
                .collect());
        }

        let mut totals = BTreeMap::new();
        for (excluded_tags, required_tags) in tags_by_exclusions {
            let mut values = Vec::new();
            let required_values = required_tags
                .iter()
                .enumerate()
                .map(|(index, tag)| {
                    values.push(Value::from(tag.clone()));
                    format!("(${}::TEXT, {})", values.len(), index)
                })
                .collect::<Vec<_>>()
                .join(", ");
            values.push(Value::from(current_site_id));
            let site_parameter = values.len();
            values.push(Value::from(category.category_id));
            let category_parameter = values.len();
            let exclusion_predicates = excluded_tags
                .iter()
                .map(|tag| {
                    values.push(Value::from(tag.clone()));
                    format!("AND NOT (revision.tags @> ARRAY[${}::TEXT])", values.len())
                })
                .collect::<Vec<_>>()
                .join(" ");
            let sql = format!(
                "WITH requested(tag, ordinal) AS (VALUES {required_values}) \
                 SELECT requested.tag, COUNT(matched.page_id)::BIGINT AS total \
                 FROM requested \
                 LEFT JOIN LATERAL ( \
                   SELECT page.page_id \
                   FROM page_revision revision \
                   JOIN page ON page.latest_revision_id = revision.revision_id \
                   WHERE revision.tags @> ARRAY[requested.tag]::TEXT[] \
                     AND page.site_id = ${site_parameter} \
                     AND page.page_category_id = ${category_parameter} \
                     AND page.deleted_at IS NULL \
                     AND regexp_replace(page.slug, '^.*:', '') NOT LIKE '\\_%' ESCAPE '\\' \
                     {exclusion_predicates} \
                   LIMIT {MAX_LISTPAGES_RENDER_SCAN_ROWS} \
                 ) matched ON TRUE \
                 GROUP BY requested.tag, requested.ordinal \
                 ORDER BY requested.ordinal"
            );
            let txn = ctx.transaction();
            let statement =
                Statement::from_sql_and_values(txn.get_database_backend(), sql, values);
            let rows = CountPagesRequiredTagTotal::find_by_statement(statement)
                .all(txn)
                .await
                .or_raise(|| {
                    Error::new(
                        "failed to batch CountPages required-tag totals",
                        ErrorType::Render,
                    )
                })?;
            for row in rows {
                totals.insert(
                    (excluded_tags.clone(), row.tag),
                    count_pages_required_tag_batch_result(row.total, Some(can_view)),
                );
            }
        }

        Ok(totals)
    }

    #[cfg(test)]
    fn protect_wikidot_css_modules_before_first_list_pages(
        wikitext: &mut String,
        settings: &WikitextSettings,
    ) -> Option<CompatTextFragments> {
        protect_css_modules_before_first_list_pages(wikitext, settings)
    }

    #[cfg(test)]
    fn extract_wikidot_css_modules(
        wikitext: &mut String,
        settings: &WikitextSettings,
    ) -> Vec<String> {
        extract_css_modules(wikitext, settings)
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

        let mut titles = WikidotCompatLinkTitleMap::new();
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

    pub(super) fn wikidot_residual_div_attributes(marker: &str) -> Option<String> {
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

    pub(super) fn page_info_full_slug(page_info: &PageInfo<'_>) -> String {
        let page = page_info.page.as_ref();
        match Self::page_info_category_slug(page_info).as_ref() {
            "_default" => page.to_owned(),
            category => format!("{category}:{page}"),
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn render_list_pages_block(
        ctx: &ServiceContext<'_>,
        page_context: ListPagesPageContext,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        arguments: ListPagesArguments,
        template: &ListPagesTemplatePlan,
        mut include_budget: IncludeExpansionBudget,
        mut prefetched_pages: Option<FoundPages>,
        prefetched_displays: Option<&ListPagesBatchDisplays>,
        include_source_cache: &mut IncludeSourceCache,
        content_cache: &mut ListPagesContentCache,
        expansion_budget: &mut ListPagesExpansionBudget,
        permission_cache: &mut BTreeMap<(i64, Option<i64>), bool>,
        score_filter_cache: &mut PageQueryScoreFilterCache,
        author_resolution_cache: &mut BTreeMap<
            ListPagesAuthorCacheKey,
            ResolvedListPagesAuthors,
        >,
        compat_text: &mut CompatTextFragments,
    ) -> Result<ListPagesBlockRenderResult> {
        let ListPagesPageContext {
            site_id: current_site_id,
            page_id: current_page_id,
        } = page_context;
        let ajax_module_response = current_page_id == 0;
        let initial_remaining_include_expansions = include_budget.remaining;
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
            author_filter_present,
            order,
            limit,
            count_pages_explicit_limit: _,
            count_pages_per_page,
            offset,
            exclude_current_page,
            page_type,
            page_parent,
            creation_date,
            update_date,
            score,
            slug,
            name_pattern,
            data_form_fields,
            prepend_line,
            separate,
            wrapper,
            unsupported_author_filter: _,
            unsupported_score_filter: _,
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
        let query_limit = list_pages_row_scan_target(
            requested_limit,
            limit,
            count_pages_per_page,
            offset,
            exclude_current_page,
        );
        let wants_content = template.uses_content();
        if wants_content
            && render_page_query_uses_single_scan(order)
            && query_limit > expansion_budget.remaining_content_rows() as u64
        {
            // Avoid a broad random scan when its scan target exceeds the remaining deterministic content-expansion budget.
            return Ok(ListPagesBlockRenderResult::PreserveOriginal);
        }
        if wants_content
            && query_limit > 0
            && !expansion_budget.try_start_content_module()
        {
            return Ok(ListPagesBlockRenderResult::PreserveOriginal);
        }
        let included_categories = if category_all {
            IncludedCategories::All
        } else {
            IncludedCategories::List(&categories)
        };

        let wants_created_by = template.uses_created_by();
        let wants_created_at = template.uses_created_at();
        let wants_updated_by = template.uses_updated_by();
        let wants_updated_at = template.uses_updated_at();
        let wants_rating_votes = template.uses_rating_votes();
        let resolved_authors = Self::resolve_list_pages_authors_cached(
            ctx,
            current_site_id,
            current_page_id,
            &authors,
            author_filter_present,
            author_resolution_cache,
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
            creation_date,
            update_date,
            author: resolved_authors.as_selector(),
            score: &score,
            votes: &[],
            offset: 0,
            range: RangeSelector::Current,
            name: name_pattern,
            slug,
            slugs: &[],
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
            fields: template.fields(),
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
        } else if let Some(pages) = prefetched_pages.take() {
            pages
        } else {
            let query_target =
                if wants_content && !render_page_query_uses_single_scan(order) {
                    list_pages_content_query_target(
                        query_limit,
                        requested_limit,
                        expansion_budget.remaining_content_rows(),
                        offset,
                        exclude_current_page,
                        count_pages_per_page.is_some(),
                    )
                } else {
                    query_limit
                };
            let found = RenderRuntime::new(ctx)
                .find_viewable_list_pages_rows(
                    query,
                    query_target.min(usize::MAX as u64) as usize,
                    permission_cache,
                    Some(score_filter_cache),
                )
                .await?;
            if page_query_cap_requires_original_module(&found.metadata) {
                return Ok(ListPagesBlockRenderResult::PreserveOriginal);
            }
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
        let body = template.body();
        if wants_content && !expansion_budget.can_expand_content_rows(total) {
            return Ok(ListPagesBlockRenderResult::PreserveOriginal);
        }
        let wants_data_form_values = template.uses_data_form();
        if wants_content || wants_data_form_values {
            let mut missing_by_site = BTreeMap::<i64, Vec<i64>>::new();
            for page in &pages {
                let cache_key = (page.site_id, page.page_id);
                if !content_cache.wikitext.contains_key(&cache_key) {
                    missing_by_site
                        .entry(page.site_id)
                        .or_default()
                        .push(page.page_id);
                }
            }
            for (site_id, page_ids) in missing_by_site {
                let loaded = PageRevisionService::get_wikitext_optional_batch(
                    ctx, site_id, &page_ids,
                )
                .await?;
                content_cache.wikitext.extend(
                    loaded
                        .into_iter()
                        .map(|(page_id, wikitext)| ((site_id, page_id), wikitext)),
                );
            }
        }
        let category_ids = pages
            .iter()
            .filter_map(|page| page.page_category_id)
            .collect::<BTreeSet<_>>();
        let category_slugs = if category_ids.is_empty() {
            BTreeMap::new()
        } else {
            PageCategory::find()
                .filter(page_category::Column::CategoryId.is_in(category_ids))
                .all(ctx.transaction())
                .await
                .or_raise(|| {
                    Error::new(
                        "failed to load ListPages page categories",
                        ErrorType::Render,
                    )
                })?
                .into_iter()
                .map(|category| (category.category_id, category.slug))
                .collect::<BTreeMap<_, _>>()
        };
        let loaded_user_displays =
            if (wants_created_by || wants_updated_by) && prefetched_displays.is_none() {
                Some(Self::load_wikidot_user_displays(ctx, &pages).await?)
            } else {
                None
            };
        let empty_user_displays = BTreeMap::new();
        let user_displays = prefetched_displays
            .map(|displays| &displays.user_displays)
            .or(loaded_user_displays.as_ref())
            .unwrap_or(&empty_user_displays);
        let wants_comments = template.uses_comments();
        let wants_commented_by = template.uses_commented_by();
        let wants_commented_at = template.uses_commented_at();
        let wants_snapshot_displays = wants_created_by
            || wants_updated_by
            || wants_created_at
            || wants_updated_at
            || wants_comments
            || wants_commented_by
            || wants_commented_at
            || wants_rating_votes;
        let loaded_snapshot_displays =
            if wants_snapshot_displays && prefetched_displays.is_none() {
                Some(Self::load_list_pages_snapshot_displays(ctx, &pages).await?)
            } else {
                None
            };
        let empty_snapshot_displays = BTreeMap::new();
        let snapshot_displays = prefetched_displays
            .map(|displays| &displays.snapshot_displays)
            .or(loaded_snapshot_displays.as_ref())
            .unwrap_or(&empty_snapshot_displays);
        let mut output = String::new();
        if wrapper {
            output.push_str("[[div class=\"list-pages-box\"]]\n");
        }
        let mut included_pages = Vec::new();
        if let Some(prepend_line) = prepend_line {
            output.push_str(&prepend_line);
            output.push('\n');
        }

        let render_generated_html =
            template.output_shape() == ListPagesOutputShape::TableRows;
        for (index, page) in pages.iter().enumerate() {
            if separate {
                output.push_str("[[div class=\"list-pages-item\"]]\n");
            }
            let cache_key = (page.site_id, page.page_id);
            let page_wikitext = if wants_content || wants_data_form_values {
                content_cache
                    .wikitext
                    .get(&cache_key)
                    .cloned()
                    .unwrap_or_default()
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
            let isolated_content_section =
                if wants_content && template.content_sections().len() == 1 {
                    template
                        .content_sections()
                        .iter()
                        .next()
                        .copied()
                        .flatten()
                        .and_then(|section| {
                            page_wikitext.as_deref().and_then(|wikitext| {
                                isolate_wikidot_content_section(wikitext, section)
                                    .map(|content| (section, content))
                            })
                        })
                } else {
                    None
                };
            let expanded_page_content = if wants_content {
                match page_wikitext.as_deref() {
                    Some(wikitext) => {
                        if page.site_id != current_site_id {
                            return Err(Error::new(
                                format!(
                                    "ListPages content row page ID {} belongs to site ID {}, not current site ID {}",
                                    page.page_id, page.site_id, current_site_id,
                                ),
                                ErrorType::Render,
                            )
                            .into());
                        }
                        let source_attachment_page_slug = page.slug.as_deref().ok_or_else(
                            || {
                                Error::new(
                                    format!(
                                        "ListPages content row for page ID {} is missing its attachment-owner slug",
                                        page.page_id,
                                    ),
                                    ErrorType::Render,
                                )
                            },
                        )?;
                        let source_attachment_owner = AttachmentOwner {
                            site_slug: page_info.site.to_string(),
                            page_slug: source_attachment_page_slug.to_owned(),
                        };
                        let expansion = Self::expand_includes(
                            ctx,
                            isolated_content_section
                                .as_ref()
                                .map(|(_, content)| content.as_str())
                                .unwrap_or(wikitext)
                                .to_owned(),
                            page_info,
                            page_info.site.as_ref(),
                            settings,
                            IncludeExpansionOptions {
                                current_site_id: Some(page.site_id),
                                source_attachment_owner: Some(source_attachment_owner),
                                source_cache: include_source_cache,
                                compat_text,
                                expand_wikidot_image_blocks: false,
                                budget: include_budget,
                            },
                        )
                        .await?;
                        include_budget.consume(expansion.expanded_include_count);
                        included_pages.extend(expansion.included_pages);
                        Some(expansion.wikitext)
                    }
                    None => None,
                }
            } else {
                None
            };
            let expanded_content = expanded_page_content
                .as_deref()
                .map(|expanded| {
                    if let Some((section, _)) = isolated_content_section.as_ref() {
                        BTreeMap::from([(
                            Some(*section),
                            expanded.trim_matches('\n').to_owned(),
                        )])
                    } else {
                        template
                            .content_sections()
                            .iter()
                            .copied()
                            .map(|section| {
                                (section, wikidot_content_section(expanded, section))
                            })
                            .collect::<BTreeMap<_, _>>()
                    }
                })
                .unwrap_or_default();
            let substitution_context = ListPagesSubstitutionContext {
                rendered_limit: requested_limit as usize,
                ajax_module_response,
                category: page
                    .page_category_id
                    .and_then(|category_id| category_slugs.get(&category_id))
                    .map(String::as_str)
                    .unwrap_or_default(),
                user_displays,
                snapshot_displays,
                page_wikitext: None,
                expanded_content: Some(&expanded_content),
                data_form_values: &data_form_values,
                render_generated_html,
            };
            let mut body = if template.uses_only_rating() {
                substitute_list_pages_rating_only(body, page)
            } else {
                let mut generated_fragments = CompatHtmlFragments::new(body);
                let body = substitute_list_pages_variables_with_fragments(
                    body,
                    page,
                    index + 1,
                    total,
                    &substitution_context,
                    &mut generated_fragments,
                );
                generated_fragments.restore(&body)
            };
            neutralize_authored_markers(&mut body);
            if let Some(table) = render_list_pages_table_rows(&body) {
                output.push_str(&table);
            } else {
                output.push_str(&render_list_pages_numbered_rows(&body));
            }
            if separate {
                output.push_str("\n[[/div]]\n");
            } else {
                output.push('\n');
            }
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

        if wrapper {
            output.push_str("[[/div]]");
        }
        if wants_content {
            expansion_budget.consume_content_rows(total);
        }
        Ok(ListPagesBlockRenderResult::Expanded(IncludeExpansion {
            wikitext: output,
            included_pages,
            expanded_include_count: initial_remaining_include_expansions
                .saturating_sub(include_budget.remaining),
        }))
    }

    async fn render_count_pages_block(
        ctx: &ServiceContext<'_>,
        page_context: ListPagesPageContext,
        page_info: &PageInfo<'_>,
        arguments: ListPagesArguments,
        body: &str,
        permission_cache: &mut BTreeMap<(i64, Option<i64>), bool>,
    ) -> Result<CountPagesBlockRenderResult> {
        let ListPagesPageContext {
            site_id: current_site_id,
            page_id: current_page_id,
        } = page_context;
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
            author_filter_present,
            order,
            limit,
            count_pages_explicit_limit,
            count_pages_per_page: _,
            offset,
            exclude_current_page,
            page_type,
            page_parent,
            creation_date,
            update_date,
            score,
            slug,
            name_pattern,
            prepend_line: _,
            data_form_fields,
            unsupported_author_filter: _,
            unsupported_score_filter: _,
            unsupported_count_pages_filter: _,
            separate: _,
            wrapper: _,
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
        let resolved_authors = Self::resolve_list_pages_authors(
            ctx,
            current_site_id,
            current_page_id,
            &authors,
            author_filter_present,
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
            creation_date,
            update_date,
            author: resolved_authors.as_selector(),
            score: &score,
            votes: &[],
            offset: 0,
            range: RangeSelector::Current,
            name: name_pattern,
            slug,
            slugs: &[],
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
        let mut raw_scan_completion = CountPagesRawScanCompletion::Complete;
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
            let found = RenderRuntime::new(ctx)
                .find_viewable_count_pages_rows(query, target_count, permission_cache)
                .await?;
            count_pages_metadata = Some((
                found.metadata.clone(),
                found.view_permission_filtering_applied,
            ));
            raw_scan_completion = found.raw_scan_completion;
            found.pages
        };
        if let Some((metadata, view_permission_filtering_applied)) = count_pages_metadata
        {
            let preserve_original = page_query_cap_requires_original_module(&metadata);
            let diagnostics = count_pages_exact_count_render_diagnostics(
                metadata,
                view_permission_filtering_applied,
                exclude_current_page,
                offset > 0,
                count_pages_explicit_limit,
                count_pages_query_limit,
            );
            debug!("CountPages exact count eligibility diagnostics: {diagnostics:?}");
            if preserve_original {
                return Ok(CountPagesBlockRenderResult::PreserveOriginal);
            }
        }
        if count_pages_scan_requires_preservation(
            raw_scan_completion,
            pages.pages.len(),
            count_pages_query_limit.min(usize::MAX as u64) as usize,
        ) {
            return Ok(CountPagesBlockRenderResult::PreserveOriginal);
        }
        let pages = pages
            .pages
            .into_iter()
            .filter(|page| !exclude_current_page || page.page_id != current_page_id)
            .skip(offset as usize);
        let total = match count_pages_explicit_limit {
            Some(limit) => pages.take(limit.min(usize::MAX as u64) as usize).count(),
            None => {
                let Some(total) =
                    count_pages_unbounded_total(raw_scan_completion, pages.count())
                else {
                    return Ok(CountPagesBlockRenderResult::PreserveOriginal);
                };
                total
            }
        };

        Ok(CountPagesBlockRenderResult::Expanded(
            substitute_count_pages_variables(body, total),
        ))
    }

    async fn current_page_list_pages_row(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        current_page_id: i64,
        page_info: &PageInfo<'_>,
        fields: &FoundPageFields,
    ) -> Result<FoundPages> {
        if let Some(row) = current_page_info_list_pages_row(
            current_site_id,
            current_page_id,
            page_info,
            fields,
        ) {
            return Ok(FoundPages { pages: vec![row] });
        }

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
                        GetPageRevision {
                            site_id: current_site_id,
                            page_id: current_page_id,
                            revision_number: 0,
                        },
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

    async fn resolve_list_pages_authors_cached(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        current_page_id: i64,
        author_names: &[Cow<'static, str>],
        author_filter_present: bool,
        cache: &mut BTreeMap<ListPagesAuthorCacheKey, ResolvedListPagesAuthors>,
    ) -> Result<ResolvedListPagesAuthors> {
        let key = list_pages_author_cache_key(author_names, author_filter_present);
        if let Some(resolved) = cache.get(&key) {
            return Ok(resolved.clone());
        }
        let resolved = Self::resolve_list_pages_authors(
            ctx,
            current_site_id,
            current_page_id,
            author_names,
            author_filter_present,
        )
        .await?;
        cache.insert(key, resolved.clone());
        Ok(resolved)
    }

    async fn resolve_list_pages_authors(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        current_page_id: i64,
        author_names: &[Cow<'static, str>],
        author_filter_present: bool,
    ) -> Result<ResolvedListPagesAuthors> {
        if !author_filter_present {
            return Ok(ResolvedListPagesAuthors::All);
        }

        let mut snapshot_names = BTreeSet::new();
        let mut user_ids = BTreeSet::new();
        let mut include_current_page_author = false;
        for author in author_names {
            if author.as_ref() == "=" {
                include_current_page_author = true;
            } else {
                let author = normalize_wikidot_author_name(author);
                if !author.is_empty() {
                    snapshot_names.insert(author);
                }
            }
        }

        if include_current_page_author {
            let current_page_author = Self::load_current_page_author_source(
                ctx,
                current_site_id,
                current_page_id,
            )
            .await?;
            match current_page_author {
                Some(CurrentPageAuthorSource {
                    snapshot_present: true,
                    created_by_name: Some(created_by_name),
                    ..
                }) => {
                    let created_by_name = normalize_wikidot_author_name(&created_by_name);
                    if !created_by_name.is_empty() {
                        snapshot_names.insert(created_by_name);
                    }
                }
                Some(CurrentPageAuthorSource {
                    snapshot_present: true,
                    created_by_name: None,
                    ..
                })
                | Some(CurrentPageAuthorSource {
                    from_wikidot: true,
                    snapshot_present: false,
                    ..
                })
                | None => {}
                Some(CurrentPageAuthorSource {
                    from_wikidot: false,
                    snapshot_present: false,
                    ..
                }) => {
                    if let Some(revision) = PageRevisionService::get_earliest_optional(
                        ctx,
                        current_site_id,
                        current_page_id,
                    )
                    .await?
                    {
                        user_ids.insert(revision.user_id);
                    }
                }
            }
        }

        user_ids.extend(Self::load_wikidot_author_ids(ctx, &snapshot_names).await?);
        if user_ids.is_empty() && snapshot_names.is_empty() {
            Ok(ResolvedListPagesAuthors::None)
        } else {
            Ok(ResolvedListPagesAuthors::Any {
                user_ids: user_ids.into_iter().collect(),
                wikidot_snapshot_names: snapshot_names
                    .into_iter()
                    .map(Cow::Owned)
                    .collect(),
            })
        }
    }

    async fn load_wikidot_author_ids(
        ctx: &ServiceContext<'_>,
        wanted: &BTreeSet<String>,
    ) -> Result<Vec<i64>> {
        if wanted.is_empty() {
            return Ok(Vec::new());
        }

        let make_error = || {
            Error::new(
                "failed to load Wikidot author IDs for ListPages render",
                ErrorType::Render,
            )
        };
        let users = WikidotUser::find()
            .all(ctx.transaction())
            .await
            .or_raise(make_error)?;

        let author_ids = users
            .into_iter()
            .filter(|user| {
                user.name.as_ref().is_some_and(|name| {
                    wanted.contains(&normalize_wikidot_author_name(name))
                }) || user.slug.as_ref().is_some_and(|slug| {
                    wanted.contains(&normalize_wikidot_author_name(slug))
                })
            })
            .map(|user| i64::from(user.user_id))
            .collect::<Vec<_>>();

        Ok(author_ids)
    }

    async fn load_current_page_author_source(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        current_page_id: i64,
    ) -> Result<Option<CurrentPageAuthorSource>> {
        let make_error = || {
            Error::new(
                "failed to load current page author provenance for ListPages render",
                ErrorType::Render,
            )
        };
        let txn = ctx.transaction();
        let statement = Statement::from_sql_and_values(
            txn.get_database_backend(),
            "SELECT page.from_wikidot, snapshot.page_id IS NOT NULL AS snapshot_present, snapshot.created_by_name \
             FROM page \
             LEFT JOIN wikidot_page_snapshot snapshot ON snapshot.page_id = page.page_id \
             WHERE page.site_id = $1 AND page.page_id = $2 AND page.deleted_at IS NULL",
            [Value::from(current_site_id), Value::from(current_page_id)],
        );

        CurrentPageAuthorSource::find_by_statement(statement)
            .one(txn)
            .await
            .or_raise(make_error)
    }
}

fn register_generated_list_pages_html(
    value: String,
    compat_html: &mut CompatHtmlFragments,
) -> String {
    if !value.contains("data-wikijump-compat-") {
        return value;
    }
    let literal_regions = LiteralRegionIndex::new(&value);
    GENERATED_LISTPAGES_HTML_REGEX
        .replace_all(&value, |captures: &regex::Captures<'_>| {
            let full_match = captures.get(0).expect("compat fragment capture exists");
            if literal_regions.contains(full_match.start()) {
                return full_match.as_str().to_owned();
            }

            let html = compat_html.restore(full_match.as_str());
            compat_html.push_html(html)
        })
        .into_owned()
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

fn render_native_bullet_list_with_wikipedia_links(
    lines: &[&str],
    wikipedia_links: &mut Vec<WikidotWikipediaLink>,
) -> String {
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
        let depth = raw_depth
            .saturating_sub(base_depth)
            .min(MAX_NATIVE_LIST_COMPAT_DEPTH);
        let has_children = items.get(index + 1).is_some_and(|(next_depth, _)| {
            next_depth
                .saturating_sub(base_depth)
                .min(MAX_NATIVE_LIST_COMPAT_DEPTH)
                > depth
        });

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
        output.push_str(&render_native_list_item_content(
            content,
            has_children,
            wikipedia_links,
        ));
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

fn render_native_list_item_content(
    content: &str,
    has_children: bool,
    wikipedia_links: &mut Vec<WikidotWikipediaLink>,
) -> String {
    let rendered =
        render_native_list_inline_html_with_wikipedia_links(content, wikipedia_links);
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

    while let Some(offset) = html[cursor..].find('<') {
        cursor += offset;

        if html[cursor..].starts_with("<ul") {
            depth += 1;
            cursor += "<ul".len();
            continue;
        }

        if html[cursor..].starts_with("</ul>") {
            if depth == 0 {
                return None;
            }

            depth -= 1;
            cursor += "</ul>".len();
            if depth == 0 {
                return Some(cursor);
            }
            continue;
        }

        cursor += '<'.len_utf8();
    }

    None
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

pub(super) fn native_numbered_list_content(line: &str) -> Option<&str> {
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

pub(super) fn render_native_list_inline_html(value: &str) -> String {
    render_native_list_inline_html_with_titles(value, None)
}

fn render_native_list_inline_html_with_wikipedia_links(
    value: &str,
    wikipedia_links: &mut Vec<WikidotWikipediaLink>,
) -> String {
    render_native_list_inline_html_with_titles_and_wikipedia_links(
        value,
        None,
        Some(wikipedia_links),
    )
}

fn render_native_list_inline_html_with_titles(
    value: &str,
    link_titles: Option<&WikidotCompatLinkTitleMap>,
) -> String {
    render_native_list_inline_html_with_titles_and_wikipedia_links(
        value,
        link_titles,
        None,
    )
}

fn render_native_list_inline_html_with_titles_and_wikipedia_links(
    value: &str,
    link_titles: Option<&WikidotCompatLinkTitleMap>,
    mut wikipedia_links: Option<&mut Vec<WikidotWikipediaLink>>,
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
            let link = build_wikidot_wikipedia_link(
                &captures["target"],
                captures.name("label").map(|matched| matched.as_str()),
            );
            let anchor = link.anchor.clone();
            if let Some(links) = wikipedia_links.as_deref_mut() {
                links.push(link);
            }
            anchor
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

pub(super) fn render_native_list_inline_wikidot_strong(value: &str) -> String {
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

pub(super) fn render_native_list_inline_wikidot_underlines(value: &str) -> String {
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
    render_native_list_inline_wikidot_spans_at_depth(value, 0)
}

const MAX_NATIVE_LIST_WIKIDOT_SPAN_NESTING: usize = 64;

fn render_native_list_inline_wikidot_spans_at_depth(value: &str, depth: usize) -> String {
    if depth >= MAX_NATIVE_LIST_WIKIDOT_SPAN_NESTING {
        return escape_list_pages_html_text(value);
    }

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

        let Some(open_tag) = wikidot_inline_span_marker_open(marker) else {
            output.push_str(&escape_list_pages_html_text(marker));
            rest = after_marker;
            continue;
        };

        let Some(close_start) = find_matching_wikidot_span_close(after_marker) else {
            output.push_str(&escape_list_pages_html_text(marker_start));
            return output;
        };

        output.push_str(&open_tag);
        output.push_str(&render_native_list_inline_wikidot_spans_at_depth(
            &after_marker[..close_start],
            depth + 1,
        ));
        output.push_str("</span>");
        rest = &after_marker[close_start + "[[/span]]".len()..];
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

pub(super) fn wikidot_inline_span_marker_open(marker: &str) -> Option<String> {
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

pub(super) fn escape_list_pages_html_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(super) fn escape_list_pages_html_attr(value: &str) -> String {
    escape_list_pages_html_text(value).replace('"', "&quot;")
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

pub(super) fn render_read_only_rate_module(
    score: ftml::data::ScoreValue,
    language: &str,
) -> String {
    let score = format_score_value(score);
    let labels = wikidot_rate_module_labels(language);

    format!(
        concat!(
            "<div class=\"page-rate-widget-box\">",
            "<span class=\"rate-points\">{}",
            "<span class=\"number prw54353\">{}</span>",
            "</span>",
            "<span class=\"rateup btn btn-default\">",
            "<a href=\"javascript:;\" onclick=\"WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, 1)\" title=\"{}\">+</a>",
            "</span>",
            "<span class=\"ratedown btn btn-default\">",
            "<a href=\"javascript:;\" onclick=\"WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, -1)\" title=\"{}\">–</a>",
            "</span>",
            "<span class=\"cancel btn btn-default\">",
            "<a href=\"javascript:;\" onclick=\"WIKIDOT.modules.PageRateWidgetModule.listeners.cancelVote(event)\" title=\"{}\">x</a>",
            "</span>",
            "</div>"
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

pub(super) fn wikidot_module_argument<'a>(head: &'a str, name: &str) -> Option<&'a str> {
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

pub(super) fn render_backlinks_module_box(pages: &[BacklinksModulePage]) -> String {
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

pub(super) fn render_members_module_placeholder(group: &str) -> String {
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

pub(super) fn render_new_page_module(head: &str) -> String {
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

pub(super) fn render_clone_module(head: &str) -> String {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RenderContext {
    current_site_id: Option<i64>,
    current_page_id: Option<i64>,
    text_block_page_id: Option<i64>,
}

#[derive(Clone, Copy, Debug)]
struct RenderInnerOptions<'a> {
    render_context: RenderContext,
    max_include_expansions: usize,
    trace: Option<(&'a CorpusRenderTrace, CorpusRenderScope)>,
    persist_compiled_text: bool,
}

#[derive(Clone, Copy, Debug)]
struct RenderExpansionOptions<'a> {
    current_site_id: Option<i64>,
    current_page_id: Option<i64>,
    max_include_expansions: usize,
    trace: Option<(&'a CorpusRenderTrace, CorpusRenderScope)>,
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

    fn ajax_module(site_id: i64) -> Self {
        Self {
            current_site_id: Some(site_id),
            current_page_id: Some(0),
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

#[derive(Debug)]
struct IncludeExpansionOptions<'a> {
    current_site_id: Option<i64>,
    /// Canonical attachment owner for source text originating on a page other
    /// than the page currently being rendered, such as a ListPages content row.
    source_attachment_owner: Option<AttachmentOwner>,
    source_cache: &'a mut IncludeSourceCache,
    compat_text: &'a mut CompatTextFragments,
    expand_wikidot_image_blocks: bool,
    budget: IncludeExpansionBudget,
}

#[derive(Clone, Copy, Debug)]
struct IncludeExpansionBudget {
    maximum: usize,
    remaining: usize,
}

impl IncludeExpansionBudget {
    fn new(maximum: usize) -> Self {
        Self {
            maximum,
            remaining: maximum,
        }
    }

    fn consume(&mut self, count: usize) {
        self.remaining = self.remaining.saturating_sub(count);
    }
}

#[derive(Clone, Copy, Debug)]
struct ListPagesPageContext {
    site_id: i64,
    page_id: i64,
}

#[derive(Debug, Default)]
struct ListPagesContentCache {
    wikitext: BTreeMap<(i64, i64), Option<String>>,
}

#[derive(Debug)]
struct ListPagesExpansionBudget {
    remaining_content_modules: usize,
    remaining_content_rows: usize,
}

impl ListPagesExpansionBudget {
    fn new() -> Self {
        Self {
            remaining_content_modules: MAX_LISTPAGES_CONTENT_MODULES_PER_RENDER,
            remaining_content_rows: MAX_LISTPAGES_CONTENT_ROWS_PER_RENDER,
        }
    }

    fn try_start_content_module(&mut self) -> bool {
        if self.remaining_content_modules == 0 {
            return false;
        }
        self.remaining_content_modules -= 1;
        true
    }

    fn remaining_content_rows(&self) -> usize {
        self.remaining_content_rows
    }

    fn can_expand_content_rows(&self, rows: usize) -> bool {
        rows <= self.remaining_content_rows
    }

    fn consume_content_rows(&mut self, rows: usize) {
        debug_assert!(self.can_expand_content_rows(rows));
        self.remaining_content_rows = self.remaining_content_rows.saturating_sub(rows);
    }
}

#[derive(Clone, Copy, Debug)]
struct ListPagesExpansionOptions {
    current_site_id: Option<i64>,
    current_page_id: Option<i64>,
    include_budget: IncludeExpansionBudget,
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
    expand_wikidot_image_blocks: bool,
    max_total_includes: usize,
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
        let mut expanded = String::with_capacity(content.len());
        let mut previous_end = 0;
        let mut matched = false;
        let mut changed = false;

        for capture in INCLUDE_VARIABLE_REGEX.captures_iter(content) {
            let mtch = capture.get(0).unwrap();
            let name = &capture["name"];

            if let Some(value) = include
                .variables()
                .get(name)
                .map(|value| Cow::Borrowed(trim_include_variable_value(value)))
                .or_else(|| default_include_variable_value(name).map(Cow::Owned))
            {
                expanded.push_str(&content[previous_end..mtch.start()]);
                expanded.push_str(&value);
                previous_end = mtch.end();
                matched = true;
                changed |= value != mtch.as_str();
            }
        }

        if !matched {
            break;
        }

        expanded.push_str(&content[previous_end..]);
        *content = expanded;
        if !changed {
            break;
        }
    }
}

fn apply_include_variables_before_resolving_iftags(
    content: &mut String,
    include: &IncludeRef<'_>,
    page_info: &PageInfo<'_>,
) {
    apply_include_variables(content, include);
    resolve_include_variable_iftags(content, include.variables(), page_info);
}

fn prepare_include_source_variables_and_comment_branches(
    content: &mut String,
    include: &IncludeRef<'_>,
    page_info: &PageInfo<'_>,
    compat_text: &mut CompatTextFragments,
) {
    apply_include_variables_before_resolving_iftags(content, include, page_info);
    // A comment branch is local to the included source once its callsite
    // variables are bound. Remove inactive branches before recursively
    // preparing that source so their conditional and include delimiters
    // cannot pair with delimiters from sibling expansions.
    remove_unresolved_include_comment_branches_source_local(content, compat_text);
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
    if !content.contains("{$") {
        return;
    }
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

fn has_include_opening_candidate(content: &str) -> bool {
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

fn unprotect_include_variables(content: &mut String) {
    *content = content
        .replace(INCLUDE_VARIABLE_OPEN_SENTINEL, "{$")
        .replace(INCLUDE_VARIABLE_CLOSE_SENTINEL, "}");
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

fn direct_wdfiles_local_file_url(host: &str, path: &str) -> Option<String> {
    if !path.starts_with("/local--files/") {
        return None;
    }

    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();
    let site_slug = host.strip_suffix(".wikidot.com")?;
    if site_slug.is_empty()
        || !site_slug
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
        || !site_slug
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        || !site_slug
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_alphanumeric)
    {
        return None;
    }

    Some(format!("https://{site_slug}.wdfiles.com{path}"))
}

#[allow(dead_code)]
fn public_url_port_suffix(port: Option<u16>) -> String {
    port.map(|port| format!(":{port}")).unwrap_or_default()
}

pub(super) fn rendered_wikidot_mailform_attribute(
    head: &str,
    name: &str,
) -> Option<String> {
    let prefix = format!("{name}=&quot;");
    let start = head.find(&prefix)? + prefix.len();
    let rest = &head[start..];
    let end = rest.find("&quot;")?;
    Some(rest[..end].to_owned())
}

#[cfg(test)]
mod tests;
