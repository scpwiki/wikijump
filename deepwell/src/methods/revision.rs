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
use crate::models::page_revision::Model as PageRevisionModel;
use crate::services::revision::{
    PageRevisionModelFiltered, RevisionCountOutput, UpdateRevision,
};
use crate::web::RevisionLimitQuery;

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

    txn.commit().await?;
    build_revision_response(revision, StatusCode::Ok)
}

pub async fn page_revision_put(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

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

    RevisionService::update(&ctx, revision.revision_id, input)
        .await
        .to_api()?;

    txn.commit().await?;
    build_revision_response(revision, StatusCode::Ok)
}

pub async fn page_revision_range_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let RevisionLimitQuery { limit } = req.query()?;
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

    txn.commit().await?;
    build_revision_list_response(revisions, StatusCode::Ok)
}

fn build_revision_response(
    revision: PageRevisionModel,
    status: StatusCode,
) -> ApiResponse {
    let filtered_revision = PageRevisionModelFiltered::from(revision);
    let body = Body::from_json(&filtered_revision)?;
    let response = Response::builder(status).body(body).into();
    Ok(response)
}

fn build_revision_list_response(
    revisions: Vec<PageRevisionModel>,
    status: StatusCode,
) -> ApiResponse {
    let filtered_revisions = revisions
        .into_iter()
        .map(PageRevisionModelFiltered::from)
        .collect::<Vec<_>>();

    let body = Body::from_json(&filtered_revisions)?;
    let response = Response::builder(status).body(body).into();
    Ok(response)
}
