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
use super::super::compat::text_fragments::CompatTextFragments;
use super::super::include_attachment_owners::AttachmentOwner;
use super::super::literal_regions::*;
use super::super::prelude::*;
use super::super::runtime::*;
use super::super::runtime_page_queries::*;
use super::super::service::*;
use super::content_sections::{isolate_wikidot_content_section, wikidot_content_section};
use super::parents::load_list_pages_parent_fullnames;
use super::scanner::{
    CountPagesCloseReachabilityIndex, find_list_pages_module_matches,
    has_count_pages_module_opening_candidate, has_list_pages_module_opening_candidate,
};
use super::template::{ListPagesOutputShape, ListPagesTemplatePlan};
use super::*;
use crate::models::page::{self, Entity as Page};
use crate::models::page_category::{self, Entity as PageCategory};
use crate::models::page_revision;
use crate::models::user::{self, Entity as UserTable};
use crate::models::wikidot_user::{self, Entity as WikidotUser};
use crate::services::page_query::*;
use crate::services::page_revision::GetPageRevision;
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::text_block::{MIME_HTML, TextBlock, TextBlockService};
use crate::services::{CategoryService, PageRevisionService, PageService, TextService};
use crate::types::{Action, Permission, Resource, TextBlockType};
use sea_orm::{ColumnTrait, EntityTrait, FromQueryResult, QueryFilter, Statement, Value};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub(in crate::services::render) enum ListPagesBlockRenderResult {
    Expanded(IncludeExpansion),
    PreserveOriginal,
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
pub(in crate::services::render) struct ListPagesPageContext {
    pub(in crate::services::render) site_id: i64,
    pub(in crate::services::render) page_id: i64,
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
pub(in crate::services::render) struct ListPagesExpansionOptions {
    pub(in crate::services::render) current_site_id: Option<i64>,
    pub(in crate::services::render) current_page_id: Option<i64>,
    pub(in crate::services::render) include_budget: IncludeExpansionBudget,
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
        Ok(ListPagesBatchDisplays {
            user_displays,
            snapshot_displays,
        })
    }

    pub(in crate::services::render) async fn expand_count_pages(
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

    pub(in crate::services::render) async fn load_count_pages_required_tag_totals(
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
            reverse,
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
            append_line,
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
        let wants_size = template.uses_size();
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
        let wants_created_by_unix = template.uses_created_by_unix();
        let wants_created_at = template.uses_created_at();
        let wants_updated_by = template.uses_updated_by();
        let wants_updated_at = template.uses_updated_at();
        let wants_rating_votes = template.uses_rating_votes();
        let wants_site_domain = template.uses_site_domain();
        let wants_parent_fullname = template.uses_parent_fullname();
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
        let mut pages = selected_pages
            .into_iter()
            .take(requested_limit as usize)
            .collect::<Vec<_>>();
        if reverse {
            pages.reverse();
        }
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
        if wants_created_by_unix
            && pages
                .iter()
                .any(|page| list_pages_created_by_unix(page, user_displays).is_none())
        {
            // Wikidot emits the creator's stored unix name, which is separate
            // from the display name. Missing account data must remain literal.
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
            || wants_parent_fullname;
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
        let relational_parent_fullnames = if wants_parent_fullname
            && pages
                .iter()
                .any(|page| !snapshot_displays.contains_key(&page.page_id))
        {
            load_list_pages_parent_fullnames(ctx, &pages).await?
        } else {
            BTreeMap::new()
        };
        let mut output = String::new();
        if wrapper {
            output.push_str("[[div class=\"list-pages-box\"]]\n");
        }
        let mut included_pages = Vec::new();
        if !pages.is_empty()
            && let Some(prepend_line) = prepend_line
        {
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
                site: page_info.site.as_ref(),
                category: page
                    .page_category_id
                    .and_then(|category_id| category_slugs.get(&category_id))
                    .map(String::as_str)
                    .unwrap_or_default(),
                user_displays,
                snapshot_displays,
                page_wikitext: None,
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
                    &relational_parent_fullnames,
                ),
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
                    index + offset as usize + 1,
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

        if !pages.is_empty()
            && let Some(append_line) = append_line
        {
            output.push_str(&append_line);
            output.push('\n');
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

    pub(in crate::services::render) async fn render_count_pages_block(
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
            reverse: _,
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
            append_line: _,
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

    pub(in crate::services::render) async fn current_page_list_pages_row(
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

    pub(in crate::services::render) async fn current_page_matches_data_form_fields(
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

    pub(in crate::services::render) async fn load_wikidot_user_displays(
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

    pub(in crate::services::render) async fn load_list_pages_snapshot_displays(
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
            parent_fullname: Option<String>,
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
                        snapshot.parent_fullname, \
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
                             parent_fullname,
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
                                    parent_fullname,
                                },
                            )
                        },
                    )
                    .collect()
            })
    }

    pub(in crate::services::render) async fn resolve_list_pages_authors_cached(
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

    pub(in crate::services::render) async fn resolve_list_pages_authors(
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

    pub(in crate::services::render) async fn load_wikidot_author_ids(
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

    pub(in crate::services::render) async fn load_current_page_author_source(
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

pub(in crate::services::render) fn register_generated_list_pages_html(
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
