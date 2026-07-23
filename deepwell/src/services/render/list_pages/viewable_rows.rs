/*
 * services/render/list_pages/viewable_rows.rs
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

use super::super::service::{MAX_LISTPAGES_RENDER_LIMIT, MAX_LISTPAGES_RENDER_SCAN_ROWS};
use crate::error::Result;
use crate::services::page_query::{
    FoundPageRow, FoundPages, OrderBySelector, OrderProperty, PageQuery,
    PageQueryResultMetadata, PageQueryScoreFilterCache, PageQueryScoreFilterSession,
};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::{PageQueryService, ServiceContext};
use crate::types::{Action, Permission, Reference, Resource};
use std::collections::BTreeMap;

#[derive(Debug)]
pub(in crate::services::render) struct ViewableCountPagesRows {
    pub(in crate::services::render) pages: FoundPages,
    pub(in crate::services::render) metadata: PageQueryResultMetadata,
    pub(in crate::services::render) view_permission_filtering_applied: bool,
    pub(in crate::services::render) raw_scan_completion: CountPagesRawScanCompletion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::render) enum CountPagesRawScanCompletion {
    Complete,
    Capped,
}

#[derive(Debug)]
pub(in crate::services::render) struct ViewableListPagesRows {
    pub(in crate::services::render) pages: FoundPages,
    pub(in crate::services::render) metadata: PageQueryResultMetadata,
    pub(in crate::services::render) view_permission_filtering_applied: bool,
}

pub(in crate::services::render) async fn find_viewable_list_pages_rows(
    ctx: &ServiceContext<'_>,
    query: PageQuery<'_>,
    target_count: usize,
    permission_cache: &mut BTreeMap<(i64, Option<i64>), bool>,
    score_filter_cache: Option<&mut PageQueryScoreFilterCache>,
) -> Result<ViewableListPagesRows> {
    let found = find_viewable_render_page_rows(
        ctx,
        query,
        target_count,
        permission_cache,
        score_filter_cache,
        true,
    )
    .await?;
    Ok(ViewableListPagesRows {
        pages: found.pages,
        metadata: found.metadata,
        view_permission_filtering_applied: found.view_permission_filtering_applied,
    })
}

pub(in crate::services::render) async fn find_viewable_count_pages_rows(
    ctx: &ServiceContext<'_>,
    query: PageQuery<'_>,
    target_count: usize,
    permission_cache: &mut BTreeMap<(i64, Option<i64>), bool>,
) -> Result<ViewableCountPagesRows> {
    find_viewable_render_page_rows(
        ctx,
        query,
        target_count,
        permission_cache,
        None,
        false,
    )
    .await
}

async fn find_viewable_render_page_rows(
    ctx: &ServiceContext<'_>,
    query: PageQuery<'_>,
    target_count: usize,
    permission_cache: &mut BTreeMap<(i64, Option<i64>), bool>,
    mut score_filter_cache: Option<&mut PageQueryScoreFilterCache>,
    retain_score_filter_session: bool,
) -> Result<ViewableCountPagesRows> {
    let mut score_filter_session = PageQueryScoreFilterSession::default();
    if target_count > 0 && render_page_query_uses_single_scan(query.order) {
        let mut query = query;
        query.offset = 0;
        query.pagination.limit = Some(random_page_query_scan_limit(target_count));
        let mut found = PageQueryService::find_with_metadata_cached(
            ctx,
            query,
            score_filter_cache.as_deref_mut(),
            retain_score_filter_session.then_some(&mut score_filter_session),
        )
        .await?;
        if found.metadata.cap_exceeded {
            found.metadata.cap_exceeded = false;
        }
        let raw_count = found.pages.pages.len();
        let mut pages =
            filter_viewable_rows(ctx, found.pages.pages, permission_cache).await?;
        let view_permission_filtering_applied = pages.len() != raw_count;
        pages.truncate(target_count);
        return Ok(ViewableCountPagesRows {
            pages: FoundPages { pages },
            metadata: found.metadata,
            view_permission_filtering_applied,
            raw_scan_completion: count_pages_raw_scan_completion(raw_count),
        });
    }

    let mut pages = Vec::new();
    let mut raw_offset = 0;
    let mut metadata = None;
    let mut view_permission_filtering_applied = false;
    let mut raw_scan_completion = CountPagesRawScanCompletion::Complete;

    while pages.len() < target_count && raw_offset < MAX_LISTPAGES_RENDER_SCAN_ROWS {
        let mut query = query.clone();
        query.offset = raw_offset;
        let batch_limit =
            render_page_query_batch_limit(target_count, pages.len(), raw_offset);
        query.pagination.limit = Some(batch_limit);

        let found = PageQueryService::find_with_metadata_cached(
            ctx,
            query,
            score_filter_cache.as_deref_mut(),
            retain_score_filter_session.then_some(&mut score_filter_session),
        )
        .await?;
        let cap_exceeded = found.metadata.cap_exceeded;
        merge_render_page_query_metadata(&mut metadata, found.metadata);
        if cap_exceeded {
            return Ok(ViewableCountPagesRows {
                pages: FoundPages { pages: Vec::new() },
                metadata: metadata.unwrap_or_default(),
                view_permission_filtering_applied: false,
                raw_scan_completion: CountPagesRawScanCompletion::Capped,
            });
        }
        let raw_count = found.pages.pages.len();
        if raw_count == 0 {
            break;
        }
        let viewable =
            filter_viewable_rows(ctx, found.pages.pages, permission_cache).await?;
        view_permission_filtering_applied |= viewable.len() != raw_count;
        pages.extend(viewable);
        if raw_count < batch_limit as usize {
            break;
        }
        raw_offset = raw_offset.saturating_add(raw_count as u32);
        if raw_offset >= MAX_LISTPAGES_RENDER_SCAN_ROWS {
            raw_scan_completion = CountPagesRawScanCompletion::Capped;
        }
    }

    Ok(ViewableCountPagesRows {
        pages: FoundPages { pages },
        metadata: metadata.unwrap_or_default(),
        view_permission_filtering_applied,
        raw_scan_completion,
    })
}

async fn filter_viewable_rows(
    ctx: &ServiceContext<'_>,
    pages: Vec<FoundPageRow>,
    category_permissions: &mut BTreeMap<(i64, Option<i64>), bool>,
) -> Result<Vec<FoundPageRow>> {
    let mut viewable = Vec::with_capacity(pages.len());
    for page in pages {
        let permission_key = (page.site_id, page.page_category_id);
        let can_view = if let Some(can_view) = category_permissions.get(&permission_key) {
            *can_view
        } else {
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
            category_permissions.insert(permission_key, can_view);
            can_view
        };
        if can_view {
            viewable.push(page);
        }
    }

    Ok(viewable)
}

pub(in crate::services::render) fn count_pages_raw_scan_completion(
    raw_count: usize,
) -> CountPagesRawScanCompletion {
    if raw_count >= MAX_LISTPAGES_RENDER_SCAN_ROWS as usize {
        CountPagesRawScanCompletion::Capped
    } else {
        CountPagesRawScanCompletion::Complete
    }
}

pub(in crate::services::render) fn render_page_query_batch_limit(
    target_count: usize,
    viewable_count: usize,
    raw_offset: u32,
) -> u64 {
    let needed = target_count.saturating_sub(viewable_count);
    let remaining = (MAX_LISTPAGES_RENDER_SCAN_ROWS - raw_offset) as usize;
    needed
        .max(MAX_LISTPAGES_RENDER_LIMIT as usize)
        .min(remaining) as u64
}

pub(in crate::services::render) fn render_page_query_uses_single_scan(
    order: Option<OrderBySelector>,
) -> bool {
    order.is_some_and(|order| order.property == OrderProperty::Random)
}

pub(in crate::services::render) fn random_page_query_scan_limit(
    _target_count: usize,
) -> u64 {
    u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS)
}

pub(super) fn merge_render_page_query_metadata(
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
