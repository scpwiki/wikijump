/*
 * web/revision_direction.rs
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
use strum_macros::EnumIter;
use std::str::FromStr;

#[derive(
    EnumIter,
    Serialize,
    Deserialize,
    Debug,
    Copy,
    Clone,
    Hash,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
)]
#[serde(rename_all = "camelCase")]
pub enum RevisionDirection {
    /// Retrieves revisions prior (earlier) to this one.
    Before,

    /// Retrieves revisions after (later than) this one.
    After,
}

impl RevisionDirection {
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            RevisionDirection::Before => "before",
            RevisionDirection::After => "after",
        }
    }
}

impl FromStr for RevisionDirection {
    type Err = ServiceError;

    fn from_str(value: &str) -> Result<RevisionDirection, ServiceError> {
        match value {
            "before" => Ok(RevisionDirection::Before),
            "after" => Ok(RevisionDirection::After),
            _ => Err(ServiceError::InvalidEnumValue),
        }
    }
}

/// Ensure `RevisionDirection::name()` produces the same output as serde.
#[test]
fn name_serde() {
    use strum::IntoEnumIterator;

    for variant in RevisionDirection::iter() {
        let output = serde_json::to_string(&variant).expect("Unable to serialize JSON");
        let serde_name: String =
            serde_json::from_str(&output).expect("Unable to deserialize JSON");

        assert_eq!(
            &serde_name,
            variant.name(),
            "Serde name does not match variant name",
        );

        let converted: RevisionDirection = serde_name
            .as_str()
            .parse()
            .expect("Could not convert item");

        assert_eq!(converted, variant, "Converted item does not match variant");
    }
}
