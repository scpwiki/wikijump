/*
 * services/session/struct.rs
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

use crate::types::UserPermissions;
use crate::{models::session::Model as SessionModel, types::Reference};
use std::net::IpAddr;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateSession {
    pub user_id: i64,
    pub ip_address: IpAddr,
    pub user_agent: String,
    pub restricted: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RenewSession {
    pub old_session_token: String,
    pub user_id: i64,
    pub ip_address: IpAddr,
    pub user_agent: String,
}

pub type GetOtherSessions = InvalidateOtherSessions;

#[derive(Serialize, Debug, Clone)]
pub struct GetOtherSessionsOutput {
    pub current: SessionModel,
    pub others: Vec<SessionModel>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct InvalidateOtherSessions {
    pub session_token: String,
    pub user_id: i64,
}

#[derive(Debug, Clone)]
pub struct PrefetchSessionInput<'a> {
    pub session_token: Option<String>,
    pub site_id: Option<i64>,
    pub page_reference: Option<Reference<'a>>,
}

#[derive(Serialize, Debug, Clone)]
pub struct UserSession {
    pub session: Option<SessionModel>,
    pub user_id: Option<i64>,
    pub user_permissions: UserPermissions,
}
