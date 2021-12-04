/*
 * methods/user/types.rs
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

use chrono::{NaiveDate, NaiveDateTime};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum UserDetails {
    Identity,
    Info,
    Profile,
}

impl Default for UserDetails {
    #[inline]
    fn default() -> Self {
        UserDetails::Identity
    }
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum UserResponse {
    Identity(UserIdentity),
    Info(UserInfo),
    Profile(UserProfile),
}

#[derive(Serialize, Debug)]
pub struct UserIdentity {
    id: u64,
    username: String,
    tinyavatar: String,
    karma: u8,
    role: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    id: u64,
    username: String,
    tinyavatar: String,
    karma: u8,
    role: String,
    about: String,
    avatar: String,
    signature: String,
    since: NaiveDateTime,
    last_active: NaiveDateTime,
    blocked: bool,
}

#[derive(Serialize, Debug)]
pub struct UserProfile {
    id: u64,
    username: String,
    tinyavatar: String,
    karma: u8,
    role: String,
    about: String,
    avatar: String,
    signature: String,
    since: NaiveDateTime,
    last_active: NaiveDateTime,
    blocked: bool,
    realname: String,
    pronouns: Option<String>,
    birthday: Option<NaiveDate>,
    location: Option<String>,
    links: HashMap<String, String>,
}
