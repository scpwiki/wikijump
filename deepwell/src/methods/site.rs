/*
 * methods/site.rs
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
use crate::models::site::Model as SiteModel;
use crate::models::site_domain::Model as SiteDomainModel;
use crate::services::domain::CreateCustomDomain;
use crate::services::site::{CreateSite, GetSite, GetSiteOutput, UpdateSite};

pub async fn site_create(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: CreateSite = req.body_json().await?;
    let output = SiteService::create(&ctx, input).await.to_api()?;
    txn.commit().await?;

    let body = Body::from_json(&output)?;
    let response = Response::builder(StatusCode::Created).body(body).into();
    Ok(response)
}

pub async fn site_get(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let GetSite { site } = req.body_json().await?;
    tide::log::info!("Getting site {:?}", site);

    let site = SiteService::get(&ctx, site).await.to_api()?;
    let (aliases, domains) = try_join!(
        async { Ok(vec![]) }, // TODO create SiteAliasService
        DomainService::domains_for_site(&ctx, site.site_id),
    )?;

    build_site_response(site, aliases, domains, StatusCode::Ok)
}

pub async fn site_put(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let UpdateSite { site, body } = req.body_json().await?;
    tide::log::info!("Updating site {:?}", site);

    SiteService::update(&ctx, site, body).await.to_api()?;

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

pub async fn site_domain_get(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let domain = req.body_string().await?;
    let model = DomainService::site_from_domain(&ctx, &domain)
        .await
        .to_api()?;

    let body = Body::from_json(&model)?;
    txn.commit().await?;
    Ok(body.into())
}

pub async fn site_domain_post(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: CreateCustomDomain = req.body_json().await?;
    DomainService::create(&ctx, input).await.to_api()?;

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

pub async fn site_domain_delete(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let domain = req.body_string().await?;
    DomainService::delete(&ctx, domain).await.to_api()?;

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

pub async fn site_get_from_domain(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let domain = req.param("domain")?;
    let model = DomainService::site_from_domain(&ctx, domain)
        .await
        .to_api()?;

    let body = Body::from_json(&model)?;
    txn.commit().await?;
    Ok(body.into())
}

fn build_site_response(
    site: SiteModel,
    aliases: Vec<()>, // TODO impl site aliases
    domains: Vec<SiteDomainModel>,
    status: StatusCode,
) -> ApiResponse {
    let output = GetSiteOutput {
        site,
        aliases,
        domains,
    };

    let body = Body::from_json(&output)?;
    let response = Response::builder(status).body(body).into();
    Ok(response)
}
