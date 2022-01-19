/*
 * services/user/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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
use crate::models::users::{self, Entity as User, Model as UserModel};
use crate::utils::replace_in_place;
use wikidot_normalize::normalize;

#[derive(Debug)]
pub struct UserService;

impl UserService {
    pub async fn create(
        ctx: &ServiceContext<'_>,
        input: CreateUser,
    ) -> Result<CreateUserOutput> {
        let txn = ctx.transaction();
        let slug = get_user_slug(&input.username);

        // Check for conflicts
        let result = User::find()
            .filter(
                Condition::all()
                    .add(
                        Condition::any()
                            .add(users::Column::Username.eq(input.username.as_str()))
                            .add(users::Column::Email.eq(input.email.as_str()))
                            .add(users::Column::Slug.eq(slug.as_str())),
                    )
                    .add(users::Column::DeletedAt.is_null()),
            )
            .one(txn)
            .await?;

        if result.is_some() {
            return Err(Error::Conflict);
        }

        // Insert new model
        let user = users::ActiveModel {
            username: Set(input.username),
            slug: Set(slug.clone()),
            email: Set(input.email),
            email_verified_at: Set(None),
            password: Set(input.password),
            multi_factor_secret: Set(None),
            multi_factor_recovery_codes: Set(None),
            remember_token: Set(None),
            language: Set(input.language),
            karma_points: Set(0),
            karma_level: Set(0),
            real_name: Set(None),
            pronouns: Set(None),
            dob: Set(None),
            bio: Set(None),
            about_page: Set(None),
            avatar_path: Set(None),
            created_at: Set(Some(now_naive())),
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
            Reference::Id(id) => User::find_by_id(id).one(txn).await?,
            Reference::Slug(slug) => {
                User::find()
                    .filter(
                        Condition::all()
                            .add(users::Column::Slug.eq(slug))
                            .add(users::Column::DeletedAt.is_null()),
                    )
                    .one(txn)
                    .await?
            }
        };

        Ok(user)
    }

    pub async fn get(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
    ) -> Result<UserModel> {
        match Self::get_optional(ctx, reference).await? {
            Some(user) => Ok(user),
            None => Err(Error::NotFound),
        }
    }

    pub async fn update(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
        input: UpdateUser,
    ) -> Result<()> {
        let txn = ctx.transaction();
        let model = Self::get(ctx, reference).await?;
        let mut user: users::ActiveModel = model.clone().into();

        // Add each field
        if let ProvidedValue::Set(username) = input.username {
            // TODO: add old alias
            // TODO: check for conflicts

            let slug = get_user_slug(&username);
            user.username = Set(username);
            user.username_changes = Set(user.username_changes.unwrap() + 1);
            user.slug = Set(slug);
        }

        if let ProvidedValue::Set(email) = input.email {
            user.email = Set(email);
        }

        if let ProvidedValue::Set(email_verified) = input.email_verified {
            let value = if email_verified {
                Some(now_naive())
            } else {
                None
            };

            user.email_verified_at = Set(value);
        }

        if let ProvidedValue::Set(password) = input.password {
            user.password = Set(password);
        }

        if let ProvidedValue::Set(multi_factor_secret) = input.multi_factor_secret {
            user.multi_factor_secret = Set(multi_factor_secret);
        }

        if let ProvidedValue::Set(multi_factor_recovery_codes) =
            input.multi_factor_recovery_codes
        {
            user.multi_factor_recovery_codes = Set(multi_factor_recovery_codes);
        }

        if let ProvidedValue::Set(remember_token) = input.remember_token {
            user.remember_token = Set(remember_token);
        }

        if let ProvidedValue::Set(language) = input.language {
            user.language = Set(language);
        }

        if let ProvidedValue::Set(karma_points) = input.karma_points {
            user.karma_points = Set(karma_points);
        }

        if let ProvidedValue::Set(karma_level) = input.karma_level {
            user.karma_level = Set(karma_level);
        }

        if let ProvidedValue::Set(real_name) = input.real_name {
            user.real_name = Set(real_name);
        }

        if let ProvidedValue::Set(pronouns) = input.pronouns {
            user.pronouns = Set(pronouns);
        }

        if let ProvidedValue::Set(dob) = input.dob {
            user.dob = Set(dob);
        }

        if let ProvidedValue::Set(bio) = input.bio {
            user.bio = Set(bio);
        }

        if let ProvidedValue::Set(about_page) = input.about_page {
            user.about_page = Set(about_page);
        }

        if let ProvidedValue::Set(avatar_path) = input.avatar_path {
            user.avatar_path = Set(avatar_path);
        }

        // Set update flag
        // TODO update to add TZ
        user.updated_at = Set(Some(now_naive()));

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
        let mut user: users::ActiveModel = model.clone().into();

        // Set deletion flag
        // TODO update to add TZ
        user.deleted_at = Set(Some(now_naive()));

        // Update and return
        user.update(txn).await?;
        Ok(model)
    }
}

// Helpers

fn get_user_slug(username: &str) -> String {
    let mut slug = str!(username);
    replace_in_place(&mut slug, ":", "-");
    normalize(&mut slug);
    slug
}
