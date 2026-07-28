/*
 * constants.rs
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

#![allow(dead_code)]

use std::net::{IpAddr, Ipv6Addr};

// See seeder data for these values
pub const ADMIN_USER_ID: i64 = -1;
pub const SYSTEM_USER_ID: i64 = -2;
pub const ANONYMOUS_USER_ID: i64 = -3;
pub const UNKNOWN_USER_ID: i64 = -4;
pub const SAMPLE_USER_ID: i64 = -5;

/// The IP address to use in audit log entries for actions performed by the system.
pub const SYSTEM_IP_ADDRESS: IpAddr = IpAddr::V6(Ipv6Addr::LOCALHOST);
