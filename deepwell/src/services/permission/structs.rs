/*
 * services/permission/struct.rs
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

use crate::types::{Action, Permission, Reference, Resource};

#[derive(Deserialize, Debug, Clone)]
pub struct PermissionInput<'a> {
    pub resource_type: Resource,
    pub resource_category: Option<Reference<'a>>,
    pub action: Action,
}

#[derive(Serialize, Debug, Clone)]
pub struct DecoratedPermission {
    pub permission: Permission,
    pub active: bool,
    pub addable: bool,
    pub removable: bool,
}

/// Context for permission checks.
///
/// Contains all information needed to evaluate whether a user can perform
/// an action on a resource.
#[derive(Debug, Clone)]
pub struct CheckPermissionContext<'a> {
    pub user_id: Option<i64>,
    pub site_id: i64,
    pub page_reference: Option<Reference<'a>>,
}

#[derive(Debug, Clone)]
pub struct PrefetchPermissionsInput<'a> {
    pub session_token: Option<String>,
    pub site_id: Option<i64>,
    pub page_reference: Option<Reference<'a>>,
}
