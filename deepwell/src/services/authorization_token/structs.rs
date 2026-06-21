/*
 * services/authorization_token/structs.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
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

use std::net::IpAddr;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AuthorizedObject {
    /// Authorizes the creation of a site.
    Site,

    /// Authorizes the creation of a new user (even if registrations are disabled).
    User,

    /// Authorizes the creation of a bot user.
    BotUser,
}

impl AuthorizedObject {
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            AuthorizedObject::Site => "site",
            AuthorizedObject::User => "user",
            AuthorizedObject::BotUser => "bot-user",
        }
    }

    #[inline]
    pub fn code(self) -> char {
        match self {
            AuthorizedObject::Site => 'S',
            AuthorizedObject::User => 'U',
            AuthorizedObject::BotUser => 'B',
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct CreateAuthorizationToken {
    pub r#type: AuthorizedObject,
    pub description: String,
    pub creating_user_id: i64,
    pub ip_address: IpAddr,
}
