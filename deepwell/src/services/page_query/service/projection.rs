/*
 * services/page_query/service/projection.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Projection of selected page-query models into public result rows.

use super::{
    BTreeMap, ColumnTrait, EntityTrait, Error, ErrorType, FoundPageFields, FoundPageRow,
    FoundPages, OrderBySelector, OrderProperty, Ordering, PageQueryResultEnvelope,
    PageQueryResultMetadata, PaginationSelector, ParentService, QueryFilter, QueryOrder,
    QuerySelect, Reference, Result, ResultExt, ScoreService, ServiceContext,
    compare_ordering_titles, data_form_field_value_ordering,
    list_pages_deferred_ordering, load_page_ordering_identities,
    load_page_ordering_titles, load_pages_static_data_form_values, page, page_revision,
    project_effective_revision_count, score_to_f32,
};
use sea_orm::FromQueryResult;

#[derive(Debug)]
pub(super) struct PageQueryProjection {
    pub(super) pages: Vec<page::Model>,
    pub(super) effective_revision_count_by_page_id: BTreeMap<i64, u64>,
    pub(super) fields: FoundPageFields,
    pub(super) order: OrderBySelector,
    pub(super) offset: u32,
    pub(super) pagination: PaginationSelector,
    pub(super) candidate_count: Option<usize>,
    pub(super) cap_exceeded: bool,
    pub(super) sql_limit_offset_applied: bool,
    pub(super) filtering_deferred_to_rust: bool,
    pub(super) ordering_deferred_to_rust: bool,
}

#[derive(Debug, FromQueryResult)]
pub(super) struct PageQuerySelectedPage {
    #[sea_orm(nested)]
    pub(super) page: page::Model,
    pub(super) effective_revision_count: Option<i64>,
}

// Resolving this list ahead of the main query avoids the full sequential scan produced by
// PostgreSQL's equivalent page_parent self-join.
pub(super) async fn current_parent_ids(
    ctx: &ServiceContext<'_>,
    current_site_id: i64,
    current_page_id: i64,
) -> Result<Vec<i64>> {
    ParentService::get_parents(ctx, current_site_id, Reference::Id(current_page_id))
        .await
        .map(|parents| {
            parents
                .into_iter()
                .map(|parent| parent.parent_page_id)
                .collect()
        })
}

pub(super) async fn project_page_query_results(
    ctx: &ServiceContext<'_>,
    PageQueryProjection {
        mut pages,
        effective_revision_count_by_page_id,
        fields,
        order,
        offset,
        pagination,
        candidate_count,
        cap_exceeded,
        sql_limit_offset_applied,
        filtering_deferred_to_rust,
        ordering_deferred_to_rust,
    }: PageQueryProjection,
) -> Result<PageQueryResultEnvelope> {
    let txn = ctx.transaction();
    let make_error =
        || Error::new("failed to project ListPages query", ErrorType::PageQuery);

    let mut page_ids = pages.iter().map(|page| page.page_id).collect::<Vec<_>>();
    let score_ordering = matches!(order.property, OrderProperty::Score);
    let score_by_page_id: BTreeMap<i64, f32> =
        if (fields.score || score_ordering) && !page_ids.is_empty() {
            ScoreService::scores_bulk(ctx, &page_ids)
                .await
                .or_raise(make_error)?
                .into_iter()
                .map(|(page_id, score)| (page_id, score_to_f32(score)))
                .collect()
        } else {
            BTreeMap::new()
        };
    let ordering_identity_by_page_id = if ordering_deferred_to_rust {
        load_page_ordering_identities(ctx, &page_ids)
            .await
            .or_raise(make_error)?
    } else {
        BTreeMap::new()
    };
    // Live Wikidot resolves equal scores by title ascending, regardless of
    // whether the score direction is ascending or descending.
    let ordering_title_by_page_id = if score_ordering {
        load_page_ordering_titles(ctx, &pages)
            .await
            .or_raise(make_error)?
    } else {
        BTreeMap::new()
    };

    let defer_offset_limit = ordering_deferred_to_rust || filtering_deferred_to_rust;
    if defer_offset_limit {
        if ordering_deferred_to_rust {
            match &order.property {
                OrderProperty::Score => {
                    pages.sort_by(|left, right| {
                        let left_score =
                            score_by_page_id.get(&left.page_id).copied().unwrap_or(0.0);
                        let right_score =
                            score_by_page_id.get(&right.page_id).copied().unwrap_or(0.0);
                        let ordering = left_score
                            .partial_cmp(&right_score)
                            .unwrap_or(Ordering::Equal);
                        let score_comparison = if order.ascending {
                            ordering
                        } else {
                            ordering.reverse()
                        };
                        score_comparison
                            .then_with(|| {
                                compare_ordering_titles(
                                    ordering_title_by_page_id
                                        .get(&left.page_id)
                                        .map(String::as_str),
                                    ordering_title_by_page_id
                                        .get(&right.page_id)
                                        .map(String::as_str),
                                )
                            })
                            .then_with(|| {
                                list_pages_deferred_ordering(
                                    Ordering::Equal,
                                    order.ascending,
                                    ordering_identity_by_page_id
                                        .get(&left.page_id)
                                        .copied()
                                        .unwrap_or(left.page_id),
                                    ordering_identity_by_page_id
                                        .get(&right.page_id)
                                        .copied()
                                        .unwrap_or(right.page_id),
                                    left,
                                    right,
                                )
                            })
                    });
                }
                OrderProperty::DataFormFieldName { field, numeric } => {
                    let values_by_page_id =
                        load_pages_static_data_form_values(ctx, &pages)
                            .await
                            .or_raise(make_error)?;
                    pages.sort_by(|left, right| {
                        let left_value = values_by_page_id
                            .get(&left.page_id)
                            .and_then(|values| values.get(field.as_ref()))
                            .map(String::as_str);
                        let right_value = values_by_page_id
                            .get(&right.page_id)
                            .and_then(|values| values.get(field.as_ref()))
                            .map(String::as_str);
                        let ordering = data_form_field_value_ordering(
                            left_value,
                            right_value,
                            *numeric,
                        );
                        list_pages_deferred_ordering(
                            ordering,
                            order.ascending,
                            ordering_identity_by_page_id
                                .get(&left.page_id)
                                .copied()
                                .unwrap_or(left.page_id),
                            ordering_identity_by_page_id
                                .get(&right.page_id)
                                .copied()
                                .unwrap_or(right.page_id),
                            left,
                            right,
                        )
                    });
                }
                _ => {}
            }
        }
        if offset > 0 {
            let skip = (offset as usize).min(pages.len());
            pages.drain(..skip);
        }
        if let Some(limit) = pagination.limit {
            pages.truncate(std::cmp::Ord::min(limit, usize::MAX as u64) as usize);
        }
        page_ids = pages.iter().map(|page| page.page_id).collect();
    }

    let revision_fields_requested =
        fields.title || fields.alt_title || fields.tags || fields.updated_by;
    let revisions_by_id: BTreeMap<i64, page_revision::Model> =
        if revision_fields_requested {
            let revision_ids = pages
                .iter()
                .filter_map(|page| page.latest_revision_id)
                .collect::<Vec<_>>();

            if revision_ids.is_empty() {
                BTreeMap::new()
            } else {
                page_revision::Entity::find()
                    .filter(page_revision::Column::RevisionId.is_in(revision_ids))
                    .all(txn)
                    .await
                    .or_raise(make_error)?
                    .into_iter()
                    .map(|revision| (revision.revision_id, revision))
                    .collect()
            }
        } else {
            BTreeMap::new()
        };

    let created_by_by_page_id: BTreeMap<i64, i64> =
        if fields.created_by && !page_ids.is_empty() {
            let mut created_by_by_page_id = BTreeMap::new();
            for (page_id, user_id) in page_revision::Entity::find()
                .select_only()
                .column(page_revision::Column::PageId)
                .column(page_revision::Column::UserId)
                .filter(page_revision::Column::PageId.is_in(page_ids))
                .order_by_asc(page_revision::Column::PageId)
                .order_by_asc(page_revision::Column::RevisionNumber)
                .order_by_asc(page_revision::Column::RevisionId)
                .into_tuple::<(i64, i64)>()
                .all(txn)
                .await
                .or_raise(make_error)?
            {
                created_by_by_page_id.entry(page_id).or_insert(user_id);
            }
            created_by_by_page_id
        } else {
            BTreeMap::new()
        };

    let rows = pages
        .into_iter()
        .map(|page| {
            let revision = page
                .latest_revision_id
                .and_then(|revision_id| revisions_by_id.get(&revision_id));

            FoundPageRow {
                page_id: page.page_id,
                site_id: page.site_id,
                slug: fields.slug.then_some(page.slug),
                page_category_id: fields
                    .page_category_id
                    .then_some(page.page_category_id),
                page_revision_id: fields
                    .page_revision_id
                    .then_some(page.latest_revision_id)
                    .flatten(),
                created_at: fields.created_at.then_some(page.created_at),
                updated_at: fields.updated_at.then_some(page.updated_at).flatten(),
                title: fields
                    .title
                    .then(|| revision.map(|revision| revision.title.clone()))
                    .flatten(),
                alt_title: fields
                    .alt_title
                    .then(|| revision.and_then(|revision| revision.alt_title.clone()))
                    .flatten(),
                tags: fields
                    .tags
                    .then(|| revision.map(|revision| revision.tags.clone()))
                    .flatten(),
                created_by: fields
                    .created_by
                    .then(|| created_by_by_page_id.get(&page.page_id).copied())
                    .flatten(),
                updated_by: fields
                    .updated_by
                    .then(|| revision.map(|revision| revision.user_id))
                    .flatten(),
                score: fields
                    .score
                    .then(|| score_by_page_id.get(&page.page_id).copied().or(Some(0.0)))
                    .flatten(),
                revision_count: project_effective_revision_count(
                    &fields,
                    effective_revision_count_by_page_id
                        .get(&page.page_id)
                        .copied(),
                ),
            }
        })
        .collect();

    let pages = FoundPages { pages: rows };
    if filtering_deferred_to_rust || ordering_deferred_to_rust || cap_exceeded {
        return Ok(PageQueryResultEnvelope::deferred(
            pages,
            candidate_count,
            filtering_deferred_to_rust,
            ordering_deferred_to_rust,
            cap_exceeded,
        ));
    }

    Ok(PageQueryResultEnvelope {
        pages,
        metadata: PageQueryResultMetadata {
            candidate_count,
            cap_exceeded,
            sql_limit_offset_applied,
            filtering_deferred_to_rust,
            ordering_deferred_to_rust,
            exact_count_safe: true,
            unsupported_reason: None,
        },
    })
}
