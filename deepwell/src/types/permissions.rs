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
use std::{borrow::Cow, str::FromStr};

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
