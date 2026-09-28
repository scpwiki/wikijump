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

#[derive(Serialize, Debug, Clone)]
pub struct DecoratedPermission<'a> {
    pub permission: Permission<'a>,
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
    pub resource_type: Resource,
    pub resource_reference: Option<Reference<'a>>,
}

#[derive(Deserialize, Debug, Clone)]
pub enum PermissionTarget<'a> {
    Site,
    Page {
        page_ref: Reference<'a>,
        category_id: i64,
    },
}

impl<'a> PermissionTarget<'a> {
    pub fn resource_type(&self) -> Resource {
        match self {
            PermissionTarget::Site { .. } => Resource::Site,
            PermissionTarget::Page { .. } => Resource::Page,
        }
    }

    pub fn category_id(&self) -> Option<i64> {
        match self {
            PermissionTarget::Page { category_id, .. } => Some(*category_id),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CheckPermissionInput<'a> {
    pub user_id: Option<i64>,
    pub site_id: Option<i64>,
    pub action: Action,
    pub target: PermissionTarget<'a>,
}
