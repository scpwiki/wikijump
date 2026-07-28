/*
 * services/render/list_pages/current_page.rs
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

//! Resolution of the current page as a ListPages query row.

use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::services::page_query::{
    DataFormSelector, FoundPageFields, FoundPageRow, FoundPages, PageQueryResultMetadata,
    parse_static_wikidot_data_form_values, static_wikidot_data_form_matches,
};
use crate::services::page_revision::GetPageRevision;
use crate::services::render::runtime_page_queries::CountPagesRawScanCompletion;
use crate::services::render::service::{MAX_LISTPAGES_RENDER_SCAN_ROWS, RenderService};
use crate::services::{
    CategoryService, PageRevisionService, PageService, ServiceContext,
};
use crate::types::Reference;
use ftml::data::PageInfo;
use std::borrow::Cow;

pub(in crate::services::render) fn count_pages_unbounded_total(
    raw_scan_completion: CountPagesRawScanCompletion,
    scanned_total: usize,
) -> Option<usize> {
    match raw_scan_completion {
        CountPagesRawScanCompletion::Complete => Some(scanned_total),
        CountPagesRawScanCompletion::Capped => None,
    }
}

pub(in crate::services::render) fn page_query_cap_requires_original_module(
    metadata: &PageQueryResultMetadata,
) -> bool {
    metadata.cap_exceeded
}

pub(in crate::services::render) fn count_pages_scan_requires_preservation(
    raw_scan_completion: CountPagesRawScanCompletion,
    viewable_count: usize,
    target_count: usize,
) -> bool {
    raw_scan_completion == CountPagesRawScanCompletion::Capped
        && viewable_count < target_count
}

pub(in crate::services::render) fn list_pages_row_scan_target(
    requested_limit: u64,
    overall_limit: Option<u64>,
    per_page: Option<u64>,
    offset: u32,
    exclude_current_page: bool,
) -> u64 {
    let rows = if per_page.is_some() {
        overall_limit.unwrap_or(u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS))
    } else {
        requested_limit
    };
    rows.saturating_add(u64::from(offset))
        .saturating_add(u64::from(exclude_current_page))
        .min(u64::from(MAX_LISTPAGES_RENDER_SCAN_ROWS))
}

pub(in crate::services::render) fn list_pages_content_query_target(
    query_limit: u64,
    requested_limit: u64,
    remaining_content_rows: usize,
    offset: u32,
    exclude_current_page: bool,
    has_pager: bool,
) -> u64 {
    if has_pager {
        return query_limit;
    }
    let selected_rows_needed = requested_limit.min(
        u64::try_from(remaining_content_rows)
            .unwrap_or(u64::MAX)
            .saturating_add(1),
    );
    query_limit.min(
        selected_rows_needed
            .saturating_add(u64::from(offset))
            .saturating_add(u64::from(exclude_current_page)),
    )
}

pub(in crate::services::render) fn should_render_current_page_list_pages_row(
    current_page_only: bool,
    limit: Option<u64>,
    offset: u32,
) -> bool {
    current_page_only && limit.unwrap_or(1) > 0 && offset == 0
}

pub(in crate::services::render) fn requested_page_info_score(
    fields: &FoundPageFields,
    page_info: &PageInfo<'_>,
) -> Option<f32> {
    fields.score.then(|| page_info.score.to_f64() as f32)
}

pub(in crate::services::render) fn current_page_info_list_pages_row(
    current_site_id: i64,
    current_page_id: i64,
    page_info: &PageInfo<'_>,
    fields: &FoundPageFields,
) -> Option<FoundPageRow> {
    if fields.page_category_id
        || fields.page_revision_id
        || fields.created_at
        || fields.created_by
        || fields.updated_at
        || fields.updated_by
    {
        return None;
    }

    Some(FoundPageRow {
        page_id: current_page_id,
        site_id: current_site_id,
        title: fields.title.then(|| page_info.title.to_string()),
        alt_title: fields
            .alt_title
            .then_some(page_info.alt_title.as_ref())
            .flatten()
            .map(ToString::to_string),
        slug: fields
            .slug
            .then(|| RenderService::page_info_full_slug(page_info)),
        page_category_id: None,
        page_revision_id: None,
        tags: fields
            .tags
            .then(|| page_info.tags.iter().map(ToString::to_string).collect()),
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        score: requested_page_info_score(fields, page_info),
    })
}

impl RenderService {
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
}
