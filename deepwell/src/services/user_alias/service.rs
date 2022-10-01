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
use crate::models::user::{self, Entity as User};
use crate::models::user_alias::{self, Entity as UserAlias, Model as UserAliasModel};
use crate::services::user::UserService;
use crate::utils::get_user_slug;
use crate::web::Reference;

#[derive(Debug)]
pub struct UserAliasService;

impl UserAliasService {
    /// Creates a new user alias.
    pub async fn create(
        ctx: &ServiceContext<'_>,
        input: CreateUserAlias,
    ) -> Result<CreateUserAliasOutput> {
        let output = Self::create_no_verify(ctx, input).await?;
        Self::verify(ctx, &output.slug).await?;
        Ok(output)
    }

    /// Creates a new user alias, but does not perform row checks.
    ///
    /// This method should only be invoked when the corresponding user
    /// row has not been updated, if in doubt use `UserAliasService::create()`.
    ///
    /// The caller is responsible for calling `UserAliasService::verify()` after
    /// all its database changes have been made.
    pub(crate) async fn create_no_verify(
        ctx: &ServiceContext<'_>,
        input: CreateUserAlias,
    ) -> Result<CreateUserAliasOutput> {
        let txn = ctx.transaction();
        let slug = get_user_slug(input.slug);

        tide::log::info!("Creating user alias with slug '{}'", slug);

        // Check for conflicts
        //
        // This also checks aliases, though we verify down below that
        // it actually finds conflicts properly.
        if UserService::exists(ctx, Reference::Slug(&slug)).await? {
            tide::log::error!(
                "User with conflicting slug '{slug}' already exists, cannot create",
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

    #[inline]
    #[allow(dead_code)] // TEMP
    pub async fn get(ctx: &ServiceContext<'_>, slug: &str) -> Result<UserAliasModel> {
        find_or_error(Self::get_optional(ctx, slug)).await
    }

    #[inline]
    pub async fn exists(ctx: &ServiceContext<'_>, slug: &str) -> Result<bool> {
        Self::get_optional(ctx, slug)
            .await
            .map(|alias| alias.is_some())
    }

    /// Used for when a user renames to an old slug.
    ///
    /// This takes the old user alias and renames the slug in-place, without having to do
    /// `create()` / `delete()` (which runs into a dependency issue as `create()` checks
    /// `UserService` that the user doesn't already exist with that name.
    ///
    /// The database uniqueness constraint enforces that the `slug` doesn't collide with another
    /// user's alias.
    pub async fn swap(
        ctx: &ServiceContext<'_>,
        alias_id: i64,
        new_slug: &str,
    ) -> Result<()> {
        let txn = ctx.transaction();

        tide::log::info!(
            "Swapping user alias ID {} to use slug '{}'",
            alias_id,
            new_slug,
        );

        let model = user_alias::ActiveModel {
            alias_id: Set(alias_id),
            slug: Set(str!(new_slug)),
            ..Default::default()
        };

        model.update(txn).await?;
        Ok(())
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

    /// Verifies that the `user` and `user_alias` tables are consistent.
    ///
    /// These tables have a uniqueness invariant wherein a slug is only
    /// present in at most one of these two tables, but not both.
    pub async fn verify(ctx: &ServiceContext<'_>, slug: &str) -> Result<()> {
        tide::log::info!(
            "Verifying user and user alias table consistency for slug '{}'",
            slug,
        );

        let txn = ctx.transaction();
        let (user_result, alias_result) = try_join!(
            User::find().filter(user::Column::Slug.eq(slug)).one(txn),
            UserAlias::find()
                .filter(user_alias::Column::Slug.eq(slug))
                .one(txn),
        )?;

        if let (Some(user), Some(alias)) = (user_result, alias_result) {
            // Both tables bear the same slug!
            // Invariant was not held, panic

            tide::log::error!(
                "Consistency error! Both user and user_alias tables have the slug '{}'",
                slug,
            );

            panic!(
                "Slug appears as both a user and an alias!\nUser: {user:#?}\nAlias: {alias:#?}",
            );
        }

        Ok(())
    }
}
