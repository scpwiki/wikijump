/*
 * services/user/mod.rs
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
use chrono::NaiveDate;
use wikidot_normalize::normalize;

// Helper structs

#[derive(Deserialize, Debug)]
pub struct CreateUser {
    username: String,
    email: String,
    password: String,
    language: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct UpdateUser {
    username: Maybe<String>,
    email: Maybe<String>,
    email_verified: Maybe<bool>,
    password: Maybe<String>,
    multi_factor_secret: Maybe<Option<String>>,
    multi_factor_recovery_codes: Maybe<Option<String>>,
    remember_token: Maybe<Option<String>>,
    language: Maybe<Option<String>>,
    karma_points: Maybe<i32>,
    karma_level: Maybe<i16>,
    real_name: Maybe<Option<String>>,
    pronouns: Maybe<Option<String>>,
    dob: Maybe<Option<NaiveDate>>,
    bio: Maybe<Option<String>>,
    about_page: Maybe<Option<String>>,
    avatar_path: Maybe<Option<String>>,
}

// Service

#[derive(Debug)]
pub struct UserService(ApiServerState);

impl_service_constructor!(UserService);

impl UserService {
    pub async fn create(&self, input: &CreateUser) -> Result<UserModel> {
        let db = &self.0.database;
        let slug = get_user_slug(&input.username);

        // Check for conflicts
        /*
        let result = User::find()
            .filter(
                Condition::any()
                    .add(users::Column::Username.eq(input.username.as_str()))
                    .add(users::Column::Email.eq(input.email.as_str()))
                    .add(users::Column::Slug.eq(slug.as_str())),
            )
            .one(db)
            .await?;

        if result.is_some() {
            return Err(Error::Conflict);
        }

        // Insert new model
        let model = users::ActiveModel {
            username: Set(input.username),
            slug: Set(slug),
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
            created_at: Set(Some(now())),
            updated_at: Set(None),
            deleted_at: Set(None),
            ..Default::default()
        };

        let user = User::insert(model).exec(&db).await?;
        Ok(user)
            */
        todo!()
    }

    pub async fn get_optional(
        &self,
        reference: ItemReference<'_>,
    ) -> Result<Option<UserModel>> {
        let db = &self.0.database;

        let user = match reference {
            ItemReference::Id(id) => User::find_by_id(id).one(db).await?,
            ItemReference::Slug(slug) => {
                User::find()
                    .filter(users::Column::Slug.eq(slug))
                    .one(db)
                    .await?
            }
        };

        Ok(user)
    }

    pub async fn get(&self, reference: ItemReference<'_>) -> Result<UserModel> {
        match self.get_optional(reference).await? {
            Some(user) => Ok(user),
            None => Err(Error::NotFound),
        }
    }

    pub async fn update(
        &self,
        reference: ItemReference<'_>,
        input: &UpdateUser,
    ) -> Result<UserModel> {
        let db = &self.0.database;
        let model = self.get(reference).await?;
        let mut user: users::ActiveModel = model.into();

        /*
        // Add each field
        if let Some(username) = input.username {
            let slug = get_user_slug(&username);
            user.username = Set(username);
            user.username_changes = Set(user.username_changes.unwrap() + 1);
            user.slug = Set(slug);
        }

        if let Some(email) = input.email {
            user.email = Set(email);
        }

        if let Some(email_verified) = input.email_verified {
            let value = if email_verified { Some(now()) } else { None };
            user.email_verified_at = Set(value);
        }

        if let Some(password) = input.password {
            user.password = Set(password);
        }

        if let Some(multi_factor_secret) = input.multi_factor_secret {
            user.multi_factor_secret = Set(multi_factor_secret);
        }

        if let Some(multi_factor_recovery_codes) = input.multi_factor_recovery_codes {
            user.multi_factor_recovery_codes = Set(multi_factor_recovery_codes);
        }

        if let Some(remember_token) = input.remember_token {
            user.remember_token = Set(remember_token);
        }

        if let Some(language) = input.language {
            user.language = Set(language);
        }

        if let Some(karma_points) = input.karma_points {
            user.karma_points = Set(karma_points);
        }

        if let Some(karma_level) = input.karma_level {
            user.karma_level = Set(karma_level);
        }

        if let Some(real_name) = input.real_name {
            user.real_name = Set(real_name);
        }

        if let Some(pronouns) = input.pronouns {
            user.pronouns = Set(pronouns);
        }

        if let Some(dob) = input.dob {
            user.dob = Set(dob);
        }

        if let Some(bio) = input.bio {
            user.bio = Set(bio);
        }

        if let Some(about_page) = input.about_page {
            user.about_page = Set(about_page);
        }

        if let Some(avatar_path) = input.avatar_path {
            user.avatar_path = Set(avatar_path);
        }

        // Set update flag
        user.updated_at = Set(Some(now()));

        // Update and return
        user.update(db).await?;
        Ok(model)
            */
        todo!()
    }

    pub async fn delete(&self, reference: ItemReference<'_>) -> Result<UserModel> {
        let db = &self.0.database;
        let model = self.get(reference).await?;
        todo!()
        /*
        let mut user: users::ActiveModel = model.into();

        // Set deletion flag
        user.updated_at = Set(Some(now()));
        user.deleted_at = Set(Some(now()));

        // Update and return
        user.update(db).await?;
        Ok(model)
        */
    }
}

// Helpers

fn get_user_slug(username: &str) -> String {
    let mut slug = str!(username);
    replace_in_place(&mut slug, ":", "-");
    normalize(&mut slug);
    slug
}
