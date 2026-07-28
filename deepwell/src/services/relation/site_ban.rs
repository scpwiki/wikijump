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

use super::prelude::*;
use super::site_member::{GetSiteMember, RemoveSiteMember};
use crate::constants::{SYSTEM_IP_ADDRESS, SYSTEM_USER_ID};
use crate::models::relation::{self, Entity as Relation};
use crate::models::user_role::{self, Entity as UserRole};
use crate::services::audit::{AuditEvent, AuditService};
use crate::services::role::{RevokeUserRoleInput, RoleService};
use std::net::IpAddr;
use time::Date;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SiteBanData {
    pub banned_until: Option<Date>,
    pub reason: String,
}

impl_relation_new! {
    name => SiteBan,
    dest => site_id: Site,
    from => user_id: User,
    data => SiteBanData,
    create_fn => false,
    remove_fn => false,
}

impl RelationService {
    pub async fn create_site_ban(
        ctx: &ServiceContext<'_>,
        CreateSiteBan {
            site_id,
            user_id,
            created_by,
            metadata,
        }: CreateSiteBan,
        ip_address: IpAddr,
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

        if Self::site_member_exists(ctx, GetSiteMember { site_id, user_id })
            .await
            .or_raise(make_error)?
        {
            Self::remove_site_member(
                ctx,
                RemoveSiteMember {
                    site_id,
                    user_id,
                    removed_by: created_by,
                },
                ip_address,
                &metadata.reason,
            )
            .await
            .or_raise(make_error)?;
        }

        // TODO: remove site member applications

        let user_roles = UserRole::find()
            .filter(
                Condition::all()
                    .add(user_role::Column::UserId.eq(user_id))
                    .add(user_role::Column::SiteId.eq(site_id))
                    .add(user_role::Column::DeletedAt.is_null()),
            )
            .all(ctx.transaction())
            .await
            .or_raise(make_error)?;

        for user_role in user_roles {
            RoleService::revoke_role_from_user(
                ctx,
                RevokeUserRoleInput {
                    user_id,
                    role_id: user_role.role_id,
                    site_id,
                    revoking_user_id: created_by,
                    ip_address,
                },
            )
            .await
            .or_raise(make_error)?;
        }

        Self::create_site_ban_inner(
            ctx,
            CreateSiteBanInner {
                site_id,
                user_id,
                created_by,
                metadata: &metadata,
            },
        )
        .await
        .or_raise(make_error)?;

        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::SiteBanCreate {
                site_id,
                user_id,
                banning_user_id: created_by,
                banned_until: metadata.banned_until,
                reason: &metadata.reason,
            },
        )
        .await
        .or_raise(make_error)?;

        Ok(())
    }

    pub async fn remove_site_ban(
        ctx: &ServiceContext<'_>,
        RemoveSiteBan {
            site_id,
            user_id,
            removed_by,
        }: RemoveSiteBan,
        ip_address: IpAddr,
        reason: &str,
    ) -> Result<RelationModel> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to remove ban from site ID {} for user ID {}, as removed by ID {}",
                    site_id, user_id, removed_by,
                ),
                ErrorType::SiteBanRelation,
            )
        };

        let relation = Self::remove_site_ban_inner(
            ctx,
            RemoveSiteBan {
                site_id,
                user_id,
                removed_by,
            },
        )
        .await
        .or_raise(make_error)?;

        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::SiteBanRemove {
                site_id,
                user_id,
                unbanning_user_id: removed_by,
                reason,
            },
        )
        .await
        .or_raise(make_error)?;

        Ok(relation)
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

        if Self::site_ban_exists(ctx, body)
            .await
            .or_raise(make_error)?
        {
            error!(
                "User ID {} cannot {} because they are banned on site ID {}",
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

    /// Soft-deletes all active site bans whose expiry date has passed.
    ///
    /// Permanent bans, represented by a null `banned_until`, are left unmodified.
    ///
    /// # Returns
    /// The number of site bans lifted.
    pub async fn lift_expired_site_bans(ctx: &ServiceContext<'_>) -> Result<u64> {
        info!("Lifting expired site bans");

        let make_error = || {
            Error::new(
                "failed to lift expired site bans",
                ErrorType::SiteBanRelation,
            )
        };

        let txn = ctx.transaction();
        let today = now().date();

        let site_bans = Relation::find()
            .filter(
                Condition::all()
                    .add(relation::Column::RelationType.eq(RelationType::SiteBan))
                    .add(relation::Column::OverwrittenAt.is_null())
                    .add(relation::Column::DeletedAt.is_null()),
            )
            .all(txn)
            .await
            .or_raise(make_error)?;

        let mut lifted = 0;

        for site_ban in site_bans {
            let metadata: SiteBanData =
                serde_json::from_value(site_ban.metadata.clone()).or_raise(make_error)?;

            let Some(banned_until) = metadata.banned_until else {
                // Null means this is a permanent ban.
                continue;
            };

            if banned_until > today {
                // The ban has not expired yet.
                continue;
            }

            Self::remove_site_ban(
                ctx,
                RemoveSiteBan {
                    site_id: site_ban.dest_id,
                    user_id: site_ban.from_id,
                    removed_by: SYSTEM_USER_ID,
                },
                SYSTEM_IP_ADDRESS,
                "Site ban expired",
            )
            .await
            .or_raise(make_error)?;

            lifted += 1;
        }

        debug!("{lifted} expired site bans were lifted");
        Ok(lifted)
    }
}
