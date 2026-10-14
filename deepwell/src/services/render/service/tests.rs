/*
 * services/render/service/tests.rs
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

use super::super::compat::text_fragments::COMPAT_TEXT_MARKER_PREFIX;
use super::super::iftags::wikidot_tag_conditions_match;
use super::super::list_pages::content_sections::wikidot_content_section;
use super::super::list_pages::scanner::{
    find_list_pages_module_matches, first_list_pages_module_opening_candidate,
    has_count_pages_module_opening_candidate, has_list_pages_module_opening_candidate,
};
use super::super::list_pages::template::ListPagesTemplatePlan;
use super::super::list_pages::{
    AJAX_MODULE_LITERAL_MARKER_PREFIX, ListPagesBatchDisplayRequirements,
    ListPagesExpansionBudget, ListPagesOffsetOrigin, ListPagesPagerRoute,
    ListPagesParentDisplay, ListPagesRuntimeDisplay, ListPagesSnapshotDisplay,
    ListPagesSubstitutionContext, WikidotUserDisplay,
    advance_literal_cursor_and_check_count_pages_capture_containment,
    build_wikidot_list_pages_module_request, build_wikidot_list_pages_module_source,
    count_pages_exact_count_render_diagnostics, count_pages_required_tag_batch_result,
    count_pages_required_tag_batch_selector, count_pages_scan_requires_preservation,
    count_pages_should_remain_literal, count_pages_unbounded_total,
    current_page_info_list_pages_row, exact_name_list_pages_batch_key,
    format_list_pages_created_at, list_pages_author_cache_key,
    list_pages_body_is_no_visible_tracking_markup, list_pages_body_uses_content_variable,
    list_pages_body_variables_supported, list_pages_content_query_target,
    list_pages_feed_info_html, list_pages_has_unsupported_page_type_selector,
    list_pages_has_unsupported_parent_selector, list_pages_parent_fullname,
    list_pages_row_scan_target, list_pages_tag_link_href,
    page_query_cap_requires_original_module, parse_list_pages_arguments,
    parse_list_pages_arguments_with_url, parse_list_pages_date_selector,
    push_list_pages_pager, register_generated_list_pages_html, render_list_pages_tags,
    repair_list_pages_block_boundaries, requested_page_info_score,
    should_render_current_page_list_pages_row, substitute_count_pages_variables,
    substitute_list_pages_variables, substitute_list_pages_variables_with_fragments,
    unsupported_list_pages_replacement, url_offset_list_pages_content_bytes,
};
use super::super::literal_regions::ListPagesSourceProjection;
use super::super::new_page_module::{NewPageTemplateRendering, render_new_page_module};
use super::super::render_options::RenderLifecycle;
use super::super::runtime_page_queries::{
    CountPagesRawScanCompletion, count_pages_raw_scan_completion,
    random_page_query_scan_limit, render_page_query_batch_limit,
    render_page_query_uses_single_scan,
};
use super::{
    AttachmentOwner, AttachmentProvenanceRegistry, AttachmentVariableOwners,
    COUNTPAGES_MODULE_REGEX, CodeBlock, CollectingIncluder, CompatHtmlFragments,
    CompatTextFragments, CorpusReplayExpandedWikitext, CorpusReplayPreparationStage,
    CountPagesRequiredTagBatchResult, IncludeExpansionContext, IncludeSourceCache,
    LiteralRegionIndex, MAX_FTML_COMPAT_COLLAPSIBLE_BLOCKS,
    MAX_FTML_COMPAT_DENSE_PARSE_SCORE, MAX_FTML_COMPAT_PARSE_BYTES,
    MAX_LISTPAGES_CONTENT_ROWS_PER_RENDER, MAX_LISTPAGES_RENDER_OFFSET,
    MAX_LISTPAGES_RENDER_SCAN_ROWS, MAX_NATIVE_LIST_COMPAT_DEPTH,
    MAX_NATIVE_LIST_WIKIDOT_SPAN_NESTING, MIN_DENSE_FTML_COMPAT_RENDER_TIMEOUT_SECS,
    MIN_FTML_COMPAT_TABBED_FALLBACK_BYTES, MIN_FTML_COMPAT_TABBED_FALLBACK_MARKERS,
    MIN_URL_OFFSET_LISTPAGES_CONTENT_BYTES, MIN_URL_OFFSET_LISTPAGES_RENDER_TIMEOUT_SECS,
    PreparedIncluder, RenderContext, RenderService, WIKIDOT_COLOR_SPAN_SENTINEL_PREFIX,
    WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX, WIKIDOT_COMPAT_LINK_SENTINEL_PREFIX,
    WIKIDOT_INLINE_HTML_SENTINEL_PREFIX, WIKIDOT_WIKIPEDIA_LINK_SENTINEL_PREFIX,
    WikidotCompatLinkTitleMap, extract_css_modules as extract_css_modules_with_registry,
    find_balanced_ul_end, has_include_opening_candidate, include_error,
    members_compat_html_fixture, native_list_page_link_default_label,
    parse_wikidot_compat_color_descriptor, protect_forwarded_attachment_variables,
    render_clone_module, render_list_pages_numbered_rows_with_titles,
    render_list_pages_table_rows, render_native_list_inline_wikidot_spans,
    render_native_list_page_link, wikidot_no_such_include_replacement,
};
use crate::config::Config;
use crate::constants::ADMIN_USER_ID;
use crate::models::site::Model as SiteModel;
use crate::services::PageExistenceSnapshot;
use crate::services::page_query::{
    ComparisonOperation, DataFormSelector, DateSelector, DateTimeResolution,
    FoundPageFields, FoundPageRow, MAX_PAGE_QUERY_SCORE_SELECTORS, OrderBySelector,
    OrderProperty, PageQueryResultMetadata, parse_static_wikidot_data_form_values,
    static_wikidot_data_form_matches,
};
use crate::services::render::rate_module::render_read_only_rate_module;
use crate::services::render::render_budget::RenderCostBudget;
use crate::services::render::runtime::IncludeSource;
use crate::services::render::runtime_modules::RateModuleContext;
use crate::services::render::{UrlArgumentPair, UrlArguments};
use crate::services::settings::PageRatingType;
use crate::types::{License, PageId};
use crate::utils::{locale_for_ftml, now};
use ftml::data::PageRef;
use ftml::includes::IncludeRef;
use ftml::layout::Layout;
use ftml::render::{Render, html::HtmlRender};
use ftml::settings::{WikitextMode, WikitextSettings};
use ftml::tree::VariableMap;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

mod fallback;
mod includes;
mod list_pages;
mod markup_compat;
mod postprocess;

#[test]
#[should_panic(expected = "render protection bundle dropped before restoration")]
fn dropping_prepared_wikitext_without_restoration_is_rejected() {
    let source = "plain source";
    let page_info = fallback_test_page_info("guard", "Guard");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let prepared = RenderService::prepare_outer_render_wikitext(
        super::ExpandedRenderWikitext {
            theme_previewer_blank: false,
            wikitext: source.to_owned(),
            included_pages: Vec::new(),
            expanded_include_count: 0,
            url_offset_list_pages_content_bytes: 0,
            runtime_css_insertions: Vec::new(),
            wikidot_compat_html: CompatHtmlFragments::new(source),
            wikidot_compat_text: CompatTextFragments::new(source),
        },
        &page_info,
        &settings,
    );
    drop(prepared);
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
fn scp9506_full_seed_survives_the_ftml_wikidot_pipeline() {
    let mut source = include_str!("../../../../seeder/scp-9506.ftml").to_owned();
    assert!(
        source.contains("San José Public Library"),
        "the full-page canary must retain the multibyte quoted-line witness"
    );

    let page_info = fallback_test_page_info("scp-9506", "National Fog Safety Initiative");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    ftml::preprocess_for_layout(&mut source, settings.layout);
    let tokens = ftml::tokenize(&source);
    let (tree, _errors) = ftml::parse(&tokens, &page_info, &settings).into();
    let rendered = HtmlRender.render(&tree, &page_info, &settings).body;

    assert!(
        rendered.contains("San José Public Library"),
        "the full page must reach HTML rendering without losing the multibyte canary"
    );
}

fn prepare_test_wikidot_conditionals(
    wikitext: &mut String,
    page_info: &ftml::data::PageInfo<'_>,
) {
    let mut preserved = CompatTextFragments::new(wikitext);
    RenderService::prepare_wikidot_conditionals_for_include_expansion(
        wikitext,
        page_info,
        &mut preserved,
    );
    *wikitext = preserved.restore(wikitext);
}

fn prepare_test_wikidot_conditionals_before_include_expansion(
    wikitext: &mut String,
    page_info: &ftml::data::PageInfo<'_>,
) {
    let mut preserved = CompatTextFragments::new(wikitext);
    RenderService::prepare_wikidot_conditionals_before_include_expansion(
        wikitext,
        page_info,
        &mut preserved,
        0,
    );
    *wikitext = preserved.restore(wikitext);
}

fn resolve_test_wikidot_iftags(
    wikitext: &mut String,
    page_info: &ftml::data::PageInfo<'_>,
) {
    let mut preserved = CompatTextFragments::new(wikitext);
    RenderService::resolve_wikidot_iftags(wikitext, page_info, &mut preserved);
    *wikitext = preserved.restore(wikitext);
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
    let fragments =
        RenderService::protect_generated_wikidot_compat_html(&mut wikitext, &settings);
    ftml::preprocess_for_layout(&mut wikitext, settings.layout);
    let tokens = ftml::tokenize(&wikitext);
    let result = ftml::parse(&tokens, &page_info, &settings);
    let (tree, _) = result.into();
    let rendered = HtmlRender.render(&tree, &page_info, &settings).body;
    RenderService::restore_protected_generated_wikidot_compat_html(rendered, &fragments)
}

fn render_saved_wikidot_page_body(page: &'static str, wikitext: &str) -> String {
    let page_info = fallback_test_page_info(page, page);
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut wikitext = wikitext.to_owned();
    ftml::preprocess_for_layout(&mut wikitext, settings.layout);
    let tokens = ftml::tokenize(&wikitext);
    let (tree, errors) = ftml::parse(&tokens, &page_info, &settings).into();
    assert!(errors.is_empty(), "{errors:#?}");
    HtmlRender.render(&tree, &page_info, &settings).body
}

#[test]
fn saved_page_bare_attachment_images_follow_wikidot_file_type_default() {
    // Saved Wikidot pages (issue #2349): scp-9506 line 1, 1-hachiro, and
    // 12-aromantic-abatement render their bare .png attachments as the plain
    // original; the shared-corpus saved HTML renders .jpeg/.webp/.svg
    // attachments as the linked medium.jpg resize.
    for (page, source, original) in [
        (
            "scp-9506",
            r#"[[image NFSI.png class="crom-thumbnail" style="display: none"]]"#,
            "https://scp-wiki.wjfiles.com/local--files/scp-9506/NFSI.png",
        ),
        (
            "1-hachiro",
            "[[=image Hachiro.png]]",
            "https://scp-wiki.wjfiles.com/local--files/1-hachiro/Hachiro.png",
        ),
        (
            "12-aromantic-abatement",
            "[[f>image AAG.png]]",
            "https://scp-wiki.wjfiles.com/local--files/12-aromantic-abatement/AAG.png",
        ),
    ] {
        let html = render_saved_wikidot_page_body(page, source);
        assert!(
            html.contains(&format!(r#"<img src="{original}""#)),
            "{html}"
        );
        assert!(!html.contains("local--resized-images"), "{html}");
        assert!(!html.contains("<a "), "{html}");
    }

    let html = render_saved_wikidot_page_body("scp-4875", "[[image ATACAMA.jpeg]]");
    assert_eq!(
        html,
        concat!(
            r#"<a href="https://scp-wiki.wjfiles.com/local--files/scp-4875/ATACAMA.jpeg">"#,
            r#"<img src="https://scp-wiki.wjfiles.com/local--resized-images/scp-4875/ATACAMA.jpeg/medium.jpg" class="image" alt="ATACAMA.jpeg">"#,
            "</a>",
        ),
    );
}

fn render_trusted_wikidot_block_html(html: String) -> String {
    let page_info = fallback_test_page_info("scp-7243", "SCP-7243");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut fragments = CompatHtmlFragments::new("");
    let mut wikitext = fragments.push_block_html(html);
    ftml::preprocess_for_layout(&mut wikitext, settings.layout);
    let tokens = ftml::tokenize(&wikitext);
    let result = ftml::parse(&tokens, &page_info, &settings);
    let (tree, errors) = result.into();
    assert!(errors.is_empty(), "{errors:#?}");
    let rendered = HtmlRender.render(&tree, &page_info, &settings).body;
    fragments.restore(&rendered)
}

fn render_wikidot_conditionals_with_tags(wikitext: &str, tags: &[&str]) -> String {
    let page_info = ftml::data::PageInfo {
        tags: tags
            .iter()
            .map(|tag| Cow::Owned((*tag).to_owned()))
            .collect(),
        ..fallback_test_page_info("conditional", "Conditional")
    };
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let outer = RenderService::prepare_outer_render_wikitext(
        super::ExpandedRenderWikitext {
            theme_previewer_blank: false,
            wikidot_compat_html: CompatHtmlFragments::new(wikitext),
            wikidot_compat_text: CompatTextFragments::new(wikitext),
            wikitext: wikitext.to_owned(),
            included_pages: Vec::new(),
            expanded_include_count: 0,
            url_offset_list_pages_content_bytes: 0,
            runtime_css_insertions: Vec::new(),
        },
        &page_info,
        &settings,
    );
    let inner = RenderService::prepare_inner_render_wikitext(outer, &settings);
    let tokens = ftml::tokenize(&inner.wikitext);
    let (tree, errors) = ftml::parse(&tokens, &page_info, &settings).into();
    assert!(errors.is_empty(), "{wikitext:?}: {errors:#?}");
    let html = HtmlRender.render(&tree, &page_info, &settings).body;
    inner
        .protection
        .restore(|protection| protection.compat_text().restore(&html))
}

#[test]
fn compatibility_protection_preserves_ftml_lexical_ownership() {
    for (source, expected) in [
        ("##|A##", "<p>##|A##</p>"),
        ("##red|A", "<p>##red|A</p>"),
        (
            "##url(javascript:alert(1))|A##",
            "<p>##url(javascript:alert(1))|A##</p>",
        ),
        ("{{$x}}", "<p><tt>$x</tt></p>"),
    ] {
        assert_eq!(
            render_wikidot_conditionals_with_tags(source, &[]),
            expected,
            "{source:?}",
        );
    }
}

fn render_wikidot_fallback_after_generated_compat_restore(wikitext: &str) -> String {
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut wikitext = wikitext.to_owned();
    let fragments =
        RenderService::protect_generated_wikidot_compat_html(&mut wikitext, &settings);
    let rendered =
        RenderService::render_wikidot_compatibility_fallback_with_code_blocks(&wikitext);

    RenderService::restore_protected_generated_wikidot_compat_html(rendered, &fragments)
}

fn render_wikidot_css_after_extraction(
    wikitext: &str,
    fallback: bool,
) -> (String, Vec<String>) {
    let page_info = fallback_test_page_info("css", "CSS");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut protected = wikitext.to_owned();
    let styles = extract_wikidot_css_modules(&mut protected, &settings);
    let rendered = if fallback {
        RenderService::render_wikidot_compatibility_fallback_with_code_blocks(&protected)
    } else {
        ftml::preprocess_for_layout(&mut protected, settings.layout);
        let tokens = ftml::tokenize(&protected);
        let (tree, _) = ftml::parse(&tokens, &page_info, &settings).into();
        HtmlRender.render(&tree, &page_info, &settings).body
    };
    (rendered, styles)
}

fn extract_wikidot_css_modules(
    wikitext: &mut String,
    settings: &WikitextSettings,
) -> Vec<String> {
    let page_info = fallback_test_page_info("css", "CSS");
    let mut compat_html = CompatHtmlFragments::new(wikitext);
    extract_css_modules_with_registry(wikitext, &page_info, settings, &mut compat_html)
}

#[test]
fn renders_nested_plain_parentheses_directly_through_ftml() {
    let rendered = render_wikidot_page_body_after_compat_restore("before (a (b)) after");

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

    assert!(rendered.contains(r#"class="bibcite""#), "{rendered}");
    assert!(rendered.contains("before (a (b)) after"));
}

#[test]
fn preserves_unclosed_bibliography_after_footnote() {
    let rendered = render_wikidot_page_body_after_compat_restore(
        "A claim[[footnote]]with a note[[/footnote]].\n\n[[bibliography]]",
    );

    assert!(rendered.contains("<p>[[bibliography]]</p>"), "{rendered}");
    assert!(
        rendered.contains(r#"<div class="footnotes-footer">"#),
        "{rendered}"
    );
    assert!(
        !rendered.contains(r#"<div class="bibitems">"#),
        "{rendered}"
    );
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
fn include_preflight_requires_an_include_block_candidate() {
    assert!(has_include_opening_candidate("[[include component:thing]]"));
    assert!(has_include_opening_candidate(
        "[[  InClUdE component:thing]]"
    ));
    assert!(!has_include_opening_candidate("include component:thing"));
    assert!(!has_include_opening_candidate("[[module css]] .include {}"));
}

#[test]
fn renders_wikidot_read_only_rate_module_with_downvote() {
    let rendered = render_read_only_rate_module(
        ftml::data::ScoreValue::Integer(19),
        "en",
        PageRatingType::PlusMinus,
    );

    assert!(rendered.contains("<span class=\"rate-points\">rating:\u{00a0}"));
    assert!(rendered.contains(r#"<span class="number prw54353">+19</span>"#));
    assert!(rendered.contains(r#"<span class="rateup btn btn-default">"#));
    assert!(rendered.contains(r#"listeners.rate(event, 1)"#));
    assert!(rendered.contains(r#"</span><span class="rateup btn btn-default">"#));
    assert!(rendered.contains(r#"<span class="ratedown btn btn-default">"#));
    assert!(rendered.contains(r#"listeners.rate(event, -1)"#));
    assert!(rendered.contains(r#"</span><span class="ratedown btn btn-default">"#));
    assert!(rendered.contains(r#"title="I don't like it">–</a>"#));
    assert!(rendered.contains(r#"<span class="cancel btn btn-default">"#));
    assert!(rendered.contains(r#"listeners.cancelVote(event)"#));
    assert!(rendered.contains(r#"</span><span class="cancel btn btn-default">"#));
}

#[test]
fn renders_wikidot_plus_only_rate_module_without_downvote() {
    let rendered = render_read_only_rate_module(
        ftml::data::ScoreValue::Integer(0),
        "en",
        PageRatingType::Plus,
    );

    assert!(rendered.contains(r#"listeners.rate(event, 1)"#));
    assert!(!rendered.contains("ratedown"));
    assert!(!rendered.contains("rate(event, -1)"));
    assert!(rendered.contains(r#"listeners.cancelVote(event)"#));
}

#[test]
fn renders_japanese_wikidot_read_only_rate_module_labels() {
    let rendered = render_read_only_rate_module(
        ftml::data::ScoreValue::Integer(35),
        "ja",
        PageRatingType::PlusMinus,
    );

    assert!(rendered.contains("<span class=\"rate-points\">評価:\u{00a0}"));
    assert!(rendered.contains(r#"<span class="number prw54353">+35</span>"#));
    assert!(rendered.contains(r#"title="好き""#));
    assert!(rendered.contains(r#"title="好きじゃない""#));
    assert!(rendered.contains(r#"title="投票を取り消す""#));
}

#[test]
fn protects_wikidot_members_module_html_before_parsing() {
    let mut wikitext = members_compat_html_fixture("moderators");
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
        members_compat_html_fixture("moderators"),
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
        "[[module NewPage]]".to_owned(),
        &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
    );

    assert!(rendered.contains(
        r#"<div class="new-page-box" style="text-align: center; margin: 1em 0;">"#
    ));
    assert!(rendered.contains(r#"<form action="dummy.html" method="get" onsubmit="WIKIDOT.modules.NewPageHelperModule.listeners.create(event);">"#));
    assert!(
        rendered.contains(r#"<input class="text" name="pageName" type="text" size="30" maxlength="128" style="margin: 1px" aria-label="Name of the new page"/>"#)
    );
    assert!(
        rendered.contains(r#"<input type="submit" class="button" value="Create page" style="margin: 1px;"/>"#)
    );
    assert!(!rendered.contains("[[module NewPage"));
}

#[test]
fn multiple_new_page_modules_have_independent_accessible_names_without_ids() {
    let rendered = RenderService::expand_new_page_modules(
        "[[module NewPage]]\n[[module NewPage size=15]]".to_owned(),
        &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
    );

    assert_eq!(
        rendered
            .matches(r#"aria-label="Name of the new page""#)
            .count(),
        2
    );
    assert_eq!(rendered.matches(r#"name="pageName""#).count(), 2);
    assert!(!rendered.contains("id="));
}

#[test]
fn renders_wikidot_new_page_module_documented_hidden_fields() {
    let rendered = RenderService::expand_new_page_modules(
        r#"[[module NewPage size="30" category="band" parent="bands" tags="rock" format="/^[0-9]{5}$/" mode="save-and-go" goTo="target" button="Add a new rock band"]]"#
            .to_owned(),
        &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
    );

    assert!(
        rendered.contains(r#"<input type="hidden" name="mode" value="save-and-go"/>"#)
    );
    assert!(rendered.contains(r#"<input type="hidden" name="goTo" value="target"/>"#));
    assert!(
        rendered.contains(r#"<input type="hidden" name="categoryName" value="band"/>"#)
    );
    assert!(
        rendered.contains(r#"<input type="hidden" name="format" value="/^[0-9]{5}$/"/>"#)
    );
    assert!(rendered.contains(r#"<input type="hidden" name="tags" value="rock"/>"#));
    assert!(rendered.contains(r#"<input type="hidden" name="parent" value="bands"/>"#));
    assert!(
        rendered.contains(r#"<input type="submit" class="button" value="Add a new rock band" style="margin: 1px;"/>"#)
    );
}

#[test]
fn renders_wikidot_new_page_module_uses_live_argument_quirks() {
    let rendered = RenderService::expand_new_page_modules(
        r#"[[module NewPage size="999" button="   " SIZE="15" button='ignored' category=doc]]"#
            .to_owned(),
        &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
    );

    assert!(
        rendered.contains(r#"<input class="text" name="pageName" type="text" size="999" maxlength="128" style="margin: 1px" aria-label="Name of the new page"/>"#)
    );
    assert!(rendered.contains(
        r#"<input type="submit" class="button" value="   " style="margin: 1px;"/>"#
    ));
    assert!(!rendered.contains(r#"name="categoryName""#));
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
fn renders_wikidot_join_module_anonymous_dom() {
    let rendered = RenderService::expand_join_modules(
        concat!(
            "[[module Join]]\n",
            "[[module Join button=\"Custom <join>\" class=\"join-module\" ",
            "id=\"ignored\" style=\"display: flex\"]]",
        )
        .to_owned(),
        &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
    );

    assert!(rendered.contains(r#"<div class="join-box">"#));
    assert!(rendered.contains(r#"<div class="join-module">"#));
    assert!(rendered.contains("Custom &lt;join&gt;"));
    assert!(!rendered.contains("id=\"ignored\""));
    assert!(!rendered.contains("display: flex"));
    assert_eq!(rendered.matches("WIKIDOT.page.listeners.join").count(), 2);
}

#[test]
fn clone_html_is_registered_only_by_its_runtime_producer() {
    let source = "[[module Clone button=\"Clone <now>\"]]";
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut fragments = CompatHtmlFragments::new(source);
    let protected = RenderService::expand_registry_modules_with_registry(
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
fn registry_module_expansion_does_not_match_name_prefixes() {
    let source = concat!(
        "[[module MembershipByPassword]] ",
        "[[module MembershipEmailInvitation]] ",
        "[[module NewPageExtra]] ",
        "[[module Joinery]] ",
        "[[module CloneExtra]] ",
        "[[module Clone]]",
    );
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut fragments = CompatHtmlFragments::new(source);
    let protected = RenderService::expand_registry_modules_with_registry(
        source.to_owned(),
        &settings,
        &mut fragments,
    );

    assert_eq!(
        protected
            .matches(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX)
            .count(),
        1,
        "only the exact Clone module should be expanded:\n{protected}",
    );
    assert!(protected.contains("[[module MembershipByPassword]]"));
    assert!(protected.contains("[[module MembershipEmailInvitation]]"));
    assert!(protected.contains("[[module NewPageExtra]]"));
    assert!(protected.contains("[[module Joinery]]"));
    assert!(protected.contains("[[module CloneExtra]]"));

    let restored = fragments.restore(&protected);
    assert!(restored.contains(r#"data-wikijump-compat-clone="1""#));
    assert!(!restored.contains("[[module Clone]]"));
    assert!(!restored.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
}

#[test]
fn registry_module_expansion_ignores_literal_attribute_and_comment_occurrences() {
    let modules = "[[module Clone]]";
    let source = format!(
        concat!(
            "@@{modules}@@\n",
            "[[code]]\n{modules}\n[[/code]]\n",
            "[[raw]]\n{modules}\n[[/raw]]\n",
            "[!-- {modules} --]\n",
            "[[div data-module=\"[[module Clone]]\"]]clone[[/div]]\n",
            "<div data-module=\"[[module Clone]]\">clone</div>\n",
            "<pre>{modules}</pre>\n",
            "<!-- {modules} -->\n",
            "[[module Clone button=\"clone-first\"]]\n",
        ),
        modules = modules
    );
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut fragments = CompatHtmlFragments::new(&source);
    let protected = RenderService::expand_registry_modules_with_registry(
        source,
        &settings,
        &mut fragments,
    );

    assert_eq!(
        protected
            .matches(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX)
            .count(),
        1,
    );
    assert_eq!(protected.matches("[[module Clone]]").count(), 8);

    let restored = fragments.restore(&protected);
    assert!(restored.contains("clone-first"));
    assert!(!restored.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));

    let mut output =
        RenderService::render_wikidot_compatibility_fallback_output_for_context(
            &protected,
            Some("module-literal-boundary"),
            Some("scp-wiki"),
            None,
        );
    output.body = fragments.restore(&output.body);
    assert!(!output.body.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
}

#[test]
fn registry_module_expansion_does_not_reclassify_a_later_literal_candidate() {
    let source = r#"[[module Clone button="@@"]][[module NewPage]]@@"#;
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut fragments = CompatHtmlFragments::new(source);
    let protected = RenderService::expand_registry_modules_with_registry(
        source.to_owned(),
        &settings,
        &mut fragments,
    );

    assert_eq!(
        protected
            .matches(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX)
            .count(),
        1,
    );
    assert!(protected.contains("[[module NewPage]]@@"));

    let restored = fragments.restore(&protected);
    assert!(restored.contains(r#"<a class="button""#));
    assert!(restored.contains("[[module NewPage]]@@"));
    assert!(!restored.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
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
        restored.contains(r#"<a class="button" href="javascript:;">Clone this site</a>"#)
    );
    assert!(!restored.contains("data-wikijump-compat-clone"));
}

#[test]
fn protects_wikidot_new_page_module_html_before_parsing() {
    let mut wikitext = render_new_page_module(
        r#" size="15" button="new <page>""#,
        NewPageTemplateRendering::None,
        "Name of the new page",
    )
    .replacen(
        r#"<div class="new-page-box""#,
        r#"<div class="new-page-box" data-wikijump-compat-new-page="1""#,
        1,
    );
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
        r#"<div class="new-page-box" style="text-align: center; margin: 1em 0;">"#
    ));
    assert!(restored.contains(r#"value="new &lt;page&gt;""#));
    assert!(!restored.contains("data-wikijump-compat-new-page"));
}

#[test]
fn authored_compat_markers_are_neutralized_before_html_protection() {
    let forgeries = [
        r#"<table class="wiki-content-table" data-wikijump-compat-listpages="1"><tr><td><img src=x onerror="alert(1)"></td></tr></table>"#,
        r#"<ol data-wikijump-compat-listpages="1"><li><img src=x onerror="alert(1)"></li></ol>"#,
        r#"<ul data-wikijump-compat-list="1"><li><img src=x onerror="alert(1)"></li></ul>"#,
        r#"<div id="ml-1" data-wikijump-compat-members="1"><img src=x onerror="alert(1)"></div>"#,
        r#"<div class="backlinks-module-box" data-wikijump-compat-backlinks="1"><img src=x onerror="alert(1)"></div>"#,
        r#"<div class="new-page-box" data-wikijump-compat-new-page="1"><img src=x onerror="alert(1)"></div>"#,
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
    let rendered =
        members_compat_html_fixture("</script><img src=x onerror='alert(1)'>&\u{2028}");

    assert_eq!(rendered.matches("</script>").count(), 1);
    assert!(!rendered.contains("</script><img"));
    assert!(rendered.contains(r#"\x3C/script\x3E\x3Cimg"#));
    assert!(rendered.contains(r#"\x26\u2028"#));
}

#[test]
fn protects_only_generated_plain_text_wikidot_date_html() {
    let mut wikitext = r#"<span class="odate time_-123 format_%25Y%20%25b%20%25e" data-wikijump-compat-date="1">1 Jan &amp; 1970</span>"#.to_owned();
    let fragments = RenderService::protect_generated_wikidot_compat_html(
        &mut wikitext,
        &WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot),
    );

    assert_eq!(fragments.len(), 1);
    assert!(wikitext.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX));
    assert!(!fragments[0].html.contains("data-wikijump-compat-date"));
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
fn optional_no_visible_wikidot_includes_do_not_render_missing_page_text() {
    assert_eq!(
        wikidot_no_such_include_replacement(&PageRef::page_and_site("drizzles", "raven")),
        "",
    );
    assert_eq!(
        wikidot_no_such_include_replacement(&PageRef::page_and_site("crom", "pixel")),
        "",
    );
    assert_eq!(
        wikidot_no_such_include_replacement(&PageRef::page_and_site("scp-jp", "missing")),
        "[[div class=\"error-block\"]]\nIncluded page \"missing\" does not exist ([[a href=\"http://scp-jp.wikidot.com/missing/edit/true\"]]create it now[[/a]])\n[[/div]]",
    );

    let replacement = wikidot_no_such_include_replacement(&PageRef::page_only("banana"));
    assert_eq!(
        render_wikidot_page_body_after_compat_restore(&replacement),
        r#"<div class="error-block"><p>Included page &quot;banana&quot; does not exist (<a href="/banana/edit/true">create it now</a>)</p></div>"#,
    );
}

#[test]
fn missing_cross_site_include_uses_raw_page_for_display_and_canonical_ref_for_lookup() {
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let source = "[[include :scp-wiki:deleted:protected:component:magic]]";
    let mut display_pages = super::collect_include_display_pages(source);
    let canonical =
        PageRef::page_and_site("scp-wiki", "deleted:protected:component:magic");
    let includes = vec![IncludeRef::page_only(canonical.clone())];
    let mut compat_text = CompatTextFragments::new(source);
    let missing_replacements = super::collect_missing_include_replacements(
        &includes,
        &[None],
        &mut display_pages,
        &mut compat_text,
    );
    let (expanded, included_pages) = ftml::include(
        source,
        &settings,
        PreparedIncluder {
            pages: vec![None],
            missing_replacements,
        },
        include_error,
    )
    .expect("missing include should render a live-compatible error");
    assert_eq!(included_pages, vec![canonical]);
    assert_eq!(
        expanded,
        concat!(
            "[[div class=\"error-block\"]]\n",
            "Included page \"deleted:protected:component:magic\" does not exist ",
            "([[a href=\"http://scp-wiki.wikidot.com/deleted:protected:component:magic/edit/true\"]]",
            "create it now[[/a]])\n",
            "[[/div]]",
        ),
    );
}

#[test]
fn missing_include_with_spaced_empty_separator_matches_live_browser_dom() {
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let page_info = fallback_test_page_info("debug", "Debug");
    let source = "[[include PAGE | ]]";
    let mut display_pages = super::collect_include_display_pages(source);
    let includes = vec![
        IncludeRef::page_only(PageRef::page_only("PAGE"))
            .with_spaced_empty_separator(true),
    ];
    let mut compat_text = CompatTextFragments::new(source);
    let missing_replacements = super::collect_missing_include_replacements(
        &includes,
        &[None],
        &mut display_pages,
        &mut compat_text,
    );
    let (mut expanded, _) = ftml::include(
        source,
        &settings,
        PreparedIncluder {
            pages: vec![None],
            missing_replacements,
        },
        include_error,
    )
    .expect("missing include should expand");

    ftml::preprocess_for_layout(&mut expanded, settings.layout);
    let tokens = ftml::tokenize(&expanded);
    let (tree, _) = ftml::parse(&tokens, &page_info, &settings).into();
    let rendered = HtmlRender.render(&tree, &page_info, &settings).body;
    let restored =
        RenderService::restore_wikidot_render_compatibility_for_context_with_resources(
            &rendered,
            None,
            &Config::integration_testing(),
            true,
            &[],
        );
    let restored = compat_text.restore(&restored);

    assert_eq!(
        restored,
        concat!(
            "<p>[[div class=&quot;error-block&quot;]]<br>\n",
            "Included page &quot;page&quot; does not exist ",
            "(<a href=\"/page/edit/true\">create it now</a>)</p>",
        ),
    );
}

#[test]
fn page_view_and_nav_contexts_keep_current_page_without_text_block_target() {
    let expected = RenderContext {
        current_site_id: Some(7),
        current_category_id: Some(9),
        current_page_id: Some(11),
        text_block_page_id: None,
        lifecycle: RenderLifecycle::SavedPage,
        suppress_nested_list_pages: false,
    };

    for context in [
        RenderContext::page_view(7, 9, 11),
        RenderContext::page_nav(7, 9, 11),
    ] {
        assert_eq!(context, expected);
    }
}

#[test]
fn page_render_context_uses_current_page_as_text_block_target() {
    assert_eq!(
        RenderContext::page(7, 9, 11),
        RenderContext {
            current_site_id: Some(7),
            current_category_id: Some(9),
            current_page_id: Some(11),
            text_block_page_id: Some(11),
            lifecycle: RenderLifecycle::SavedPage,
            suppress_nested_list_pages: false,
        },
    );
}

#[test]
fn wikidot_compatibility_fallback_centers_read_only_rate_module() {
    let source = "[[=]]\n[[module Rate]]\n[[/=]]\n";
    let mut page_info = fallback_test_page_info("scp-9506", "SCP-9506");
    page_info.score = ftml::data::ScoreValue::Integer(396);
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut fragments = CompatHtmlFragments::new(source);
    let mut compat_text = CompatTextFragments::new(source);
    let protected = RenderService::expand_rate_modules_with_registry(
        source.to_owned(),
        &page_info,
        &settings,
        RateModuleContext {
            rating_type: PageRatingType::PlusMinus,
            score: page_info.score,
            rating_votes: None,
        },
        &mut fragments,
        &mut compat_text,
    );

    let mut output =
        RenderService::render_wikidot_compatibility_fallback_output_for_context(
            &protected,
            Some("scp-anthology-2024"),
            Some("scp-wiki"),
            None,
        );
    output.body = fragments.restore(&output.body);

    assert!(output.body.contains(
        r#"<div style="text-align: center;"><div class="page-rate-widget-box">"#
    ));
    assert!(output.body.contains("<span class=\"rate-points\">rating:\u{00a0}<span class=\"number prw54353\">+396</span></span>"));
    assert!(output.body.contains(r#"<span class="rateup btn btn-default"><a href="javascript:;" onclick="WIKIDOT.modules.PageRateWidgetModule.listeners.rate(event, 1)" title="I like it">+</a></span>"#));
    assert!(output.body.contains("</div></div>"));
    assert!(
        !output
            .body
            .contains(r#"<div class="page-rate-widget-box"><p>"#)
    );
    assert!(
        !output
            .body
            .contains(r#"<a href="javascript:;"><span class="rateup"#)
    );
    assert_eq!(output.body.matches(r#"class="rate-points""#).count(), 1);
    assert!(!output.body.contains("[[=]]"));
    assert!(!output.body.contains("[[/=]]"));
}

#[test]
fn rate_module_plusminus_body_is_consumed_like_live_wikidot() {
    let source = concat!(
        "[[module Rate]]\n",
        "Average Rating %%rating%% from %%rating_votes%% votes percent=%%rating_percent%% decimal=%%rating_decimal%%\n",
        "[[/module]]",
    );
    let page_info = fallback_test_page_info("rate-plusminus-body", "Rate plusminus body");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut fragments = CompatHtmlFragments::new(source);
    let mut compat_text = CompatTextFragments::new(source);
    let protected = RenderService::expand_rate_modules_with_registry(
        source.to_owned(),
        &page_info,
        &settings,
        RateModuleContext {
            rating_type: PageRatingType::PlusMinus,
            score: page_info.score,
            rating_votes: Some(0),
        },
        &mut fragments,
        &mut compat_text,
    );

    let mut output =
        RenderService::render_wikidot_compatibility_fallback_output_for_context(
            &protected,
            Some("rate-plusminus-body"),
            Some("scp-wiki"),
            None,
        );
    output.body = fragments.restore(&output.body);

    assert_eq!(
        output
            .body
            .matches(r#"class="page-rate-widget-box""#)
            .count(),
        1
    );
    assert!(output.body.contains(r#"class="ratedown btn btn-default""#));
    assert!(!output.body.contains("Average Rating"));
    assert!(!output.body.contains("%%rating"));
    assert!(!output.body.contains("[[/module]]"));
}

#[test]
fn rate_module_balanced_scope_does_not_bind_nested_closer_to_outer_rate() {
    let source = concat!(
        "[[module Rate]]\n",
        "body before a later rate head\n",
        "[[module Rate]]\n",
        "body after the later rate head\n",
        "[[/module]]\n",
        "visible tail\n",
    );
    let page_info = fallback_test_page_info("rate-overlap", "Rate overlap");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut fragments = CompatHtmlFragments::new(source);
    let mut compat_text = CompatTextFragments::new(source);

    let protected = RenderService::expand_rate_modules_with_registry(
        source.to_owned(),
        &page_info,
        &settings,
        RateModuleContext {
            rating_type: PageRatingType::PlusMinus,
            score: page_info.score,
            rating_votes: Some(0),
        },
        &mut fragments,
        &mut compat_text,
    );

    assert_eq!(protected.matches("WIKIJUMPWIKIDOTCOMPATHTML").count(), 2,);
    assert!(protected.ends_with("\nvisible tail\n"), "{protected}");
    assert!(!protected.contains("[[module Rate]]"), "{protected}");
    assert!(
        protected.contains("body before a later rate head"),
        "{protected}"
    );
}

#[test]
fn rate_module_block_fragment_restores_only_at_root_and_div_contexts() {
    let source = "[[module Rate]]\n";
    let mut page_info = fallback_test_page_info("scp-9506", "SCP-9506");
    page_info.score = ftml::data::ScoreValue::Integer(396);
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut fragments = CompatHtmlFragments::new(source);
    let mut compat_text = CompatTextFragments::new(source);
    let protected = RenderService::expand_rate_modules_with_registry(
        source.to_owned(),
        &page_info,
        &settings,
        RateModuleContext {
            rating_type: PageRatingType::PlusMinus,
            score: page_info.score,
            rating_votes: None,
        },
        &mut fragments,
        &mut compat_text,
    );

    let root = fragments.restore(&format!("<p>{protected}</p>"));
    assert!(root.contains(r#"<div class="page-rate-widget-box">"#));
    assert!(!root.contains("<p><div"));

    let div = fragments.restore(&format!(
        "<div class=\"rate-shell\"><p>{protected}</p></div>"
    ));
    assert!(
        div.contains(r#"<div class="rate-shell"><div class="page-rate-widget-box">"#)
    );
    assert!(!div.contains("<p><div"));

    assert_eq!(
        fragments.restore(&format!("<span><p>{protected}</p></span>")),
        format!("<span><p>{protected}</p></span>"),
    );
}

fn render_rate_module_through_ftml(source: &str) -> String {
    let mut page_info = fallback_test_page_info("rate-containers", "Rate containers");
    page_info.score = ftml::data::ScoreValue::Integer(0);
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut fragments = CompatHtmlFragments::new(source);
    let mut compat_text = CompatTextFragments::new(source);
    let mut protected = RenderService::expand_rate_modules_with_registry(
        source.to_owned(),
        &page_info,
        &settings,
        RateModuleContext {
            rating_type: PageRatingType::PlusMinus,
            score: page_info.score,
            rating_votes: None,
        },
        &mut fragments,
        &mut compat_text,
    );
    ftml::preprocess_for_layout(&mut protected, settings.layout);
    let tokens = ftml::tokenize(&protected);
    let (tree, _) = ftml::parse(&tokens, &page_info, &settings).into();
    let rendered = HtmlRender.render(&tree, &page_info, &settings).body;
    fragments.restore(&rendered)
}

#[test]
fn rate_module_renders_directly_inside_header_cells_and_list_items() {
    // Anonymous Wikidot PagePreviewModule (scp-wiki, 2026-10-11, issue #2348):
    // [[module Rate]] inside [[hcell]], [[cell]], [[ul]][[li]], and [[ol]][[li]]
    // renders <th>/<td>/<li><div class="page-rate-widget-box">…</div></th|td|li>
    // with no paragraph between the container and the widget.
    for (source, parent) in [
        (
            "[[table]]\n[[row]]\n[[hcell]]\n[[module Rate]]\n[[/hcell]]\n[[/row]]\n[[/table]]\n",
            "<th>",
        ),
        (
            "[[table]]\n[[row]]\n[[hcell class=\"namerow\" colspan=\"2\"]]\n[[module Rate]]\n[[/hcell]]\n[[/row]]\n[[/table]]\n",
            r#"<th class="namerow" colspan="2">"#,
        ),
        (
            "[[table]]\n[[row]]\n[[cell]]\n[[module Rate]]\n[[/cell]]\n[[/row]]\n[[/table]]\n",
            "<td>",
        ),
        (
            "[[ul]]\n[[li]]\n[[module Rate]]\n[[/li]]\n[[/ul]]\n",
            "<li>",
        ),
        (
            "[[ol]]\n[[li]]\n[[module Rate]]\n[[/li]]\n[[/ol]]\n",
            "<li>",
        ),
    ] {
        let rendered = render_rate_module_through_ftml(source);

        assert!(
            rendered.contains(&format!(r#"{parent}<div class="page-rate-widget-box">"#)),
            "{source}\n{rendered}"
        );
        assert_eq!(
            rendered.matches(r#"class="page-rate-widget-box""#).count(),
            1,
            "{rendered}"
        );
        assert!(
            !rendered.contains(WIKIDOT_COMPAT_HTML_SENTINEL_PREFIX),
            "{source}\n{rendered}"
        );
    }
}

#[test]
fn rate_module_expansion_leaves_wikidot_quote_depths_literal() {
    let source = concat!(
        "> [[module Rate show=\"DEPTH_ONE\"]]\n",
        ">> [[module Rate show=\"DEPTH_TWO\"]]\n",
        "[[module Rate]]\n",
    );
    let page_info = fallback_test_page_info("rate-quotes", "Rate quotes");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut fragments = CompatHtmlFragments::new(source);
    let mut compat_text = CompatTextFragments::new(source);
    let protected = RenderService::expand_rate_modules_with_registry(
        source.to_owned(),
        &page_info,
        &settings,
        RateModuleContext {
            rating_type: PageRatingType::Plus,
            score: page_info.score,
            rating_votes: None,
        },
        &mut fragments,
        &mut compat_text,
    );

    assert!(protected.contains("[[module Rate show=\"DEPTH_ONE\"]]"));
    assert!(protected.contains("[[module Rate show=\"DEPTH_TWO\"]]"));
    assert_eq!(protected.matches("WIKIJUMPWIKIDOTCOMPATHTML").count(), 1);
}

#[test]
fn rate_module_expansion_leaves_footnote_body_literal() {
    let source = "[[footnote]][[module Rate]][[/footnote]]";
    let page_info = fallback_test_page_info("rate-footnote", "Rate footnote");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut fragments = CompatHtmlFragments::new(source);
    let mut compat_text = CompatTextFragments::new(source);
    let protected = RenderService::expand_rate_modules_with_registry(
        source.to_owned(),
        &page_info,
        &settings,
        RateModuleContext {
            rating_type: PageRatingType::PlusMinus,
            score: page_info.score,
            rating_votes: None,
        },
        &mut fragments,
        &mut compat_text,
    );

    assert_ne!(protected, source);
    let mut protected = protected;
    ftml::preprocess_for_layout(&mut protected, settings.layout);
    let tokens = ftml::tokenize(&protected);
    let result = ftml::parse(&tokens, &page_info, &settings);
    let (tree, _) = result.into();
    let rendered = HtmlRender.render(&tree, &page_info, &settings).body;
    let rendered = fragments.restore(&rendered);
    let rendered = compat_text.restore(&rendered);

    assert!(rendered.contains("1</a>. [[module Rate]]"), "{rendered}",);
    assert!(!rendered.contains(r#"class="page-rate-widget-box""#));
}

#[test]
fn rate_module_expansion_ignores_literal_and_attribute_occurrences() {
    let source = concat!(
        "@@[[module Rate]]@@\n",
        "[[code]]\n[[module Rate]]\n[[/code]]\n",
        "[[raw]]\n[[module Rate]]\n[[/raw]]\n",
        "[!-- [[module Rate]] --]\n",
        "[[div data-rate=\"[[module Rate]]\"]]body[[/div]]\n",
        "<div data-rate=\"[[module Rate]]\">body</div>\n",
        "before [[module Rate]] after\n",
        "[[module Rate]][[module Rate]]\n",
    );
    let mut page_info = fallback_test_page_info("rate-boundary", "Rate boundary");
    page_info.score = ftml::data::ScoreValue::Integer(7);
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut fragments = CompatHtmlFragments::new(source);
    let mut compat_text = CompatTextFragments::new(source);
    let protected = RenderService::expand_rate_modules_with_registry(
        source.to_owned(),
        &page_info,
        &settings,
        RateModuleContext {
            rating_type: PageRatingType::PlusMinus,
            score: page_info.score,
            rating_votes: None,
        },
        &mut fragments,
        &mut compat_text,
    );

    assert_eq!(protected.matches("WIKIJUMPWIKIDOTCOMPATHTML").count(), 3);
    assert_eq!(protected.matches("[[module Rate]]").count(), 6);

    let mut output =
        RenderService::render_wikidot_compatibility_fallback_output_for_context(
            &protected,
            Some("rate-boundary"),
            Some("scp-wiki"),
            None,
        );
    output.body = fragments.restore(&output.body);
    assert_eq!(output.body.matches(r#"class="rate-points""#).count(), 3);
    assert!(!output.body.contains("<p><div"));
    assert!(!output.body.contains(r#"data-rate="<div"#));
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
    ftml::preprocess_for_layout(&mut wikitext, settings.layout);
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
        "\n",
        r#"{$missing}"#,
        "\n[[/div]]\n",
    )
    .to_owned();
    let mut fragments = CompatTextFragments::new(&wikitext);

    RenderService::protect_wikidot_unbound_include_variables(
        &mut wikitext,
        &mut fragments,
    );

    assert!(wikitext.contains(COMPAT_TEXT_MARKER_PREFIX));
    let page_info = fallback_test_page_info("001-blank-i", "Proposal Blank the First");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    ftml::preprocess_for_layout(&mut wikitext, settings.layout);
    let tokens = ftml::tokenize(&wikitext);
    let result = ftml::parse(&tokens, &page_info, &settings);
    let (tree, _) = result.into();
    let rendered = HtmlRender.render(&tree, &page_info, &settings).body;
    let restored = fragments.restore(&rendered);

    assert!(
        restored.contains(r#"<div class="anom-bar-container item-SCP-001 {$american}">"#)
    );
    assert!(restored.contains(r#"<span class="item">Item#:</span>"#));
    assert!(restored.contains("{$missing}"));
    assert!(!restored.contains("[[span"));
    assert!(!restored.contains(COMPAT_TEXT_MARKER_PREFIX));
}

#[test]
fn protects_unbound_include_variables_in_all_rendered_positions() {
    let authored_legacy = "wikijump-include-var-american";
    let mut wikitext = format!(
        concat!(
            "[[div class=\"valid {{$alpha_1-beta}} {}\"]]\n",
            "[[span class=\"valid {{$z9}}\"]]ok[[/span]]\n",
            "[[div class=\"invalid {{$space name}} {{$dot.name}} {{$}}\"]]\n",
            "[[div id=\"{{$id}}\" class='{{$single}}']]\n",
            "text class=\"{{$plain}}\"\n",
            "[[table class=\"{{$table}}\"]]\n",
        ),
        authored_legacy,
    );
    let original = wikitext.clone();
    let mut fragments = CompatTextFragments::new(&wikitext);

    RenderService::protect_wikidot_unbound_include_variables(
        &mut wikitext,
        &mut fragments,
    );

    assert_eq!(wikitext.matches(COMPAT_TEXT_MARKER_PREFIX).count(), 6);
    assert!(wikitext.contains(authored_legacy));
    assert!(wikitext.contains("{$space name}"));
    assert!(wikitext.contains("{$dot.name}"));
    assert!(!wikitext.contains("id=\"{$id}\""));
    assert!(!wikitext.contains("class='{$single}'"));
    assert!(!wikitext.contains("text class=\"{$plain}\""));
    assert!(!wikitext.contains("[[table class=\"{$table}\"]]"));
    assert_eq!(fragments.restore(&wikitext), original);
}

/// Parse protected Wikidot source, optionally restoring compatibility text
/// in link targets first, and return the restored HTML with its links.
fn render_protected_links(
    source: &str,
    restore_targets: bool,
) -> (String, ftml::data::Backlinks<'static>) {
    let mut wikitext = source.to_owned();
    let mut fragments = CompatTextFragments::new(&wikitext);
    RenderService::protect_wikidot_unbound_include_variables(
        &mut wikitext,
        &mut fragments,
    );
    // A stray `##` is protected the same way as an unrendered color marker.
    wikitext = wikitext.replace("##", &fragments.push("##"));
    let page_info = fallback_test_page_info("component:links", "Links");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    ftml::preprocess_for_layout(&mut wikitext, settings.layout);
    let tokens = ftml::tokenize(&wikitext);
    let (mut tree, _) = ftml::parse(&tokens, &page_info, &settings).into();
    if restore_targets {
        super::super::compat::link_targets::restore_compat_text_in_link_targets(
            &mut tree, &fragments,
        );
    }
    let output = HtmlRender.render(&tree, &page_info, &settings);
    let backlinks = ftml::data::Backlinks {
        included_pages: output.backlinks.included_pages.clone(),
        internal_links: output.backlinks.internal_links.clone(),
        external_links: output
            .backlinks
            .external_links
            .iter()
            .map(|link| Cow::Owned(link.to_string()))
            .collect(),
    };
    (fragments.restore(&output.body), backlinks)
}

#[test]
fn link_targets_keep_the_authored_text_of_protected_fragments() {
    // Wikidot links an unbound variable target by its authored text
    // (component:earthworm: `[[[{$previous-url} | {$previous-title}]]]`
    // links to /previous-url) and keeps a stray `##` inside a URL.
    let source = concat!(
        "[[[{$previous-url} | {$previous-title}]]]\n\n",
        "[[[adult:{$page} | Adult]]]\n\n",
        "[https://example.com/scp-7493## SCP-7493]\n",
    );

    let (html, links) = render_protected_links(source, true);

    assert!(html.contains(r#"href="/previous-url""#), "{html}");
    assert!(html.contains(">{$previous-title}</a>"), "{html}");
    assert!(html.contains(r#"href="/adult:page""#), "{html}");
    assert!(
        html.contains(r#"href="https://example.com/scp-7493##""#),
        "{html}"
    );
    assert!(
        !html
            .to_ascii_uppercase()
            .contains(COMPAT_TEXT_MARKER_PREFIX),
        "{html}"
    );
    let recorded = format!("{:?} {:?}", links.internal_links, links.external_links);
    assert!(
        !recorded
            .to_ascii_uppercase()
            .contains(COMPAT_TEXT_MARKER_PREFIX),
        "recorded links must not carry protection markers: {recorded}",
    );
    assert!(
        recorded.contains("https://example.com/scp-7493##"),
        "{recorded}"
    );

    // Without the pass the marker survives into the page target, which is
    // the leak this guards against.
    let (unrestored, _) = render_protected_links(source, false);
    assert!(
        unrestored
            .to_ascii_uppercase()
            .contains(COMPAT_TEXT_MARKER_PREFIX),
        "{unrestored}",
    );
}

#[test]
fn densely_protects_and_restores_marker_class_variables() {
    let mut wikitext = String::from("[[div class=\"");
    for index in 0..10_000 {
        wikitext.push_str(&format!("item-{{$variable_{index}}} "));
    }
    wikitext.push_str("\"]]body[[/div]]\n");
    let original = wikitext.clone();
    let mut fragments = CompatTextFragments::new(&wikitext);

    RenderService::protect_wikidot_unbound_include_variables(
        &mut wikitext,
        &mut fragments,
    );

    assert_eq!(wikitext.matches(COMPAT_TEXT_MARKER_PREFIX).count(), 10_000);
    assert_eq!(fragments.restore(&wikitext), original);
}

#[test]
fn restores_marker_class_variables_after_fallback_rendering() {
    let mut wikitext = "[[div class=\"anom-bar {$american}\"]]body[[/div]]\n".to_owned();
    let mut fragments = CompatTextFragments::new(&wikitext);
    RenderService::protect_wikidot_unbound_include_variables(
        &mut wikitext,
        &mut fragments,
    );

    let fallback =
        RenderService::render_wikidot_compatibility_fallback_output_for_context(
            &wikitext,
            Some("fixture"),
            Some("fixture-site"),
            None,
        );
    let restored = fragments.restore(&fallback.body);

    assert!(
        restored.contains(r#"class="anom-bar {$american}""#),
        "{restored}",
    );
    assert!(!restored.contains(COMPAT_TEXT_MARKER_PREFIX));
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
    ftml::preprocess_for_layout(&mut wikitext, settings.layout);
    let tokens = ftml::tokenize(&wikitext);
    let result = ftml::parse(&tokens, &page_info, &settings);
    let (tree, _) = result.into();
    let rendered = HtmlRender.render(&tree, &page_info, &settings).body;

    assert!(rendered.contains(r#"href="/an-incredibly-importanterest-announcement""#));
    assert!(rendered.contains("Creck Fection Contest 2 (TWO DAY EXTRAVAGANZA!)"));
    assert!(!rendered.contains("[[[an-incredibly-importanterest-announcement"));
}

#[test]
fn multiline_page_links_respect_canonical_literal_regions() {
    let mut wikitext = concat!(
        "[[code]]\n[[[code-link|Nope\nPlease]]]\n[[/code]]\n",
        "[[html]]\n[[[html-link|Nope\nPlease]]]\n[[/html]]\n",
        "@@[[[escape-link|Nope\nPlease]]]@@\n",
        "[!-- [[[comment-link|Nope\nPlease]]] --]\n",
        "<pre>[[[pre-link|Nope\nPlease]]]</pre>\n",
        "[[[page-link|Yes\nPlease]]]\n",
    )
    .to_owned();

    RenderService::normalize_wikidot_multiline_page_links(&mut wikitext);

    for literal in [
        "[[[code-link|Nope\nPlease]]]",
        "[[[html-link|Nope\nPlease]]]",
        "@@[[[escape-link|Nope\nPlease]]]@@",
        "[!-- [[[comment-link|Nope\nPlease]]] --]",
        "<pre>[[[pre-link|Nope\nPlease]]]</pre>",
    ] {
        assert!(
            wikitext.contains(literal),
            "missing literal region: {literal}"
        );
    }
    assert!(wikitext.contains("[[[page-link|Yes Please]]]"));
}

#[test]
fn multiline_page_links_preserve_malformed_and_unicode_input() {
    let mut wikitext = concat!(
        "[[[\u{30da}\u{30fc}\u{30b8}|\u{4e00}\u{884c}\u{76ee}\n\u{4e8c}\u{884c}\u{76ee}]]]\n",
        "[[[missing-label|\n]]]\n",
    )
    .to_owned();

    RenderService::normalize_wikidot_multiline_page_links(&mut wikitext);

    assert!(wikitext.contains(
        "[[[\u{30da}\u{30fc}\u{30b8}|\u{4e00}\u{884c}\u{76ee} \u{4e8c}\u{884c}\u{76ee}]]]"
    ));
    assert!(wikitext.contains("[[[missing-label|\n]]]"));

    for malformed in [
        "[[[missing-close|line one\nline two\n",
        "[[[|empty target\nlabel]]]\n",
    ] {
        let mut value = malformed.to_owned();
        RenderService::normalize_wikidot_multiline_page_links(&mut value);
        assert_eq!(value, malformed);
    }
}

#[test]
fn multiline_page_link_normalization_handles_dense_literal_regions_linearly() {
    const COUNT: usize = 2_048;
    let mut wikitext = String::new();
    for index in 0..COUNT {
        if index % 2 == 0 {
            wikitext.push_str("[!-- [[[literal|");
            wikitext.push_str(&index.to_string());
            wikitext.push_str("\nvalue]]] --]\n");
        } else {
            wikitext.push_str("[[[page-");
            wikitext.push_str(&index.to_string());
            wikitext.push_str("|first\nsecond]]]\n");
        }
    }

    RenderService::normalize_wikidot_multiline_page_links(&mut wikitext);

    assert_eq!(wikitext.matches("\nvalue]]] --]").count(), COUNT / 2);
    assert_eq!(wikitext.matches("|first second]]]").count(), COUNT / 2);
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

    ftml::preprocess_for_layout(&mut wikitext, settings.layout);
    let tokens = ftml::tokenize(&wikitext);
    let result = ftml::parse(&tokens, &page_info, &settings);
    let (tree, _) = result.into();
    let mut rendered = HtmlRender.render(&tree, &page_info, &settings).body;
    rendered = RenderService::restore_protected_wikidot_compat_links(rendered, &links);
    rendered = RenderService::restore_wikidot_email_obfuscation(&rendered);
    rendered = RenderService::remove_spurious_wikidot_email_classes(&rendered);

    assert!(rendered.contains(r#"<a href="javascript:;">phmd@scip.net</a>:// ./msg <a href="/scp-6276" target="_blank">ext_server-6276</a> !undata recipient 0000"#));
    assert!(!rendered.contains("//:]ten.pics|dmhp"));
    assert!(!rendered.contains("[# phmd@scip.net]"));
    assert!(!rendered.contains("[*/scp-6276 ext_server-6276]"));
}

