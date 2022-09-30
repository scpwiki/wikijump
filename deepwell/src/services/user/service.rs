/*
 * services/user/service.rs
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
use crate::models::sea_orm_active_enums::UserType;
use crate::models::user::{self, Entity as User, Model as UserModel};
use crate::services::{user_alias::CreateUserAlias, UserAliasService};
use crate::utils::get_user_slug;
use std::cmp;

// TODO make these configurable
const DEFAULT_NAME_CHANGES: i16 = 3;
const MAX_NAME_CHANGES: i16 = 3;

#[derive(Debug)]
pub struct UserService;

impl UserService {
    pub async fn create(
        ctx: &ServiceContext<'_>,
        input: CreateUser,
    ) -> Result<CreateUserOutput> {
        let txn = ctx.transaction();
        let slug = get_user_slug(&input.name);

        // Check for conflicts
        let result = User::find()
            .filter(
                Condition::all()
                    .add(
                        Condition::any()
                            .add(user::Column::Name.eq(input.name.as_str()))
                            .add(user::Column::Email.eq(input.email.as_str()))
                            .add(user::Column::Slug.eq(slug.as_str())),
                    )
                    .add(user::Column::DeletedAt.is_null()),
            )
            .one(txn)
            .await?;

        if result.is_some() {
            tide::log::error!("User with conflicting name, slug, or email already exists, cannot create");
            return Err(Error::Conflict);
        }

        // Check for alias conflicts
        let result = UserAliasService::get_optional(ctx, &slug).await?;
        if result.is_some() {
            tide::log::error!(
                "User alias with conflicting slug already exists, cannot create",
            );

            return Err(Error::Conflict);
        }

        // Set up password field depending on type
        let password = match input.user_type {
            UserType::Regular => {
                tide::log::info!("Creating regular user '{slug}' with password");

                match input.password {
                    Some(password) => hash_password(password),
                    None => {
                        tide::log::warn!("No password specified");
                        return Err(Error::BadRequest);
                    }
                }
            }
            UserType::System => {
                tide::log::info!("Creating system user '{slug}'");

                if input.password.is_some() {
                    tide::log::warn!("Password was specified for system user");
                    return Err(Error::BadRequest);
                }

                // Disabled password
                str!("!")
            }
            UserType::Bot => {
                tide::log::info!("Creating bot user '{slug}'");

                if input.password.is_some() {
                    tide::log::warn!("Password was specified for bot user");
                    return Err(Error::BadRequest);
                }

                // TODO assign bot token
                str!("TODO bot token")
            }
        };

        // Insert new model
        let user = user::ActiveModel {
            user_type: Set(input.user_type),
            name: Set(input.name),
            slug: Set(slug.clone()),
            name_changes_left: Set(DEFAULT_NAME_CHANGES),
            email: Set(input.email),
            email_verified_at: Set(None),
            password: Set(password),
            multi_factor_secret: Set(None),
            multi_factor_recovery_codes: Set(None),
            locale: Set(input.locale),
            avatar_s3_hash: Set(None),
            display_name: Set(None),
            gender: Set(None),
            birthday: Set(None),
            biography: Set(None),
            user_page: Set(None),
            created_at: Set(now()),
            updated_at: Set(None),
            deleted_at: Set(None),
            ..Default::default()
        };

        let user_id = User::insert(user).exec(txn).await?.last_insert_id;
        Ok(CreateUserOutput { user_id, slug })
    }

    #[inline]
    pub async fn exists(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
    ) -> Result<bool> {
        Self::get_optional(ctx, reference)
            .await
            .map(|user| user.is_some())
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        mut reference: Reference<'_>,
    ) -> Result<Option<UserModel>> {
        let txn = ctx.transaction();

        // If slug, determine if this is a user alias.
        //
        // NOTE: Originally I tried having a direct query to
        //       select both the user and user_alias table at
        //       the same time. I tried a JOIN and a subquery,
        //       but for both the query planner indictated that
        //       they would be slower than doing queries on
        //       simple indexes directly, which is why we are
        //       doing it this way.
        if let Reference::Slug(slug) = reference {
            // If present, proceed with SELECT by id.
            // If absent, then this user is missing, return.
            let alias = match UserAliasService::get_optional(ctx, slug).await? {
                Some(alias) => alias,
                None => return Ok(None),
            };

            // Rewrite reference so in the "real" user search
            // we locate directly via user ID.
            reference = Reference::Id(alias.user_id);
        }

        let user = match reference {
            // Get directly from ID
            Reference::Id(id) => User::find_by_id(id).one(txn).await?,

            // Since a slug can be an alias, check for a redirect
            Reference::Slug(slug) => {
                User::find()
                    .filter(
                        Condition::all()
                            .add(user::Column::Slug.eq(slug))
                            .add(user::Column::DeletedAt.is_null()),
                    )
                    .one(txn)
                    .await?
            }
        };

        Ok(user)
    }

    #[inline]
    pub async fn get(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
    ) -> Result<UserModel> {
        find_or_error(Self::get_optional(ctx, reference)).await
    }

    pub async fn update(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
        input: UpdateUser,
    ) -> Result<()> {
        let txn = ctx.transaction();
        let user = Self::get(ctx, reference).await?;
        let mut model = user::ActiveModel {
            user_id: Set(user.user_id),
            ..Default::default()
        };

        // Add each field
        if let ProvidedValue::Set(name) = input.name {
            Self::update_name(ctx, name, &user, &mut model).await?;
        }

        if let ProvidedValue::Set(email) = input.email {
            model.email = Set(email);
        }

        if let ProvidedValue::Set(email_verified) = input.email_verified {
            let timestamp = if email_verified { Some(now()) } else { None };
            model.email_verified_at = Set(timestamp);
        }

        if let ProvidedValue::Set(password) = input.password {
            model.password = Set(hash_password(password));
        }

        if let ProvidedValue::Set(locale) = input.locale {
            model.locale = Set(locale);
        }

        if let ProvidedValue::Set(display_name) = input.display_name {
            model.display_name = Set(display_name);
        }

        if let ProvidedValue::Set(gender) = input.gender {
            model.gender = Set(gender);
        }

        if let ProvidedValue::Set(birthday) = input.birthday {
            model.birthday = Set(birthday);
        }

        if let ProvidedValue::Set(biography) = input.biography {
            model.biography = Set(biography);
        }

        if let ProvidedValue::Set(user_page) = input.user_page {
            model.user_page = Set(user_page);
        }

        if let ProvidedValue::Set(avatar) = input.avatar {
            // TODO store avatar in S3
            model.avatar_s3_hash = Set(avatar);
        }

        // Set update flag
        model.updated_at = Set(Some(now()));

        // Update and return
        model.update(txn).await?;
        Ok(())
    }

    async fn update_name(
        ctx: &ServiceContext<'_>,
        name: String,
        user: &UserModel,
        model: &mut user::ActiveModel,
    ) -> Result<()> {
        // Regardless of the number of name change tokens,
        // the user can always change their name if the slug is
        // unaltered, or if the slug is a prior name of theirs
        // (i.e. they have a user alias for it).

        let slug = get_user_slug(&name);

        if user.slug == slug {
            tide::log::debug!("User slug is the same, rename is free");

            // Set model, but return early, we don't deduct a name change token
            model.name = Set(name);
            return Ok(());
        }

        if let Some(alias) = UserAliasService::get_optional(ctx, &slug).await? {
            tide::log::debug!("User slug is a past alias, rename is free");

            // Swap old user alias
            UserAliasService::swap(ctx, alias.alias_id, &slug).await?;

            // Set model, but return early, we don't deduct a name change token
            model.name = Set(name);
            model.slug = Set(slug);
            return Ok(());
        }

        // All changes beyond this point involve creating a new alias, so
        // a name change token must be consumed.
        if user.name_changes_left == 0 {
            tide::log::error!("User ID {} has no remaining name changes", user.user_id);
            return Err(Error::InsufficientNameChanges);
        }

        tide::log::debug!("Creating user alias for {} and deducting name change", slug);

        // Deduct name change token and add user alias for old slug.
        //
        // The "created by" is the user themselves, since
        // they initiatived the rename.
        UserAliasService::create(
            ctx,
            CreateUserAlias {
                slug: user.slug.clone(),
                target_user_id: user.user_id,
                created_by_user_id: user.user_id,
            },
        )
        .await?;

        model.name_changes_left = Set(user.name_changes_left - 1);
        model.name = Set(name);
        model.slug = Set(slug);
        Ok(())
    }

    /// Adds an additional rename token, up to the cap.
    ///
    /// # Returns
    /// The current number of rename tokens the user has.
    pub async fn add_name_change_token(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
    ) -> Result<i16> {
        let txn = ctx.transaction();
        let user = Self::get(ctx, reference).await?;

        let name_changes = cmp::min(user.name_changes_left + 1, MAX_NAME_CHANGES);
        let model = user::ActiveModel {
            user_id: Set(user.user_id),
            name_changes_left: Set(name_changes),
            updated_at: Set(Some(now())),
            ..Default::default()
        };

        tide::log::info!(
            "Adding name change token to user ID {} (was {}, now {}, max {})",
            user.user_id,
            user.name_changes_left,
            name_changes,
            MAX_NAME_CHANGES,
        );

        model.update(txn).await?;
        Ok(name_changes)
    }

    pub async fn delete(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
    ) -> Result<UserModel> {
        let txn = ctx.transaction();
        let user = Self::get(ctx, reference).await?;
        tide::log::info!("Deleting user with ID {}", user.user_id);

        // Delete all user aliases
        UserAliasService::delete_all(ctx, user.user_id).await?;

        // Set deletion flag
        let model = user::ActiveModel {
            user_id: Set(user.user_id),
            deleted_at: Set(Some(now())),
            ..Default::default()
        };

        // Update and return
        let user = model.update(txn).await?;
        Ok(user)
    }
}

// Helpers

// TEMP helper, so it's easier to replace when implemented
// TODO replace
fn hash_password(value: String) -> String {
    // TODO Securely hash password
    value
}
