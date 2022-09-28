/*
 * services/user_alias/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2022 Wikijump Team
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
use crate::models::user::{self, Entity as User, Model as UserModel};
use crate::models::user_alias::{self, Entity as UserAlias, Model as UserAliasModel};
use crate::services::user::UserService;
use crate::utils::get_user_slug;
use crate::web::Reference;
use sea_orm::query::JoinType;

#[derive(Debug)]
pub struct UserAliasService;

impl UserAliasService {
    pub async fn create(
        ctx: &ServiceContext<'_>,
        input: CreateUserAlias,
    ) -> Result<CreateUserAliasOutput> {
        let txn = ctx.transaction();
        let slug = get_user_slug(input.slug);

        // Check for conflicts
        if UserService::exists(ctx, Reference::Slug(&slug)).await? {
            tide::log::error!(
                "User with conflicting slug '{slug}' already exists, cannot create",
            );

            return Err(Error::Conflict);
        }

        // The above also checks aliases, but here we do a separate query just to be sure
        let result = UserAlias::find()
            .filter(user_alias::Column::Slug.eq(slug.as_str()))
            .one(txn)
            .await?;

        if result.is_some() {
            tide::log::error!(
                "User alias with conflicting slug '{slug}' already exists, cannot create",
            );

            return Err(Error::Conflict);
        }

        // Insert new model
        let alias = user_alias::ActiveModel {
            created_by: Set(input.created_by_user_id),
            user_id: Set(input.target_user_id),
            slug: Set(slug.clone()),
            ..Default::default()
        };

        let alias_id = UserAlias::insert(alias).exec(txn).await?.last_insert_id;
        Ok(CreateUserAliasOutput { alias_id, slug })
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        slug: &str,
    ) -> Result<Option<UserAliasModel>> {
        let txn = ctx.transaction();

        let alias = UserAlias::find()
            .filter(user::Column::Slug.eq(slug))
            .one(txn)
            .await?;

        Ok(alias)
    }

    pub async fn get(ctx: &ServiceContext<'_>, slug: &str) -> Result<UserAliasModel> {
        Self::get_optional(ctx, slug).await?.ok_or(Error::NotFound)
    }

    /// If this alias exists, then get the `UserModel` for it.
    /// Otherwise, return `None`.
    pub async fn get_redirect_optional(
        ctx: &ServiceContext<'_>,
        slug: &str,
    ) -> Result<Option<UserModel>> {
        let txn = ctx.transaction();

        /*
         * Get a user by slug, or a joined user_alias by slug.
         *
         * SELECT *
         * FROM "user"
         * JOIN user_alias
         * ON "user".user_id = user_alias.user_id
         * WHERE "user".slug = $1
         * AND user_alias.slug = $1
         *
         */

        let user = User::find()
            .join_rev(
                JoinType::Join,
                UserAlias::belongs_to(User)
                    .from(user_alias::Column::UserId)
                    .to(user::Column::UserId)
                    .into(),
            )
            .filter(
                Condition::any()
                    .add(user::Column::Slug.eq(slug))
                    .add(user_alias::Column::Slug.eq(slug)),
            )
            .one(txn)
            .await?;

        Ok(user)
    }

    pub async fn delete(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
    ) -> Result<()> {
        let txn = ctx.transaction();

        match reference {
            Reference::Id(id) => {
                tide::log::info!("Deleting user alias ID {id}");
                UserAlias::delete_by_id(id).exec(txn).await?;
            }
            Reference::Slug(slug) => {
                tide::log::info!("Deleting user alias '{slug}'");
                let alias = Self::get(ctx, slug).await?;
                alias.delete(txn).await?;
            }
        }

        Ok(())
    }
}
