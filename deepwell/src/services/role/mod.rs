/*
 * services/role/mod.rs
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

use strum_macros::{Display, EnumString, IntoStaticStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display, IntoStaticStr)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
#[allow(dead_code)]
pub enum SystemRole {
    Root,
    Member,
    Guest,
    Registered,
    Anonymous,
    Everyone,
    PageAuthor,
    Banned,
}

mod service;
mod structs;

pub use self::service::RoleService;
pub use self::structs::{
    CreateRoleInput, DeleteRoleInput, GetRoleInput, GetRolePermissionsInput,
    GetUserRolesInput, GrantUserRoleInput, InternalCreateRoleInput,
    InternalReparentRoleInput, ListSiteRolesInput, ReparentRoleInput,
    RevokeUserRoleInput, UpdateRoleInput, UpdateRolePermissionsInput,
};
