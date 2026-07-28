/*
 * services/render/list_pages/rendering.rs
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

use super::super::compat::CompatHtmlFragments;
use super::super::compat::preparation::neutralize_authored_markers;
use super::super::compat::text_fragments::{CompatTextFragments, escape_html_text};
use super::super::include_attachment_owners::AttachmentOwner;
use super::super::literal_regions::{ListPagesSourceProjection, LiteralRegionIndex};
use super::super::runtime::{IncludeSourceCache, RenderRuntime};
use super::super::runtime_page_queries::{
    CountPagesRawScanCompletion, render_page_query_uses_single_scan,
};
use super::super::service::{
    COUNTPAGES_MODULE_REGEX, CountPagesRequiredTagBatchResult,
    DEFAULT_LISTPAGES_PER_PAGE, IncludeExpansion, IncludeExpansionBudget,
    IncludeExpansionOptions, MAX_LISTPAGES_CONTENT_MODULES_PER_RENDER,
    MAX_LISTPAGES_CONTENT_ROWS_PER_RENDER, MAX_LISTPAGES_RENDER_LIMIT,
    MAX_LISTPAGES_RENDER_SCAN_ROWS, RenderService, render_list_pages_numbered_rows,
    render_list_pages_table_rows,
};
use super::content_sections::{isolate_wikidot_content_section, wikidot_content_section};
use super::parents::{load_list_pages_child_counts, load_list_pages_parent_displays};
use super::scanner::{
    CountPagesCloseReachabilityIndex, find_list_pages_module_matches,
    has_count_pages_module_opening_candidate, has_list_pages_module_opening_candidate,
    list_pages_body_has_standalone_count_pages_opening,
    list_pages_body_inline_count_pages_legacy_tail, list_pages_runtime_head_can_execute,
};
use super::template::{ListPagesOutputShape, ListPagesTemplatePlan};
use super::{
    ExactNameListPagesBatchKey, ListPagesArguments, ListPagesAuthorCacheKey,
    ListPagesBatchDisplayRequirements, ListPagesBatchDisplays,
    ListPagesSubstitutionContext, ResolvedListPagesAuthors,
    count_pages_capture_is_literal, count_pages_exact_count_render_diagnostics,
    count_pages_required_tag_batch_result, count_pages_required_tag_batch_selector,
    count_pages_scan_requires_preservation, count_pages_should_remain_literal,
    count_pages_unbounded_total, exact_name_list_pages_batch_key,
    is_list_pages_visible_tag, list_pages_argument_error,
    list_pages_content_query_target, list_pages_created_by_unix,
    list_pages_feed_info_html, list_pages_has_unsupported_page_type_selector,
    list_pages_has_unsupported_parent_selector, list_pages_parent_fullname,
    list_pages_revision_count, list_pages_row_scan_target,
    list_pages_static_parent_fullname, load_list_pages_data_form_definitions,
    page_query_cap_requires_original_module, parse_list_pages_arguments,
    parse_list_pages_arguments_with_url,
    preserve_list_pages_following_paragraph_boundary,
    protect_ajax_module_literal_markers, push_list_pages_pager,
    register_generated_list_pages_html, should_render_current_page_list_pages_row,
    substitute_count_pages_variables, substitute_list_pages_rating_only,
    substitute_list_pages_variables_with_fragments, union_found_page_fields,
    unsupported_list_pages_replacement, url_offset_list_pages_content_bytes,
};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::hash::{TextHash, k12_hash};
use crate::models::page_category::{self, Entity as PageCategory};
use crate::services::ServiceContext;
use crate::services::page_query::{
    AuthorSelector, CategoriesSelector, ComparisonOperation, DateSelector,
    DateTimeResolution, FoundPageFields, FoundPageRow, FoundPages, IncludedCategories,
    ListPagesRenderDiagnosticsInput, OrderProperty, PageParentSelector, PageQuery,
    PageQueryScoreFilterCache, PageTypeSelector, PaginationSelector, RangeSelector,
    ScoreSelector, TagCondition, list_pages_render_diagnostics,
    parse_static_wikidot_data_form_values,
};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::render::UrlArguments;
use crate::services::{
    CategoryService, PageQueryService, PageRevisionService, PageService, SiteService,
};
use crate::types::{Action, Permission, Reference, Resource};
use ftml::data::PageInfo;
use ftml::settings::WikitextSettings;
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, FromQueryResult, QueryFilter, Statement,
    Value,
};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

const MAX_NESTED_LISTPAGES_DEPTH: usize = 8;
const MAX_NESTED_LISTPAGES_MODULES_PER_PASS: usize = 64;

fn list_pages_body_starts_with_preparsed_block(body: &str) -> bool {
    let body = body.trim_start_matches(char::is_whitespace);
    body.get(.."[[code]]".len())
        .is_some_and(|opening| opening.eq_ignore_ascii_case("[[code]]"))
        || body
            .get(.."[[html]]".len())
            .is_some_and(|opening| opening.eq_ignore_ascii_case("[[html]]"))
}

#[derive(Debug)]
pub(in crate::services::render) enum ListPagesBlockRenderResult {
    Expanded(IncludeExpansion),
    PreserveOriginal,
}

#[derive(Debug)]
pub(in crate::services::render) struct ListPagesExpansion {
    pub(in crate::services::render) wikitext: String,
    pub(in crate::services::render) included_pages: Vec<ftml::data::PageRef>,
    pub(in crate::services::render) expanded_include_count: usize,
    pub(in crate::services::render) url_offset_content_bytes: usize,
}

#[derive(Debug, Clone)]
pub(in crate::services::render) enum CountPagesBlockRenderResult {
    Expanded(String),
    PreserveOriginal,
}

#[derive(Debug, FromQueryResult)]
pub(in crate::services::render) struct CountPagesRequiredTagTotal {
    pub(in crate::services::render) tag: String,
    pub(in crate::services::render) total: i64,
}

pub(in crate::services::render) struct CountPagesRequiredTagSource<'a> {
    pub(in crate::services::render) literal_regions: &'a LiteralRegionIndex,
    pub(in crate::services::render) close_reachability:
        &'a CountPagesCloseReachabilityIndex,
    pub(in crate::services::render) source_projection:
        Option<&'a ListPagesSourceProjection>,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::services::render) struct ListPagesPageContext<'a> {
    pub(in crate::services::render) site_id: i64,
    pub(in crate::services::render) page_id: Option<i64>,

    /// Wikidot URL path arguments from the current page view.
    pub(in crate::services::render) url: UrlArguments<'a>,
}

#[derive(Debug, Default)]
pub(in crate::services::render) struct ListPagesContentCache {
    pub(in crate::services::render) wikitext: BTreeMap<(i64, i64), Option<String>>,
    pub(in crate::services::render) wikitext_scalar_count:
        BTreeMap<(i64, i64), Option<usize>>,
}

#[derive(Debug)]
pub(in crate::services::render) struct ListPagesExpansionBudget {
    pub(in crate::services::render) remaining_content_modules: usize,
    pub(in crate::services::render) remaining_content_rows: usize,
}

impl ListPagesExpansionBudget {
    pub(in crate::services::render) fn new() -> Self {
        Self {
            remaining_content_modules: MAX_LISTPAGES_CONTENT_MODULES_PER_RENDER,
            remaining_content_rows: MAX_LISTPAGES_CONTENT_ROWS_PER_RENDER,
        }
    }

    pub(in crate::services::render) fn try_start_content_module(&mut self) -> bool {
        if self.remaining_content_modules == 0 {
            return false;
        }
        self.remaining_content_modules -= 1;
        true
    }

    pub(in crate::services::render) fn remaining_content_rows(&self) -> usize {
        self.remaining_content_rows
    }

    pub(in crate::services::render) fn can_expand_content_rows(
        &self,
        rows: usize,
    ) -> bool {
        rows <= self.remaining_content_rows
    }

    pub(in crate::services::render) fn consume_content_rows(&mut self, rows: usize) {
        debug_assert!(self.can_expand_content_rows(rows));
        self.remaining_content_rows = self.remaining_content_rows.saturating_sub(rows);
    }
}

#[derive(Clone, Copy, Debug)]
pub(in crate::services::render) struct ListPagesExpansionOptions<'a> {
    pub(in crate::services::render) current_site_id: Option<i64>,
    pub(in crate::services::render) current_page_id: Option<i64>,
    pub(in crate::services::render) include_budget: IncludeExpansionBudget,

    /// The Wikidot URL path arguments this request carried.
    pub(in crate::services::render) url: UrlArguments<'a>,
}

/// The request a CountPages expansion is answering.
pub(in crate::services::render) struct CountPagesExpansionOptions<'a> {
    pub(in crate::services::render) current_site_id: Option<i64>,
    pub(in crate::services::render) current_page_id: Option<i64>,

    /// The Wikidot URL path arguments this request carried; a `tags` selector
    /// can name the tag as `@URL`, and `/p/<n>` picks the rendered page.
    pub(in crate::services::render) url: UrlArguments<'a>,
}

impl RenderService {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::services::render) async fn expand_list_pages(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        compat_html: &mut CompatHtmlFragments,
        include_source_cache: &mut IncludeSourceCache,
        compat_text: &mut CompatTextFragments,
        options: ListPagesExpansionOptions<'_>,
    ) -> Result<ListPagesExpansion> {
        let mut expansion_budget = ListPagesExpansionBudget::new();
        let mut seen = BTreeSet::new();
        Box::pin(Self::expand_list_pages_nested(
            ctx,
            wikitext,
            page_info,
            settings,
            compat_html,
            include_source_cache,
            compat_text,
            options,
            &mut expansion_budget,
            &mut seen,
            0,
        ))
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn expand_list_pages_nested(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        compat_html: &mut CompatHtmlFragments,
        include_source_cache: &mut IncludeSourceCache,
        compat_text: &mut CompatTextFragments,
        options: ListPagesExpansionOptions<'_>,
        expansion_budget: &mut ListPagesExpansionBudget,
        seen: &mut BTreeSet<TextHash>,
        depth: usize,
    ) -> Result<ListPagesExpansion> {
        let ListPagesExpansionOptions {
            current_site_id,
            current_page_id,
            mut include_budget,
            url,
        } = options;
        let Some(current_site_id) = current_site_id else {
            return Ok(ListPagesExpansion {
                wikitext,
                included_pages: Vec::new(),
                expanded_include_count: 0,
                url_offset_content_bytes: 0,
            });
        };
        let requested_current_page_id = current_page_id;
        let current_page_id = current_page_id.unwrap_or(0);

        if !settings.enable_page_syntax {
            return Ok(ListPagesExpansion {
                wikitext,
                included_pages: Vec::new(),
                expanded_include_count: 0,
                url_offset_content_bytes: 0,
            });
        }

        if !has_list_pages_module_opening_candidate(&wikitext) {
            return Ok(ListPagesExpansion {
                wikitext,
                included_pages: Vec::new(),
                expanded_include_count: 0,
                url_offset_content_bytes: 0,
            });
        }

        if !seen.insert(k12_hash(wikitext.as_bytes())) {
            return Ok(ListPagesExpansion {
                wikitext,
                included_pages: Vec::new(),
                expanded_include_count: 0,
                url_offset_content_bytes: 0,
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
                legacy_tail: Option<String>,
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
        let module_matches = find_list_pages_module_matches(&wikitext);
        if depth > 0 && module_matches.len() > MAX_NESTED_LISTPAGES_MODULES_PER_PASS {
            return Ok(ListPagesExpansion {
                wikitext,
                included_pages: Vec::new(),
                expanded_include_count: 0,
                url_offset_content_bytes: 0,
            });
        }
        let static_parent_references = module_matches
            .iter()
            .filter(|module| list_pages_runtime_head_can_execute(module.head))
            .filter_map(|module| list_pages_static_parent_fullname(module.head))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|parent| Reference::Slug(Cow::Borrowed(parent)))
            .collect::<Vec<_>>();
        let existing_static_parents =
            PageService::get_pages(ctx, current_site_id, &static_parent_references)
                .await?
                .into_iter()
                .map(|page| page.slug)
                .collect::<BTreeSet<_>>();
        let blocks = module_matches
            .into_iter()
            .map(|module| {
                let head = module.head;
                // Wikidot's code/html pass owns a leading body block before
                // ListPages evaluates. The remaining ListPages opening is
                // therefore an empty, unclosed module using the default
                // template, while the owned block and closing module remain
                // downstream source.
                let body_was_preparsed =
                    list_pages_body_starts_with_preparsed_block(module.body);
                let body = if body_was_preparsed { "" } else { module.body };
                let module_end = if body_was_preparsed {
                    module.body_start
                } else {
                    module.end
                };
                let module_original = if body_was_preparsed {
                    &wikitext[module.start..module.body_start]
                } else {
                    module.original
                };
                let head_can_execute = list_pages_runtime_head_can_execute(head);
                let feed_only_plan = head_can_execute
                    .then(|| parse_list_pages_arguments_with_url(head, url))
                    .flatten()
                    .filter(|arguments| {
                        arguments.rss_only
                            && arguments
                                .rss_title
                                .as_deref()
                                .is_some_and(|title| !title.is_empty())
                    })
                    .and_then(|arguments| {
                        ListPagesTemplatePlan::compile("").map(|template| {
                            ListPagesBlockPlan::Render {
                                arguments,
                                template,
                                batch_key: None,
                                legacy_tail: None,
                            }
                        })
                    });
                let plan = if let Some(plan) = feed_only_plan {
                    plan
                } else if let Some(error) =
                    list_pages_argument_error(head, requested_current_page_id.is_some())
                {
                    ListPagesBlockPlan::Static(compat_html.push_block_html(format!(
                        r#"<div class="error-block">{error}</div>"#,
                    )))
                } else if let Some(parent) = list_pages_static_parent_fullname(head)
                    && !existing_static_parents.contains(parent)
                {
                    ListPagesBlockPlan::Static(compat_html.push_block_html(format!(
                        r#"<div class="error-block">Parent page {} does not exist</div>"#,
                        escape_html_text(parent),
                    )))
                } else if !head_can_execute
                    || list_pages_has_unsupported_parent_selector(head)
                    || list_pages_has_unsupported_page_type_selector(head)
                {
                    ListPagesBlockPlan::PreserveOriginal
                } else if let Some(arguments) =
                    parse_list_pages_arguments_with_url(head, url)
                {
                    if arguments.unsupported_author_filter
                        || arguments.unsupported_list_pages_filter
                        || arguments.unsupported_score_filter
                    {
                        ListPagesBlockPlan::PreserveOriginal
                    } else if let Some(legacy_tail) =
                        list_pages_body_inline_count_pages_legacy_tail(body)
                    {
                        ListPagesTemplatePlan::compile("").map_or_else(
                            || unsupported_plan(module_original, body),
                            |template| ListPagesBlockPlan::Render {
                                arguments,
                                template,
                                batch_key: None,
                                legacy_tail: Some(legacy_tail),
                            },
                        )
                    } else if list_pages_body_has_standalone_count_pages_opening(body) {
                        ListPagesTemplatePlan::compile("").map_or_else(
                            || unsupported_plan(module_original, body),
                            |template| {
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
                                    legacy_tail: None,
                                }
                            },
                        )
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
                            legacy_tail: None,
                        }
                    } else {
                        unsupported_plan(module_original, body)
                    }
                } else {
                    unsupported_plan(module_original, body)
                };
                ListPagesBlock {
                    start: module.start,
                    end: module_end,
                    plan,
                }
            })
            .collect::<Vec<_>>();

        let mut expanded = String::with_capacity(wikitext.len());
        let mut included_pages = Vec::new();
        let mut url_offset_content_bytes = 0usize;
        let mut content_cache = ListPagesContentCache::default();
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
                        legacy_tail: _,
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
                        legacy_tail,
                        ..
                    } = block.plan
                    else {
                        unreachable!();
                    };
                    let offset_origin = arguments.offset_origin;
                    let uses_content = template.uses_content();
                    let slug = arguments.slug.as_ref().unwrap().to_string();
                    let prefetched_pages =
                        prefetched.as_ref().map(|prefetched| FoundPages {
                            pages: prefetched.get(&slug).cloned().unwrap_or_default(),
                        });
                    let rendered = Box::pin(Self::render_list_pages_block(
                        ctx,
                        ListPagesPageContext {
                            site_id: current_site_id,
                            page_id: requested_current_page_id,
                            url,
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
                        expansion_budget,
                        &mut permission_cache,
                        &mut score_filter_cache,
                        &mut author_resolution_cache,
                        compat_text,
                    ))
                    .await?;
                    match rendered {
                        ListPagesBlockRenderResult::Expanded(IncludeExpansion {
                            wikitext: mut replacement,
                            included_pages: replacement_included_pages,
                            expanded_include_count: replacement_expanded_include_count,
                        }) => {
                            include_budget.consume(replacement_expanded_include_count);
                            let following_source =
                                legacy_tail.as_deref().unwrap_or(&wikitext[block.end..]);
                            preserve_list_pages_following_paragraph_boundary(
                                &mut replacement,
                                following_source,
                            );
                            let replacement = register_generated_list_pages_html(
                                replacement,
                                compat_html,
                            );
                            url_offset_content_bytes = url_offset_content_bytes
                                .saturating_add(url_offset_list_pages_content_bytes(
                                    offset_origin,
                                    uses_content,
                                    &replacement,
                                ));
                            expanded.push_str(&replacement);
                            if let Some(legacy_tail) = legacy_tail {
                                expanded.push_str(&legacy_tail);
                            }
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
                    legacy_tail,
                    ..
                } => {
                    let offset_origin = arguments.offset_origin;
                    let uses_content = template.uses_content();
                    let rendered = Box::pin(Self::render_list_pages_block(
                        ctx,
                        ListPagesPageContext {
                            site_id: current_site_id,
                            page_id: requested_current_page_id,
                            url,
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
                        expansion_budget,
                        &mut permission_cache,
                        &mut score_filter_cache,
                        &mut author_resolution_cache,
                        compat_text,
                    ))
                    .await?;
                    match rendered {
                        ListPagesBlockRenderResult::Expanded(IncludeExpansion {
                            wikitext: mut replacement,
                            included_pages: replacement_included_pages,
                            expanded_include_count: replacement_expanded_include_count,
                        }) => {
                            include_budget.consume(replacement_expanded_include_count);
                            let following_source =
                                legacy_tail.as_deref().unwrap_or(&wikitext[block.end..]);
                            preserve_list_pages_following_paragraph_boundary(
                                &mut replacement,
                                following_source,
                            );
                            let replacement = register_generated_list_pages_html(
                                replacement,
                                compat_html,
                            );
                            url_offset_content_bytes = url_offset_content_bytes
                                .saturating_add(url_offset_list_pages_content_bytes(
                                    offset_origin,
                                    uses_content,
                                    &replacement,
                                ));
                            expanded.push_str(&replacement);
                            if let Some(legacy_tail) = legacy_tail {
                                expanded.push_str(&legacy_tail);
                            }
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
        let expanded = if page_info.page.as_ref() == "_ajax-module-connector" {
            protect_ajax_module_literal_markers(expanded, compat_text)
        } else {
            expanded
        };
        let mut expansion = ListPagesExpansion {
            wikitext: expanded,
            included_pages,
            expanded_include_count: initial_remaining_include_expansions
                .saturating_sub(include_budget.remaining),
            url_offset_content_bytes,
        };
        if depth < MAX_NESTED_LISTPAGES_DEPTH
            && has_list_pages_module_opening_candidate(&expansion.wikitext)
        {
            let nested = Box::pin(Self::expand_list_pages_nested(
                ctx,
                std::mem::take(&mut expansion.wikitext),
                page_info,
                settings,
                compat_html,
                include_source_cache,
                compat_text,
                ListPagesExpansionOptions {
                    current_site_id: Some(current_site_id),
                    current_page_id: requested_current_page_id,
                    include_budget,
                    url,
                },
                expansion_budget,
                seen,
                depth + 1,
            ))
            .await?;
            expansion.wikitext = nested.wikitext;
            expansion.included_pages.extend(nested.included_pages);
            expansion.expanded_include_count = expansion
                .expanded_include_count
                .saturating_add(nested.expanded_include_count);
            expansion.url_offset_content_bytes = expansion
                .url_offset_content_bytes
                .saturating_add(nested.url_offset_content_bytes);
        }
        Ok(expansion)
    }

    pub(in crate::services::render) async fn load_exact_name_list_pages_batch(
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
                untagged: false,
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

    pub(in crate::services::render) async fn load_list_pages_batch_displays(
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
        let runtime_displays = if requirements.runtime {
            Self::load_list_pages_runtime_displays(ctx, pages).await?
        } else {
            BTreeMap::new()
        };
        Ok(ListPagesBatchDisplays {
            user_displays,
            snapshot_displays,
            runtime_displays,
        })
    }

    pub(in crate::services::render) async fn expand_count_pages(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        options: CountPagesExpansionOptions<'_>,
        compat_text: &mut CompatTextFragments,
    ) -> Result<String> {
        let CountPagesExpansionOptions {
            current_site_id,
            current_page_id,
            url,
        } = options;
        let Some(current_site_id) = current_site_id else {
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
            // CountPages renders a total, not a page of rows, so a `/p/<n>`
            // in the path does not change what it counts.
            url,
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

            let Some(arguments) = parse_list_pages_arguments_with_url(head, url) else {
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

    pub(in crate::services::render) async fn load_count_pages_required_tag_totals(
        ctx: &ServiceContext<'_>,
        wikitext: &str,
        source: CountPagesRequiredTagSource<'_>,
        page_info: &PageInfo<'_>,
        page_context: ListPagesPageContext<'_>,
        permission_cache: &mut BTreeMap<(i64, Option<i64>), bool>,
    ) -> Result<BTreeMap<(Vec<String>, String), CountPagesRequiredTagBatchResult>> {
        let ListPagesPageContext {
            site_id: current_site_id,
            page_id: current_page_identity,
            // CountPages renders a total, so the requested page does not apply.
            url: _,
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
            // This prefetch only serves CountPages, whose own URL-argument
            // behavior is uncaptured, so a head naming `@URL` keeps the
            // module literal rather than resolving here.
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
                    page_reference: current_page_identity.map(Reference::Id),
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

    pub(in crate::services::render) fn categories_with_current_page_category(
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

    pub(in crate::services::render) fn page_info_category_slug<'a>(
        page_info: &'a PageInfo<'_>,
    ) -> Cow<'a, str> {
        page_info
            .category
            .as_ref()
            .map(|category| Cow::Borrowed(category.as_ref()))
            .unwrap_or(Cow::Borrowed("_default"))
    }

    pub(in crate::services::render) fn page_info_full_slug(
        page_info: &PageInfo<'_>,
    ) -> String {
        let page = page_info.page.as_ref();
        match Self::page_info_category_slug(page_info).as_ref() {
            "_default" => page.to_owned(),
            category => format!("{category}:{page}"),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::services::render) async fn render_list_pages_block(
        ctx: &ServiceContext<'_>,
        page_context: ListPagesPageContext<'_>,
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
            page_id: current_page_identity,
            url,
        } = page_context;
        let current_page_id = current_page_identity.unwrap_or(0);
        let ajax_module_response = page_info.page.as_ref() == "_ajax-module-connector";
        let initial_remaining_include_expansions = include_budget.remaining;
        let feed_info = list_pages_feed_info_html(page_info, &arguments);
        if arguments.rss_only
            && let Some(feed_info) = feed_info
        {
            return Ok(ListPagesBlockRenderResult::Expanded(IncludeExpansion {
                wikitext: feed_info,
                included_pages: Vec::new(),
                expanded_include_count: 0,
            }));
        }
        let ListPagesArguments {
            current_page_only,
            category_selector_present,
            category_all,
            include_current_category,
            categories,
            excluded_categories,
            mut any_tags,
            mut all_tags,
            default_tags,
            no_tags,
            untagged,
            same_visible_tags,
            exact_visible_tags,
            authors,
            author_filter_present,
            order,
            reverse,
            limit,
            count_pages_explicit_limit: _,
            count_pages_per_page,
            url_attr_prefix,
            offset,
            offset_origin: _,
            offset_beyond_render_window,
            exclude_current_page,
            relative_range,
            page_type,
            page_parent,
            static_parent_fullname,
            mut creation_date,
            mut update_date,
            creation_date_current_page,
            update_date_current_page,
            mut score,
            score_equals_current_page,
            mut votes,
            votes_equals_current_page,
            slug,
            name_pattern,
            data_form_fields,
            prepend_line,
            append_line,
            separate,
            wrapper,
            rss_title: _,
            rss_description: _,
            rss_home: _,
            rss_limit: _,
            rss_only: _,
            rss_path: _,
            exclude_current_page_author,
            unsupported_author_filter: _,
            unsupported_list_pages_filter: _,
            link_to,
            unsupported_score_filter: _,
            unsupported_count_pages_filter: _,
        } = arguments;
        any_tags.extend(default_tags);
        let current_visible_tags = page_info
            .tags
            .iter()
            .filter(|tag| is_list_pages_visible_tag(tag))
            .map(|tag| tag.to_string())
            .collect::<BTreeSet<_>>();
        if same_visible_tags {
            any_tags.extend(current_visible_tags.iter().cloned().map(Cow::Owned));
        }
        if exact_visible_tags {
            all_tags.extend(current_visible_tags.iter().cloned().map(Cow::Owned));
        }
        let current_page_date_missing = current_page_identity.is_none()
            && (creation_date_current_page || update_date_current_page);
        if let Some(current_page_id) = current_page_identity
            && (creation_date_current_page || update_date_current_page)
        {
            let page = PageService::get_direct(ctx, current_page_id, false)
                .await
                .or_raise(|| {
                    Error::new(
                        "failed to load current page dates for ListPages render",
                        ErrorType::Render,
                    )
                })?;
            if creation_date_current_page {
                creation_date = DateSelector::Span {
                    timestamp: page.created_at,
                    resolution: DateTimeResolution::Day,
                    comparison: ComparisonOperation::Equal,
                };
            }
            if update_date_current_page {
                update_date = DateSelector::Span {
                    timestamp: page.updated_at.unwrap_or(page.created_at),
                    resolution: DateTimeResolution::Day,
                    comparison: ComparisonOperation::Equal,
                };
            }
        }
        if score_equals_current_page {
            score.push(ScoreSelector {
                score: page_info.score,
                comparison: ComparisonOperation::Equal,
            });
        }
        let mut votes_equal_current_zero_votes = false;
        if votes_equals_current_page {
            let current_votes = if current_page_identity.is_some() {
                PageQueryService::effective_vote_count(ctx, current_page_id)
                    .await
                    .or_raise(|| {
                        Error::new(
                            "failed to load current page vote count for ListPages render",
                            ErrorType::Render,
                        )
                    })?
            } else {
                0
            };
            if current_page_identity.is_some() && current_votes == 0 {
                votes_equal_current_zero_votes = true;
            } else {
                votes.push(ScoreSelector {
                    score: ftml::data::ScoreValue::Integer(current_votes),
                    comparison: ComparisonOperation::Equal,
                });
            }
        }
        let current_page_full_slug = Self::page_info_full_slug(page_info);
        let link_to_references = link_to
            .iter()
            .map(|slug| {
                let slug = if slug.as_ref() == "." {
                    current_page_full_slug.as_str()
                } else {
                    slug.as_ref()
                };
                Reference::Slug(Cow::Borrowed(slug))
            })
            .collect::<Vec<_>>();
        let static_parent_references = static_parent_fullname
            .as_ref()
            .map(|parent| [Reference::Slug(Cow::Borrowed(parent.as_ref()))]);
        let page_parent = static_parent_references
            .as_ref()
            .map_or(page_parent, |parents| {
                PageParentSelector::HasParents(parents)
            });
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
        let per_page = count_pages_per_page
            .unwrap_or(DEFAULT_LISTPAGES_PER_PAGE)
            .min(MAX_LISTPAGES_RENDER_LIMIT);
        let url_page = url.page_for_prefix(url_attr_prefix.as_deref());
        let oversized_offset_initial_page =
            offset_beyond_render_window.is_some() && url_page.unwrap_or(1) <= 1;
        let offset = match (offset_beyond_render_window, url_page) {
            (Some(raw_offset), Some(page)) if page > 1 => (raw_offset % per_page) as u32,
            _ => offset,
        };
        let query_limit = list_pages_row_scan_target(
            per_page,
            if relative_range.is_some() {
                None
            } else {
                limit
            },
            Some(per_page),
            offset,
            exclude_current_page,
        );
        let wants_content = template.uses_content();
        let wants_size = template.uses_size();
        if wants_content
            && render_page_query_uses_single_scan(order.clone())
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
        let wants_created_by_unix = template.uses_created_by_unix();
        let wants_created_at = template.uses_created_at();
        let wants_updated_by = template.uses_updated_by();
        let wants_updated_at = template.uses_updated_at();
        let wants_rating_votes = template.uses_rating_votes();
        let wants_site_domain = template.uses_site_domain();
        let wants_parent_metadata = template.uses_parent_metadata();
        let wants_revisions = template.uses_revisions();
        let wants_children = template.uses_children();
        let resolved_authors = Self::resolve_list_pages_authors_cached(
            ctx,
            current_site_id,
            current_page_id,
            &authors,
            author_filter_present,
            exclude_current_page_author,
            author_resolution_cache,
        )
        .await?;
        let mut query_fields = template.fields();
        query_fields.tags |= exact_visible_tags;
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
                untagged,
            },
            page_parent,
            contains_outgoing_links: &link_to_references,
            creation_date,
            update_date,
            author: resolved_authors.as_selector(),
            score: &score,
            votes: &votes,
            offset: 0,
            range: RangeSelector::Current,
            name: name_pattern,
            slug,
            slugs: &[],
            data_form_fields: &data_form_fields,
            order: order.clone(),
            candidate_limit: if data_form_fields.is_empty()
                && !matches!(
                    order.as_ref().map(|order| &order.property),
                    Some(OrderProperty::Score | OrderProperty::DataFormFieldName { .. })
                ) {
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
            fields: query_fields,
        };

        let mut list_pages_metadata = None;
        let missing_current_page_for_selector = current_page_identity.is_none()
            && (current_page_only || exclude_current_page_author);
        let pages = if oversized_offset_initial_page
            || current_page_date_missing
            || votes_equal_current_zero_votes
            || missing_current_page_for_selector
            || (same_visible_tags && current_visible_tags.is_empty())
        {
            FoundPages { pages: Vec::new() }
        } else if current_page_only
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
                if wants_content && !render_page_query_uses_single_scan(order.clone()) {
                    list_pages_content_query_target(
                        query_limit,
                        per_page,
                        expansion_budget.remaining_content_rows(),
                        offset,
                        exclude_current_page,
                        true,
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
        let query_returned_every_match =
            list_pages_metadata.as_ref().is_some_and(|(metadata, _)| {
                !metadata.cap_exceeded
                    && !metadata.filtering_deferred_to_rust
                    && metadata.candidate_count.is_some_and(|candidate_count| {
                        (candidate_count as u64) < MAX_LISTPAGES_RENDER_LIMIT
                    })
            });
        if let Some((metadata, view_permission_filtering_applied)) = list_pages_metadata {
            let diagnostics =
                list_pages_render_diagnostics(ListPagesRenderDiagnosticsInput {
                    metadata,
                    view_permission_filtering_applied,
                    post_query_exclusion_applied: exclude_current_page,
                    post_query_offset_applied: offset > 0,
                    requested_limit: per_page,
                    query_limit,
                });
            debug!("ListPages render diagnostics: {diagnostics:?}");
        }
        let all_selected_pages = pages
            .pages
            .into_iter()
            .filter(|page| {
                !exact_visible_tags
                    || page
                        .tags
                        .as_deref()
                        .unwrap_or_default()
                        .iter()
                        .filter(|tag| is_list_pages_visible_tag(tag))
                        .map(String::as_str)
                        .collect::<BTreeSet<_>>()
                        == current_visible_tags
                            .iter()
                            .map(String::as_str)
                            .collect::<BTreeSet<_>>()
            })
            .collect::<Vec<_>>();
        let all_selected_pages = if let Some(relative_range) = relative_range {
            let current_index = all_selected_pages
                .iter()
                .position(|page| page.page_id == current_page_id);
            match (relative_range, current_index) {
                (RangeSelector::Before, Some(index)) => {
                    all_selected_pages.into_iter().take(index).collect()
                }
                (RangeSelector::After, Some(index)) => {
                    all_selected_pages.into_iter().skip(index + 1).collect()
                }
                _ => Vec::new(),
            }
        } else {
            all_selected_pages
        };
        let all_selected_pages = all_selected_pages
            .into_iter()
            .filter(|page| !exclude_current_page || page.page_id != current_page_id)
            .skip(offset as usize)
            .collect::<Vec<_>>();
        let all_selected_total = all_selected_pages.len();
        let selected_pages = all_selected_pages
            .into_iter()
            .take(
                limit
                    .and_then(|limit| usize::try_from(limit).ok())
                    .unwrap_or(usize::MAX),
            )
            .collect::<Vec<_>>();
        let total_selected = selected_pages.len();

        // `/p/<n>` picks which page of an already-paginated module to render.
        // Live counts pages after the module's own `offset`, so the count and
        // the clamp both come from `total_selected` rather than the raw match
        // count, and a number past the end renders the last page.
        let page_count = (total_selected as u64).div_ceil(per_page).max(1);
        let page = u64::from(url_page.unwrap_or(1)).clamp(1, page_count);
        let url_page_skip = usize::try_from((page - 1) * per_page).unwrap_or(usize::MAX);
        let mut pages = selected_pages
            .into_iter()
            .skip(url_page_skip)
            .take(per_page as usize)
            .collect::<Vec<_>>();
        if reverse {
            pages.reverse();
        }
        let exact_total =
            (query_returned_every_match && offset == 0 && !exclude_current_page)
                .then_some(all_selected_total);
        if template.uses_total() && exact_total.is_none() {
            return Ok(ListPagesBlockRenderResult::PreserveOriginal);
        }
        let rendered_rows = pages.len();
        let total = exact_total.unwrap_or(rendered_rows);
        let body = template.body();
        if wants_content && !expansion_budget.can_expand_content_rows(rendered_rows) {
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
        if wants_size {
            let mut missing_by_site = BTreeMap::<i64, Vec<i64>>::new();
            for page in &pages {
                let cache_key = (page.site_id, page.page_id);
                if content_cache.wikitext_scalar_count.contains_key(&cache_key) {
                    continue;
                }
                if let Some(wikitext) = content_cache.wikitext.get(&cache_key) {
                    content_cache.wikitext_scalar_count.insert(
                        cache_key,
                        wikitext.as_deref().map(|wikitext| wikitext.chars().count()),
                    );
                } else {
                    missing_by_site
                        .entry(page.site_id)
                        .or_default()
                        .push(page.page_id);
                }
            }
            for (site_id, page_ids) in missing_by_site {
                let loaded =
                    PageRevisionService::get_wikitext_scalar_count_optional_batch(
                        ctx, site_id, &page_ids,
                    )
                    .await?;
                content_cache.wikitext_scalar_count.extend(
                    loaded.into_iter().map(|(page_id, scalar_count)| {
                        ((site_id, page_id), scalar_count)
                    }),
                );
            }
            if pages.iter().any(|page| {
                content_cache
                    .wikitext_scalar_count
                    .get(&(page.site_id, page.page_id))
                    .copied()
                    .flatten()
                    .is_none()
            }) {
                // Wikidot reports the Unicode scalar-value count of the normalized saved source.
                // A missing latest source cannot be replaced with a plausible zero.
                return Ok(ListPagesBlockRenderResult::PreserveOriginal);
            }
        }
        let category_ids = pages
            .iter()
            .filter_map(|page| page.page_category_id)
            .collect::<BTreeSet<_>>();
        let categories = if category_ids.is_empty() {
            Vec::new()
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
        };
        let category_slugs = categories
            .iter()
            .map(|category| (category.category_id, category.slug.clone()))
            .collect::<BTreeMap<_, _>>();
        let data_form_definitions = if wants_data_form_values {
            load_list_pages_data_form_definitions(ctx, &categories).await?
        } else {
            BTreeMap::new()
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
        if let ResolvedListPagesAuthors::NotAny {
            user_ids,
            wikidot_snapshot_names,
        } = &resolved_authors
            && user_ids.is_empty()
            && wikidot_snapshot_names.is_empty()
            && current_page_identity.is_some()
        {
            // The excluded author did not resolve, and rendering without the
            // exclusion would return exactly the pages the author excluded.
            return Ok(ListPagesBlockRenderResult::PreserveOriginal);
        }
        if wants_site_domain && page_info.site.is_empty() {
            return Ok(ListPagesBlockRenderResult::PreserveOriginal);
        }
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
            || wants_rating_votes
            || wants_parent_metadata
            || wants_revisions;
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
        let wants_rating = template.uses_rating();
        let wants_rating_percent = template.uses_rating_percent();
        let wants_runtime_displays = wants_comments
            || wants_commented_by
            || wants_commented_at
            || wants_rating
            || wants_rating_percent
            || wants_rating_votes;
        let loaded_runtime_displays =
            if wants_runtime_displays && prefetched_displays.is_none() {
                Some(Self::load_list_pages_runtime_displays(ctx, &pages).await?)
            } else {
                None
            };
        let empty_runtime_displays = BTreeMap::new();
        let runtime_displays = prefetched_displays
            .map(|displays| &displays.runtime_displays)
            .or(loaded_runtime_displays.as_ref())
            .unwrap_or(&empty_runtime_displays);
        if wants_created_by_unix
            && pages.iter().any(|page| {
                list_pages_created_by_unix(page, user_displays, snapshot_displays)
                    .is_none()
            })
        {
            // An imported row's local creating revision belongs to the importer,
            // so its account slug cannot stand in for the Wikidot author's unix name.
            return Ok(ListPagesBlockRenderResult::PreserveOriginal);
        }
        let child_counts = if wants_children {
            load_list_pages_child_counts(ctx, &pages).await?
        } else {
            BTreeMap::new()
        };
        let revision_counts = if wants_revisions {
            let mut missing_by_site = BTreeMap::<i64, Vec<i64>>::new();
            for page in &pages {
                if !snapshot_displays.contains_key(&page.page_id) {
                    missing_by_site
                        .entry(page.site_id)
                        .or_default()
                        .push(page.page_id);
                }
            }
            let mut revision_counts = BTreeMap::<i64, u64>::new();
            for (site_id, page_ids) in missing_by_site {
                revision_counts.extend(
                    PageRevisionService::get_revision_count_batch(
                        ctx, site_id, &page_ids,
                    )
                    .await?,
                );
            }
            if pages.iter().any(|page| {
                list_pages_revision_count(page, snapshot_displays, &revision_counts)
                    .is_none()
            }) {
                return Ok(ListPagesBlockRenderResult::PreserveOriginal);
            }
            revision_counts
        } else {
            BTreeMap::new()
        };
        let relational_parent_displays = if wants_parent_metadata {
            load_list_pages_parent_displays(ctx, &pages).await?
        } else {
            BTreeMap::new()
        };
        let site_title = if template.uses_site_title() {
            Some(
                SiteService::get(ctx, Reference::Id(current_site_id))
                    .await?
                    .name,
            )
        } else {
            None
        };
        let mut output = String::new();
        if wrapper {
            output.push_str("[[div class=\"list-pages-box\"]]\n");
        }
        let mut included_pages = Vec::new();
        if template.has_sections() && pages.is_empty() {
            return Ok(ListPagesBlockRenderResult::PreserveOriginal);
        }
        if !separate
            && !pages.is_empty()
            && let Some(prepend_line) = prepend_line
        {
            output.push_str(&prepend_line);
            output.push('\n');
        }
        if let Some(head) = template.head_section() {
            output.push_str(head);
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
                authored_limit: limit,
                ajax_module_response,
                site: page_info.site.as_ref(),
                site_title: site_title.as_deref().unwrap_or_default(),
                category: page
                    .page_category_id
                    .and_then(|category_id| category_slugs.get(&category_id))
                    .map(String::as_str)
                    .unwrap_or_default(),
                user_displays,
                snapshot_displays,
                runtime_displays,
                page_wikitext: page_wikitext.as_deref(),
                page_wikitext_scalar_count: wants_size.then(|| {
                    content_cache
                        .wikitext_scalar_count
                        .get(&cache_key)
                        .copied()
                        .flatten()
                        .expect("size-backed ListPages rows were validated before substitution")
                }),
                page_parent_fullname: list_pages_parent_fullname(
                    page,
                    snapshot_displays,
                    &relational_parent_displays,
                ),
                page_parent_display: relational_parent_displays.get(&page.page_id),
                page_child_count: wants_children
                    .then(|| child_counts.get(&page.page_id).copied().unwrap_or(0)),
                page_revision_count: wants_revisions.then(|| {
                    list_pages_revision_count(page, snapshot_displays, &revision_counts)
                        .expect(
                            "revision-backed ListPages rows were validated before substitution",
                        )
                }),
                expanded_content: Some(&expanded_content),
                data_form_values: &data_form_values,
                data_form_definition: page
                    .page_category_id
                    .and_then(|category_id| data_form_definitions.get(&category_id)),
                render_generated_html,
            };
            let uses_star_rating = runtime_displays
                .get(&page.page_id)
                .is_some_and(|display| display.rating_type == "stars");
            let body = if template.uses_only_rating() && !uses_star_rating {
                let mut body = substitute_list_pages_rating_only(body, page);
                neutralize_authored_markers(&mut body);
                body
            } else {
                let mut generated_fragments = CompatHtmlFragments::new(body);
                let mut body = substitute_list_pages_variables_with_fragments(
                    body,
                    page,
                    index + offset as usize + url_page_skip + 1,
                    total,
                    &substitution_context,
                    &mut generated_fragments,
                );
                neutralize_authored_markers(&mut body);
                generated_fragments.restore(&body)
            };
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

        if let Some(foot) = template.foot_section() {
            output.push_str(foot);
            output.push('\n');
        }

        if !separate
            && !pages.is_empty()
            && let Some(append_line) = append_line
        {
            output.push_str(&append_line);
            output.push('\n');
        }

        push_list_pages_pager(
            &mut output,
            page_info,
            url,
            url_attr_prefix.as_deref(),
            // The pager numbers pages from after the module's own offset,
            // so it reads the URL-derived skip, not the raw offset.
            u32::try_from(url_page_skip).unwrap_or(u32::MAX),
            per_page,
            total_selected,
        );

        if let Some(feed_info) = feed_info {
            output.push_str(&feed_info);
        }
        if wrapper {
            output.push_str("[[/div]]");
        }
        if wants_content {
            expansion_budget.consume_content_rows(rendered_rows);
        }
        Ok(ListPagesBlockRenderResult::Expanded(IncludeExpansion {
            wikitext: output,
            included_pages,
            expanded_include_count: initial_remaining_include_expansions
                .saturating_sub(include_budget.remaining),
        }))
    }

    pub(in crate::services::render) async fn render_count_pages_block(
        ctx: &ServiceContext<'_>,
        page_context: ListPagesPageContext<'_>,
        page_info: &PageInfo<'_>,
        arguments: ListPagesArguments,
        body: &str,
        permission_cache: &mut BTreeMap<(i64, Option<i64>), bool>,
    ) -> Result<CountPagesBlockRenderResult> {
        let ListPagesPageContext {
            site_id: current_site_id,
            page_id: current_page_identity,
            // CountPages renders a total, so the requested page does not apply.
            url: _,
        } = page_context;
        let current_page_id = current_page_identity.unwrap_or(0);
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
            untagged: _,
            same_visible_tags: _,
            exact_visible_tags: _,
            authors,
            author_filter_present,
            order,
            reverse: _,
            limit,
            count_pages_explicit_limit,
            count_pages_per_page: _,
            url_attr_prefix: _,
            offset,
            offset_origin: _,
            offset_beyond_render_window,
            exclude_current_page,
            relative_range: _,
            page_type,
            page_parent,
            static_parent_fullname: _,
            creation_date,
            update_date,
            creation_date_current_page: _,
            update_date_current_page: _,
            score,
            score_equals_current_page: _,
            votes,
            votes_equals_current_page: _,
            slug,
            name_pattern,
            prepend_line: _,
            append_line: _,
            data_form_fields,
            exclude_current_page_author: _,
            unsupported_author_filter: _,
            unsupported_list_pages_filter: _,
            link_to,
            unsupported_score_filter: _,
            unsupported_count_pages_filter: _,
            separate: _,
            wrapper: _,
            rss_title: _,
            rss_description: _,
            rss_home: _,
            rss_limit: _,
            rss_only: _,
            rss_path: _,
        } = arguments;
        if offset_beyond_render_window.is_some() {
            return Ok(CountPagesBlockRenderResult::Expanded(
                substitute_count_pages_variables(body, 0),
            ));
        }
        let count_pages_query_limit = count_pages_explicit_limit
            .map(|limit| {
                limit
                    .saturating_add(u64::from(offset))
                    .saturating_add(u64::from(exclude_current_page))
            })
            .unwrap_or(u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS))
            .min(u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS));
        any_tags.extend(default_tags);
        let link_to_references = link_to
            .iter()
            .map(|slug| Reference::Slug(Cow::Borrowed(slug.as_ref())))
            .collect::<Vec<_>>();
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
            // CountPages keeps its existing literal behavior for the exclusion
            // sentinel, which `unsupported_count_pages_filter` already drives.
            false,
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
                untagged: false,
            },
            page_parent,
            contains_outgoing_links: &link_to_references,
            creation_date,
            update_date,
            author: resolved_authors.as_selector(),
            score: &score,
            votes: &votes,
            offset: 0,
            range: RangeSelector::Current,
            name: name_pattern,
            slug,
            slugs: &[],
            data_form_fields: &data_form_fields,
            order: order.clone(),
            candidate_limit: if data_form_fields.is_empty()
                && !matches!(
                    order.as_ref().map(|order| &order.property),
                    Some(OrderProperty::Score | OrderProperty::DataFormFieldName { .. })
                ) {
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
        let pages = if current_page_only && current_page_identity.is_none() {
            FoundPages { pages: Vec::new() }
        } else if current_page_only
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
}
