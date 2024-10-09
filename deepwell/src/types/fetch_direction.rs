/*
 * types/fetch_direction.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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
#[serde(rename_all = "kebab-case")]
pub enum FetchDirection {
    /// Retrieves items prior (earlier) to this one.
    Before,

    /// Retrieves items after (later than) this one.
    After,
}

impl FetchDirection {
    #[cfg(test)]
    pub fn name(self) -> &'static str {
        match self {
            FetchDirection::Before => "before",
            FetchDirection::After => "after",
        }
    }
}

impl FromStr for FetchDirection {
    type Err = ServiceError;

    fn from_str(value: &str) -> Result<FetchDirection, ServiceError> {
        match value {
            "before" => Ok(FetchDirection::Before),
            "after" => Ok(FetchDirection::After),
            _ => Err(ServiceError::InvalidEnumValue),
        }
    }
}

/// Ensure `FetchDirection::name()` produces the same output as serde.
#[test]
fn name_serde() {
    use strum::IntoEnumIterator;

    for variant in FetchDirection::iter() {
        let output = serde_json::to_string(&variant).expect("Unable to serialize JSON");
        let serde_name: String =
            serde_json::from_str(&output).expect("Unable to deserialize JSON");

        assert_eq!(
            &serde_name,
            variant.name(),
            "Serde name does not match variant name",
        );

        let converted: FetchDirection =
            serde_name.as_str().parse().expect("Could not convert item");

        assert_eq!(converted, variant, "Converted item does not match variant");
    }
}
