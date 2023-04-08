/*
 * services/user/structs.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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
use crate::models::user::Model as UserModel;
use crate::utils::DateTimeWithTimeZone;
use chrono::NaiveDate;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateUser {
    pub name: String,
    pub email: String,
    pub locale: String,
    pub password: String,

    #[serde(default)]
    pub bypass_filter: bool,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserOutput {
    pub user_id: i64,
    pub slug: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetUser {
    pub user: Reference<'static>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateUser {
    pub name: ProvidedValue<String>,
    pub email: ProvidedValue<String>,
    pub email_verified: ProvidedValue<bool>,
    pub password: ProvidedValue<String>,
    pub locale: ProvidedValue<String>,
    pub avatar: ProvidedValue<Option<Vec<u8>>>,
    pub real_name: ProvidedValue<Option<String>>,
    pub gender: ProvidedValue<Option<String>>,
    pub birthday: ProvidedValue<Option<NaiveDate>>,
    pub location: ProvidedValue<Option<String>>,
    pub biography: ProvidedValue<Option<String>>,
    pub user_page: ProvidedValue<Option<String>>,

    #[serde(default)]
    pub bypass_filter: bool,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserIdentityOutput {
    id: i64,
    name: String,
    slug: String,
    tinyavatar: Option<String>, // TODO
    role: String,
}

impl From<&UserModel> for UserIdentityOutput {
    fn from(user: &UserModel) -> Self {
        Self {
            id: user.user_id,
            name: user.name.clone(),
            slug: user.slug.clone(),
            tinyavatar: None,    // TODO
            role: String::new(), // TODO
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserInfoOutput {
    #[serde(flatten)]
    identity: UserIdentityOutput,

    biography: Option<String>,
    avatar: Option<String>, // TODO
    since: DateTimeWithTimeZone,
    last_active: Option<DateTimeWithTimeZone>,
}

impl From<&UserModel> for UserInfoOutput {
    fn from(user: &UserModel) -> Self {
        Self {
            identity: UserIdentityOutput::from(user),
            biography: user.biography.clone(),
            avatar: None, // TODO
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

    real_name: Option<String>,
    gender: Option<String>,
    birthday: Option<NaiveDate>,
    location: Option<String>,
    links: HashMap<String, String>,
}

impl From<&UserModel> for UserProfileOutput {
    fn from(user: &UserModel) -> Self {
        Self {
            info: UserInfoOutput::from(user),
            real_name: user.real_name.clone(),
            gender: user.gender.clone(),
            birthday: user.birthday,
            location: None,        // TODO
            links: HashMap::new(), // TODO
        }
    }
}
