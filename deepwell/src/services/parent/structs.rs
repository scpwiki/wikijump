/*
 * services/parent/structs.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParentDescription<'a> {
    pub site_id: i64,
    pub parent: Reference<'a>,
    pub child: Reference<'a>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ParentalRelationshipType {
    Parent,
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
