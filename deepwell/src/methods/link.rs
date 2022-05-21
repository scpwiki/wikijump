/*
 * methods/link.rs
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

pub async fn page_links_from_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    let reference = Reference::try_from(&req)?;
    tide::log::info!("Getting page links for page {reference:?} in site ID {site_id}");

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
    tide::log::info!("Getting page links from page {reference:?} in site ID {site_id}");

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
    tide::log::info!(
        "Getting missing page links from page slug {page_slug} in site ID {site_id}",
    );

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
    tide::log::info!(
        "Getting external links from page {reference:?} in site ID {site_id}",
    );

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
    tide::log::info!("Getting external links to URL {url} in site ID {site_id}");

    let output = LinkService::get_external_to(&ctx, site_id, url)
        .await
        .to_api()?;

    let body = Body::from_json(&output)?;
    txn.commit().await?;
    Ok(body.into())
}
