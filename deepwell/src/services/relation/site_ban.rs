/*
 * services/relation/site_ban.rs
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
use super::site_member::RemoveSiteMember;
use super::structs::{RelationDirection, RelationObject, RelationReference};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::relation::Model as RelationModel;
use crate::services::ServiceContext;
use crate::types::RelationType;
use crate::utils::now;
use paste::paste;
use serde::Serialize;
use time::Date;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SiteBanData {
    pub banned_until: Option<Date>,
    pub reason: String,
}

impl_relation!(
    SiteBan,
    Site,
    site_id,
    User,
    user_id,
    SiteBanData,
    NO_CREATE_IMPL,
);

impl RelationService {
    #[allow(dead_code)] // TEMP
    pub async fn active_site_ban_exists(
        ctx: &ServiceContext<'_>,
        body: GetSiteBan,
    ) -> Result<bool> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to check active ban of user ID {} in site ID {}",
                    body.user_id, body.site_id,
                ),
                ErrorType::SiteBanRelation,
            )
        };

        let Some(relation) = Self::get_optional_site_ban(ctx, body)
            .await
            .or_raise(make_error)?
        else {
            return Ok(false);
        };

        let metadata: SiteBanData =
            serde_json::from_value(relation.metadata).or_raise(make_error)?;

        Ok(match metadata.banned_until {
            Some(banned_until) => banned_until >= now().date(),
            None => true,
        })
    }

    #[allow(dead_code)] // TEMP
    pub async fn create_site_ban(
        ctx: &ServiceContext<'_>,
        CreateSiteBan {
            site_id,
            user_id,
            created_by,
            metadata,
        }: CreateSiteBan,
    ) -> Result<()> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to create ban from site ID {} → user ID {}, as created by ID {} (metadata {:?})",
                    site_id, user_id, created_by, metadata,
                ),
                ErrorType::SiteBanRelation,
            )
        };

        Self::remove_site_member(
            ctx,
            RemoveSiteMember {
                site_id,
                user_id,
                removed_by: created_by,
            },
        )
        .await
        .or_raise(make_error)?;

        // TODO: remove site member applications
        // TODO: remove site roles

        create_operation!(
            ctx, SiteBan, Site, site_id, User, user_id, created_by, &metadata,
            make_error,
        )
    }

    /// Helper method for rejecting an relation if the user is banned.
    pub async fn check_site_ban(
        ctx: &ServiceContext<'_>,
        body: GetSiteBan,
        action: &str,
    ) -> Result<()> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to check for ban of user ID {} in site ID {}",
                    body.user_id, body.site_id,
                ),
                ErrorType::SiteBanRelation,
            )
        };

        if Self::active_site_ban_exists(ctx, body)
            .await
            .or_raise(make_error)?
        {
            error!(
                "User ID {} cannot {} site ID {} because they are banned",
                body.user_id, action, body.site_id,
            );
            bail!(Error::new(
                format!(
                    "cannot {} because user ID {} is banned on site ID {}",
                    action, body.user_id, body.site_id
                ),
                ErrorType::SiteBannedUser,
            ));
        }

        Ok(())
    }
}
