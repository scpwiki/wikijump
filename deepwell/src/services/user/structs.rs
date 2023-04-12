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
use crate::models::alias::Model as AliasModel;
use crate::models::sea_orm_active_enums::UserType;
use crate::models::user::Model as UserModel;
use chrono::NaiveDate;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateUser {
    pub user_type: UserType,
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
pub struct GetUser<'a> {
    pub user: Reference<'a>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetUserOutput {
    #[serde(flatten)]
    pub user: UserModel,
    pub aliases: Vec<AliasModel>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUser<'a> {
    pub user: Reference<'a>,

    #[serde(flatten)]
    pub body: UpdateUserBody,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateUserBody {
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
