/*
 * methods/site.rs
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
use crate::models::site::Model as SiteModel;

pub async fn site_create(_req: ApiRequest) -> ApiResponse {
    // TODO when we get to site table refactoring

    todo!()
}

pub async fn site_head(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let reference = Reference::try_from(&req)?;
    let exists = SiteService::exists(&ctx, reference).await.to_api()?;
    txn.commit().await?;
    exists_status(exists)
}

pub async fn site_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let reference = Reference::try_from(&req)?;
    let site = SiteService::get(&ctx, reference).await.to_api()?;
    build_site_response(&site, StatusCode::Ok)
}

fn build_site_response(site: &SiteModel, status: StatusCode) -> ApiResponse {
    let body = Body::from_json(site)?;
    let response = Response::builder(status).body(body).into();
    Ok(response)
}
