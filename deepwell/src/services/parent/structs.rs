/*
 * services/parent/structs.rs
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

use crate::error::prelude::{EnumConversionError, StdResult};
use crate::types::Reference;
use std::str::FromStr;

#[derive(Deserialize, Debug, Clone)]
pub struct ParentDescription<'a> {
    pub site_id: i64,
    pub parent: Reference<'a>,
    pub child: Reference<'a>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateParents<'a> {
    pub site_id: i64,
    pub child: Reference<'a>,
    #[serde(default)]
    pub user_id: Option<i64>,
    pub add: Option<Vec<Reference<'a>>>,
    pub remove: Option<Vec<Reference<'a>>>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ParentalRelationshipType {
    #[serde(rename = "parents")]
    Parent,

    #[serde(rename = "children")]
    Child,
}

impl ParentalRelationshipType {
    pub fn name(self) -> &'static str {
        match self {
            ParentalRelationshipType::Parent => "parents",
            ParentalRelationshipType::Child => "children",
        }
    }
}

impl FromStr for ParentalRelationshipType {
    type Err = EnumConversionError;

    fn from_str(value: &str) -> StdResult<ParentalRelationshipType, EnumConversionError> {
        match value {
            "parents" => Ok(ParentalRelationshipType::Parent),
            "children" => Ok(ParentalRelationshipType::Child),
            _ => Err(EnumConversionError::new("ParentalRelationshipType", value)),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetParentRelationships<'a> {
    pub site_id: i64,
    pub page: Reference<'a>,
    pub relationship_type: ParentalRelationshipType,
}

#[derive(Serialize, Debug, Copy, Clone)]
pub struct RemoveParentOutput {
    pub was_deleted: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct UpdateParentsOutput {
    pub added: Option<Vec<i64>>,
    pub removed: Option<Vec<bool>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parental_relationship_names_and_parsing_are_stable() {
        assert_eq!(ParentalRelationshipType::Parent.name(), "parents");
        assert_eq!(ParentalRelationshipType::Child.name(), "children");
        assert_eq!(
            "parents".parse::<ParentalRelationshipType>().unwrap(),
            ParentalRelationshipType::Parent,
        );
        assert_eq!(
            "children".parse::<ParentalRelationshipType>().unwrap(),
            ParentalRelationshipType::Child,
        );

        let error = "siblings".parse::<ParentalRelationshipType>().unwrap_err();
        assert_eq!(
            error.to_string(),
            "failed to convert value 'siblings' to a ParentalRelationshipType enum value"
        );
    }
}
