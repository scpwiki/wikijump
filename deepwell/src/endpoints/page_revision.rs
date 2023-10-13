/*
 * endpoints/page_revision.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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
use crate::models::page_revision::Model as PageRevisionModel;
use crate::services::page::GetPage;
use crate::services::page_revision::{
    GetPageRevision, GetPageRevisionRange, PageRevisionCountOutput,
    PageRevisionModelFiltered, UpdatePageRevision,
};
use crate::services::{Result, TextService};
use crate::web::PageDetails;

pub async fn page_revision_count(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::from_req(&req, &txn);

    let GetPage {
        site_id,
        page: reference,
        details: _,
    } = req.body_json().await?;

    tide::log::info!(
        "Getting latest revision for page {reference:?} in site ID {site_id}",
    );

    let page_id = PageService::get_id(&ctx, site_id, reference).await?;

    let revision_count = PageRevisionService::count(&ctx, site_id, page_id).await?;

    txn.commit().await?;
    let output = PageRevisionCountOutput {
        revision_count,
        first_revision: 0,
        last_revision: revision_count.get() - 1,
    };

    let body = Body::from_json(&output)?;
    let response = Response::builder(StatusCode::Ok).body(body).into();
    Ok(response)
}

pub async fn page_revision_retrieve(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::from_req(&req, &txn);

    let details: PageDetails = req.query()?;
    let GetPageRevision {
        site_id,
        page_id,
        revision_number,
    } = req.body_json().await?;

    tide::log::info!(
        "Getting revision {revision_number} for page ID {page_id} in site ID {site_id}",
    );

    let revision =
        PageRevisionService::get(&ctx, site_id, page_id, revision_number).await?;

    let response =
        build_revision_response(&ctx, revision, details, StatusCode::Ok).await?;

    txn.commit().await?;
    Ok(response)
}

pub async fn page_revision_put(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::from_req(&req, &txn);

    let details: PageDetails = req.query()?;
    let input: UpdatePageRevision = req.body_json().await?;

    tide::log::info!(
        "Editing revision ID {} for page ID {} in site ID {}",
        input.revision_id,
        input.page_id,
        input.site_id,
    );

    let revision_id = input.revision_id;
    let (_, revision) = try_join!(
        PageRevisionService::update(&ctx, input),
        PageRevisionService::get_direct(&ctx, revision_id),
    )?;

    let response =
        build_revision_response(&ctx, revision, details, StatusCode::Ok).await?;

    txn.commit().await?;
    Ok(response)
}

pub async fn page_revision_range_retrieve(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::from_req(&req, &txn);

    let details: PageDetails = req.query()?;
    let input: GetPageRevisionRange = req.body_json().await?;
    let revisions = PageRevisionService::get_range(&ctx, input).await?;

    let response =
        build_revision_list_response(&ctx, revisions, details, StatusCode::Ok).await?;

    txn.commit().await?;
    Ok(response)
}

// Helper functions
async fn filter_and_populate_revision(
    ctx: &ServiceContext<'_>,
    model: PageRevisionModel,
    mut details: PageDetails,
) -> Result<PageRevisionModelFiltered> {
    let PageRevisionModel {
        revision_id,
        revision_type,
        created_at,
        from_wikidot,
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
    } = model;

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
            _ => panic!("Unknown field name in hidden: {field}"),
        }
    }

    // Get text data, if requested
    let (wikitext, compiled_html) = try_join!(
        TextService::get_maybe(ctx, details.wikitext, &wikitext_hash),
        TextService::get_maybe(ctx, details.compiled_html, &compiled_hash),
    )?;

    Ok(PageRevisionModelFiltered {
        revision_id,
        revision_type,
        created_at,
        from_wikidot,
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
    })
}

async fn build_revision_response(
    ctx: &ServiceContext<'_>,
    revision: PageRevisionModel,
    details: PageDetails,
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
    details: PageDetails,
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
