/*
 * services/site_member/service.rs
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
use crate::models::site_member::{self, Entity as SiteMember, Model as SiteMemberModel};

#[derive(Debug)]
pub struct SiteMemberService;

impl SiteMemberService {
    /// Add a user to a site.
    pub async fn add(
        ctx: &ServiceContext<'_>,
        SiteMembership { site_id, user_id }: SiteMembership,
    ) -> Result<Option<SiteMemberModel>> {
        let txn = ctx.transaction();
        tide::log::info!(
            "Adding membership of user with ID {user_id} to site ID {site_id}"
        );

        // If the user is already a member of the target site, discontinue.
        if Self::exists(ctx, SiteMembership { site_id, user_id }).await? {
            return Ok(None);
        }

        // TODO: Check for membership qualifications (e.g. bans).

        // Insert new membership.
        let model = site_member::ActiveModel {
            user_id: Set(user_id),
            site_id: Set(site_id),
            ..Default::default()
        };

        let membership = model.insert(txn).await?;

        Ok(Some(membership))
    }

    /// Remove a user from a site.
    pub async fn remove(
        ctx: &ServiceContext<'_>,
        SiteMembership { site_id, user_id }: SiteMembership,
    ) -> Result<SiteMemberModel> {
        let txn = ctx.transaction();
        tide::log::info!(
            "Removing the membership of user ID {user_id} from site ID {site_id}"
        );

        let model = match Self::get_optional(ctx, SiteMembership { site_id, user_id })
            .await?
        {
            // If membership is found, remove it by setting the leave date.
            Some(member_model) => {
                let mut model = member_model.into_active_model();
                model.deleted_at = Set(Some(now()));
                model.update(txn).await?
            }

            // If no membership is found, return BadRequest error.
            None => {
                tide::log::error!(
                    "Could not remove user ID {user_id} from site ID {site_id} as they are not a member."
                );
                return Err(Error::BadRequest);
            }
        };

        Ok(model)
    }

    #[inline]
    pub async fn exists(ctx: &ServiceContext<'_>, key: SiteMembership) -> Result<bool> {
        Self::get_optional(ctx, key)
            .await
            .map(|member| member.is_some())
    }

    #[inline]
    pub async fn get(
        ctx: &ServiceContext<'_>,
        key: SiteMembership,
    ) -> Result<SiteMemberModel> {
        find_or_error(Self::get_optional(ctx, key)).await
    }

    /// Get whether a user is a member of a site or not. Returns `None` if no membership is found.
    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        SiteMembership { site_id, user_id }: SiteMembership,
    ) -> Result<Option<SiteMemberModel>> {
        let txn = ctx.transaction();
        let model = SiteMember::find()
            .filter(
                Condition::all()
                    .add(site_member::Column::SiteId.eq(site_id))
                    .add(site_member::Column::UserId.eq(user_id))
                    .add(site_member::Column::DeletedAt.is_null()),
            )
            .one(txn)
            .await?;
        Ok(model)
    }

    /// Get all users of a site, ordered by oldest to newest.
    pub async fn get_site_members(
        ctx: &ServiceContext<'_>,
        site_id: i64,
    ) -> Result<Vec<SiteMemberModel>> {
        let txn = ctx.transaction();

        let models = SiteMember::find()
            .filter(
                Condition::all()
                    .add(site_member::Column::SiteId.eq(site_id))
                    .add(site_member::Column::DeletedAt.is_null()),
            )
            .order_by_asc(site_member::Column::MembershipId)
            .all(txn)
            .await?;

        Ok(models)
    }

    /// Get all sites a user is a member of, ordered by oldest to newest.
    pub async fn get_user_sites(
        ctx: &ServiceContext<'_>,
        user_id: i64,
    ) -> Result<Vec<SiteMemberModel>> {
        let txn = ctx.transaction();

        let models = SiteMember::find()
            .filter(
                Condition::all()
                    .add(site_member::Column::UserId.eq(user_id))
                    .add(site_member::Column::DeletedAt.is_null()),
            )
            .order_by_asc(site_member::Column::MembershipId)
            .all(txn)
            .await?;

        Ok(models)
    }
}
