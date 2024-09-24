/*
 * services/parent/structs.rs
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

use crate::services::Error;
use crate::web::Reference;
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
    type Err = Error;

    fn from_str(value: &str) -> Result<ParentalRelationshipType, Error> {
        match value {
            "parents" => Ok(ParentalRelationshipType::Parent),
            "children" => Ok(ParentalRelationshipType::Child),
            _ => Err(Error::InvalidEnumValue),
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
