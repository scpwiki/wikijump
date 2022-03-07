/*
 * services/user/structs.rs
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
use crate::models::users::Model as UserModel;
use chrono::{NaiveDate, NaiveDateTime};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub language: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserOutput {
    pub user_id: i64,
    pub slug: String,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateUser {
    pub username: ProvidedValue<String>,
    pub email: ProvidedValue<String>,
    pub email_verified: ProvidedValue<bool>,
    pub password: ProvidedValue<String>,
    pub multi_factor_secret: ProvidedValue<Option<String>>,
    pub multi_factor_recovery_codes: ProvidedValue<Option<String>>,
    pub remember_token: ProvidedValue<Option<String>>,
    pub language: ProvidedValue<Option<String>>,
    pub karma_points: ProvidedValue<i32>,
    pub karma_level: ProvidedValue<i16>,
    pub real_name: ProvidedValue<Option<String>>,
    pub pronouns: ProvidedValue<Option<String>>,
    pub dob: ProvidedValue<Option<NaiveDate>>,
    pub bio: ProvidedValue<Option<String>>,
    pub about_page: ProvidedValue<Option<String>>,
    pub avatar_path: ProvidedValue<Option<String>>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserIdentityOutput {
    id: i64,
    username: String,
    tinyavatar: Option<String>, // TODO
    karma: u8,
    role: String,
}

impl From<&UserModel> for UserIdentityOutput {
    fn from(user: &UserModel) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            tinyavatar: None, // TODO
            karma: user.karma_level as u8,
            role: String::new(), // TODO
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserInfoOutput {
    #[serde(flatten)]
    identity: UserIdentityOutput,

    about: Option<String>,
    avatar: Option<String>, // TODO
    signature: Option<String>,
    since: Option<NaiveDateTime>,
    last_active: Option<NaiveDateTime>,
}

impl From<&UserModel> for UserInfoOutput {
    fn from(user: &UserModel) -> Self {
        Self {
            identity: UserIdentityOutput::from(user),
            about: user.bio.clone(),
            avatar: user.avatar_path.clone(),
            signature: None, // TODO
            since: user.created_at,
            last_active: user.updated_at,
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserProfileOutput {
    #[serde(flatten)]
    info: UserInfoOutput,

    realname: Option<String>,
    pronouns: Option<String>,
    birthday: Option<NaiveDate>,
    location: Option<String>,
    links: HashMap<String, String>,
}

impl From<&UserModel> for UserProfileOutput {
    fn from(user: &UserModel) -> Self {
        Self {
            info: UserInfoOutput::from(user),
            realname: user.real_name.clone(),
            pronouns: user.pronouns.clone(),
            birthday: user.dob,
            location: None,        // TODO
            links: HashMap::new(), // TODO
        }
    }
}
