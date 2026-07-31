/*
 * services/page_lock/structs.rs
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
use time::OffsetDateTime;

use crate::types::{PageLockType, Reference};

// TODO: Add ip_address to request context
#[derive(Deserialize, Debug, Clone)]
pub struct RemovePageLockInput {
    pub page: Reference<'static>,
    pub ip_address: IpAddr,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CreatePageLockInput {
    pub page: Reference<'static>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub expires_at: Option<OffsetDateTime>,
    #[serde(default)]
    pub from_wikidot: bool,
    pub lock_type: PageLockType,
    pub reason: Option<String>,
    #[serde(default)]
    pub override_existing: bool,
    pub ip_address: IpAddr,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetPageLockHistoryInput {
    pub page: Reference<'static>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CheckLockBypassOutput {
    pub lock_present: bool,
    pub can_edit: bool,
}
