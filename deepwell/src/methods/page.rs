/*
 * methods/page.rs
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
use crate::models::page::Model as PageModel;
use crate::models::page_revision::Model as PageRevisionModel;
use crate::services::page::{CreatePage, DeletePage, EditPage, UndeletePage};

#[derive(Serialize, Debug)]
struct PageOutput<'a> {
    page: &'a PageModel,
    revision: &'a PageRevisionModel,
}

pub async fn page_invalid(req: ApiRequest) -> ApiResponse {
    tide::log::warn!("Received invalid /page path: {}", req.url());
    Ok(Response::new(StatusCode::BadRequest))
}

pub async fn page_head_direct(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let page_id = req.param("page_id")?.parse()?;
    let exists = PageService::exists_direct(&ctx, page_id).await.to_api()?;
    txn.commit().await?;
    exists_status(exists)
}

pub async fn page_get_direct(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let page_id = req.param("page_id")?.parse()?;
    let page = PageService::get_direct(&ctx, page_id).await.to_api()?;
    let revision = RevisionService::get_latest(&ctx, page.site_id, page.page_id)
        .await
        .to_api()?;
    txn.commit().await?;

    build_page_response(&page, &revision, StatusCode::Ok)
}

pub async fn page_create(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: CreatePage = req.body_json().await?;
    let site_id = req.param("site_id")?.parse()?;
    let output = PageService::create(&ctx, site_id, input).await.to_api()?;
    let body = Body::from_json(&output)?;
    txn.commit().await?;

    Ok(body.into())
}

pub async fn page_head(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    let reference = Reference::try_from(&req)?;
    let exists = PageService::exists(&ctx, site_id, reference)
        .await
        .to_api()?;

    txn.commit().await?;
    exists_status(exists)
}

pub async fn page_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    let reference = Reference::try_from(&req)?;
    let page = PageService::get(&ctx, site_id, reference).await.to_api()?;
    let revision = RevisionService::get_latest(&ctx, site_id, page.page_id)
        .await
        .to_api()?;

    txn.commit().await?;
    build_page_response(&page, &revision, StatusCode::Ok)
}

pub async fn page_edit(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: EditPage = req.body_json().await?;
    let site_id = req.param("site_id")?.parse()?;
    let reference = Reference::try_from(&req)?;
    let output = PageService::edit(&ctx, site_id, reference, input)
        .await
        .to_api()?;

    txn.commit().await?;
    let body = Body::from_json(&output)?;
    Ok(body.into())
}

pub async fn page_delete(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: DeletePage = req.body_json().await?;
    let site_id = req.param("site_id")?.parse()?;
    let reference = Reference::try_from(&req)?;
    let output = PageService::delete(&ctx, site_id, reference, input)
        .await
        .to_api()?;

    txn.commit().await?;
    let body = Body::from_json(&output)?;
    Ok(body.into())
}

pub async fn page_rerender(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    let page_id = req.param("page_id")?.parse()?;
    RevisionService::rerender(&ctx, site_id, page_id)
        .await
        .to_api()?;

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

pub async fn page_undelete(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: UndeletePage = req.body_json().await?;
    let site_id = req.param("site_id")?.parse()?;
    let page_id = req.param("page_id")?.parse()?;
    let output = PageService::undelete(&ctx, site_id, page_id, input)
        .await
        .to_api()?;

    txn.commit().await?;
    let body = Body::from_json(&output)?;
    Ok(body.into())
}

pub async fn page_links_from_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    let reference = Reference::try_from(&req)?;
    let page = PageService::get(&ctx, site_id, reference).await.to_api()?;
    let output = LinkService::get_from(&ctx, page.page_id).await.to_api()?;
    let body = Body::from_json(&output)?;
    txn.commit().await?;

    Ok(body.into())
}

pub async fn page_links_to_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    let reference = Reference::try_from(&req)?;
    let page = PageService::get(&ctx, site_id, reference).await.to_api()?;
    let output = LinkService::get_to(&ctx, page.page_id, None)
        .await
        .to_api()?;
    let body = Body::from_json(&output)?;
    txn.commit().await?;

    Ok(body.into())
}

pub async fn page_links_to_missing_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    let page_slug = req.param("page_slug")?;
    let output = LinkService::get_to_missing(&ctx, site_id, page_slug, None)
        .await
        .to_api()?;

    let body = Body::from_json(&output)?;
    txn.commit().await?;

    Ok(body.into())
}

pub async fn page_links_external_from(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    let reference = Reference::try_from(&req)?;
    let page = PageService::get(&ctx, site_id, reference).await.to_api()?;
    let output = LinkService::get_external_from(&ctx, page.page_id)
        .await
        .to_api()?;

    let body = Body::from_json(&output)?;
    txn.commit().await?;

    Ok(body.into())
}

pub async fn page_links_external_to(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    let url = req.param("url")?;
    let output = LinkService::get_external_to(&ctx, site_id, url)
        .await
        .to_api()?;

    let body = Body::from_json(&output)?;
    txn.commit().await?;

    Ok(body.into())
}

fn build_page_response(
    page: &PageModel,
    revision: &PageRevisionModel,
    status: StatusCode,
) -> ApiResponse {
    let body = Body::from_json(&PageOutput { page, revision })?;
    let response = Response::builder(status).body(body).into();
    Ok(response)
}
