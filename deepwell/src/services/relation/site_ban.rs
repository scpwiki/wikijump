/*
 * services/relation/site_ban.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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
use super::site_member::RemoveSiteMember;
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
    pub async fn create_site_ban(
        ctx: &ServiceContext,
        CreateSiteBan {
            site_id,
            user_id,
            created_by,
            metadata,
        }: CreateSiteBan,
    ) -> Result<()> {
        Self::remove_site_member(
            ctx,
            RemoveSiteMember {
                site_id,
                user_id,
                removed_by: created_by,
            },
        )
        .await?;
        // TODO: remove site member applications
        // TODO: remove site roles

        create_operation!(
            ctx, SiteBan, Site, site_id, User, user_id, created_by, &metadata,
        )
    }

    /// Helper method for rejecting an relation if the user is banned.
    pub async fn check_site_ban(
        ctx: &ServiceContext,
        body: GetSiteBan,
        action: &str,
    ) -> Result<()> {
        if Self::site_ban_exists(ctx, body).await? {
            error!(
                "User ID {} cannot {} site ID {} because they are banned",
                body.user_id, action, body.site_id,
            );

            return Err(Error::SiteBlockedUser);
        }

        Ok(())
    }
}
