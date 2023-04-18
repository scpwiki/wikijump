/*
 * endpoints/site.rs
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
use crate::models::alias::Model as AliasModel;
use crate::models::sea_orm_active_enums::AliasType;
use crate::models::site::Model as SiteModel;
use crate::models::site_domain::Model as SiteDomainModel;
use crate::services::domain::CreateCustomDomain;
use crate::services::site::{CreateSite, GetSite, GetSiteOutput, UpdateSite};

pub async fn site_create(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: CreateSite = req.body_json().await?;
    let output = SiteService::create(&ctx, input).await?;
    txn.commit().await?;

    let body = Body::from_json(&output)?;
    let response = Response::builder(StatusCode::Created).body(body).into();
    Ok(response)
}

pub async fn site_retrieve(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let GetSite { site } = req.body_json().await?;
    tide::log::info!("Getting site {:?}", site);

    let site = SiteService::get(&ctx, site).await?;
    let (aliases, domains) = try_join!(
        AliasService::get_all(&ctx, AliasType::Site, site.site_id),
        DomainService::list_custom(&ctx, site.site_id),
    )?;

    build_site_response(site, aliases, domains, StatusCode::Ok)
}

pub async fn site_put(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let UpdateSite {
        site,
        body,
        user_id,
    } = req.body_json().await?;

    tide::log::info!("Updating site {:?}", site);

    SiteService::update(&ctx, site, body, user_id).await?;

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

pub async fn site_custom_domain_retrieve(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let domain = req.body_string().await?;
    let model = DomainService::site_from_domain(&ctx, &domain).await?;

    let body = Body::from_json(&model)?;
    txn.commit().await?;
    Ok(body.into())
}

pub async fn site_custom_domain_post(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: CreateCustomDomain = req.body_json().await?;
    DomainService::create_custom(&ctx, input).await?;

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

pub async fn site_custom_domain_delete(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let domain = req.body_string().await?;
    DomainService::delete_custom(&ctx, domain).await?;

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

pub async fn site_get_from_domain(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let domain = req.param("domain")?;
    let model = DomainService::site_from_domain(&ctx, domain).await?;

    let body = Body::from_json(&model)?;
    txn.commit().await?;
    Ok(body.into())
}

fn build_site_response(
    site: SiteModel,
    aliases: Vec<AliasModel>,
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
