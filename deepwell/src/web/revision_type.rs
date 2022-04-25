/*
 * web/revision_type.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2022 Wikijump Team
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
use std::str::FromStr;
use strum_macros::EnumIter;

#[derive(EnumIter, Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RevisionType {
    /// This is a regular revision.
    Regular,

    /// This revision created the page.
    Create,

    /// This revision deleted the page.
    Delete,

    /// This revision undeleted (restored) the page.
    Undelete,
}

impl RevisionType {
    pub fn name(self) -> &'static str {
        match self {
            RevisionType::Regular => "regular",
            RevisionType::Create => "create",
            RevisionType::Delete => "delete",
            RevisionType::Undelete => "undelete",
        }
    }
}

impl FromStr for RevisionType {
    type Err = ServiceError;

    fn from_str(value: &str) -> Result<RevisionType, ServiceError> {
        match value {
            "regular" => Ok(RevisionType::Regular),
            "create" => Ok(RevisionType::Create),
            "delete" => Ok(RevisionType::Delete),
            "undelete" => Ok(RevisionType::Undelete),
            _ => Err(ServiceError::InvalidEnumValue),
        }
    }
}

/// Ensure `RevisionType::name()` produces the same output as serde.
#[test]
fn name_serde() {
    use strum::IntoEnumIterator;

    for variant in RevisionType::iter() {
        let output = serde_json::to_string(&variant).expect("Unable to serialize JSON");
        let serde_name: String =
            serde_json::from_str(&output).expect("Unable to deserialize JSON");

        assert_eq!(
            &serde_name,
            variant.name(),
            "Serde name does not match variant name",
        );

        let converted: RevisionDirection =
            serde_name.as_str().parse().expect("Could not convert item");

        assert_eq!(converted, variant, "Converted item does not match variant");
    }
}
