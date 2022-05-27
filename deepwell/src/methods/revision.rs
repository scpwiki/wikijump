/*
 * methods/revision.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2022 Wikijump Team
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
use crate::json_utils::json_to_string_list;
use crate::models::page_revision::Model as PageRevisionModel;
use crate::services::revision::{
    PageRevisionModelFiltered, RevisionCountOutput, UpdateRevision,
};
use crate::services::{Result, TextService};
use crate::web::{PageDetailsQuery, PageLimitQuery};

pub async fn page_revision_info(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    let reference = Reference::try_from(&req)?;

    tide::log::info!(
        "Getting latest revision for page {reference:?} in site ID {site_id}",
    );

    let page = PageService::get(&ctx, site_id, reference).await.to_api()?;
    let revision_count = RevisionService::count(&ctx, site_id, page.page_id)
        .await
        .to_api()?;

    txn.commit().await?;
    let output = RevisionCountOutput {
        revision_count,
        first_revision: 0,
        last_revision: revision_count.get() - 1,
    };

    let body = Body::from_json(&output)?;
    let response = Response::builder(StatusCode::Ok).body(body).into();
    Ok(response)
}

pub async fn page_revision_head(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    let revision_number = req.param("revision_number")?.parse()?;
    let reference = Reference::try_from(&req)?;

    tide::log::info!(
        "Checking existence of revision {revision_number} for page {reference:?} in site ID {site_id}",
    );

    let page = PageService::get(&ctx, site_id, reference).await.to_api()?;
    let exists = RevisionService::exists(&ctx, site_id, page.page_id, revision_number)
        .await
        .to_api()?;

    txn.commit().await?;
    exists_status(exists)
}

pub async fn page_revision_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let details: PageDetailsQuery = req.query()?;
    let site_id = req.param("site_id")?.parse()?;
    let revision_number = req.param("revision_number")?.parse()?;
    let reference = Reference::try_from(&req)?;

    tide::log::info!(
        "Getting revision {revision_number} for page {reference:?} in site ID {site_id}",
    );

    let page = PageService::get(&ctx, site_id, reference).await.to_api()?;
    let revision = RevisionService::get(&ctx, site_id, page.page_id, revision_number)
        .await
        .to_api()?;

    let response = build_revision_response(&ctx, revision, details, StatusCode::Ok)
        .await
        .to_api()?;

    txn.commit().await?;
    Ok(response)
}

pub async fn page_revision_put(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let details: PageDetailsQuery = req.query()?;
    let input: UpdateRevision = req.body_json().await?;
    let site_id = req.param("site_id")?.parse()?;
    let revision_number = req.param("revision_number")?.parse()?;
    let reference = Reference::try_from(&req)?;

    tide::log::info!(
        "Editing revision {revision_number} for page {reference:?} in site ID {site_id}",
    );

    let page = PageService::get(&ctx, site_id, reference).await.to_api()?;
    let revision = RevisionService::get(&ctx, site_id, page.page_id, revision_number)
        .await
        .to_api()?;

    RevisionService::update(&ctx, site_id, page.page_id, revision.revision_id, input)
        .await
        .to_api()?;

    let response = build_revision_response(&ctx, revision, details, StatusCode::Ok)
        .await
        .to_api()?;

    txn.commit().await?;
    Ok(response)
}

pub async fn page_revision_range_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let PageLimitQuery {
        wikitext,
        compiled_html,
        limit,
    } = req.query()?;

    let details = PageDetailsQuery {
        wikitext,
        compiled_html,
    };

    let site_id = req.param("site_id")?.parse()?;
    let revision_number = req.param("revision_number")?.parse()?;
    let direction = req.param("direction")?.parse()?;
    let reference = Reference::try_from(&req)?;

    let page = PageService::get(&ctx, site_id, reference).await.to_api()?;
    let revisions = RevisionService::get_range(
        &ctx,
        site_id,
        page.page_id,
        revision_number,
        direction,
        limit.into(),
    )
    .await
    .to_api()?;

    let response = build_revision_list_response(&ctx, revisions, details, StatusCode::Ok)
        .await
        .to_api()?;

    txn.commit().await?;
    Ok(response)
}

// Helper functions
async fn filter_and_populate_revision(
    ctx: &ServiceContext<'_>,
    model: PageRevisionModel,
    mut details: PageDetailsQuery,
) -> Result<PageRevisionModelFiltered> {
    let PageRevisionModel {
        revision_id,
        revision_type,
        created_at,
        revision_number,
        page_id,
        site_id,
        user_id,
        changes,
        wikitext_hash,
        compiled_hash,
        compiled_at,
        compiled_generator,
        comments,
        hidden,
        title,
        mut alt_title,
        slug,
        tags,
        file_id,
    } = model;

    // Convert string list fields
    let changes = json_to_string_list(changes)?;
    let hidden = json_to_string_list(hidden)?;
    let tags = json_to_string_list(tags)?;

    // Strip hidden fields
    let mut comments = Some(comments);
    let mut title = Some(title);
    // alt-title is already Option and we're not doubling up
    let mut slug = Some(slug);
    let mut tags = Some(tags);

    for field in &hidden {
        // TODO hidden fields aren't standardized yet
        match field.as_str() {
            "wikitext" => details.wikitext = false,
            "compiled" => details.compiled_html = false,
            "comments" => comments = None,
            "title" => title = None,
            "alt_title" => alt_title = None,
            "slug" => slug = None,
            "tags" => tags = None,
            _ => panic!("Unknown field name in hidden: {}", field),
        }
    }

    // Get text data, if requested
    let (wikitext, compiled_html) = try_join!(
        TextService::get_maybe(ctx, details.wikitext, &wikitext_hash),
        TextService::get_maybe(ctx, details.compiled_html, &compiled_hash),
    )
    .to_api()?;

    Ok(PageRevisionModelFiltered {
        revision_id,
        revision_type,
        created_at,
        revision_number,
        page_id,
        site_id,
        user_id,
        changes,
        wikitext,
        compiled_html,
        compiled_at,
        compiled_generator,
        comments,
        hidden,
        title,
        alt_title,
        slug,
        tags,
        file_id_changed: file_id,
    })
}

async fn build_revision_response(
    ctx: &ServiceContext<'_>,
    revision: PageRevisionModel,
    details: PageDetailsQuery,
    status: StatusCode,
) -> Result<Response> {
    let filtered_revision = filter_and_populate_revision(ctx, revision, details).await?;
    let body = Body::from_json(&filtered_revision)?;
    let response = Response::builder(status).body(body).into();
    Ok(response)
}

async fn build_revision_list_response(
    ctx: &ServiceContext<'_>,
    revisions: Vec<PageRevisionModel>,
    details: PageDetailsQuery,
    status: StatusCode,
) -> Result<Response> {
    let filtered_revisions = {
        let mut f_revisions = Vec::new();

        for revision in revisions {
            let f_revision = filter_and_populate_revision(ctx, revision, details).await?;
            f_revisions.push(f_revision);
        }

        f_revisions
    };

    let body = Body::from_json(&filtered_revisions)?;
    let response = Response::builder(status).body(body).into();
    Ok(response)
}
