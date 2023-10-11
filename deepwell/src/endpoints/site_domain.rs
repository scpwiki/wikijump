/*
 * endpoints/site_domain.rs
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
use crate::services::domain::CreateCustomDomain;

pub async fn site_get_from_domain(
    state: ServerState,
    params: Params<'static>,
) -> Result<SiteModel> {
    let txn = state.database.begin().await?;
    let ctx = ServiceContext::from_raw(&state, &txn);
    let domain: String = params.one()?;
    let site = DomainService::site_from_domain(&ctx, &domain).await?;
    txn.commit().await?;
    Ok(site)
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
