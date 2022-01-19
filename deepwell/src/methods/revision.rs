/*
 * methods/revision.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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
use crate::services::revision::CreateRevision;

pub async fn page_revision_create(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: CreateRevision = req.body_json().await?;
    let site_id = req.param("site_id")?.parse()?;
    let reference = Reference::try_from(&req)?;
    let page = PageService::get(&ctx, site_id, reference).await.to_api()?;
    let output = RevisionService::create(&ctx, site_id, page.page_id, input)
        .await
        .to_api()?;
    let body = Body::from_json(&output)?;
    txn.commit().await?;

    Ok(body.into())
}

pub async fn page_revision_latest(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    let reference = Reference::try_from(&req)?;
    let page = PageService::get(&ctx, site_id, reference).await.to_api()?;
    let revision = RevisionService::get_latest(&ctx, site_id, page.page_id)
        .await
        .to_api()?;
    txn.commit().await?;

    build_revision_response(&revision, StatusCode::Ok)
}

pub async fn page_revision_head(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    let revision_number = req.param("revision_number")?.parse()?;
    let reference = Reference::try_from(&req)?;
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
    let page = PageService::get(&ctx, site_id, reference).await.to_api()?;
    let revision = RevisionService::get(&ctx, site_id, page.page_id, revision_number)
        .await
        .to_api()?;
    txn.commit().await?;

    build_revision_response(&revision, StatusCode::Ok)
}

pub async fn page_revision_edit(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    todo!()
}

// TODO: get list of revisions before/after a spec, max limit 100, default 10
// app.at("/page/:site_id/:type/:id_or_slug/revision/:revision_num/:direction")
pub async fn page_revision_range_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    todo!()
}

// TODO: filter out hidden fields
fn build_revision_response(
    revision: &PageRevisionModel,
    status: StatusCode,
) -> ApiResponse {
    let body = Body::from_json(revision)?;
    let response = Response::builder(status).body(body).into();
    Ok(response)
}
