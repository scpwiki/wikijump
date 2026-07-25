/*
 * services/relation/site_member.rs
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

use super::RelationService;
use super::site_ban::GetSiteBan;
use super::structs::{RelationDirection, RelationObject, RelationReference};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::relation::Model as RelationModel;
use crate::services::ServiceContext;
use crate::types::RelationType;
use paste::paste;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "cause", content = "user_id")]
pub enum SiteMemberAccepted {
    CreatedSite,
    SelfJoined,
    Password,
    Accepted(i64),
    Invitation(i64),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SiteMemberData {
    pub accepted: SiteMemberAccepted,
}

impl_relation!(
    SiteMember,
    Site,
    site_id,
    User,
    user_id,
    SiteMemberData,
    NO_CREATE_IMPL,
);

impl RelationService {
    pub async fn create_site_member(
        ctx: &ServiceContext<'_>,
        CreateSiteMember {
            site_id,
            user_id,
            metadata,
            created_by,
        }: CreateSiteMember,
    ) -> Result<()> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to add user ID {} as member of site ID {}, created by user ID {}",
                    user_id, site_id, created_by,
                ),
                ErrorType::SiteMemberRelation,
            )
        };

        // Cannot join if banned
        Self::check_site_ban(ctx, GetSiteBan { site_id, user_id }, "join")
            .await
            .or_raise(make_error)?;

        create_operation!(
            ctx, SiteMember, Site, site_id, User, user_id, created_by, &metadata,
            make_error,
        )
    }
}
