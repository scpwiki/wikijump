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
        SiteMembership { site_id, user_id }: SiteMembership
    ) -> Result<Option<SiteMemberModel>> {
        let txn = ctx.transaction();
        tide::log::info!("Adding membership of user with ID {user_id} to site ID {site_id}");

        // If the user is already a member of the target site, discontinue.
        if Self::get_optional(ctx, SiteMembership { site_id, user_id }).await?.is_some() {
            return Ok(None);
        }

        // TODO: Check for membership qualifications (e.g. bans).

        // Insert new membership.
        let model = site_member::ActiveModel {
            user_id: Set(user_id),
            site_id: Set(site_id),
            date_left: Set(None),
            ..Default::default()
        };

        let membership = model.insert(txn).await?;

        Ok(Some(membership))
    }

    /// Remove a user from a site.
    pub async fn remove(ctx: &ServiceContext<'_>, SiteMembership { site_id, user_id }: SiteMembership) -> Result<Option<SiteMemberModel>> {
        let txn = ctx.transaction();
        tide::log::info!("Removing the membership of user ID {user_id} from site ID {site_id}");

        let model = match Self::get_optional(ctx, SiteMembership { site_id, user_id }).await? {
            // If membership is found, remove it by setting the leave date.
            Some(member_model) => {
                let mut model = member_model.into_active_model();
                model.date_left = Set(Some(now()));
                model.update(txn).await?
            },

            // If no membership is found, return BadRequest error.
            None => {
                tide::log::error!(
                    "Could not remove user ID {user_id} from site ID {site_id} as they are not a member."
                );
                return Err(Error::BadRequest);
            }
        };

        Ok(Some(model))
    }

    #[inline]
    pub async fn get(ctx: &ServiceContext<'_>, key: SiteMembership) -> Result<SiteMemberModel> {
        find_or_error(Self::get_optional(ctx, key)).await
    }

    /// Get whether a user is a member of a site or not. Returns `None` if no membership is found. 
    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        SiteMembership { site_id, user_id }: SiteMembership
    ) -> Result<Option<SiteMemberModel>> {
        let txn = ctx.transaction();
        let model = SiteMember::find()
            .filter(
                Condition::all()
                    .add(site_member::Column::SiteId.eq(site_id))
                    .add(site_member::Column::UserId.eq(user_id))
                    .add(site_member::Column::DateLeft.is_null()),
            )
            .one(txn)
            .await?;
        Ok(model)
    }

    /// Get all users of a site, ordered by oldest to newest.
    pub async fn get_site_members(ctx: &ServiceContext<'_>, site_id: i64) -> Result<Vec<SiteMemberModel>> {
        let txn = ctx.transaction();

        let models = SiteMember::find()
        .filter(
            Condition::all()
                .add(site_member::Column::SiteId.eq(site_id))
                .add(site_member::Column::DateLeft.is_null())
        )
        .order_by_asc(site_member::Column::MembershipId)
        .all(txn)
        .await?;

        Ok(models)
    }

    /// Get membership history.
    /// 
    /// The `start_id` argument gives the start ID to search from, exclusive.
    /// If `0`, then it means "everything".
    /// 
    /// Both `user_id` and `site_id` are presented as separate arguments to allow for
    /// neither to be provided as input (returning all membership history in general), one
    /// or the other to be provided, or both (which tracks a user's membership on a specific site).
    /// 
    /// The `current_members` argument:
    /// * If it is `Some(true)`, then it only returns current memberships.
    /// * If it is `Some(false)`, then it doesn't return any current memberships.
    /// * If it is `None`, then it returns all memberships, regardless of currency.
    pub async fn get_history(
        ctx: &ServiceContext<'_>,
        SiteMembershipHistory {
            user_id,
            site_id,
            current_members,
            start_id,
            limit
        }: SiteMembershipHistory
    ) -> Result<Vec<SiteMemberModel>> {
        let txn = ctx.transaction();
        let condition = Self::build_history_condition(user_id, site_id, current_members, start_id);

        let model = SiteMember::find()
            .filter(condition)
            .order_by_asc(site_member::Column::MembershipId)
            .limit(limit)
            .all(txn)
            .await?;

        Ok(model)
    }

    /// Counts the number of memberships according to the same specifications of `get_history()`.
    /// See the method for more information.
    pub async fn get_history_count(
        ctx: &ServiceContext<'_>,
        SiteMembershipHistory {
            user_id,
            site_id,
            current_members,
            start_id,
            limit
        }: SiteMembershipHistory
    ) -> Result<u64> {
        let txn = ctx.transaction();
        let condition = Self::build_history_condition(user_id, site_id, current_members, start_id);

        let count = SiteMember::find()
            .filter(condition)
            .order_by_asc(site_member::Column::MembershipId)
            .limit(limit)
            .count(txn)
            .await?;

        Ok(count)
    }

    fn build_history_condition(
        user_id: Option<i64>,
        site_id: Option<i64>,
        current_members: Option<bool>,
        start_id: i64
    ) -> Condition {
        let user_condition = user_id.map(|id| site_member::Column::UserId.eq(id));
        let site_condition = site_id.map(|id| site_member::Column::SiteId.eq(id));

        let members_condition = match current_members {
            Some(true) => Some(site_member::Column::DateLeft.is_null()),
            Some(false) => Some(site_member::Column::DateLeft.is_not_null()),
            None => None,
        };

        Condition::all()
            .add(site_member::Column::MembershipId.gt(start_id))
            .add_option(user_condition)
            .add_option(site_condition)
            .add_option(members_condition)
    }
}