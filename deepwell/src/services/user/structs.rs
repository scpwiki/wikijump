/*
 * services/user/structs.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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
use crate::types::Bytes;
use time::Date;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateUser {
    pub user_type: UserType,
    pub name: String,
    pub email: String,
    pub locales: Vec<String>,
    pub password: String,

    #[serde(default)]
    pub bypass_filter: bool,
    #[serde(default)]
    pub bypass_email_verification: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct CreateUserOutput {
    pub user_id: i64,
    pub slug: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetUser<'a> {
    pub user: Reference<'a>,
}

#[derive(Serialize, Debug, Clone)]
pub struct GetUserOutput {
    #[serde(flatten)]
    pub user: UserModel,
    pub aliases: Vec<AliasModel>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateUser<'a> {
    pub user: Reference<'a>,

    #[serde(flatten)]
    pub body: UpdateUserBody,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct UpdateUserBody {
    pub name: ProvidedValue<String>,
    pub email: ProvidedValue<String>,
    pub email_verified: ProvidedValue<bool>,
    pub password: ProvidedValue<String>,
    pub locales: ProvidedValue<Vec<String>>,
    pub avatar_uploaded_blob_id: ProvidedValue<Option<String>>,
    pub real_name: ProvidedValue<Option<String>>,
    pub gender: ProvidedValue<Option<String>>,
    pub birthday: ProvidedValue<Option<Date>>,
    pub location: ProvidedValue<Option<String>>,
    pub biography: ProvidedValue<Option<String>>,
    pub user_page: ProvidedValue<Option<String>>,

    #[serde(default)]
    pub bypass_filter: bool,
}
