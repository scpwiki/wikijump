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
use crate::services::UserAliasService;
use crate::utils::get_user_slug;

// TODO make this configurable
const DEFAULT_NAME_CHANGES: i16 = 3;

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
        reference: Reference<'_>,
    ) -> Result<Option<UserModel>> {
        let txn = ctx.transaction();
        let user = match reference {
            // Get directly from ID
            Reference::Id(id) => User::find_by_id(id).one(txn).await?,

            // Since a slug can be an alias, check for a redirect
            Reference::Slug(slug) => {
                UserAliasService::get_redirect_optional(ctx, slug).await?
            }
        };

        Ok(user)
    }

    pub async fn get(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
    ) -> Result<UserModel> {
        Self::get_optional(ctx, reference)
            .await?
            .ok_or(Error::NotFound)
    }

    pub async fn update(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
        input: UpdateUser,
    ) -> Result<()> {
        let txn = ctx.transaction();
        let model = Self::get(ctx, reference).await?;
        let mut user: user::ActiveModel = model.into();

        // Add each field
        if let ProvidedValue::Set(name) = input.name {
            // TODO: add old alias
            // TODO: check for conflicts

            let slug = get_user_slug(&name);
            user.name = Set(name);
            user.name_changes_left = Set(user.name_changes_left.unwrap() - 1); // TODO
            user.slug = Set(slug);
        }

        if let ProvidedValue::Set(email) = input.email {
            user.email = Set(email);
        }

        if let ProvidedValue::Set(email_verified) = input.email_verified {
            user.email_verified_at = Set(if email_verified { Some(now()) } else { None });
        }

        if let ProvidedValue::Set(password) = input.password {
            user.password = Set(hash_password(password));
        }

        if let ProvidedValue::Set(locale) = input.locale {
            user.locale = Set(locale);
        }

        if let ProvidedValue::Set(display_name) = input.display_name {
            user.display_name = Set(display_name);
        }

        if let ProvidedValue::Set(gender) = input.gender {
            user.gender = Set(gender);
        }

        if let ProvidedValue::Set(birthday) = input.birthday {
            user.birthday = Set(birthday);
        }

        if let ProvidedValue::Set(biography) = input.biography {
            user.biography = Set(biography);
        }

        if let ProvidedValue::Set(user_page) = input.user_page {
            user.user_page = Set(user_page);
        }

        if let ProvidedValue::Set(avatar) = input.avatar {
            // TODO store avatar in S3
            user.avatar_s3_hash = Set(avatar);
        }

        // Set update flag
        user.updated_at = Set(Some(now()));

        // Update and return
        user.update(txn).await?;
        Ok(())
    }

    pub async fn delete(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
    ) -> Result<UserModel> {
        let txn = ctx.transaction();
        let model = Self::get(ctx, reference).await?;
        let mut user: user::ActiveModel = model.clone().into();

        // Set deletion flag
        user.deleted_at = Set(Some(now()));

        // Update and return
        user.update(txn).await?;
        Ok(model)
    }
}

// Helpers

// TEMP helper, so it's easier to replace when implemented
// TODO replace
fn hash_password(value: String) -> String {
    // TODO Securely hash password
    value
}
