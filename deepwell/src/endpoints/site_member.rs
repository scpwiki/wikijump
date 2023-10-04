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
use crate::services::interaction::{
    InteractionObject, InteractionReference, InteractionType, SiteMemberAccepted,
    SiteMemberData,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SiteMembership {
    pub site_id: i64,
    pub user_id: i64,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateSiteMembership {
    pub site_id: i64,
    pub site_user_id: i64,
    pub acting_user_id: i64,
    pub accepted: SiteMemberAccepted,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSiteMembership {
    pub site_id: i64,
    pub site_user_id: i64,
    pub acting_user_id: i64,
}

pub async fn membership_retrieve(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let SiteMembership { site_id, user_id } = req.body_json().await?;
    let output = InteractionService::get_site_member(&ctx, site_id, user_id).await?;
    txn.commit().await?;

    build_json_response(&output, StatusCode::Ok)
}

pub async fn membership_put(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let CreateSiteMembership {
        site_id,
        site_user_id,
        acting_user_id,
        accepted,
    } = req.body_json().await?;

    let output = InteractionService::add_site_member(
        &ctx,
        site_id,
        site_user_id,
        acting_user_id,
        &SiteMemberData { accepted },
    )
    .await?;

    txn.commit().await?;
    build_json_response(&output, StatusCode::Created)
}

pub async fn membership_delete(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let DeleteSiteMembership {
        site_id,
        site_user_id,
        acting_user_id,
    } = req.body_json().await?;

    let output = InteractionService::remove(
        &ctx,
        InteractionReference::Relationship {
            interaction_type: InteractionType::SiteMember,
            dest: InteractionObject::Site(site_id),
            from: InteractionObject::User(site_user_id),
        },
        acting_user_id,
    )
    .await?;

    txn.commit().await?;
    build_json_response(&output, StatusCode::Ok)
}
