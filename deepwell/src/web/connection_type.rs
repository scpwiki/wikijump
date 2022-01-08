/*
 * web/connection_type.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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

use crate::services::Error as ServiceError;
use std::convert::TryFrom;
use strum_macros::EnumIter;

#[derive(EnumIter, Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ConnectionType {
    IncludeMessy,
    IncludeElements,
    Component,
    Link,
    Redirect,
}

impl ConnectionType {
    pub fn name(self) -> &'static str {
        match self {
            ConnectionType::IncludeMessy => "include-messy",
            ConnectionType::IncludeElements => "include-elements",
            ConnectionType::Component => "component",
            ConnectionType::Link => "link",
            ConnectionType::Redirect => "redirect",
        }
    }
}

impl TryFrom<&'_ str> for ConnectionType {
    type Error = ServiceError;

    fn try_from(value: &'_ str) -> Result<ConnectionType, ServiceError> {
        match value {
            "include-messy" => Ok(ConnectionType::IncludeMessy),
            "include-elements" => Ok(ConnectionType::IncludeElements),
            "component" => Ok(ConnectionType::Component),
            "link" => Ok(ConnectionType::Link),
            "redirect" => Ok(ConnectionType::Redirect),
            _ => Err(ServiceError::InvalidEnumValue),
        }
    }
}

/// Ensure `ConnectionType::name()` produces the same output as serde.
#[test]
fn name_serde() {
    use strum::IntoEnumIterator;

    for variant in ConnectionType::iter() {
        let output = serde_json::to_string(&variant).expect("Unable to serialize JSON");
        let serde_name: String =
            serde_json::from_str(&output).expect("Unable to deserialize JSON");

        assert_eq!(
            &serde_name,
            variant.name(),
            "Serde name does not match variant name",
        );

        let converted: ConnectionType = serde_name.as_str().try_into().expect("Could not convert item");
        assert_eq!(
            converted,
            variant,
            "Converted item does not match variant",
        );
    }
}
