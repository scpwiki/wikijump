/*
 * endpoints/site_member.rs
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
use crate::services::site_member::SiteMembership;
use serde::Serialize;

pub async fn membership_retrieve(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: SiteMembership = req.body_json().await?;
    let output = SiteMemberService::get(&ctx, input).await?;
    txn.commit().await?;

    build_membership_response(&output, StatusCode::Ok)
}

pub async fn membership_put(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: SiteMembership = req.body_json().await?;
    let created = SiteMemberService::add(&ctx, input).await?;
    txn.commit().await?;

    match created {
        Some(model) => build_membership_response(&model, StatusCode::Created),
        None => Ok(Response::new(StatusCode::NoContent)),
    }
}

pub async fn membership_delete(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: SiteMembership = req.body_json().await?;
    let output = SiteMemberService::remove(&ctx, input).await?;
    txn.commit().await?;

    build_membership_response(&output, StatusCode::Ok)
}

pub async fn membership_site_retrieve(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: i64 = req.body_json().await?;
    let output = SiteMemberService::get_site_members(&ctx, input).await?;
    txn.commit().await?;

    build_membership_response(&output, StatusCode::Ok)
}

pub async fn membership_user_retrieve(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: i64 = req.body_json().await?;
    let output = SiteMemberService::get_user_sites(&ctx, input).await?;
    txn.commit().await?;

    build_membership_response(&output, StatusCode::Ok)
}

fn build_membership_response<T: Serialize>(data: &T, status: StatusCode) -> ApiResponse {
    let body = Body::from_json(data)?;
    let response = Response::builder(status).body(body).into();
    Ok(response)
}
