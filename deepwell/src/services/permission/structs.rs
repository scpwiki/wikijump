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

use crate::types::{Action, Permission, Resource};

#[derive(Serialize, Debug, Clone)]
pub struct DecoratedPermission<'a> {
    pub permission: Permission<'a>,
    pub active: bool,
    pub addable: bool,
    pub removable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PermissionTarget {
    Site,
    Page { page_id: i64, category_id: i64 },
    Lock,
}

impl PermissionTarget {
    pub fn resource_type(&self) -> Resource {
        match self {
            PermissionTarget::Site => Resource::Site,
            PermissionTarget::Lock => Resource::PageLock,
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