#[test]
fn ftml_renders_valid_wikidot_named_anchor_markers_without_visible_brackets() {
    let page_info = fallback_test_page_info("scp-7243", "SCP-7243");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut wikitext = "[[# tabanchor]]\nVisible text".to_owned();

    let links = RenderService::protect_wikidot_compat_links(&mut wikitext, &settings);

    assert!(links.is_empty());
    assert!(wikitext.contains("[[# tabanchor]]"));

    ftml::preprocess_for_layout(&mut wikitext, settings.layout);
    let tokens = ftml::tokenize(&wikitext);
    let result = ftml::parse(&tokens, &page_info, &settings);
    let (tree, _) = result.into();
    let rendered = HtmlRender.render(&tree, &page_info, &settings).body;

    assert!(rendered.contains(r#"<a name="tabanchor"></a>"#));
    assert!(rendered.contains("Visible text"));
    assert!(!rendered.contains("[tabanchor]"));
    assert!(!rendered.contains("[# tabanchor]"));
}

#[test]
fn named_anchor_candidates_reach_ftml_without_deepwell_preprotection() {
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);

    for source in [
        "[[# tabanchor]]",
        "[[# alpha beta]]",
        "[[# symbol$%_foo]]",
        "[[# 日本語🙂]]",
        "[[[scp-002|[[# alpha]]]]]",
        "[https://example.com [[# alpha]]]",
        "[[span title=\"[[# alpha]]\"]]X[[/span]]",
    ] {
        let mut protected = source.to_owned();
        let links =
            RenderService::protect_wikidot_compat_links(&mut protected, &settings);

        assert!(links.is_empty(), "{source:?}");
        assert_eq!(protected, source, "{source:?}");
    }
}

#[test]
fn compat_link_markers_do_not_restore_predictable_literal_sentinels() {
    let page_info = fallback_test_page_info("scp-7243", "SCP-7243");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut wikitext = concat!(
        "[# safe label]\n",
        "[[span class=\"WIKIJUMPWIKIDOTCOMPATLINK0X\"]]hover[[/span]]",
    )
    .to_owned();

    let links = RenderService::protect_wikidot_compat_links(&mut wikitext, &settings);

    assert_eq!(links.len(), 1);
    assert_ne!(links[0].marker, "WIKIJUMPWIKIDOTCOMPATLINK0X");
    assert!(wikitext.contains(&links[0].marker));
    assert!(wikitext.contains("WIKIJUMPWIKIDOTCOMPATLINK0X"));

    ftml::preprocess_for_layout(&mut wikitext, settings.layout);
    let tokens = ftml::tokenize(&wikitext);
    let result = ftml::parse(&tokens, &page_info, &settings);
    let (tree, _) = result.into();
    let rendered = RenderService::restore_protected_wikidot_compat_links(
        HtmlRender.render(&tree, &page_info, &settings).body,
        &links,
    );

    assert!(rendered.contains(r#"<a href="javascript:;">safe label</a>"#));
    assert!(
        rendered.contains(r#"<span class="WIKIJUMPWIKIDOTCOMPATLINK0X">hover</span>"#)
    );
    assert!(!rendered.contains(r#"<span class="<a name="#));
}

#[test]
fn named_anchor_candidates_inside_attributes_never_issue_compat_markers() {
    let page_info = fallback_test_page_info("scp-7243", "SCP-7243");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut wikitext = concat!(
        "[[span class=\"[[# x onmouseover=alert(1) y]]\"]]",
        "hover",
        "[[/span]]",
    )
    .to_owned();

    let links = RenderService::protect_wikidot_compat_links(&mut wikitext, &settings);

    assert!(links.is_empty());
    assert!(wikitext.contains("[[# x onmouseover=alert(1) y]]"));

    ftml::preprocess_for_layout(&mut wikitext, settings.layout);
    let tokens = ftml::tokenize(&wikitext);
    let result = ftml::parse(&tokens, &page_info, &settings);
    let (tree, _) = result.into();
    let rendered = HtmlRender.render(&tree, &page_info, &settings).body;

    assert!(rendered.contains("hover"), "{rendered}");
    assert!(!rendered.contains("onmouseover"));
    assert!(!rendered.contains(WIKIDOT_COMPAT_LINK_SENTINEL_PREFIX));
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

    let links = RenderService::protect_wikidot_wikipedia_links(&mut wikitext, &settings);

    assert_eq!(links.len(), 1);
    assert!(wikitext.contains(WIKIDOT_WIKIPEDIA_LINK_SENTINEL_PREFIX));
    assert!(!wikitext.contains("[wikipedia:Canonical_bundle"));

    ftml::preprocess_for_layout(&mut wikitext, settings.layout);
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
fn wikipedia_link_restoration_only_replaces_issued_text_markers() {
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut wikitext = "[wikipedia:Canonical_bundle Canonical Bundle]".to_owned();
    let links = RenderService::protect_wikidot_wikipedia_links(&mut wikitext, &settings);

    assert_eq!(links.len(), 1);
    let marker = &links[0].marker;
    let wrong_index = format!("{}9X", marker.strip_suffix("0X").unwrap());
    let legacy_marker = format!("{WIKIDOT_WIKIPEDIA_LINK_SENTINEL_PREFIX}0X");
    let html = format!(
        r#"<span data-double=">{marker}" data-single='>{marker}'>{marker}</span>{wrong_index}{legacy_marker}"#,
    );

    let restored = RenderService::restore_protected_wikidot_wikipedia_links(html, &links);

    assert!(restored.contains(&format!(
        r#"<span data-double=">{marker}" data-single='>{marker}'>"#,
    )));
    assert!(restored.contains(&wrong_index));
    assert!(restored.contains(&legacy_marker));
    assert_eq!(restored.matches("<a href=").count(), 1);
}

#[test]
fn restores_dense_wikidot_wikipedia_links_without_repeated_scans() {
    const LINK_COUNT: usize = 10_000;

    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut wikitext = String::with_capacity(LINK_COUNT * 64);
    for index in 0..LINK_COUNT {
        wikitext.push_str(&format!(
            "[wikipedia:Canonical_bundle_{index} Canonical Bundle {index}]\n",
        ));
    }

    let links = RenderService::protect_wikidot_wikipedia_links(&mut wikitext, &settings);
    assert_eq!(links.len(), LINK_COUNT);

    let restored =
        RenderService::restore_protected_wikidot_wikipedia_links(wikitext, &links);

    assert_eq!(restored.matches("<a href=").count(), LINK_COUNT);
    assert!(!restored.contains(WIKIDOT_WIKIPEDIA_LINK_SENTINEL_PREFIX));
}

#[test]
fn renders_wikidot_wikipedia_links_with_language_and_default_label() {
    assert_eq!(
        super::build_wikidot_wikipedia_link("it:Albert_Einstein", Some("Albert")).anchor,
        r#"<a href="http://it.wikipedia.org/wiki/Albert_Einstein" onclick="window.open(this.href, '_blank'); return false;">Albert</a>"#,
    );
    assert_eq!(
        super::build_wikidot_wikipedia_link("Canonical_bundle", None).anchor,
        r#"<a href="http://en.wikipedia.org/wiki/Canonical_bundle" onclick="window.open(this.href, '_blank'); return false;">Canonical bundle</a>"#,
    );
}

#[test]
fn leaves_wikidot_wikipedia_links_inside_literal_regions_unchanged() {
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut escaped = "@@[wikipedia:Canonical_bundle Canonical Bundle]@@".to_owned();
    let links = RenderService::protect_wikidot_wikipedia_links(&mut escaped, &settings);

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
    assert!(
        rendered.contains(
            r#"<span class="fncon">For clarity: payloads will be absent.</span>"#
        )
    );
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

    let mut fragments = CompatHtmlFragments::new(&wikitext);
    let (protected, links) = RenderService::render_long_native_list_runs_with_registry(
        wikitext,
        &mut fragments,
    );
    let rendered = fragments.restore(&protected);
    let mut backlinks = ftml::data::Backlinks::new();
    RenderService::record_wikidot_wikipedia_backlinks(&mut backlinks, &links);

    assert!(rendered.contains(r#"<li>Source <a href="http://en.wikipedia.org/wiki/Canonical_bundle" onclick="window.open(this.href, '_blank'); return false;">Canonical Bundle</a></li>"#));
    assert!(!rendered.contains("[wikipedia:Canonical_bundle"));
    assert_eq!(links.len(), 1);
    assert_eq!(
        backlinks.external_links,
        vec![Cow::Borrowed(
            "http://en.wikipedia.org/wiki/Canonical_bundle"
        )],
    );
}

fn render_native_list_page_for_regression(
    source: &str,
    require_clean_parse: bool,
) -> String {
    let page_info = fallback_test_page_info("nav:top", "Top Bar");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let outer = RenderService::prepare_outer_render_wikitext(
        super::ExpandedRenderWikitext {
            theme_previewer_blank: false,
            wikidot_compat_html: CompatHtmlFragments::new(source),
            wikidot_compat_text: CompatTextFragments::new(source),
            wikitext: source.to_owned(),
            included_pages: Vec::new(),
            expanded_include_count: 0,
            url_offset_list_pages_content_bytes: 0,
            runtime_css_insertions: Vec::new(),
        },
        &page_info,
        &settings,
    );
    assert!(!outer.compatibility_fallback);

    let inner = RenderService::prepare_inner_render_wikitext(outer, &settings);
    let tokens = ftml::tokenize(&inner.wikitext);
    let (tree, errors) = ftml::parse(&tokens, &page_info, &settings).into();
    if require_clean_parse {
        assert!(errors.is_empty(), "{errors:#?}");
    }
    inner.protection.restore(|protection| {
        protection
            .compat_html()
            .restore(&HtmlRender.render(&tree, &page_info, &settings).body)
    })
}

#[test]
fn restores_long_native_list_as_direct_div_child() {
    let source = concat!(
        "[[div class=\"top-bar\"]]\n",
        "* About\n",
        "* Library\n",
        "* Community\n",
        "* Resources\n",
        "* Rules\n",
        "* Contact\n",
        "* Help\n",
        "* News\n",
        "[[/div]]",
    );
    let restored = render_native_list_page_for_regression(source, true);

    let top_bar_start = restored.find(r#"<div class="top-bar">"#).expect(&restored);
    let top_bar_end = restored[top_bar_start..]
        .find("</div>")
        .map(|offset| top_bar_start + offset)
        .expect(&restored);
    let top_bar = &restored[top_bar_start..top_bar_end];
    assert!(
        top_bar.contains(r#"<ul data-wikijump-compat-list="1">"#),
        "{restored}"
    );
    assert!(
        restored.contains(r#"<div class="top-bar"><ul data-wikijump-compat-list="1">"#),
        "{restored}"
    );
    assert!(!top_bar.contains("<p>"), "{restored}");
    assert!(!restored.contains("<p><ul"), "{restored}");
    assert!(
        !restored.contains("WIKIJUMPWIKIDOTCOMPATHTML"),
        "{restored}"
    );
}

#[test]
fn keeps_long_native_lists_native_inside_cross_tree_inline_scopes() {
    let items = concat!(
        "* One\n",
        "* Two\n",
        "* Three\n",
        "* Four\n",
        "* Five\n",
        "* Six\n",
        "* Seven\n",
        "* Eight\n",
    );
    for source in [
        format!("[[span class=\"inline\"]]\n{items}[[/span]]"),
        format!("[[div]]\n[[span]]\n{items}[[/span]]\n[[/div]]"),
        format!("[[span class=\"inline\"]]\n{items}"),
        format!("[[size 120%]]\n{items}[[/size]]"),
    ] {
        let mut fragments = CompatHtmlFragments::new(&source);
        let (protected, links) =
            RenderService::render_long_native_list_runs_with_registry(
                source.clone(),
                &mut fragments,
            );

        assert_eq!(protected, source);
        assert!(links.is_empty());
        assert!(!protected.contains("WIKIJUMPWIKIDOTCOMPATHTML"));
    }
}

#[test]
fn keeps_long_native_lists_native_inside_unsafe_contexts() {
    let items = concat!(
        "* One\n",
        "* Two\n",
        "* Three\n",
        "* Four\n",
        "* Five\n",
        "* Six\n",
        "* Seven\n",
        "* Eight\n",
    );
    for source in [
        format!("[[hidden]]\n{items}[[/hidden]]"),
        format!("[[invisible]]\n{items}[[/invisible]]"),
        format!("[[b]]\n{items}[[/b]]"),
        format!("[[bold]]\n{items}[[/b]]"),
        format!("[[a href=\"/target\"]]\n{items}[[/a]]"),
        format!("[[a_ href=\"/target\"]]\n{items}[[/a]]"),
        format!("[[*a href=\"/target\"]]\n{items}[[/a]]"),
        format!("[[*anchor href=\"/target\"]]\n{items}[[/a]]"),
        format!("[[* a href=\"/target\"]]\n{items}[[/a]]"),
        format!("[[* anchor href=\"/target\"]]\n{items}[[/a]]"),
        format!("[[span_ class=\"inline\"]]\n{items}[[/span]]"),
        format!("[[hidden]]\n{items}"),
        format!("[[hidden]]\n[[/hidden bogus]]\n{items}"),
        format!("**\n{items}**"),
        format!("//\n{items}//"),
        format!("{{{{\n{items}}}}}"),
        format!("--\n{items}--"),
        format!("~~\n{items}~~"),
        format!("##red|\n{items}##"),
    ] {
        let mut fragments = CompatHtmlFragments::new(&source);
        let (protected, links) =
            RenderService::render_long_native_list_runs_with_registry(
                source.clone(),
                &mut fragments,
            );

        assert_eq!(protected, source);
        assert!(links.is_empty());
        assert!(!protected.contains("WIKIJUMPWIKIDOTCOMPATHTML"));
    }
}

#[test]
fn does_not_leak_long_native_list_markers_for_unclosed_or_aliased_inline_scopes() {
    let items = concat!(
        "* One\n",
        "* Two\n",
        "* Three\n",
        "* Four\n",
        "* Five\n",
        "* Six\n",
        "* Seven\n",
        "* Eight\n",
    );

    for (name, source) in [
        ("unclosed hidden", format!("[[hidden]]\n{items}")),
        ("bold alias", format!("[[bold]]\n{items}[[/b]]")),
        (
            "anchor score suffix",
            format!("[[a_ href=\"/target\"]]\n{items}[[/a]]"),
        ),
        (
            "starred anchor",
            format!("[[*a href=\"/target\"]]\n{items}[[/a]]"),
        ),
        (
            "starred anchor alias",
            format!("[[*anchor href=\"/target\"]]\n{items}[[/a]]"),
        ),
        (
            "spaced starred anchor",
            format!("[[* a href=\"/target\"]]\n{items}[[/a]]"),
        ),
        (
            "spaced starred anchor alias",
            format!("[[* anchor href=\"/target\"]]\n{items}[[/a]]"),
        ),
    ] {
        let restored = render_native_list_page_for_regression(&source, false);
        assert!(
            !restored.contains("WIKIJUMPWIKIDOTCOMPATHTML"),
            "scope: {name}; html: {restored}"
        );
        assert!(
            !restored.contains(r#"data-wikijump-compat-list="1""#),
            "scope: {name}; html: {restored}"
        );
    }
}

#[test]
fn keeps_long_native_lists_native_after_invalid_inline_close() {
    let items = concat!(
        "* [wikipedia:One]\n",
        "* Two\n",
        "* Three\n",
        "* Four\n",
        "* Five\n",
        "* Six\n",
        "* Seven\n",
        "* Eight\n",
    );
    for invalid_close in ["[[/span bogus]]", "[[/span\n]]"] {
        let source = format!("[[span]]\n{invalid_close}\n{items}[[/span]]");
        let mut fragments = CompatHtmlFragments::new(&source);
        let (protected, links) =
            RenderService::render_long_native_list_runs_with_registry(
                source.clone(),
                &mut fragments,
            );

        assert_eq!(protected, source, "close: {invalid_close:?}");
        assert!(links.is_empty(), "close: {invalid_close:?}");
        assert!(!protected.contains("WIKIJUMPWIKIDOTCOMPATHTML"));
    }
}

#[test]
fn resumes_long_native_list_block_rendering_after_inline_span_scope_closes() {
    let source = concat!(
        "[[span class=\"inline\"]]label[[/span]]\n",
        "[[div class=\"top-bar\"]]\n",
        "* About\n",
        "* Library\n",
        "* Community\n",
        "* Resources\n",
        "* Rules\n",
        "* Contact\n",
        "* Help\n",
        "* News\n",
        "[[/div]]",
    );

    let restored = render_native_list_page_for_regression(source, true);
    assert!(
        restored.contains(r#"<div class="top-bar"><ul data-wikijump-compat-list="1">"#),
        "{restored}"
    );
    assert!(
        !restored.contains("WIKIJUMPWIKIDOTCOMPATHTML"),
        "{restored}"
    );
}

#[test]
fn native_list_wikipedia_backlinks_follow_only_emitted_anchors() {
    let wikitext = concat!(
        "* Item 1\n",
        "* Item 2\n",
        "* Item 3\n",
        "* Item 4\n",
        "* Item 5\n",
        "* Item 6\n",
        "* Malformed [wikipedia:] and [wikipedia:Missing close\n",
        "* Sources [wikipedia:fr:Paris Paris] [wikipedia:Rust_(lang)]\n",
        "After [wikipedia:Not_emitted Outside]\n",
    )
    .to_owned();
    let mut fragments = CompatHtmlFragments::new(&wikitext);

    let (protected, links) = RenderService::render_long_native_list_runs_with_registry(
        wikitext,
        &mut fragments,
    );
    let rendered = fragments.restore(&protected);

    assert_eq!(
        links
            .iter()
            .map(|link| link.href.as_str())
            .collect::<Vec<_>>(),
        vec![
            "http://fr.wikipedia.org/wiki/Paris",
            "http://en.wikipedia.org/wiki/Rust_(lang)",
        ],
    );
    assert!(rendered.contains(">Paris</a>"));
    assert!(rendered.contains(">Rust (lang)</a>"));
    assert!(rendered.contains("After [wikipedia:Not_emitted Outside]"));
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
        rendered
            .contains(r#"<span class="outer">a <span class="inner">b</span> c</span>"#)
    );
    assert!(!rendered.contains("[[span"));
    assert!(!rendered.contains("[[/span]]"));
}

#[test]
fn caps_deep_inline_wikidot_span_nesting_inside_preprocessed_native_list_runs() {
    let mut wikitext = concat!(
        "* Item 1\n",
        "* Item 2\n",
        "* Item 3\n",
        "* Item 4\n",
        "* Item 5\n",
        "* Item 6\n",
        "* Item 7\n",
        "* Nested ",
    )
    .to_owned();
    wikitext.push_str(&"[[span]]".repeat(MAX_NATIVE_LIST_WIKIDOT_SPAN_NESTING + 1));
    wikitext.push_str("capped");
    wikitext.push_str(&"[[/span]]".repeat(MAX_NATIVE_LIST_WIKIDOT_SPAN_NESTING + 1));
    wikitext.push('\n');

    let rendered = RenderService::render_long_native_list_runs(wikitext);

    assert!(rendered.contains("capped"));
    assert!(rendered.contains("[[span]]"));
    assert!(rendered.contains("[[/span]]"));
}

#[test]
fn leaves_many_unclosed_native_list_wikidot_spans_literal() {
    let mut item = String::from("attack ");
    for _ in 0..20_000 {
        item.push_str(r#"[[span class="safe"]]"#);
    }
    item.push_str("text");

    let started = Instant::now();
    let rendered = render_native_list_inline_wikidot_spans(&item);

    assert!(
        started.elapsed() < Duration::from_secs(2),
        "unclosed span scan exceeded its linear-work budget: {:?}",
        started.elapsed(),
    );
    assert!(rendered.starts_with(r#"attack [[span class="safe"]]"#));
    assert!(rendered.ends_with("text"));
    assert!(!rendered.contains("<span"));
}

#[test]
fn leaves_an_unterminated_native_list_span_opener_literal() {
    let rendered = render_native_list_inline_wikidot_spans("prefix [[span");

    assert_eq!(rendered, "prefix [[span");
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
            "[!-- 6 --]\n",
            "<code>8</code>\n",
            "resolved 10",
        )
    );
}

#[test]
fn list_pages_parser_function_boundary_preserves_literal_examples() {
    let signed = concat!(
        "@@[[#ifexpr -3 > -1 | + | - ]][[#expr abs(-3)]]@@ ",
        "[[#ifexpr -3 > -1 | + | - ]][[#expr abs(-3)]]",
    );
    let signed = RenderService::resolve_wikidot_parser_functions(signed);
    assert_eq!(
        signed,
        "@@[[#ifexpr -3 > -1 | + | - ]][[#expr abs(-3)]]@@ -3"
    );

    let numeric = concat!(
        "[[code]]\n[[#ifexpr 2 > 1 | code | hidden]]\n[[/code]] ",
        "[[#ifexpr 2 > 1 | visible | hidden]]",
    );
    assert_eq!(
        RenderService::resolve_wikidot_parser_functions(numeric),
        "[[code]]\n[[#ifexpr 2 > 1 | code | hidden]]\n[[/code]] visible"
    );
}

#[test]
fn resolves_literal_wikidot_simple_if_before_ftml_parsing() {
    let mut source = "[[div class=\"[[#if 1 | folded | unfolded ]] [[#if 0 | inactive | active ]]\"]]\nbody\n[[/div]]".to_owned();
    let page_info = fallback_test_page_info("conditional", "Conditional");

    prepare_test_wikidot_conditionals(&mut source, &page_info);

    assert_eq!(source, "[[div class=\"folded active\"]]\nbody\n[[/div]]");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    ftml::preprocess_for_layout(&mut source, settings.layout);
    let tokens = ftml::tokenize(&source);
    let (_, errors) = ftml::parse(&tokens, &page_info, &settings).into();
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn direct_root_source_unwraps_unbound_dynamic_iftags() {
    let mut source =
        "before [[ift{$mode}gs +theme]]root[[/ift{$mode}gs]] after".to_owned();
    let page_info = fallback_test_page_info("root", "Root");

    prepare_test_wikidot_conditionals_before_include_expansion(&mut source, &page_info);

    assert_eq!(source, "before root after");
}

#[test]
fn direct_inactive_iftags_closes_before_unclosed_code() {
    let mut source =
        "[[iftags +missing]]\n[[code]]\n[[/iftags]]\nunclosed code".to_owned();
    let page_info = fallback_test_page_info("root", "Root");

    prepare_test_wikidot_conditionals_before_include_expansion(&mut source, &page_info);
    prepare_test_wikidot_conditionals(&mut source, &page_info);

    assert_eq!(source, "\nunclosed code");

    let rendered = render_wikidot_conditionals_with_tags(&source, &[]);
    assert!(rendered.contains("unclosed code"), "{rendered}");
    assert!(!rendered.contains("[[iftags"), "{rendered}");
    assert!(!rendered.contains("[[code]]"), "{rendered}");
}

#[test]
fn direct_balanced_iftags_without_includes_are_evaluated_by_ftml() {
    let source = concat!(
        "[[iftags +missing]]\n",
        "hidden\n",
        "[[/iftags]]\n",
        "visible",
    );

    let rendered = render_wikidot_conditionals_with_tags(source, &[]);

    assert!(!rendered.contains("hidden"), "{rendered}");
    assert!(rendered.contains("visible"), "{rendered}");
}

/// Live sandbox-for-codex observations, 2026-10-15 (run-owned pages
/// `wjct2353-*`, `wjct2353b-*`): `[[/module]]` pairs with the nearest
/// preceding module opener that starts a line, so a body-less module does not
/// reach past a later `[[module CSS]]` and swallow the `[[/iftags]]` between
/// them (#2353). Without the later CSS module the gate already resolved.
#[test]
fn bodyless_module_does_not_take_a_later_modules_closer() {
    let css = "[[module CSS]]\n.x { color: red; }\n[[/module]]\n";
    for module in ["Rate", "PageTree"] {
        for later in [css, ""] {
            let source = format!(
                "[[iftags +component]]\nDOCS-SHOULD-HIDE\n[[module {module}]]\n[[/iftags]]\n{later}AFTER-MARK",
            );
            for (tags, shown) in [(&[][..], false), (&["component"][..], true)] {
                let mut prepared = source.clone();
                let page_info = ftml::data::PageInfo {
                    tags: tags.iter().map(|tag| Cow::Borrowed(*tag)).collect(),
                    ..fallback_test_page_info("root", "Root")
                };
                prepare_test_wikidot_conditionals_before_include_expansion(
                    &mut prepared,
                    &page_info,
                );
                prepare_test_wikidot_conditionals(&mut prepared, &page_info);

                assert!(
                    !prepared.contains("iftags"),
                    "{module} {tags:?}: {prepared}"
                );
                assert_eq!(
                    prepared.contains("DOCS-SHOULD-HIDE"),
                    shown,
                    "{module} {tags:?}: {prepared}",
                );
                assert_eq!(
                    prepared.contains(&format!("[[module {module}]]")),
                    shown,
                    "{module} {tags:?}: {prepared}",
                );
                assert!(
                    prepared.ends_with(&format!("{later}AFTER-MARK")),
                    "{prepared}"
                );
            }
        }
    }
}

#[test]
fn direct_theme_source_drops_unbound_empty_nested_iftags() {
    let mut source = concat!(
        ">[[ift{$mode}gs -override]]\n",
        ">[[iftags]]\n",
        "theme css\n",
        ">[[/iftags]]\n",
        ">[[/ift{$mode}gs]]",
    )
    .to_owned();
    let mut page_info = fallback_test_page_info("direct-theme", "Direct theme");
    page_info.category = Some(Cow::Borrowed("theme"));

    prepare_test_wikidot_conditionals_before_include_expansion(&mut source, &page_info);

    assert_eq!(source, "");
}

#[test]
fn direct_component_source_unwraps_balanced_and_preserves_malformed_dynamic_iftags() {
    let mut source =
        "[[ift{$mode}gs +component]]component body[[/ift{$mode}gs]]".to_owned();
    let mut page_info = fallback_test_page_info("direct-component", "Direct component");
    page_info.category = Some(Cow::Borrowed("component"));

    prepare_test_wikidot_conditionals(&mut source, &page_info);

    assert_eq!(source, "component body");

    let mut malformed =
        "[[ift{$mode}gs +component]]component body[[/ift{$other}gs]]".to_owned();
    let expected = malformed.clone();
    prepare_test_wikidot_conditionals(&mut malformed, &page_info);
    assert_eq!(malformed, expected);
}

#[test]
fn parser_functions_apply_first_closer_before_include_collection() {
    let mut source = concat!(
        "[[#if 0 | [[include component:hidden-if]] | ",
        "[[include component:visible-if]] ]]\n",
        "[[#if aroace | [[include component:visible-string]] | ",
        "[[include component:hidden-string]] ]]\n",
        "[[#if {$code} | [[include component:visible-placeholder]] | ",
        "[[include component:hidden-placeholder]] ]]\n",
        "[[#ifexpr 2 > 1 | [[include component:visible-ifexpr]] | ",
        "[[include component:hidden-ifexpr]] ]]\n",
    )
    .to_owned();
    let page_info = fallback_test_page_info("conditional", "Conditional");

    prepare_test_wikidot_conditionals(&mut source, &page_info);

    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut includes = Vec::new();
    ftml::include(
        &source,
        &settings,
        CollectingIncluder {
            includes: &mut includes,
        },
        include_error,
    )
    .expect("selected includes should remain valid include syntax");

    assert_eq!(
        includes
            .iter()
            .map(|include| include.page_ref().page())
            .collect::<Vec<_>>(),
        [
            "component:visible-string",
            "component:visible-placeholder",
            "component:visible-ifexpr",
        ],
    );
    assert_eq!(
        source,
        concat!(
            " | [[include component:visible-if]] ]]\n",
            "[[include component:visible-string | ",
            "[[include component:hidden-string]] ]]\n",
            "[[include component:visible-placeholder | ",
            "[[include component:hidden-placeholder]] ]]\n",
            "[[include component:visible-ifexpr | ",
            "[[include component:hidden-ifexpr]] ]]\n",
        ),
    );
}

#[test]
fn include_argument_pipes_are_collected_before_parser_function_evaluation() {
    let mut source =
        "[[include component:conditional |x=[[#if 1 | 1 | 0 ]]|n=1]]".to_owned();
    let page_info = fallback_test_page_info("conditional", "Conditional");

    prepare_test_wikidot_conditionals_before_include_expansion(&mut source, &page_info);

    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut includes = Vec::new();
    ftml::include(
        &source,
        &settings,
        CollectingIncluder {
            includes: &mut includes,
        },
        include_error,
    )
    .expect("include argument grammar should run before parser functions");

    assert_eq!(includes.len(), 1);
    assert_eq!(
        includes[0].variables().get("x").map(Cow::as_ref),
        Some("[[#if 1 "),
    );
    assert_eq!(includes[0].variables().get("n").map(Cow::as_ref), Some("1"),);
}

#[test]
fn parser_functions_open_wikidot_comment_delimiters_before_ftml() {
    // Live provenance:
    // ftml-oracle-20260712T230555Z/run-parser-comment-delimiter.
    let mut source = concat!(
        "[!-- [[#if aroace | --] |  ]]OMEGA_TRUE[!-- --]\n",
        "[!-- [[#if 0 | --] |  ]]OMEGA_FALSE[!-- --]\n",
        "[!-- [[#expr 1+1]] OMEGA_COMMENT --]\n",
        "OMEGA_AFTER",
    )
    .to_owned();
    let page_info = fallback_test_page_info("conditional", "Conditional");

    prepare_test_wikidot_conditionals(&mut source, &page_info);
    ftml::preprocess_for_layout(&mut source, Layout::Wikidot);
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let tokens = ftml::tokenize(&source);
    let (tree, errors) = ftml::parse(&tokens, &page_info, &settings).into();
    assert!(errors.is_empty(), "{errors:#?}");
    let html = HtmlRender.render(&tree, &page_info, &settings).body;

    assert!(html.contains("OMEGA_TRUE"), "{html}");
    assert!(html.contains("OMEGA_AFTER"), "{html}");
    for hidden in ["OMEGA_FALSE", "OMEGA_COMMENT", "[[#expr"] {
        assert!(!html.contains(hidden), "{hidden}: {html}");
    }
}

#[test]
fn render_preparation_resolves_generated_simple_if_with_link_branch() {
    let page_info = fallback_test_page_info("conditional", "Conditional");
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let outer = RenderService::prepare_outer_render_wikitext(
        super::ExpandedRenderWikitext {
            theme_previewer_blank: false,
            wikitext: concat!(
                "[[div class=\"colmod-block\"]]\n",
                "[[div]]link[[#if 0 | | [# fallback] ]][[/div]]\n",
                "[[/div]]",
            )
            .to_owned(),
            included_pages: Vec::new(),
            expanded_include_count: 0,
            url_offset_list_pages_content_bytes: 0,
            runtime_css_insertions: Vec::new(),
            wikidot_compat_html: CompatHtmlFragments::new(""),
            wikidot_compat_text: CompatTextFragments::new(""),
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
    inner.protection.restore(|_| ());
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
        r#"<div class="wj-footnote-list"><div class="wj-title">Footnotes</div><ol></ol></div>"#,
    );

    let restored = RenderService::restore_wikidot_footnote_dom_compatibility(html);

    assert!(restored.contains(
        r#"<sup class="footnoteref"><a id="footnoteref-2" href="javascript:;" class="footnoteref" onclick="WIKIDOT.page.utils.scrollToReference('footnote-2')">2</a></sup> after"#
    ));
    assert!(restored.contains(r#"<div class="footnotes-footer">"#));
    assert!(restored.contains(r#"<div class="title">Footnotes</div>"#));
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
    assert!(!restored.contains(r#"<span class="wj-footnote-ref">"#));
    assert!(!restored.contains("wj-footnote-ref-tooltip"));
    assert!(!restored.contains("hidden note"));
}

#[test]
fn restores_wikidot_footnote_title_class_without_assuming_english_text() {
    for title in ["脚注", "The feet-noten"] {
        let html = format!(
            r#"<div class="wj-footnote-list"><div class="wj-title">{title}</div><ol></ol></div>"#
        );

        let restored = RenderService::restore_wikidot_footnote_dom_compatibility(&html);

        assert_eq!(
            restored,
            format!(
                r#"<div class="footnotes-footer"><div class="title">{title}</div></div>"#
            )
        );
    }
}

#[test]
fn wikidot_japanese_corrections_locale_localizes_ftml_footnotes() {
    let mut page_info =
        fallback_test_page_info("localized-footnote", "Localized footnote");
    page_info.language = Cow::Borrowed(locale_for_ftml("ja-corrections"));
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut wikitext = "本文[[footnote]]注記[[/footnote]]".to_owned();
    ftml::preprocess_for_layout(&mut wikitext, settings.layout);
    let tokens = ftml::tokenize(&wikitext);
    let result = ftml::parse(&tokens, &page_info, &settings);
    let (tree, _) = result.into();
    let rendered = HtmlRender.render(&tree, &page_info, &settings).body;

    assert!(
        rendered.contains(
            r#"<sup class="footnoteref"><a id="footnoteref-1" href="javascript:;" class="footnoteref""#
        ),
        "{rendered}"
    );
    assert!(
        rendered
            .contains(r#"<div class="footnotes-footer"><div class="title">脚注</div>"#),
        "{rendered}"
    );

    let restored = RenderService::restore_wikidot_footnote_dom_compatibility(&rendered);

    assert_eq!(restored, rendered);
    assert!(restored.contains(r#"<div class="footnote-footer" id="footnote-1">"#));
    assert!(restored.contains(
        r#"onclick="WIKIDOT.page.utils.scrollToReference(&#39;footnoteref-1&#39;)">1</a>. 注記"#
    ));
    assert_eq!(restored.matches("注記").count(), 1);
    assert!(!restored.contains("wj-footnote"));
    assert!(!restored.contains(">Footnotes<"));
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

    let restored = RenderService::restore_wikidot_ta_badge_default_compatibility(html);

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

    assert_eq!(
        rendered,
        r#"<p>before <span style="text-decoration: line-through;">removed</span> after</p>"#
    );
    assert_eq!(
        RenderService::remove_wikijump_underline_wrappers(&rendered),
        rendered,
    );
}

#[test]
fn matches_live_wikidot_tag_predicate_matrix() {
    let tags = [Cow::Borrowed("alpha")];

    assert!(wikidot_tag_conditions_match("+alpha", &tags));
    assert!(!wikidot_tag_conditions_match("+beta", &tags));
    assert!(!wikidot_tag_conditions_match("-alpha", &tags));
    assert!(wikidot_tag_conditions_match("-beta", &tags));
    assert!(wikidot_tag_conditions_match("alpha beta", &tags));
    assert!(!wikidot_tag_conditions_match("beta gamma", &tags));
    assert!(!wikidot_tag_conditions_match("+alpha +beta", &tags));
    assert!(wikidot_tag_conditions_match("+alpha -beta", &tags));
    assert!(!wikidot_tag_conditions_match("+alpha -alpha", &tags));
    assert!(!wikidot_tag_conditions_match("", &tags));
    assert!(wikidot_tag_conditions_match("-", &tags));
}

#[test]
fn resolves_parser_generated_iftags_predicates_before_tag_matching() {
    // Frozen theme sources use this exact nested #ifexpr-in-iftags shape.
    // The live tagged preview and saved-page matrix is retained under
    // ftml-oracle-20260713T042816Z/run-iftags-parser-predicate.
    let page_info = ftml::data::PageInfo {
        tags: vec![Cow::Borrowed("alpha")],
        ..fallback_test_page_info("tagged-page", "Tagged Page")
    };
    let mut wikitext = concat!(
        "[[iftags [[#ifexpr 1 == 1 | - ]]]]\n",
        "OMEGA_TRUE_MINUS\n",
        "[[/iftags]]\n",
        "[[iftags [[#ifexpr 1 == 0 | - ]]]]\n",
        "OMEGA_FALSE_EMPTY\n",
        "[[/iftags]]\n",
        "[[iftags [[#ifexpr 1 == 1 | +alpha | +beta ]]]]\n",
        "OMEGA_TRUE_ALPHA\n",
        "[[/iftags]]\n",
        "[[iftags [[#ifexpr 1 == 0 | +alpha | +beta ]]]]\n",
        "OMEGA_FALSE_BETA\n",
        "[[/iftags]]\n",
        "[[iftags [[#if 1 | alpha beta | +gamma ]]]]\n",
        "OMEGA_BARE_OR\n",
        "[[/iftags]]\n",
        "[[iftags [[#if 0 | +gamma | -alpha ]]]]\n",
        "OMEGA_FALSE_MINUS_ALPHA\n",
        "[[/iftags]]\n",
    )
    .to_owned();

    prepare_test_wikidot_conditionals_before_include_expansion(&mut wikitext, &page_info);

    for visible in ["OMEGA_TRUE_MINUS", "OMEGA_TRUE_ALPHA", "OMEGA_BARE_OR"] {
        assert!(wikitext.contains(visible), "{visible}: {wikitext}");
    }
    for hidden in [
        "OMEGA_FALSE_EMPTY",
        "OMEGA_FALSE_BETA",
        "OMEGA_FALSE_MINUS_ALPHA",
    ] {
        assert!(!wikitext.contains(hidden), "{hidden}: {wikitext}");
    }
    assert!(!wikitext.contains("[[iftags"), "{wikitext}");
    assert!(!wikitext.contains("[[#if"), "{wikitext}");
}

#[test]
fn parser_generated_name_and_bracket_fragments_form_iftags_boundaries() {
    // Live preview and saved-page observations are retained under
    // ftml-oracle-20260713T125500Z/run-parser-generated-partial.
    for (source, body) in [
        (
            concat!(
                "[[if[[#if 1 | tags +alpha | tags +beta]]]]\n",
                "OMEGA_KEYWORD_OPENER\n",
                "[[/iftags]]\n",
                "OMEGA_AFTER",
            ),
            "OMEGA_KEYWORD_OPENER",
        ),
        (
            concat!(
                "[[iftags +alpha]]\n",
                "OMEGA_KEYWORD_CLOSER\n",
                "[[/if[[#if 1 | tags | nope]]]]\n",
                "OMEGA_AFTER",
            ),
            "OMEGA_KEYWORD_CLOSER",
        ),
        (
            concat!(
                "[[iftags +alpha[[#if 1 | ] | X]]]\n",
                "OMEGA_RIGHT_BRACKET_OPENER\n",
                "[[/iftags]]\n",
                "OMEGA_AFTER",
            ),
            "OMEGA_RIGHT_BRACKET_OPENER",
        ),
        (
            concat!(
                "[[iftags +alpha]]\n",
                "OMEGA_LEFT_BRACKET_CLOSER\n",
                "[[[#if 1 | [/iftags | [/div]]]]\n",
                "OMEGA_AFTER",
            ),
            "OMEGA_LEFT_BRACKET_CLOSER",
        ),
    ] {
        for (tags, active) in [(&["alpha"][..], true), (&[][..], false)] {
            let html = render_wikidot_conditionals_with_tags(source, tags);

            assert_eq!(html.contains(body), active, "{source:?}: {html}");
            assert!(html.contains("OMEGA_AFTER"), "{source:?}: {html}");
            assert!(!html.contains("[[#if"), "{source:?}: {html}");
            assert!(!html.contains("[[iftags"), "{source:?}: {html}");
            assert!(!html.contains("[[/iftags]]"), "{source:?}: {html}");
        }
    }
}

#[test]
fn parser_generated_boundaries_participate_in_partial_iftags_recovery() {
    // Live preview and saved-page observations are retained under
    // ftml-oracle-20260713T125500Z/run-parser-generated-partial.
    for source in [
        concat!(
            "[[iftags +alpha]]\n",
            "OMEGA_OUTER\n",
            "[[if[[#if 1 | tags +beta | tags +gamma]]]]\n",
            "OMEGA_INNER\n",
            "[[/iftags]]\n",
            "OMEGA_AFTER",
        ),
        concat!(
            "[[iftags +alpha]]\n",
            "OMEGA_OUTER\n",
            "[[iftags +beta]]\n",
            "OMEGA_INNER\n",
            "[[/if[[#if 1 | tags | nope]]]]\n",
            "OMEGA_AFTER",
        ),
    ] {
        let active = render_wikidot_conditionals_with_tags(source, &["alpha"]);
        assert!(active.contains("OMEGA_OUTER"), "{active}");
        assert!(active.contains("OMEGA_INNER"), "{active}");
        assert!(active.contains("[[iftags +beta]]"), "{active}");
        assert!(active.contains("OMEGA_AFTER"), "{active}");
        assert!(!active.contains("[[#if"), "{active}");
        assert!(!active.contains("[[/iftags]]"), "{active}");

        let inactive = render_wikidot_conditionals_with_tags(source, &[]);
        assert!(!inactive.contains("OMEGA_OUTER"), "{inactive}");
        assert!(!inactive.contains("OMEGA_INNER"), "{inactive}");
        assert!(!inactive.contains("[[iftags"), "{inactive}");
        assert!(inactive.contains("OMEGA_AFTER"), "{inactive}");
        assert!(!inactive.contains("[[#if"), "{inactive}");
        assert!(!inactive.contains("[[/iftags]]"), "{inactive}");
    }
}

#[test]
fn parser_generated_partial_recovery_preserves_native_quote_depth() {
    // Live preview and saved-page observations are retained under
    // ftml-oracle-20260713T125500Z/run-parser-generated-partial.
    for (source, depth) in [
        (
            concat!(
                "> [[iftags +alpha]]\n",
                "> OMEGA_OUTER\n",
                "> [[iftags +beta]]\n",
                "> OMEGA_INNER\n",
                "> [[/if[[#if 1 | tags | nope]]]]\n",
                "OMEGA_AFTER",
            ),
            1,
        ),
        (
            concat!(
                ">> [[iftags +alpha]]\n",
                ">> OMEGA_OUTER\n",
                ">> [[if[[#if 1 | tags +beta | tags +gamma]]]]\n",
                ">> OMEGA_INNER\n",
                ">> [[/iftags]]\n",
                "OMEGA_AFTER",
            ),
            2,
        ),
    ] {
        let active = render_wikidot_conditionals_with_tags(source, &["alpha"]);
        assert!(active.contains("OMEGA_OUTER"), "{active}");
        assert!(active.contains("OMEGA_INNER"), "{active}");
        assert!(active.contains("OMEGA_AFTER"), "{active}");
        assert_eq!(active.matches("<blockquote>").count(), depth, "{active}");
        assert!(!active.contains("[[#if"), "{active}");
        assert!(!active.contains("[[/iftags]]"), "{active}");

        let inactive = render_wikidot_conditionals_with_tags(source, &[]);
        assert!(!inactive.contains("OMEGA_OUTER"), "{inactive}");
        assert!(!inactive.contains("OMEGA_INNER"), "{inactive}");
        assert!(inactive.contains("OMEGA_AFTER"), "{inactive}");
        assert_eq!(inactive.matches("<blockquote>").count(), 0, "{inactive}");
        assert!(!inactive.contains("[[#if"), "{inactive}");
        assert!(!inactive.contains("[[/iftags]]"), "{inactive}");
    }
}

#[test]
fn preserves_parser_generated_iftags_inside_literal_regions() {
    // Live preview and saved-page observations are retained under
    // ftml-oracle-20260713T051411Z/run-iftags-quoted-generated.
    let page_info = ftml::data::PageInfo {
        tags: vec![Cow::Borrowed("alpha")],
        ..fallback_test_page_info("tagged-page", "Tagged Page")
    };
    let mut wikitext = concat!(
        "[[code]]\n",
        ">[[iftags [[#ifexpr 1 == 1 | +alpha | +beta ]]]]\n",
        "OMEGA_CODE_BODY\n",
        ">[[/iftags]]\n",
        "[[/code]]\n",
        "@@[[iftags [[#ifexpr 1 == 1 | +alpha | +beta ]]]]",
        "OMEGA_ESCAPE_BODY[[/iftags]]@@\n",
        "> [[raw]]\n",
        "> [[iftags [[#ifexpr 1 == 1 | +alpha | +beta ]]]]\n",
        "> OMEGA_RAW_BODY\n",
        "> [[/iftags]]\n",
        "> [[/raw]]\n",
        "[!-- [[iftags [[#ifexpr 1 == 1 | +alpha | +beta ]]]]",
        "OMEGA_COMMENT_BODY[[/iftags]] --]\n",
        "[[iftags [[#ifexpr 1 == 1 | +alpha | +beta ]]]]",
        "OMEGA_VISIBLE_BODY[[/iftags]]\n",
    )
    .to_owned();

    prepare_test_wikidot_conditionals(&mut wikitext, &page_info);

    for literal in [
        ">[[iftags [[#ifexpr 1 == 1 | +alpha | +beta ]]]]",
        "OMEGA_CODE_BODY",
        "@@[[iftags [[#ifexpr 1 == 1 | +alpha | +beta ]]]]",
        "OMEGA_ESCAPE_BODY",
        "> [[iftags [[#ifexpr 1 == 1 | +alpha | +beta ]]]]",
        "OMEGA_RAW_BODY",
    ] {
        assert!(wikitext.contains(literal), "{literal}: {wikitext}");
    }
    assert!(
        wikitext.contains("[!-- [[iftags +alpha]]OMEGA_COMMENT_BODY[[/iftags]] --]"),
        "{wikitext}",
    );
    assert!(wikitext.contains("OMEGA_VISIBLE_BODY"), "{wikitext}");
}

#[test]
fn resolves_empty_and_empty_negative_iftags_like_saved_wikidot() {
    let page_info = fallback_test_page_info("tagged-page", "Tagged Page");
    let mut wikitext = concat!(
        "[[iftags]]OMEGA_NO_ARGUMENT[[/iftags]]\n",
        "[[iftags -]]OMEGA_EMPTY_NEGATIVE[[/iftags]]\n",
    )
    .to_owned();

    resolve_test_wikidot_iftags(&mut wikitext, &page_info);

    assert_eq!(wikitext, "\nOMEGA_EMPTY_NEGATIVE\n");
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

    resolve_test_wikidot_iftags(&mut wikitext, &page_info);

    assert!(wikitext.contains("[[div_  class=\"Dendo\"]]"));
    assert!(wikitext.contains("[[span class=\"visible \"]]body[[/span]]"));
    assert!(!wikitext.contains("multiline"));
    assert!(!wikitext.contains("display: flex"));
    assert!(!wikitext.contains("hidden"));
}

#[test]
fn prepares_wikidot_unicode_iftags_component_with_cross_closed_collapsible() {
    // scp-jp:component:centered-header-bhl uses this close order; Wikidot renders the outer div around the complete collapsible despite the cross-closed source markers.
    let page_info = ftml::data::PageInfo {
        tags: vec![Cow::Borrowed("theme")],
        language: Cow::Borrowed("ja-JP"),
        ..fallback_test_page_info("ashes-to-ashes", "Ashes to Ashes")
    };
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let source = concat!(
        "[[iftags +コンポーネント]]documentation[[/iftags]]\n",
        "[[div [[iftags -コンポーネント]]style=\"display: none\"[[/iftags]]]]\n",
        "-----\n",
        "[[collapsible show=\"+ show\" hide=\"- hide\"]]\n",
        "[[module CSS show=\"true\"]]\n",
        ".example { color: red; }\n",
        "[[/module]]\n",
        "[[/div]]\n",
        "[[/collapsible]]\n",
    )
    .to_owned();
    let outer = RenderService::prepare_outer_render_wikitext(
        super::ExpandedRenderWikitext {
            theme_previewer_blank: false,
            wikidot_compat_html: CompatHtmlFragments::new(&source),
            wikidot_compat_text: CompatTextFragments::new(&source),
            wikitext: source,
            included_pages: Vec::new(),
            expanded_include_count: 0,
            url_offset_list_pages_content_bytes: 0,
            runtime_css_insertions: Vec::new(),
        },
        &page_info,
        &settings,
    );
    let prepared = RenderService::prepare_inner_render_wikitext(outer, &settings);
    let tokens = ftml::tokenize(&prepared.wikitext);
    let (_, errors) = ftml::parse(&tokens, &page_info, &settings).into();

    assert!(errors.is_empty(), "{errors:#?}\n{}", prepared.wikitext);
    prepared.protection.restore(|_| ());
}

#[test]
fn preserves_wikidot_properly_nested_div_collapsible_markers() {
    let mut source = concat!(
        "[[div class=\"outer\"]]\n",
        "[[collapsible show=\"show\" hide=\"hide\"]]\n",
        "outer body\n",
        "[[/collapsible]]\n",
        "[[/div]]\n",
        "[[collapsible show=\"show\" hide=\"hide\"]]\n",
        "[[div class=\"inner\"]]\n",
        "inner body\n",
        "[[/div]]\n",
        "[[/collapsible]]\n",
        "[[code]]\n",
        "[[div]]\n",
        "[[collapsible]]\n",
        "[[/div]]\n",
        "[[/collapsible]]\n",
        "[[/code]]\n",
    )
    .to_owned();
    let expected = source.clone();

    RenderService::normalize_wikidot_cross_closed_div_collapsibles(&mut source);

    assert_eq!(source, expected);
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

    resolve_test_wikidot_iftags(&mut wikitext, &page_info);

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

    resolve_test_wikidot_iftags(&mut wikitext, &page_info);

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

    resolve_test_wikidot_iftags(&mut wikitext, &page_info);

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

    prepare_test_wikidot_conditionals_before_include_expansion(&mut wikitext, &page_info);

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
fn active_outer_preserves_nested_multiline_wikidot_iftags_literal() {
    let page_info = fallback_test_page_info("black-queen-hub", "Black Queen Hub");
    let mut wikitext = concat!(
        "[[iftags -component]]\n",
        "[[iftags +theme]]nested[[/iftags]]\n",
        "[[/iftags]]\n",
    )
    .to_owned();

    resolve_test_wikidot_iftags(&mut wikitext, &page_info);

    assert!(!wikitext.contains("[[iftags -component]]"));
    assert!(wikitext.contains("[[iftags +theme]]nested[[/iftags]]"));
}

#[test]
fn repeated_render_preparation_preserves_nested_iftags_for_ftml() {
    let page_info = ftml::data::PageInfo {
        tags: vec![Cow::Borrowed("alpha")],
        ..fallback_test_page_info("nested", "Nested")
    };
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut wikitext = concat!(
        "[[iftags +alpha]]\n",
        "outer-before\n",
        "[[iftags +beta]]inner[[/iftags]]\n",
        "outer-after\n",
        "[[/iftags]]\n",
        "root-after\n",
    )
    .to_owned();
    let mut wikidot_compat_text = CompatTextFragments::new(&wikitext);
    for _ in 0..2 {
        RenderService::prepare_wikidot_conditionals_for_include_expansion(
            &mut wikitext,
            &page_info,
            &mut wikidot_compat_text,
        );
    }
    assert!(wikitext.contains(COMPAT_TEXT_MARKER_PREFIX), "{wikitext}");

    let outer = RenderService::prepare_outer_render_wikitext(
        super::ExpandedRenderWikitext {
            theme_previewer_blank: false,
            wikitext,
            included_pages: Vec::new(),
            expanded_include_count: 0,
            url_offset_list_pages_content_bytes: 0,
            runtime_css_insertions: Vec::new(),
            wikidot_compat_html: CompatHtmlFragments::new(""),
            wikidot_compat_text,
        },
        &page_info,
        &settings,
    );
    assert!(!outer.wikitext.contains("[[iftags +alpha]]"));
    assert!(outer.wikitext.contains(COMPAT_TEXT_MARKER_PREFIX));
    assert!(outer.wikitext.contains("root-after"));

    let inner = RenderService::prepare_inner_render_wikitext(outer, &settings);
    let tokens = ftml::tokenize(&inner.wikitext);
    let (tree, errors) = ftml::parse(&tokens, &page_info, &settings).into();
    assert!(errors.is_empty(), "{errors:#?}");
    let html = HtmlRender.render(&tree, &page_info, &settings).body;
    let html = inner
        .protection
        .restore(|protection| protection.compat_text().restore(&html));
    assert!(html.contains("[[iftags +beta]]inner[[/iftags]]"), "{html}");
    assert!(html.contains("root-after"), "{html}");
}

#[test]
fn malformed_iftags_remain_literal_after_ftml_recovery() {
    let page_info = ftml::data::PageInfo {
        tags: vec![Cow::Borrowed("alpha")],
        ..fallback_test_page_info("malformed", "Malformed")
    };
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let mut wikitext = concat!(
        "[[/iftags]]\n",
        "[[iftags +alpha]]selected[[/iftags]]\n",
        "[[iftags +alpha]]unclosed\n",
        "[[iftags -alpha]]repeated\n",
        "**later bold**\n",
    )
    .to_owned();
    let mut wikidot_compat_text = CompatTextFragments::new(&wikitext);
    for _ in 0..2 {
        RenderService::prepare_wikidot_conditionals_for_include_expansion(
            &mut wikitext,
            &page_info,
            &mut wikidot_compat_text,
        );
    }

    let outer = RenderService::prepare_outer_render_wikitext(
        super::ExpandedRenderWikitext {
            theme_previewer_blank: false,
            wikitext,
            included_pages: Vec::new(),
            expanded_include_count: 0,
            url_offset_list_pages_content_bytes: 0,
            runtime_css_insertions: Vec::new(),
            wikidot_compat_html: CompatHtmlFragments::new(""),
            wikidot_compat_text,
        },
        &page_info,
        &settings,
    );
    let inner = RenderService::prepare_inner_render_wikitext(outer, &settings);
    let tokens = ftml::tokenize(&inner.wikitext);
    let (tree, errors) = ftml::parse(&tokens, &page_info, &settings).into();
    assert!(errors.is_empty(), "{errors:#?}");
    let html = HtmlRender.render(&tree, &page_info, &settings).body;
    let html = inner
        .protection
        .restore(|protection| protection.compat_text().restore(&html));
    for literal in ["[[/iftags]]", "[[iftags +alpha]]", "[[iftags -alpha]]"] {
        assert!(html.contains(literal), "{literal}: {html}");
    }
    for marker in ["selected", "unclosed", "repeated"] {
        assert!(html.contains(marker), "{marker}: {html}");
    }
    assert!(html.contains("<strong>later bold</strong>"), "{html}");
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
        favicon_source: None,
        ios_icon_source: None,
        windows_tile_source: None,
        settings_revision: 0,
        welcome_page: "system:welcome".to_owned(),
        google_analytics_enabled: false,
        google_analytics_profile: None,
        show_top_toolbar: false,
        show_bottom_toolbar: false,
        promote_on_other_sites: true,
        membership_by_application: false,
        membership_by_password: false,
        membership_password_hash: None,
        master_admin_user_id: None,
        educational: false,
        educational_organization: None,
        educational_purpose: None,
        layout: None,
        license: License::CcBySa30,
        forum_max_nest_level: 10,
    }
}
