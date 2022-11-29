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
use crate::services::filter::{FilterClass, FilterType};
use crate::services::user_alias::CreateUserAlias;
use crate::services::{FilterService, PasswordService, UserAliasService};
use crate::utils::{get_user_slug, regex_replace_in_place};
use regex::Regex;
use sea_orm::ActiveValue;
use std::cmp;

// TODO make these configurable
const DEFAULT_NAME_CHANGES: i16 = 3;
const MAX_NAME_CHANGES: i16 = 3;

lazy_static! {
    static ref LEADING_TRAILING_CHARS: Regex = Regex::new(r"(^[\-\s]+)|([\-\s+]$)").unwrap();
}

#[derive(Debug)]
pub struct UserService;

impl UserService {
    pub async fn create(
        ctx: &ServiceContext<'_>,
        user_type: UserType,
        mut input: CreateUser,
    ) -> Result<CreateUserOutput> {
        let txn = ctx.transaction();
        let slug = get_user_slug(&input.name);

        tide::log::debug!(
            "Normalizing user data (name '{}', slug '{}')",
            input.name,
            slug,
        );
        regex_replace_in_place(&mut input.name, &LEADING_TRAILING_CHARS, "");

        tide::log::info!("Attempting to create user '{}' ('{}')", input.name, slug);

        // Perform filter validation
        if !input.bypass_filter {
            Self::run_filter(ctx, &input.name, &slug).await?;
        }

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
        if UserAliasService::exists(ctx, &slug).await? {
            tide::log::error!(
                "User alias with conflicting slug already exists, cannot create",
            );

            return Err(Error::Conflict);
        }

        // Set up password field depending on type
        let password = match user_type {
            UserType::Regular => {
                tide::log::info!("Creating regular user '{slug}' with password");
                PasswordService::new_hash(&input.password)?
            }
            UserType::System => {
                tide::log::info!("Creating system user '{slug}'");

                if !input.password.is_empty() {
                    tide::log::warn!("Password was specified for system user");
                    return Err(Error::BadRequest);
                }

                // Disabled password
                str!("!")
            }
            UserType::Bot => {
                tide::log::info!("Creating bot user '{slug}'");
                // TODO assign bot token
                format!("TODO bot token: {}", input.password)
            }
        };

        // Insert new model
        let user = user::ActiveModel {
            user_type: Set(user_type),
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
            real_name: Set(None),
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

    /// Gets a user, but fails if the user type doesn't match.
    pub async fn get_with_user_type(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
        user_type: UserType,
    ) -> Result<UserModel> {
        let user = Self::get(ctx, reference).await?;

        if user.user_type == user_type {
            Ok(user)
        } else {
            Err(Error::BadRequest)
        }
    }

    pub async fn update(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
        input: UpdateUser,
    ) -> Result<()> {
        // NOTE: Filter validation occurs in update_name(), not here
        let txn = ctx.transaction();
        let user = Self::get(ctx, reference).await?;

        let mut verify_name = false;
        let mut model = user::ActiveModel {
            user_id: Set(user.user_id),
            ..Default::default()
        };

        // Add each field
        if let ProvidedValue::Set(name) = input.name {
            Self::update_name(ctx, name, &user, &mut model, input.bypass_filter).await?;
            verify_name = true;
        }

        if let ProvidedValue::Set(email) = input.email {
            model.email = Set(email);
        }

        if let ProvidedValue::Set(email_verified) = input.email_verified {
            let timestamp = if email_verified { Some(now()) } else { None };
            model.email_verified_at = Set(timestamp);
        }

        if let ProvidedValue::Set(password) = input.password {
            let password_hash = PasswordService::new_hash(&password)?;
            model.password = Set(password_hash);
        }

        if let ProvidedValue::Set(locale) = input.locale {
            model.locale = Set(locale);
        }

        if let ProvidedValue::Set(real_name) = input.real_name {
            model.real_name = Set(real_name);
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
        let new_user = model.update(txn).await?;

        // Verify, if needed
        if verify_name {
            try_join!(
                UserAliasService::verify(ctx, &user.slug),
                UserAliasService::verify(ctx, &new_user.slug),
            )?;
        }

        Ok(())
    }

    async fn update_name(
        ctx: &ServiceContext<'_>,
        new_name: String,
        user: &UserModel,
        model: &mut user::ActiveModel,
        bypass_filter: bool,
    ) -> Result<()> {
        // Regardless of the number of name change tokens,
        // the user can always change their name if the slug is
        // unaltered, or if the slug is a prior name of theirs
        // (i.e. they have a user alias for it).

        let new_slug = get_user_slug(&new_name);
        let old_slug = &user.slug;

        // Perform filter validation
        if !bypass_filter {
            Self::run_filter(ctx, &new_name, &new_slug).await?;
        }

        if new_slug == user.slug {
            tide::log::debug!("User slug is the same, rename is free");

            // Set model, but return early, we don't deduct a name change token
            model.name = Set(new_name);
            return Ok(());
        }

        if let Some(alias) = UserAliasService::get_optional(ctx, &new_slug).await? {
            tide::log::debug!("User slug is a past alias, rename is free");

            // Swap user alias for old slug
            UserAliasService::swap(ctx, alias.alias_id, old_slug).await?;

            // Set model, but return early, we don't deduct a name change token
            model.name = Set(new_name);
            model.slug = Set(new_slug);
            return Ok(());
        }

        // All changes beyond this point involve creating a new alias, so
        // a name change token must be consumed.
        if user.name_changes_left == 0 {
            tide::log::error!("User ID {} has no remaining name changes", user.user_id);
            return Err(Error::InsufficientNameChanges);
        }

        tide::log::debug!(
            "Creating user alias for '{}' -> '{}', deducting name change",
            old_slug,
            new_slug,
        );

        // Deduct name change token and add user alias for old slug.
        //
        // The "created by" is the user themselves, since
        // they initiatived the rename.
        //
        // We don't verify here because the user row hasn't been
        // updated yet, so we instead run UserAliasService::verify()
        // ourselves at the end of user updating.
        UserAliasService::create_no_verify(
            ctx,
            CreateUserAlias {
                slug: old_slug.clone(),
                target_user_id: user.user_id,
                created_by_user_id: user.user_id,
                bypass_filter,
            },
        )
        .await?;

        model.name_changes_left = Set(user.name_changes_left - 1);
        model.name = Set(new_name);
        model.slug = Set(new_slug);
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

    /// Set the MFA secret fields for a user.
    pub async fn set_mfa_secrets(
        ctx: &ServiceContext<'_>,
        user_id: i64,
        multi_factor_secret: ActiveValue<Option<String>>,
        multi_factor_recovery_codes: ActiveValue<Option<Vec<String>>>,
    ) -> Result<()> {
        tide::log::info!("Setting MFA secret fields for user ID {user_id}");

        let txn = ctx.transaction();
        let model = user::ActiveModel {
            user_id: Set(user_id),
            multi_factor_secret,
            multi_factor_recovery_codes,
            ..Default::default()
        };
        model.update(txn).await?;

        Ok(())
    }

    /// Removes a recovery code from the list provided for a user.
    pub async fn remove_recovery_code(
        ctx: &ServiceContext<'_>,
        user: &UserModel,
        recovery_code: &str,
    ) -> Result<()> {
        let txn = ctx.transaction();
        tide::log::info!("Removing recovery code from user ID {}", user.user_id);

        // Only update if there are recovery codes set for the user
        if let Some(current_codes) = &user.multi_factor_recovery_codes {
            // Clone list, but without the removed code
            let updated_codes = current_codes
                .iter()
                .filter(|code| code.as_str() != recovery_code)
                .map(String::from)
                .collect::<Vec<_>>();

            // Update with the new list
            let model = user::ActiveModel {
                user_id: Set(user.user_id),
                multi_factor_recovery_codes: Set(Some(updated_codes)),
                updated_at: Set(Some(now())),
                ..Default::default()
            };
            model.update(txn).await?;
        }

        Ok(())
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

    async fn run_filter(ctx: &ServiceContext<'_>, name: &str, slug: &str) -> Result<()> {
        tide::log::info!("Checking user data against filters...");

        let filter_matcher =
            FilterService::get_matcher(ctx, FilterClass::Platform, FilterType::User)
                .await?;

        try_join!(
            filter_matcher.verify(ctx, name),
            filter_matcher.verify(ctx, slug),
        )?;

        Ok(())
    }
}
