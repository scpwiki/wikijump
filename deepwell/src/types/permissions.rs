/*
 * services/permission/permissions
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
use crate::types::{Action, Reference, Resource};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug)]
pub struct PermissionParseError {
    pub message: String,
}

impl std::fmt::Display for PermissionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse permission error: {}", self.message)
    }
}

impl std::error::Error for PermissionParseError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Permission {
    pub resource: Resource,
    pub resource_category: Option<Reference<'static>>,
    pub action: Action,
}

impl FromStr for Permission {
    type Err = PermissionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(':').collect::<Vec<_>>();
        let (resource, resource_category, action) = match parts.as_slice() {
            [resource, action] => (resource, None, action),
            [resource, resource_category, action] => {
                // Try to parse as ID first, fall back to treating as slug
                let reference = resource_category
                    .parse::<i64>()
                    .map(Reference::Id)
                    .unwrap_or_else(|_| {
                        Reference::Slug(Cow::Owned(resource_category.to_string()))
                    });
                (resource, Some(reference), action)
            }
            _ => {
                return Err(PermissionParseError {
                    message: format!("invalid permission format: '{}'", s),
                });
            }
        };

        Ok(Self {
            resource: Resource::from_str(resource).map_err(|_| PermissionParseError {
                message: format!("invalid resource type: '{}'", resource),
            })?,
            resource_category,
            action: Action::from_str(action).map_err(|_| PermissionParseError {
                message: format!("invalid action type: '{}'", action),
            })?,
        })
    }
}

/// Macro for generating a list of all valid Permission types, for iteration/validation purposes.
macro_rules! define_permission_types {
    ( $( $resource:ident => [ $($action:ident),+ $(,)? ] ),+ $(,)? ) => {
        impl Permission {
            pub const ALL: &[Permission] = &[
                $($(
                    Permission { resource: Resource::$resource, resource_category: None, action: Action::$action },
                )+)+
            ];

            pub const fn new(resource: Resource, action: Action) -> Option<Permission> {
                match (resource, action) {
                    $($(
                        (Resource::$resource, Action::$action) => {
                            Some(Permission { resource, resource_category: None, action })
                        }
                    )+)+
                    _ => None,
                }
            }
        }
    };
}

// Define all valid permission types.
define_permission_types! {
    Page => [View, Edit, Create, Delete, Rename],
    Role => [View, Edit, Assign],
    Site => [View, Edit],
}

#[derive(Debug, Clone, Serialize)]
pub struct UserPermissions {
    permission_map: HashMap<Permission, bool>,
}

impl UserPermissions {
    pub fn new(permission_map: Option<HashMap<Permission, bool>>) -> Self {
        Self {
            permission_map: permission_map.unwrap_or_default(),
        }
    }

    pub fn has(&self, permission: &Permission) -> bool {
        *self.permission_map.get(permission).unwrap_or_else(|| {
            let default_fallback = Permission {
                resource: permission.resource,
                resource_category: None,
                action: permission.action,
            };
            self.permission_map.get(&default_fallback).unwrap_or(&false)
        })
    }
}
