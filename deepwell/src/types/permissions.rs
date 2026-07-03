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
pub struct Permission<'a> {
    pub resource_type: Resource,
    pub resource_category: Option<Reference<'a>>,
    pub action: Action,
}

impl FromStr for Permission<'static> {
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
            resource_type: Resource::from_str(resource).map_err(|_| {
                PermissionParseError {
                    message: format!("invalid resource type: '{}'", resource),
                }
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
        impl Permission<'static> {
            pub const ALL: &[Permission<'static>] = &[
                $($(
                    Permission { resource_type: Resource::$resource, resource_category: None, action: Action::$action },
                )+)+
            ];

            pub const fn new(resource: Resource, action: Action) -> Option<Permission<'static>> {
                match (resource, action) {
                    $($(
                        (Resource::$resource, Action::$action) => {
                            Some(Permission { resource_type: Resource::$resource, resource_category: None, action })
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
    Page => [View, Edit, BypassLock, Create, Delete, Rename],
    Role => [View, Edit, Assign],
    Site => [View, Edit],
}

#[test]
fn parses_permission_without_category() {
    let permission: Permission = "page:view".parse().unwrap();

    assert_eq!(permission.resource_type, Resource::Page);
    assert_eq!(permission.resource_category, None);
    assert_eq!(permission.action, Action::View);
}

#[test]
fn parses_permission_with_numeric_category() {
    let permission: Permission = "page:123:edit".parse().unwrap();

    assert_eq!(permission.resource_type, Resource::Page);
    assert_eq!(permission.resource_category, Some(Reference::Id(123)));
    assert_eq!(permission.action, Action::Edit);
}

#[test]
fn parses_permission_with_slug_category() {
    let permission: Permission = "page:staff:create".parse().unwrap();

    assert_eq!(permission.resource_type, Resource::Page);
    assert!(
        matches!(permission.resource_category, Some(Reference::Slug(ref slug)) if slug == "staff"),
    );
    assert_eq!(permission.action, Action::Create);
}

#[test]
fn rejects_invalid_permission_parts() {
    let format_error = "page:view:extra:field".parse::<Permission>().unwrap_err();
    assert!(
        format_error
            .to_string()
            .contains("invalid permission format")
    );

    let resource_error = "forum:view".parse::<Permission>().unwrap_err();
    assert!(resource_error.to_string().contains("invalid resource type"));

    let action_error = "page:publish".parse::<Permission>().unwrap_err();
    assert!(action_error.to_string().contains("invalid action type"));
}

#[test]
fn permission_catalog_lists_valid_resource_actions() {
    assert_eq!(Permission::ALL.len(), 11);
    assert!(Permission::ALL.contains(&Permission {
        resource_type: Resource::Page,
        resource_category: None,
        action: Action::BypassLock,
    }));
    assert_eq!(
        Permission::new(Resource::Role, Action::Assign),
        Some(Permission {
            resource_type: Resource::Role,
            resource_category: None,
            action: Action::Assign,
        }),
    );
    assert_eq!(Permission::new(Resource::Site, Action::Assign), None);
}
