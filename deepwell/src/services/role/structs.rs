/*
 * services/role/structs.rs
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
use crate::types::{Maybe, Permission, Reference};
use std::net::IpAddr;
use time::OffsetDateTime;

// Differentiate public facing input structs to mask internal fields that should not be set by users.
#[derive(Deserialize, Debug, Clone)]
pub struct CreateRoleInput {
    pub site_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub parent_role_id: i64,
    pub creating_user_id: i64,
    pub ip_address: IpAddr,
}

#[derive(Deserialize, Debug, Clone)]
pub struct InternalCreateRoleInput {
    pub site_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub is_virtual: bool,
    pub parent_role_id: Option<i64>,
    pub creating_user_id: i64,
    pub ip_address: IpAddr,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetRoleInput<'a> {
    pub site_id: i64,
    pub role_reference: Reference<'a>,

    #[serde(default)]
    pub acting_user_id: Option<i64>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct UpdateRoleInput {
    pub site_id: i64,
    pub role_id: i64,
    pub name: Maybe<String>,
    pub description: Maybe<String>,
    pub updating_user_id: i64,
    pub ip_address: IpAddr,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateRolePermissionsInput<'a> {
    pub site_id: i64,
    pub role_reference: Reference<'a>,
    pub new_permissions: Vec<Permission<'a>>,
    pub cascade_removals: bool,
    pub updating_user_id: i64,
    pub ip_address: IpAddr,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DeleteRoleInput {
    pub site_id: i64,
    pub role_id: i64,
    pub deleting_user_id: i64,
    pub reparent_children: bool,
    pub ip_address: IpAddr,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ListSiteRolesInput {
    pub site_id: i64,

    #[serde(default)]
    pub acting_user_id: Option<i64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ReparentRoleInput {
    pub site_id: i64,
    pub role_id: i64,
    pub new_parent_id: i64,
    pub reparenting_user_id: i64,
    pub ip_address: IpAddr,
}

#[derive(Deserialize, Debug, Clone)]
pub struct InternalReparentRoleInput {
    pub site_id: i64,
    pub role_id: i64,
    pub new_parent_id: Option<i64>,
    pub reparenting_user_id: i64,
    pub ip_address: IpAddr,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GrantUserRoleInput {
    pub user_id: i64,
    pub role_id: i64,
    pub site_id: i64,
    pub assigning_user_id: i64,
    #[serde(with = "time::serde::rfc3339::option")]
    pub expires_at: Option<OffsetDateTime>,
    pub ip_address: IpAddr,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RevokeUserRoleInput {
    pub user_id: i64,
    pub role_id: i64,
    pub site_id: i64,
    pub revoking_user_id: i64,
    pub ip_address: IpAddr,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetUserRolesInput<'a> {
    pub site_id: i64,
    pub user_id: Option<i64>,
    pub page_reference: Option<Reference<'a>>,

    #[serde(default)]
    pub acting_user_id: Option<i64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetRolePermissionsInput<'a> {
    pub site_id: i64,
    pub role_reference: Reference<'a>,

    #[serde(default)]
    pub acting_user_id: Option<i64>,

    #[serde(default)] // Defaults to false to avoid expensive computation
    pub human_readable_categories: bool,
}
