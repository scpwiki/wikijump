/*
 * services/interaction/site_ban.rs
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SiteBanData {
    pub banned_until: Option<Date>,
    pub reason: String,
}

impl_interaction!(SiteBan, site_ban, Site, site_id, User, user_id, false, SiteBanData);

impl InteractionService {
    pub async fn add_site_ban(
        ctx: &ServiceContext<'_>,
        AddSiteBan {
            site_id,
            user_id,
            created_by,
        }: AddSiteBan,
    ) -> Result<()> {
        Self::remove_site_member(ctx, dest, from, created_by).await?;
        // TODO: remove site roles

        add_operation!(SiteBan, site_id, user_id, created_by)
    }

    /// Helper method for rejecting an interaction if the user is banned.
    async fn check_site_ban(
        ctx: &ServiceContext<'_>,
        body: GetSiteBan,
        action: &str,
    ) -> Result<()> {
        if Self::get_site_ban(ctx, body).await? {
            tide::log::error!("User ID {user_id} cannot {action} site ID {site_id} because they are banned");
            return Err(Error::SiteBlockedUser);
        }

        Ok(())
    }
}
