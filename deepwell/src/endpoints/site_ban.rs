/*
 * endpoints/site_ban.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
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
use crate::models::relation::Model as RelationModel;
use crate::services::relation::{CreateSiteBan, GetSiteBan, RemoveSiteBan, SiteBanData};
use std::net::IpAddr;

#[derive(Deserialize, Debug, Clone)]
struct SetSiteBanInput {
    site_id: i64,
    user_id: i64,
    metadata: SiteBanData,
    created_by: i64,
    ip_address: IpAddr,
}

#[derive(Deserialize, Debug, Copy, Clone)]
struct RemoveSiteBanInput {
    site_id: i64,
    user_id: i64,
    removed_by: i64,
    ip_address: IpAddr,
}

pub async fn site_ban_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<RelationModel>> {
    let input: GetSiteBan = parse!(params, SiteBanRelation);
    let user_id = input.user_id;
    let site_id = input.site_id;

    RelationService::get_optional_site_ban(ctx, input)
        .await
        .or_raise(|| {
            Error::new(
                format!(
                    "failed to get site ban data for user ID {} on site ID {}",
                    user_id, site_id,
                ),
                ErrorType::SiteBanRelation,
            )
        })
}

pub async fn site_ban_set(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let SetSiteBanInput {
        site_id,
        user_id,
        metadata,
        created_by,
        ip_address,
    } = parse!(params, SiteBanRelation);

    RelationService::create_site_ban(
        ctx,
        CreateSiteBan {
            site_id,
            user_id,
            metadata,
            created_by,
        },
        ip_address,
    )
    .await
    .or_raise(|| {
        Error::new(
            format!("failed to ban user ID {} from site ID {}", user_id, site_id),
            ErrorType::SiteBanRelation,
        )
    })
}

pub async fn site_ban_remove(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<RelationModel> {
    let RemoveSiteBanInput {
        site_id,
        user_id,
        removed_by,
        ip_address,
    } = parse!(params, SiteBanRelation);

    RelationService::remove_site_ban_with_audit(
        ctx,
        RemoveSiteBan {
            site_id,
            user_id,
            removed_by,
        },
        ip_address,
    )
    .await
    .or_raise(|| {
        Error::new(
            format!(
                "failed to remove site ban for user ID {} from site ID {}",
                user_id, site_id,
            ),
            ErrorType::SiteBanRelation,
        )
    })
}
