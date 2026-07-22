/*
 * services/relation/site_user.rs
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

//! Governs the relation which tracks "site users".
//!
//! These are special users (of type `site`) which represent a site as a whole.
//! They can be messaged to send messages to staff, and can be utilized to send
//! messages on behalf of a site (for instance, a ban notification).
//!
//! This relation describes which site a site-user corresponds to.
//! As such, it is an invariant that all users linked here are of the type `site`.

use super::prelude::*;
use crate::services::UserService;
use crate::types::{RelationObjectType, RelationType, UserType};

impl_relation!(SiteUser, Site, site_id, User, user_id, (), NO_CREATE_IMPL);

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
        let make_error = || {
            Error::new(
                format!(
                    "failed to designate user ID {} as the site user for site ID {}, created by user ID {}",
                    user_id, site_id, created_by,
                ),
                ErrorType::SiteUserRelation,
            )
        };

        // User to be added must of type 'site'
        let user = UserService::get(ctx, Reference::Id(user_id))
            .await
            .or_raise(make_error)?;

        if user.user_type != UserType::Site {
            error!(
                "Can only create site user relations if the user is of type 'site', not {:?}",
                user.user_type,
            );
            bail!(Error::new(
                format!(
                    "cannot create site user relation if user is not of type 'site', here {:?}",
                    user.user_type,
                ),
                ErrorType::BadRequest
            ));
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
            error!(
                "Found a different relation with this site, cannot create relation: {:?}",
                sites,
            );
            bail!(Error::new(
                format!(
                    "cannot create site user relation, sites not 1:1 - {:?}",
                    sites,
                ),
                ErrorType::BadRequest,
            ));
        }

        let users = RelationService::get_entries(
            ctx,
            RelationType::SiteUser,
            RelationObject::User(user_id),
            RelationDirection::From,
        )
        .await
        .or_raise(make_error)?;

        if !users.is_empty() {
            error!(
                "Found a different relation with this user, cannot create relation: {:?}",
                users,
            );
            bail!(Error::new(
                format!(
                    "cannot create site user relation, users not 1:1 - {:?}",
                    sites,
                ),
                ErrorType::BadRequest,
            ));
        }

        // Checks done, create
        create_operation!(
            ctx,
            SiteUser,
            Site,
            site_id,
            User,
            user_id,
            created_by,
            &(),
            make_error,
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
        .await
        .or_raise(|| {
            Error::new(
                format!("failed to get site user for site ID {}", site_id),
                ErrorType::SiteUserRelation,
            )
        })?;

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
        .await
        .or_raise(|| {
            Error::new(
                format!("failed to get site ID for site user ID {}", user_id),
                ErrorType::SiteUserRelation,
            )
        })?;

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

    let make_error = || {
        Error::new(
            "failed to get site user relation data",
            ErrorType::SiteUserRelation,
        )
    };

    let txn = ctx.transaction();
    let model = Relation::find()
        .filter(site_user_relation_condition(condition))
        .order_by_asc(relation::Column::CreatedAt)
        .one(txn)
        .await
        .or_raise(make_error)?;

    match model {
        Some(model) => Ok(model),
        None => bail!(Error::new(
            "no site user relation found",
            ErrorType::RelationNotFound,
        )),
    }
}

fn site_user_relation_condition(condition: Condition) -> Condition {
    Condition::all()
        .add(relation_type_condition(RelationType::SiteUser))
        .add(condition)
        .add(relation::Column::OverwrittenAt.is_null())
        .add(relation::Column::DeletedAt.is_null())
}

#[cfg(test)]
mod tests {
    use super::site_user_relation_condition;
    use crate::models::relation;
    use sea_orm::{
        ColumnTrait, Condition, DatabaseBackend, EntityTrait, QueryFilter, QueryTrait,
    };

    #[test]
    fn site_user_lookup_matches_legacy_and_namespaced_database_values() {
        let statement = relation::Entity::find()
            .filter(site_user_relation_condition(
                Condition::all().add(relation::Column::DestId.eq(42)),
            ))
            .build(DatabaseBackend::Postgres);

        let sql = statement.to_string();
        assert!(
            sql.contains(r#""relation"."relation_type" IN ('user', 'site-user')"#),
            "{sql}"
        );
    }
}
