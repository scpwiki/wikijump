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
use crate::models::user_alias::{self, Entity as UserAlias, Model as UserAliasModel};
use crate::services::user::UserService;
use crate::utils::get_user_slug;
use crate::web::Reference;
use sea_orm::DeleteResult;

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
            .filter(user_alias::Column::Slug.eq(slug))
            .one(txn)
            .await?;

        Ok(alias)
    }

    pub async fn get(ctx: &ServiceContext<'_>, slug: &str) -> Result<UserAliasModel> {
        Self::get_optional(ctx, slug).await?.ok_or(Error::NotFound)
    }

    /// Deletes all user aliases for this user.
    ///
    /// # Returns
    /// The number of deleted aliases.
    pub async fn delete_all(ctx: &ServiceContext<'_>, user_id: i64) -> Result<u64> {
        let txn = ctx.transaction();

        tide::log::info!("Deleting all user aliases for user ID {user_id}");

        let DeleteResult { rows_affected } = UserAlias::delete_many()
            .filter(user_alias::Column::UserId.eq(user_id))
            .exec(txn)
            .await?;

        tide::log::debug!(
            "{rows_affected} user aliases for user ID {user_id} were deleted",
        );

        Ok(rows_affected)
    }
}
