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
use crate::models::page::{self, Entity as Page};
use crate::models::page_revision;
use crate::models::site::Model as SiteModel;
use crate::models::user::{self, Entity as UserTable};
use crate::models::wikidot_user::{self, Entity as WikidotUser};
use crate::services::page_query::{
    CategoriesSelector, DataFormSelector, DateSelector, FoundPageFields, FoundPageRow,
    FoundPages, IncludedCategories, OrderBySelector, OrderProperty, PageParentSelector,
    PageQuery, PageTypeSelector, PaginationSelector, RangeSelector, TagCondition,
    parse_static_wikidot_data_form_values, static_wikidot_data_form_matches,
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
use std::time::Duration;
use tokio::task;
use tokio::time::timeout;

#[derive(Debug)]
pub struct RenderService;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProtectedWikidotWikipediaLink {
    anchor: String,
    href: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProtectedWikidotCompatLink {
    anchor: String,
}

const MAX_INCLUDE_EXPANSION_DEPTH: usize = 8;
const MAX_INCLUDE_EXPANSION_TOTAL: usize = 256;
const DEFAULT_LISTPAGES_RENDER_LIMIT: u64 = 100;
const MAX_LISTPAGES_RENDER_LIMIT: u64 = 250;
const MAX_LISTPAGES_RENDER_OFFSET: u32 = 1_000;
const MAX_LISTPAGES_RENDER_SCAN_ROWS: u32 = 5_000;
const LONG_NATIVE_LIST_RENDER_MIN_ITEMS: usize = 8;
const MAX_FTML_COMPAT_PARSE_BYTES: usize = 768_000;
const MAX_FTML_COMPAT_DENSE_PARSE_SCORE: usize = 180_000;
const MAX_FTML_COMPAT_COLLAPSIBLE_BLOCKS: usize = 48;
const MIN_FTML_COMPAT_TABBED_RENDER_BYTES: usize = 100_000;
const MIN_FTML_COMPAT_TABBED_MARKERS: usize = 10;
const MIN_DENSE_FTML_COMPAT_RENDER_TIMEOUT_SECS: u64 = 150;
const LISTPAGES_NO_MATCH_AUTHOR_ID: &str = "-9223372036854775808";
const INCLUDE_VARIABLE_OPEN_SENTINEL: &str = "__WIKIJUMP_INCLUDE_VAR_OPEN__";
const INCLUDE_VARIABLE_CLOSE_SENTINEL: &str = "__WIKIJUMP_INCLUDE_VAR_CLOSE__";
const WIKIDOT_EMBED_IFRAME_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTEMBEDIFRAME";
const WIKIDOT_CSS_MODULE_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTCSSMODULE";
const WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTCOMPATHTML";
const WIKIDOT_COMPAT_LINK_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTCOMPATLINK";
const WIKIDOT_WIKIPEDIA_LINK_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTWIKIPEDIALINK";
const WIKIDOT_LOCAL_INTERWIKI_BASE: &str = "/-/wikidot-interwiki";
const WIKIDOT_TABVIEW_SCRIPT: &str = r#"<script src="http://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--javascript/yahooui/tabview-min.js" type="text/javascript"></script>"#;
const WIKIDOT_TABVIEW_INIT_SCRIPT: &str = r#"<script type="text/javascript"></script>"#;

static INCLUDE_VARIABLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\$(?P<name>[a-zA-Z0-9_\-]+)\}").unwrap());
static LISTPAGES_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)\[\[module\s+ListPages(?P<head>(?:"[^"]*"|'[^']*'|[^\]])*)\]\](?P<body>.*?)\[\[/module\]\]"#,
    )
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
static TAGCLOUD_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+TagCloud(?P<head>[^\]]*)\]\]").unwrap()
});
static MEMBERS_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+Members(?P<head>[^\]]*)\]\]").unwrap()
});
static NEWPAGE_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+NewPage(?P<head>[^\]]*)\]\]").unwrap()
});
static CSS_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+css[^\]]*\]\](?P<body>.*?)\[\[/module\]\]").unwrap()
});
static GENERATED_COMPAT_TABLE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)<table class="wiki-content-table">.*?</table>|<div id="ml-[0-9]+" data-wikijump-compat-members="1"[^>]*>.*?</div>|<div class="pager" data-wikijump-compat-pager="1"[^>]*>.*?</div>|<form class="new-page-box" data-wikijump-compat-new-page="1"[^>]*>.*?</form>"#,
    )
    .unwrap()
});
static WIKIDOT_RESIDUAL_DIV_PARAGRAPH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)<p>\s*(?:(?P<open>\[\[div[^\]]*\]\])|(?P<close>\[\[/div\]\]))\s*</p>"#,
    )
    .unwrap()
});
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
static WIKIDOT_USER_INLINE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[\*user\s+(?P<name>[^\]]+)\]\]").unwrap());
static WIKIDOT_CURRENT_PAGE_LINK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[#\s+(?P<label>[^\]\n]+)\]").unwrap());
static WIKIDOT_STAR_LOCAL_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\*/(?P<target>[^\s\]\n]+)\s+(?P<label>[^\]\n]+)\]").unwrap()
});
static WIKIDOT_LABELED_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[\[\[(?P<target>[^\]|\n]+)\|(?P<label>[^\]\n]*)\]\]\]").unwrap()
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
    Regex::new(r"##(?P<color>[A-Za-z0-9_-]+)\|(?P<body>.*?)##").unwrap()
});
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
    Regex::new(
        r#"(?is)\[\[iftags\s+(?P<sign>[+-])(?P<tag>[A-Za-z0-9_-]+)\]\](?P<body>.*?)\[\[/iftags\]\]"#,
    )
    .unwrap()
});
static WIKIDOT_SIMPLE_IF_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)\[\[#if\s+(?P<cond>1|0|true|false)\s*\|\s*(?P<when_true>.*?)\s*\|\s*(?P<when_false>.*?)\s*\]\]"#)
        .unwrap()
});
static WIKIDOT_IMAGE_BLOCK_INCLUDE_START_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)\[\[include\s+(?::(?P<site>[A-Za-z0-9_-]+):)?component:image-block(?P<after>\s|\||\]\])"#,
    )
    .unwrap()
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
        let mut included_pages = if settings.enable_page_syntax {
            Self::expand_wikidot_image_block_includes(&mut wikitext, page_info)
        } else {
            Vec::new()
        };

        let IncludeExpansion {
            wikitext: expanded_wikitext,
            included_pages: expanded_included_pages,
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
        included_pages.extend(expanded_included_pages);
        Self::remove_wikidot_metacomponent_documentation(&mut wikitext);
        Self::remove_unresolved_include_comment_branches(&mut wikitext);
        Self::remove_unresolved_variable_iftags_blocks(&mut wikitext);
        Self::resolve_single_line_wikidot_iftags_fragments(&mut wikitext, page_info);
        Self::resolve_simple_wikidot_iftags_blocks(&mut wikitext, page_info);
        let IncludeExpansion {
            wikitext: expanded_wikitext,
            included_pages: list_pages_included_pages,
        } = Self::expand_list_pages(
            ctx,
            wikitext,
            page_info,
            settings,
            current_site_id,
            current_page_id,
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
        wikitext = Self::expand_members_modules(wikitext, settings);
        wikitext = Self::expand_new_page_modules(wikitext, settings);
        wikitext = Self::expand_rate_modules(wikitext, page_info, settings);
        if settings.enable_page_syntax {
            Self::normalize_wikidot_div_style_url_quotes(&mut wikitext);
        }
        wikitext = Self::render_wikidot_color_spans(wikitext, settings);
        wikitext = Self::escape_unrendered_wikidot_color_markers(wikitext, settings);
        wikitext = Self::render_long_native_list_runs(wikitext);
        if Self::should_use_wikidot_compatibility_fallback(&wikitext, page_info) {
            let mut backlinks = ftml::data::Backlinks::new();
            backlinks.included_pages.extend(included_pages);
            let html_output = HtmlOutput {
                body: Self::render_oversized_wikidot_compatibility_fallback(
                    &wikitext,
                    current_site.as_ref(),
                    config,
                    page_info.page.as_ref(),
                ),
                meta: Vec::new(),
                backlinks,
            };
            let compiled_hash = TextService::create(ctx, html_output.body.clone())
                .await
                .or_raise(make_error)?;

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
        let render_timeout = Self::ftml_compat_render_timeout(&render_config, &wikitext);

        let render_task = task::spawn_blocking(move || {
            let wikidot_css_modules =
                Self::protect_wikidot_css_modules(&mut wikitext, &render_settings);
            let wikidot_compat_links =
                Self::protect_wikidot_compat_links(&mut wikitext, &render_settings);
            let wikidot_wikipedia_links =
                Self::protect_wikidot_wikipedia_links(&mut wikitext, &render_settings);
            let wikidot_compat_html = Self::protect_generated_wikidot_compat_html(
                &mut wikitext,
                &render_settings,
            );
            let wikidot_embed_iframes =
                Self::protect_wikidot_embed_iframes(&mut wikitext);

            ftml::preprocess(&mut wikitext);
            let tokens = ftml::tokenize(&wikitext);
            let result = ftml::parse(&tokens, &render_page_info, &render_settings);
            let (tree, errors) = result.into();
            let mut html_output =
                HtmlRender.render(&tree, &render_page_info, &render_settings);
            html_output.body = Self::restore_protected_wikidot_embed_iframes(
                html_output.body,
                &wikidot_embed_iframes,
            );
            html_output.body = Self::restore_protected_wikidot_css_modules(
                html_output.body,
                &wikidot_css_modules,
            );
            html_output.body = Self::restore_protected_generated_wikidot_compat_html(
                html_output.body,
                &wikidot_compat_html,
            );
            html_output.body = Self::restore_protected_wikidot_wikipedia_links(
                html_output.body,
                &wikidot_wikipedia_links,
            );
            html_output.body = Self::restore_protected_wikidot_compat_links(
                html_output.body,
                &wikidot_compat_links,
            );
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
            html_output.body =
                Self::remove_wikidot_compat_style_blocks(&html_output.body);
            html_output.backlinks.included_pages.extend(included_pages);
            let html_block_texts = tree
                .html_blocks
                .iter()
                .map(|html| {
                    Self::localize_wikidot_local_file_urls(
                        html,
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
        let html = Self::resolve_residual_wikidot_simple_if_fragments(&html);
        let html = Self::restore_wikidot_mailform_compatibility(&html);
        let html = Self::restore_residual_wikidot_div_paragraph_markers(&html);
        let html = Self::remove_residual_wikidot_iftags_fragments(&html);
        let html = Self::remove_wikijump_table_body_wrappers(&html);
        let html = Self::remove_wikidot_compat_style_blocks(&html);
        let html = Self::restore_wikidot_inline_math_compatibility(&html);
        let html = Self::restore_wikidot_ta_badge_default_compatibility(&html);
        let html = Self::remove_wikijump_plain_format_wrappers(&html);
        let html = Self::remove_wikidot_userkarma_background_styles(&html);
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
        html.replace("</wj-tabs-button>", "</a></li>").replace(
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

    fn restore_residual_wikidot_div_paragraph_markers(html: &str) -> String {
        let mut restored_open_count = 0usize;

        WIKIDOT_RESIDUAL_DIV_PARAGRAPH_REGEX
            .replace_all(html, |captures: &regex::Captures<'_>| {
                if let Some(marker) = captures.name("open") {
                    let marker = marker
                        .as_str()
                        .replace("&quot;", "\"")
                        .replace("&#34;", "\"");
                    if let Some(attributes) = Self::wikidot_compat_div_attributes(&marker)
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
            .into_owned()
    }

    fn remove_residual_wikidot_iftags_fragments(html: &str) -> String {
        WIKIDOT_RESIDUAL_IFTAGS_INLINE_REGEX
            .replace_all(html, "")
            .into_owned()
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

                    let tag = captures.name("tag").map_or("", |mtch| mtch.as_str());
                    let has_tag = page_info.tags.iter().any(|page_tag| page_tag == tag);
                    let active = match captures.name("sign").map(|mtch| mtch.as_str()) {
                        Some("+") => has_tag,
                        Some("-") => !has_tag,
                        _ => false,
                    };
                    if active {
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

    fn resolve_residual_wikidot_simple_if_fragments(html: &str) -> String {
        WIKIDOT_SIMPLE_IF_REGEX
            .replace_all(html, |captures: &regex::Captures<'_>| {
                if captures["cond"].eq_ignore_ascii_case("1")
                    || captures["cond"].eq_ignore_ascii_case("true")
                {
                    captures["when_true"].trim().to_owned()
                } else {
                    captures["when_false"].trim().to_owned()
                }
            })
            .into_owned()
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

    fn remove_wikijump_plain_format_wrappers(html: &str) -> String {
        html.replace("<u>", "")
            .replace("</u>", "")
            .replace("<s>", "")
            .replace("</s>", "")
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

    fn remove_unresolved_include_comment_branches(wikitext: &mut String) {
        const HIDDEN_BRANCH_MARKER: &str = "[!-- {$";
        const COMMENT_BOUNDARY_MARKER: &str = "[!----]";
        const SELECTED_BRANCH_MARKER: &str = "[!-- --]";

        while let Some(marker_start) = wikitext.find(HIDDEN_BRANCH_MARKER) {
            let removal_start = wikitext[..marker_start]
                .rfind('\n')
                .map_or(marker_start, |index| index + 1);
            let Some(boundary_offset) =
                wikitext[marker_start..].find(COMMENT_BOUNDARY_MARKER)
            else {
                break;
            };
            let boundary_start = marker_start + boundary_offset;
            let boundary_end = boundary_start + COMMENT_BOUNDARY_MARKER.len();
            let removal_end = wikitext[boundary_end..]
                .find('\n')
                .map_or(boundary_end, |offset| boundary_end + offset + 1);

            wikitext.replace_range(removal_start..removal_end, "");
        }

        for marker in [SELECTED_BRANCH_MARKER, COMMENT_BOUNDARY_MARKER] {
            while let Some(marker_start) = wikitext.find(marker) {
                let removal_start = wikitext[..marker_start]
                    .rfind('\n')
                    .map_or(marker_start, |index| index + 1);
                let marker_end = marker_start + marker.len();
                let removal_end = wikitext[marker_end..]
                    .find('\n')
                    .map_or(marker_end, |offset| marker_end + offset + 1);
                wikitext.replace_range(removal_start..removal_end, "");
            }
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
        Self::protect_wikidot_current_page_links(wikitext, &mut links);
        Self::protect_wikidot_star_local_links(wikitext, &mut links);
        links
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

            let marker = format!("{WIKIDOT_COMPAT_LINK_SENTINEL_PREFIX}{}X", links.len());
            links.push(ProtectedWikidotCompatLink {
                anchor: wikidot_current_page_anchor(label),
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

            let marker = format!("{WIKIDOT_COMPAT_LINK_SENTINEL_PREFIX}{}X", links.len());
            links.push(ProtectedWikidotCompatLink {
                anchor: wikidot_star_local_anchor(target, label),
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
        for (index, link) in links.iter().enumerate() {
            let marker = format!("{WIKIDOT_COMPAT_LINK_SENTINEL_PREFIX}{index}X");
            html = html.replace(&marker, &link.anchor);
        }
        html
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
            MAX_INCLUDE_EXPANSION_TOTAL,
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
        mut remaining_includes: usize,
    ) -> Pin<Box<dyn Future<Output = Result<IncludeExpansion>> + Send + 'a>> {
        Box::pin(async move {
            let mut wikitext = wikitext;
            Self::normalize_wikidot_ta_badge_multiline_includes(&mut wikitext);

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
                if remaining_includes == 0 {
                    return Err(Error::new(
                        format!(
                            "include expansion exceeded maximum total includes {}",
                            MAX_INCLUDE_EXPANSION_TOTAL,
                        ),
                        ErrorType::Render,
                    )
                    .into());
                }
                remaining_includes -= 1;

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
                Self::remove_wikidot_component_iftags_documentation(&mut source.wikitext);
                Self::remove_unresolved_variable_iftags_blocks(&mut source.wikitext);

                let expansion = Self::expand_includes_for_site(
                    ctx,
                    source.wikitext,
                    source.site_id,
                    source.site_slug,
                    settings,
                    depth + 1,
                    remaining_includes,
                )
                .await?;
                if expansion.included_pages.len() > remaining_includes {
                    return Err(Error::new(
                        format!(
                            "include expansion exceeded maximum total includes {}",
                            MAX_INCLUDE_EXPANSION_TOTAL,
                        ),
                        ErrorType::Render,
                    )
                    .into());
                }
                remaining_includes -= expansion.included_pages.len();

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
                    return Self::fetch_include_source_from_site(
                        ctx,
                        current_site_id,
                        current_site_slug,
                        page_ref.page(),
                    )
                    .await;
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
    ) -> Result<IncludeExpansion> {
        let (Some(current_site_id), Some(current_page_id)) =
            (current_site_id, current_page_id)
        else {
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
            expanded.push_str(&replacement);
            included_pages.extend(replacement_included_pages);
            cursor = mtch.end();
        }

        expanded.push_str(&wikitext[cursor..]);
        Ok(IncludeExpansion {
            wikitext: expanded,
            included_pages,
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

        let replacement = render_read_only_rate_module(page_info.score);
        RATE_MODULE_REGEX
            .replace_all(&wikitext, replacement.as_str())
            .into_owned()
    }

    fn expand_members_modules(wikitext: String, settings: &WikitextSettings) -> String {
        if !settings.enable_page_syntax {
            return wikitext;
        }

        MEMBERS_MODULE_REGEX
            .replace_all(&wikitext, |captures: &regex::Captures<'_>| {
                let head = captures.name("head").map_or("", |mtch| mtch.as_str());
                let group = wikidot_module_argument(head, "group")
                    .unwrap_or("members")
                    .trim();
                render_members_module_placeholder(group)
            })
            .into_owned()
    }

    fn expand_new_page_modules(wikitext: String, settings: &WikitextSettings) -> String {
        if !settings.enable_page_syntax {
            return wikitext;
        }

        NEWPAGE_MODULE_REGEX
            .replace_all(&wikitext, |captures: &regex::Captures<'_>| {
                let head = captures.name("head").map_or("", |mtch| mtch.as_str());
                render_new_page_module(head)
            })
            .into_owned()
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
    ) -> Vec<String> {
        if !settings.enable_page_syntax {
            return Vec::new();
        }

        let mut styles = Vec::new();
        let protected = CSS_MODULE_REGEX
            .replace_all(wikitext, |captures: &regex::Captures<'_>| {
                let body = captures.name("body").map_or("", |mtch| mtch.as_str());
                let body = body.trim_matches('\n');
                let marker =
                    format!("{WIKIDOT_CSS_MODULE_SENTINEL_PREFIX}{}X", styles.len());
                styles.push(format!("<style>\n{body}\n</style>"));
                marker
            })
            .into_owned();
        *wikitext = protected;
        styles
    }

    fn restore_protected_wikidot_css_modules(
        mut html: String,
        styles: &[String],
    ) -> String {
        for (index, style) in styles.iter().enumerate() {
            let marker = format!("{WIKIDOT_CSS_MODULE_SENTINEL_PREFIX}{index}X");
            html = html.replace(&marker, style);
        }
        html
    }

    fn protect_generated_wikidot_compat_html(
        wikitext: &mut String,
        settings: &WikitextSettings,
    ) -> Vec<String> {
        if !settings.enable_page_syntax {
            return Vec::new();
        }

        let mut fragments = Vec::new();
        *wikitext =
            Self::protect_generated_wikidot_compat_lists(wikitext, &mut fragments);
        let protected = GENERATED_COMPAT_TABLE_REGEX
            .replace_all(wikitext, |captures: &regex::Captures<'_>| {
                let marker =
                    format!("{WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX}{}X", fragments.len());
                fragments.push(
                    captures[0]
                        .replace(r#" data-wikijump-compat-list="1""#, "")
                        .replace(r#" data-wikijump-compat-members="1""#, "")
                        .replace(r#" data-wikijump-compat-pager="1""#, "")
                        .replace(r#" data-wikijump-compat-new-page="1""#, ""),
                );
                marker
            })
            .into_owned();
        *wikitext = protected;
        fragments
    }

    fn protect_generated_wikidot_compat_lists(
        wikitext: &str,
        fragments: &mut Vec<String>,
    ) -> String {
        let mut output = String::with_capacity(wikitext.len());
        let mut rest = wikitext;
        let list_start = r#"<ul data-wikijump-compat-list="1">"#;

        while let Some(start) = rest.find(list_start) {
            let (before, from_start) = rest.split_at(start);
            output.push_str(before);

            if let Some(end) = find_balanced_ul_end(from_start) {
                let fragment = &from_start[..end];
                let marker =
                    format!("{WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX}{}X", fragments.len());
                fragments.push(fragment.replace(r#" data-wikijump-compat-list="1""#, ""));
                output.push_str(&marker);
                rest = &from_start[end..];
            } else {
                output.push_str(list_start);
                rest = &from_start[list_start.len()..];
            }
        }

        output.push_str(rest);
        output
    }

    fn restore_protected_generated_wikidot_compat_html(
        mut html: String,
        fragments: &[String],
    ) -> String {
        for (index, fragment) in fragments.iter().enumerate() {
            let marker = format!("{WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX}{index}X");
            html = html.replace(&marker, fragment);
        }
        html
    }

    fn render_wikidot_color_spans(
        wikitext: String,
        settings: &WikitextSettings,
    ) -> String {
        if !settings.enable_page_syntax {
            return wikitext;
        }

        WIKIDOT_COLOR_SPAN_REGEX
            .replace_all(&wikitext, |captures: &regex::Captures<'_>| {
                format!(
                    r#"<span style="color: {color}">{body}</span>"#,
                    color = escape_list_pages_html_attr(&captures["color"]),
                    body = escape_list_pages_html_text(&captures["body"]),
                )
            })
            .into_owned()
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

    fn render_long_native_list_runs(wikitext: String) -> String {
        let lines = wikitext.split_inclusive('\n').collect::<Vec<_>>();
        let mut output = String::with_capacity(wikitext.len());
        let mut index = 0;

        while index < lines.len() {
            let mut end = index;
            while end < lines.len() && native_bullet_list_item(lines[end]).is_some() {
                end += 1;
            }

            if end - index >= LONG_NATIVE_LIST_RENDER_MIN_ITEMS {
                output.push_str(&render_native_bullet_list(&lines[index..end]));
                index = end;
            } else {
                output.push_str(lines[index]);
                index += 1;
            }
        }

        output
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

    fn render_oversized_wikidot_compatibility_fallback(
        wikitext: &str,
        current_site: Option<&SiteModel>,
        config: &Config,
        current_page: &str,
    ) -> String {
        let localized =
            Self::localize_wikidot_local_file_urls(wikitext, current_site, config);
        let localized = Self::render_wikidot_compat_fallback_css_modules(&localized);

        if localized.lines().any(|line| {
            let marker = line.trim_start().to_ascii_lowercase();
            marker.starts_with("[[code") || marker.starts_with("[[collapsible")
        }) {
            return Self::render_wikidot_compatibility_fallback_with_code_blocks_for_context(
                &localized,
                Some(current_page),
                current_site.map(|site| site.slug.as_str()),
            );
        }

        let mut body = String::with_capacity(localized.len() + 96);
        body.push_str("<div class=\"wikidot-compat-fallback\"><pre>");
        push_escaped_html(&mut body, &localized);
        body.push_str("</pre></div>");
        body
    }

    fn render_wikidot_compat_fallback_css_modules(wikitext: &str) -> String {
        CSS_MODULE_REGEX
            .replace_all(wikitext, |captures: &regex::Captures<'_>| {
                let body = captures.name("body").map_or("", |mtch| mtch.as_str());
                let body = body.trim_matches('\n');
                format!("<style data-wikijump-compat-css-module=\"1\">\n{body}\n</style>")
            })
            .into_owned()
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
        let mut body = String::with_capacity(wikitext.len() + 256);
        body.push_str("<div class=\"wikidot-compat-fallback\">");

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
                    &Self::render_wikidot_compat_fallback_text_html_for_context(
                        wikitext,
                        current_page,
                        local_file_site_slug,
                    ),
                );
                fallback.push_str("</div>");
                return fallback;
            }

            let mut fallback = String::with_capacity(wikitext.len() + 96);
            fallback.push_str("<div class=\"wikidot-compat-fallback\"><pre>");
            push_escaped_html(&mut fallback, wikitext);
            fallback.push_str("</pre></div>");
            return fallback;
        }

        body
    }

    #[allow(dead_code)]
    fn push_wikidot_compat_fallback_text_chunk(body: &mut String, chunk: &mut String) {
        Self::push_wikidot_compat_fallback_text_chunk_for_page(body, chunk, None, None);
    }

    fn push_wikidot_compat_fallback_text_chunk_for_page(
        body: &mut String,
        chunk: &mut String,
        current_page: Option<&str>,
        local_file_site_slug: Option<&str>,
    ) {
        if chunk.is_empty() {
            return;
        }

        let text = chunk.trim_end_matches('\n');
        if Self::wikidot_compat_text_has_markup(text) {
            body.push_str(&Self::render_wikidot_compat_fallback_text_html_for_context(
                text,
                current_page,
                local_file_site_slug,
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
            || text.contains("[[=image")
            || text.contains("[[[")
            || text.contains("[[*")
            || text.contains("[http")
            || text.contains("**")
            || text.contains("<style data-wikijump-compat-css-module=\"1\">")
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
        let mut output = String::with_capacity(text.len());
        let mut paragraph = String::new();
        let mut in_style = false;
        let mut tabview_open = false;
        let mut tab_open = false;
        let mut size_depth = 0usize;

        for line in text.lines() {
            let trimmed = line.trim();
            if in_style {
                output.push_str(line);
                output.push('\n');
                if trimmed.eq_ignore_ascii_case("</style>") {
                    in_style = false;
                }
                continue;
            }

            if trimmed == "<style data-wikijump-compat-css-module=\"1\">" {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                );
                output.push_str(trimmed);
                output.push('\n');
                in_style = true;
                continue;
            }

            if trimmed.eq_ignore_ascii_case("[[tabview]]") {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
                );
                if !tabview_open {
                    output.push_str(WIKIDOT_TABVIEW_SCRIPT);
                    output.push_str(
                        r#"<div class="yui-navset wikidot-compat-tabview"><div class="yui-content">"#,
                    );
                    tabview_open = true;
                }
                continue;
            }

            if let Some(title) = Self::wikidot_compat_tab_title(trimmed) {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
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
                );
                output.push_str(&image);
                continue;
            }

            if let Some(attributes) = Self::wikidot_compat_div_attributes(trimmed) {
                Self::push_wikidot_compat_fallback_paragraph_for_page(
                    &mut output,
                    &mut paragraph,
                    current_page,
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
                );
                output.push_str("</div>");
                continue;
            }

            paragraph.push_str(line);
            paragraph.push('\n');
        }

        Self::push_wikidot_compat_fallback_paragraph_for_page(
            &mut output,
            &mut paragraph,
            current_page,
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

    fn wikidot_compat_div_attributes(marker: &str) -> Option<String> {
        if !marker.starts_with("[[div") || !marker.ends_with("]]") {
            return None;
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

        if let Some(style) = Self::wikidot_marker_attr(marker, "style") {
            attributes.push_str(r#" style=""#);
            attributes.push_str(&escape_list_pages_html_attr(&style));
            attributes.push('"');
        }

        (!attributes.is_empty()).then_some(attributes)
    }

    #[allow(dead_code)]
    fn push_wikidot_compat_fallback_paragraph(body: &mut String, paragraph: &mut String) {
        Self::push_wikidot_compat_fallback_paragraph_for_page(body, paragraph, None);
    }

    fn push_wikidot_compat_fallback_paragraph_for_page(
        body: &mut String,
        paragraph: &mut String,
        current_page: Option<&str>,
    ) {
        let text = paragraph.trim_matches('\n');
        if !text.trim().is_empty() {
            body.push_str("<p>");
            body.push_str(&Self::render_wikidot_compat_fallback_inline_html_for_page(
                text,
                current_page,
            ));
            body.push_str("</p>");
        }
        paragraph.clear();
    }

    #[allow(dead_code)]
    fn render_wikidot_compat_fallback_inline_html(value: &str) -> String {
        Self::render_wikidot_compat_fallback_inline_html_for_page(value, None)
    }

    fn render_wikidot_compat_fallback_inline_html_for_page(
        value: &str,
        _current_page: Option<&str>,
    ) -> String {
        let mut output = String::with_capacity(value.len());
        let mut strong = false;
        for segment in value.split("**") {
            if strong {
                output.push_str("<strong>");
            }
            Self::push_wikidot_compat_fallback_inline_segment(&mut output, segment);
            if strong {
                output.push_str("</strong>");
            }
            strong = !strong;
        }
        output
    }

    fn push_wikidot_compat_fallback_inline_segment(output: &mut String, value: &str) {
        let mut rest = value;
        while let Some(start) = rest.find('<') {
            let (before, after_start) = rest.split_at(start);
            output.push_str(&Self::render_wikidot_compat_inline_text_segment(before));
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
                ));
                return;
            }
        }
        output.push_str(&Self::render_wikidot_compat_inline_text_segment(rest));
    }

    fn render_wikidot_compat_inline_text_segment(value: &str) -> String {
        let html = render_native_list_inline_html(value);
        Self::render_wikidot_compat_inline_size_markers(&html)
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
            let make_error = || {
                Error::new(
                    "failed to load current page category for ListPages render",
                    ErrorType::Render,
                )
            };
            let page =
                PageService::get(ctx, current_site_id, Reference::Id(current_page_id))
                    .await
                    .or_raise(make_error)?;
            let category = CategoryService::get(
                ctx,
                current_site_id,
                Reference::Id(page.page_category_id),
            )
            .await
            .or_raise(make_error)?;
            let mut categories = categories;
            if !categories.iter().any(|slug| slug.as_ref() == category.slug) {
                categories.push(Cow::Owned(category.slug));
            }
            categories
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
                score: list_pages_body_uses_variable(body, "rating"),
                ..Default::default()
            },
        };

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
            Self::find_viewable_list_pages_rows(
                ctx,
                query,
                query_limit.min(usize::MAX as u64) as usize,
            )
            .await?
        };
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
                            page_info.site.as_ref(),
                            settings,
                            Some(page.site_id),
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
            let body = substitute_list_pages_variables(
                body,
                page,
                index + 1,
                total,
                &substitution_context,
            );
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
            count_pages_per_page,
            offset,
            exclude_current_page,
            page_type,
            page_parent,
            slug,
            prepend_line: _,
            data_form_fields,
            unsupported_count_pages_filter: _,
        } = arguments;
        let per_page_only_count =
            count_pages_explicit_limit.is_none() && count_pages_per_page.is_some();
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
            let make_error = || {
                Error::new(
                    "failed to load current page category for CountPages render",
                    ErrorType::Render,
                )
            };
            let page =
                PageService::get(ctx, current_site_id, Reference::Id(current_page_id))
                    .await
                    .or_raise(make_error)?;
            let category = CategoryService::get(
                ctx,
                current_site_id,
                Reference::Id(page.page_category_id),
            )
            .await
            .or_raise(make_error)?;
            let mut categories = categories;
            if !categories.iter().any(|slug| slug.as_ref() == category.slug) {
                categories.push(Cow::Owned(category.slug));
            }
            categories
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
            Self::find_viewable_list_pages_rows(ctx, query, target_count).await?
        };
        let pages = pages
            .pages
            .into_iter()
            .filter(|page| !exclude_current_page || page.page_id != current_page_id)
            .skip(offset as usize);
        let total = match count_pages_explicit_limit {
            Some(limit) => pages.take(limit.min(usize::MAX as u64) as usize).count(),
            None => {
                let total = pages.count();
                if per_page_only_count && total >= MAX_LISTPAGES_RENDER_SCAN_ROWS as usize
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
    ) -> Result<FoundPages> {
        let mut pages = Vec::new();
        let mut raw_offset = 0;

        while pages.len() < target_count && raw_offset < MAX_LISTPAGES_RENDER_SCAN_ROWS {
            let mut query = query.clone();
            query.offset = raw_offset;
            query.pagination.limit = Some(
                MAX_LISTPAGES_RENDER_LIMIT
                    .min(u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS - raw_offset)),
            );

            let found = PageQueryService::find(ctx, query).await?;
            let raw_count = found.pages.len();
            if raw_count == 0 {
                break;
            }
            pages.extend(Self::filter_viewable_list_pages_rows(ctx, found.pages).await?);
            if raw_count < MAX_LISTPAGES_RENDER_LIMIT as usize {
                break;
            }
            raw_offset = raw_offset.saturating_add(MAX_LISTPAGES_RENDER_LIMIT as u32);
        }

        Ok(FoundPages { pages })
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

        let page = PageService::get(ctx, current_site_id, Reference::Id(current_page_id))
            .await
            .or_raise(make_error)?;
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
                slug: if fields.slug { Some(page.slug) } else { None },
                page_category_id: if fields.page_category_id {
                    Some(page.page_category_id)
                } else {
                    None
                },
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
                score: None,
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
                        snapshot.commented_at, snapshot.commented_by_name \
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
        || (count_pages_bound.is_none() && !arguments.current_page_only)
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

fn should_render_current_page_list_pages_row(
    current_page_only: bool,
    limit: Option<u64>,
    offset: u32,
) -> bool {
    current_page_only && limit.unwrap_or(1) > 0 && offset == 0
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
    if list_pages_body_has_numbered_rows(body) {
        "[[div class=\"list-pages-box\"]][[/div]]".to_owned()
    } else {
        module_source.to_owned()
    }
}

fn list_pages_body_has_numbered_rows(body: &str) -> bool {
    body.lines()
        .any(|line| native_numbered_list_content(line).is_some())
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

    output.push_str(r#"<div class="pager" data-wikijump-compat-pager="1">"#);
    output.push_str(&format!(
        r#"<span class="pager-no">page {current_page} of {page_count}</span>"#
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
            output.push_str(r#"<span class="dots">...</span>"#);
        }
        if page == current_page {
            output.push_str(&format!(r#"<span class="current">{page}</span>"#));
        } else {
            push_list_pages_pager_target(output, page_info, page, &page.to_string());
        }
        previous = page;
    }

    if current_page < page_count {
        push_list_pages_pager_target(output, page_info, current_page + 1, "next »");
    }

    output.push_str("</div>\n");
}

fn push_list_pages_pager_target(
    output: &mut String,
    page_info: &PageInfo<'_>,
    target_page: usize,
    label: &str,
) {
    output.push_str(r#"<span class="target"><a href="/"#);
    output.push_str(page_info.page.as_ref());
    output.push_str("/p/");
    output.push_str(&target_page.to_string());
    output.push_str(r#"">"#);
    output.push_str(&escape_list_pages_html_text(label));
    output.push_str("</a></span>");
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
    let title_linked = if slug.is_empty() {
        title.to_owned()
    } else {
        format!("[/{slug} {title}]")
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
    let index = index.to_string();
    let total = total.to_string();
    let rendered_limit = context.rendered_limit.to_string();

    let substituted = LISTPAGES_VARIABLE_REGEX
        .replace_all(template, |captures: &regex::Captures<'_>| {
            match captures["name"].to_ascii_lowercase().as_str() {
                "title_linked" => title_linked.clone(),
                "linked_title" => title_linked.clone(),
                "title" => title.to_owned(),
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
                "rating_votes" | "ratingvotes" => String::new(),
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

    resolve_list_pages_signed_abs_expressions(&substituted)
}

fn substitute_count_pages_variables(template: &str, total: usize) -> String {
    let total = total.to_string();
    LISTPAGES_VARIABLE_REGEX
        .replace_all(template, |captures: &regex::Captures<'_>| {
            match captures["name"].to_ascii_lowercase().as_str() {
                "total" | "count" => total.clone(),
                _ => captures
                    .get(0)
                    .map_or("", |matched| matched.as_str())
                    .to_owned(),
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
    let format = format.unwrap_or("%e %b %Y %H:%M");
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
            r#"[[span class="odate time_{} format_{}" style="cursor: help; display: inline;"]]{}[[/span]]"#,
            created_at.unix_timestamp(),
            encoded_format,
            text,
        )
    }
}

fn resolve_list_pages_signed_abs_expressions(value: &str) -> String {
    WIKIDOT_LISTPAGES_SIGNED_ABS_EXPR_REGEX
        .replace_all(value, |captures: &regex::Captures<'_>| {
            let original = captures.get(0).map_or("", |matched| matched.as_str());
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

    let mut output = String::from("<table class=\"wiki-content-table\">");
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
    let escaped = render_native_list_inline_wikidot_spans(value);
    let with_quadruple_links = WIKIDOT_QUADRUPLE_LINK_REGEX
        .replace_all(&escaped, |captures: &regex::Captures<'_>| {
            render_native_list_page_link(&captures["target"], None)
        })
        .into_owned();
    let with_labeled_links = WIKIDOT_LABELED_LINK_REGEX
        .replace_all(&with_quadruple_links, |captures: &regex::Captures<'_>| {
            render_native_list_page_link(&captures["target"], Some(&captures["label"]))
        })
        .into_owned();
    let with_unlabeled_links = WIKIDOT_UNLABELED_LINK_REGEX
        .replace_all(&with_labeled_links, |captures: &regex::Captures<'_>| {
            render_native_list_page_link(&captures["target"], None)
        })
        .into_owned();
    let with_local_links = WIKIDOT_LOCAL_LINK_REGEX
        .replace_all(&with_unlabeled_links, |captures: &regex::Captures<'_>| {
            render_native_list_page_link(&captures["target"], Some(&captures["label"]))
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

fn render_native_list_page_link(target: &str, label: Option<&str>) -> String {
    let target = target.trim();
    let label = label
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| native_list_page_link_default_label(target));
    let href = native_list_page_link_href(target);
    format!(
        r#"<a href="{href}">{label}</a>"#,
        href = escape_list_pages_html_attr(&href),
        label = label,
    )
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

fn native_list_page_link_default_label(target: &str) -> String {
    if target.starts_with("http://") || target.starts_with("https://") {
        return target.to_owned();
    }
    if target.contains(char::is_whitespace) {
        return target.to_owned();
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

fn render_read_only_rate_module(score: ftml::data::ScoreValue) -> String {
    let score = format_score_value(score);

    format!(
        concat!(
            "[[div class=\"page-rate-widget-box\"]]",
            "[[span class=\"rate-points\"]]rating: ",
            "[[span class=\"number prw54353\"]]{}[[/span]]",
            "[[/span]]",
            "[[span class=\"rateup btn btn-default\"]]",
            "[[a href=\"javascript:;\" onclick=\"WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, 1)\" title=\"I like it\"]]+[[/a]]",
            "[[/span]]",
            "[[span class=\"ratedown btn btn-default\"]]",
            "[[a href=\"javascript:;\" onclick=\"WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, -1)\" title=\"I don't like it\"]]–[[/a]]",
            "[[/span]]",
            "[[span class=\"cancel btn btn-default\"]]",
            "[[a href=\"javascript:;\" onclick=\"WIKIDOT.modules.PageRateWidgetModule.listeners.cancelVote(event)\" title=\"Cancel my vote\"]]x[[/a]]",
            "[[/span]]",
            "[[/div]]"
        ),
        score,
    )
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

fn escape_javascript_single_quoted(value: &str) -> String {
    value
        .replace('\\', r"\\")
        .replace('\'', r"\'")
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
#side-bar {
    display: none !important;
    visibility: hidden !important;
    left: -9999px !important;
}
#main-content {
    margin-left: auto !important;
    margin-right: auto !important;
    margin-top: -12rem !important;
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
        CollectingIncluder, LISTPAGES_MODULE_REGEX, ListPagesSnapshotDisplay,
        ListPagesSubstitutionContext, MAX_FTML_COMPAT_COLLAPSIBLE_BLOCKS,
        MAX_FTML_COMPAT_DENSE_PARSE_SCORE, MAX_FTML_COMPAT_PARSE_BYTES,
        MIN_DENSE_FTML_COMPAT_RENDER_TIMEOUT_SECS, OrderBySelector, OrderProperty,
        RenderContext, RenderService, WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX,
        WIKIDOT_COMPAT_LINK_SENTINEL_PREFIX, WIKIDOT_CSS_MODULE_SENTINEL_PREFIX,
        WIKIDOT_WIKIPEDIA_LINK_SENTINEL_PREFIX, WikidotUserDisplay,
        count_pages_should_remain_literal, include_error,
        list_pages_body_uses_content_variable, list_pages_body_variables_supported,
        list_pages_has_unsupported_page_type_selector,
        list_pages_has_unsupported_parent_selector, parse_list_pages_arguments,
        render_list_pages_numbered_rows, render_list_pages_table_rows,
        render_members_module_placeholder, render_new_page_module,
        render_read_only_rate_module, render_tag_cloud_box,
        resolve_list_pages_signed_abs_expressions,
        should_render_current_page_list_pages_row, substitute_list_pages_variables,
        unsupported_list_pages_replacement, wikidot_content_section,
        wikidot_module_argument,
    };
    use crate::config::Config;
    use crate::constants::ADMIN_USER_ID;
    use crate::models::site::Model as SiteModel;
    use crate::services::page_query::{
        DataFormSelector, FoundPageRow, parse_static_wikidot_data_form_values,
        static_wikidot_data_form_matches,
    };
    use crate::types::License;
    use crate::utils::now;
    use ftml::data::PageRef;
    use ftml::includes::IncludeRef;
    use ftml::layout::Layout;
    use ftml::render::{Render, html::HtmlRender};
    use ftml::settings::{WikitextMode, WikitextSettings};
    use ftml::tree::VariableMap;
    use std::borrow::Cow;
    use std::collections::BTreeMap;
    use std::time::Duration;

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
        assert_eq!(arguments.limit, Some(15));
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
        assert_eq!(arguments.limit, Some(250));
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
        assert_eq!(arguments.limit, Some(100));
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
        assert_eq!(arguments.limit, Some(20));
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
        assert_eq!(arguments.limit, Some(250));
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
    fn keeps_unbounded_count_pages_literal_even_with_static_filters() {
        let tagged = parse_list_pages_arguments(r#" category="*" tags="codex" "#)
            .expect("static tag CountPages selector should parse");
        assert!(count_pages_should_remain_literal(&tagged));

        let broad = parse_list_pages_arguments(r#" category="*" "#)
            .expect("broad CountPages selector should parse");
        assert!(count_pages_should_remain_literal(&broad));

        let exclusion_only = parse_list_pages_arguments(r#" category="* -deleted" "#)
            .expect("exclusion-only CountPages selector should parse");
        assert!(count_pages_should_remain_literal(&exclusion_only));
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
        let rendered = render_read_only_rate_module(ftml::data::ScoreValue::Integer(19));

        assert!(rendered.contains(r#"[[span class="rate-points"]]rating: "#));
        assert!(rendered.contains(r#"[[span class="number prw54353"]]+19[[/span]]"#));
        assert!(rendered.contains(r#"[[span class="rateup btn btn-default"]]"#));
        assert!(rendered.contains(r#"listeners.rate(event, 1)"#));
        assert!(rendered.contains(r#"[[span class="ratedown btn btn-default"]]"#));
        assert!(rendered.contains(r#"listeners.rate(event, -1)"#));
        assert!(rendered.contains(r#"title="I don't like it"]]–[[/a]]"#));
        assert!(rendered.contains(r#"[[span class="cancel btn btn-default"]]"#));
        assert!(rendered.contains(r#"listeners.cancelVote(event)"#));
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

        assert!(rendered.contains("<table class=\"wiki-content-table\">"));
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
        assert_eq!(
            RenderService::restore_protected_generated_wikidot_compat_html(
                protected, &fragments,
            ),
            rendered,
        );
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

        let styles = RenderService::protect_wikidot_css_modules(&mut source, &settings);

        assert_eq!(styles.len(), 1);
        assert!(styles[0].contains("<style>\n#u-change{"));
        assert!(styles[0].contains("display:none;"));
        assert!(source.contains(WIKIDOT_CSS_MODULE_SENTINEL_PREFIX));
        assert!(!source.contains("[[module css]]"));
        assert!(!source.contains("#u-change"));

        let restored =
            RenderService::restore_protected_wikidot_css_modules(source, &styles);
        assert!(restored.contains("<style>\n#u-change{"));
    }

    #[test]
    fn renders_wikidot_color_spans_before_ftml_parsing() {
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let source = "##blue|**[[include :scp-wiki:component:coltop**##\n".to_owned();

        let rendered = RenderService::render_wikidot_color_spans(source, &settings);

        assert_eq!(
            rendered,
            r#"<span style="color: blue">**[[include :scp-wiki:component:coltop**</span>"#
                .to_owned() + "\n",
        );
        assert!(!rendered.starts_with('#'));

        let escaped = RenderService::escape_unrendered_wikidot_color_markers(
            "####blue|leftover##".to_owned(),
            &settings,
        );
        assert_eq!(escaped, "&#35;&#35;&#35;&#35;blue|leftover&#35;&#35;");
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
            r#"[[span class="odate time_1782003564 format_%25d%20%25b%20%25Y" style="cursor: help; display: inline;"]]21 Jun 2026[[/span]]"#
        ));

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
        assert!(rendered.contains(r#"[[span class="odate time_1782005400"#));
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
            },
        );

        let rendered = substitute_list_pages_variables(
            "%%title%% by %%author%% on %%created_at|%Y %b %e|agohover%% -- %%comments%% Comments -- %%commented_by%% %%commented_at|%Y %b %e%%",
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
        assert!(rendered.contains(r#"style="cursor: help; display: inline;""#));
        assert!(!rendered.contains("Administrator"));
        assert!(!rendered.contains("<span"));
        assert!(!rendered.contains("user:info"));
        assert!(!rendered.contains("2020 Sep"));
        assert!(!rendered.contains("%%comments%%"));
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

        assert!(rendered.contains("<table class=\"wiki-content-table\">"));
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
        assert!(rendered.contains(r#"[[span class="odate time_1781900521 format_%25Y%20%25b%20%25e%7Cagohover" style="cursor: help; display: inline;"]]2026 Jun 20[[/span]]"#));
        assert!(rendered.contains("[/artwork-hub/tag/-scp,-goi-format,-supplement,-tale,-hub,-site,-resource,-guide,-essay,-theme,artwork artwork]"));
        assert!(rendered.contains("[/artwork-hub/tag/-scp,-goi-format,-supplement,-tale,-hub,-site,-resource,-guide,-essay,-theme,preview preview]"));
        assert!(rendered.contains("[/artwork-hub/tag/-scp,-goi-format,-supplement,-tale,-hub,-site,-resource,-guide,-essay,-theme,colored-pencil colored-pencil]"));
        assert!(rendered.contains(r#"[[span class="rating"]]+28[[/span]]"#));
        assert!(!rendered.contains("<span"));
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
            "[[div class=\"list-pages-item\"]]\n",
            "**<span class=\"odate time_123 format_%25e%20%25b%20%25Y%20%25H%3A%25M\">9 Aug 2017 13:06</span> <span style=\"color: green\">+3034</span>**\n",
            "[[/div]]\n",
            "[[/div]]\n",
        );

        let html =
            RenderService::render_wikidot_compatibility_fallback_with_code_blocks(source);

        assert!(html.contains(r#"<div class="list-pages-box">"#));
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
        let source = RenderService::render_wikidot_compat_fallback_css_modules(concat!(
            "[[module CSS]]\n",
            ".scp-pride { display: block; }\n",
            "[[/module]]\n",
            "[[div style=\"font-weight: bold; text-align: center;\"]]\n",
            "[https://example.com keep coming]\n",
            "[[/div]]\n",
        ));

        let html = RenderService::render_wikidot_compatibility_fallback_with_code_blocks(
            &source,
        );

        assert!(html.contains(r#"<style data-wikijump-compat-css-module="1">"#));
        assert!(html.contains(".scp-pride { display: block; }"));
        assert!(html.contains(r#"<div style="font-weight: bold; text-align: center;">"#));
        assert!(!html.contains("[[module CSS"));
        assert!(!html.contains("[[/module]]"));
        assert!(!html.contains("[[div"));
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

        assert!(html.contains(r#"<div class="yui-navset wikidot-compat-tabview">"#));
        assert!(html.contains(r#"<div class="wikidot-compat-tab"><h3>SCPs</h3>"#));
        assert!(html.contains(r#"<span style="font-size: 75%;">"#));
        assert!(html.contains(r#"<div class="image-container aligncenter">"#));
        assert!(html.contains(r#"src="https://scp-wiki.wdfiles.com/local--files/the-great-hippo/hippo2.jpg""#));
        assert!(html.contains(r#"class="image image-size-small""#));
        assert!(!html.contains("[[tabview"));
        assert!(!html.contains("[[tab"));
        assert!(!html.contains("[[size"));
        assert!(!html.contains("[[=image"));
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
    fn removes_unselected_include_comment_branches() {
        let mut wikitext = concat!(
            "Before\n",
            "[!----]\n",
            "[!-- {$inc-hidden}\n",
            "Hidden branch %%title%%\n",
            "[!----]\n",
            "[!-- --]\n",
            "Selected branch body\n",
            "[!----]\n",
            "[!-- {$inc-other}\n",
            "Other hidden branch\n",
            "[!----]\n",
            "After\n",
        )
        .to_owned();

        RenderService::remove_unresolved_include_comment_branches(&mut wikitext);

        assert!(wikitext.contains("Before"));
        assert!(wikitext.contains("Selected branch body"));
        assert!(wikitext.contains("After"));
        assert!(!wikitext.contains("Hidden branch"));
        assert!(!wikitext.contains("Other hidden branch"));
        assert!(!wikitext.contains("[!--"));
        assert!(!wikitext.contains("[!----]"));
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

        assert!(restored.contains(super::WIKIDOT_TABVIEW_SCRIPT));
        assert!(restored.contains(super::WIKIDOT_TABVIEW_INIT_SCRIPT));
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
            r#"<p>[[div id=&quot;unsupported&quot;]]</p>"#,
            r#"<span>Body</span>"#,
            r#"<p>[[/div]]</p>"#,
        );

        let restored =
            RenderService::restore_residual_wikidot_div_paragraph_markers(html);

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
            "</li>",
        );

        let restored = RenderService::resolve_residual_wikidot_simple_if_fragments(html);

        assert_eq!(
            restored,
            r#"<li class="folded active"><a href="javascript:;">+ Open</a><a href="javascript:;">- Close</a><span>Visible</span><span>Shown</span></li>"#,
        );
        assert!(!restored.contains("[[#if"));
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
        assert!(restored.contains("display: none !important"));
        assert!(restored.contains("margin-top: -12rem !important"));
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
    fn removes_wikijump_plain_format_wrappers_after_render() {
        let html = "<p><u>under</u> and <s>strike</s></p>";

        let restored = RenderService::remove_wikijump_plain_format_wrappers(html);

        assert_eq!(restored, "<p>under and strike</p>");
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
