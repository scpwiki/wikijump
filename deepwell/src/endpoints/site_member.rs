/*
 * endpoints/site_member.rs
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
use crate::services::relation::{CreateSiteMember, GetSiteMember, RemoveSiteMember};
use std::net::IpAddr;

#[derive(Deserialize, Debug)]
struct CreateSiteMemberInput {
    #[serde(flatten)]
    input: CreateSiteMember,
    ip_address: IpAddr,
}

#[derive(Deserialize, Debug)]
struct RemoveSiteMemberInput {
    #[serde(flatten)]
    input: RemoveSiteMember,
    ip_address: IpAddr,
}

pub async fn membership_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<RelationModel>> {
    let input: GetSiteMember = parse!(params, SiteMembership);
    let user_id = input.user_id;
    let site_id = input.site_id;

    RelationService::get_optional_site_member(ctx, input)
        .await
        .or_raise(|| {
            Error::new(
                format!(
                    "failed to get site member data for user ID {} on site ID {}",
                    user_id, site_id,
                ),
                ErrorType::SiteMembership,
            )
        })
}

pub async fn membership_set(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let CreateSiteMemberInput { input, ip_address } = parse!(params, SiteMembership);
    let user_id = input.user_id;
    let site_id = input.site_id;

    RelationService::create_site_member(ctx, input, ip_address)
        .await
        .or_raise(|| {
            Error::new(
                format!(
                    "failed to add user ID {} as a site member of site ID {}",
                    user_id, site_id,
                ),
                ErrorType::SiteMembership,
            )
        })
}

pub async fn membership_remove(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<RelationModel> {
    let RemoveSiteMemberInput { input, ip_address } = parse!(params, SiteMembership);
    let user_id = input.user_id;
    let site_id = input.site_id;

    RelationService::remove_site_member(ctx, input, ip_address)
        .await
        .or_raise(|| {
            Error::new(
                format!(
                    "failed to remove site membership for user ID {} from site ID {}",
                    user_id, site_id,
                ),
                ErrorType::SiteMembership,
            )
        })
}
