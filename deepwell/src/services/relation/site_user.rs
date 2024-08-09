/*
 * services/relation/site_user.rs
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

//! Governs the relation which tracks "site users".
//!
//! These are special users (of type `site`) which represent a site as a whole.
//! They can be messaged to send messages to staff, and can be utilized to send
//! messages on behalf of a site (for instance, a ban notification).
//!
//! This relation describes which site a site-user corresponds to.
//! As such, it is an invariant that all users linked here are of the type `site`.

use super::prelude::*;
use crate::models::sea_orm_active_enums::UserType;
use crate::services::UserService;

impl_relation!(SiteUser, Site, site_id, User, user_id, (), NO_CREATE_IMPL,);

impl RelationService {
    pub async fn create_site_user(
        ctx: &ServiceContext<'_>,
        CreateSiteUser {
            site_id,
            user_id,
            metadata: (),
            created_by,
        }: CreateSiteUser,
    ) -> Result<()> {
        // User to be added must of type 'site'
        let user = UserService::get(ctx, Reference::Id(user_id)).await?;
        if user.user_type != UserType::Site {
            error!(
                "Can only create site user relations if the user is of type 'site', not {:?}",
                user.user_type,
            );
            return Err(Error::BadRequest);
        }

        // Site <--> User must be 1:1
        //
        // This means there should be no results for both
        // this site_id -> anything and this user_id -> anything.

        let sites = RelationService::get_entries(
            ctx,
            RelationType::SiteUser,
            RelationObject::Site(site_id),
            RelationDirection::Dest,
        )
        .await?;

        if !sites.is_empty() {
            error!("Found a different relation with this site, cannot create relation: {sites:?}");
            return Err(Error::BadRequest);
        }

        let users = RelationService::get_entries(
            ctx,
            RelationType::SiteUser,
            RelationObject::User(user_id),
            RelationDirection::From,
        )
        .await?;

        if !users.is_empty() {
            error!("Found a different relation with this user, cannot create relation: {users:?}");
            return Err(Error::BadRequest);
        }

        // Checks done, create
        create_operation!(
            ctx,
            SiteMember,
            Site,
            site_id,
            User,
            user_id,
            created_by,
            &()
        )
    }

    pub async fn get_site_user_id_for_site(
        ctx: &ServiceContext<'_>,
        site_id: i64,
    ) -> Result<i64> {
        info!("Getting site user for site ID {site_id}");

        let model = get_relation(
            ctx,
            Condition::all()
                .add(relation::Column::DestType.eq(RelationObjectType::Site))
                .add(relation::Column::DestId.eq(site_id)),
        )
        .await?;

        Ok(model.from_id)
    }

    pub async fn get_site_id_for_site_user(
        ctx: &ServiceContext<'_>,
        user_id: i64,
    ) -> Result<i64> {
        let model = get_relation(
            ctx,
            Condition::all()
                .add(relation::Column::FromType.eq(RelationObjectType::User))
                .add(relation::Column::FromId.eq(user_id)),
        )
        .await?;

        Ok(model.dest_id)
    }
}

async fn get_relation(
    ctx: &ServiceContext<'_>,
    condition: Condition,
) -> Result<RelationModel> {
    // We implement our own query since it's 1:1 and we
    // don't have to worry about multiple results like
    // for get_entries().

    let txn = ctx.seaorm_transaction();
    let model = Relation::find()
        .filter(
            Condition::all()
                .add(relation::Column::RelationType.eq(RelationType::SiteUser.value()))
                .add(condition)
                .add(relation::Column::OverwrittenAt.is_null())
                .add(relation::Column::DeletedAt.is_null()),
        )
        .order_by_asc(relation::Column::CreatedAt)
        .one(txn)
        .await?;

    match model {
        Some(model) => Ok(model),
        None => Err(Error::RelationNotFound),
    }
}
