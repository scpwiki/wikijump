/*
 * services/relation/structs.rs
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

use crate::models::relation;
use crate::types::{RelationObjectType, RelationType};
use sea_orm::{ColumnTrait, Condition};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RelationObject {
    Site(i64),
    User(i64),
    Page(i64),
    File(i64),
}

impl From<RelationObject> for (RelationObjectType, i64) {
    fn from(object: RelationObject) -> (RelationObjectType, i64) {
        match object {
            RelationObject::Site(id) => (RelationObjectType::Site, id),
            RelationObject::User(id) => (RelationObjectType::User, id),
            RelationObject::Page(id) => (RelationObjectType::Page, id),
            RelationObject::File(id) => (RelationObjectType::File, id),
        }
    }
}

impl From<RelationObject> for RelationObjectType {
    fn from(object: RelationObject) -> RelationObjectType {
        let (otype, _): (RelationObjectType, i64) = object.into();
        otype
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)] // TEMP
pub enum RelationReference {
    Id(i64),
    Relationship {
        relation_type: RelationType,
        dest: RelationObject,
        from: RelationObject,
    },
}

impl RelationReference {
    pub fn condition(self) -> Condition {
        match self {
            RelationReference::Id(id) => {
                // Needs wrapping Condition due to type; ALL or ANY both work
                Condition::all().add(relation::Column::RelationId.eq(id))
            }
            RelationReference::Relationship {
                relation_type,
                dest,
                from,
            } => relation_condition(relation_type, dest, from),
        }
    }
}

pub fn relation_type_condition(relation_type: RelationType) -> Condition {
    Condition::any().add(
        relation::Column::RelationType
            .is_in(relation_type.database_values().iter().copied()),
    )
}

pub fn relation_condition(
    relation_type: RelationType,
    dest: RelationObject,
    from: RelationObject,
) -> Condition {
    let (dest_type, dest_id) = dest.into();
    let (from_type, from_id) = from.into();

    Condition::all()
        .add(relation_type_condition(relation_type))
        .add(relation::Column::DestType.eq(dest_type))
        .add(relation::Column::DestId.eq(dest_id))
        .add(relation::Column::FromType.eq(from_type))
        .add(relation::Column::FromId.eq(from_id))
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RelationObjectTypes {
    pub dest: RelationObjectType,
    pub from: RelationObjectType,
}

impl RelationObjectTypes {
    /// Asserts that the given object types match for relation type.
    pub fn check<O1, O2>(self, dest: O1, from: O2)
    where
        O1: Into<RelationObjectType>,
        O2: Into<RelationObjectType>,
    {
        let dest = dest.into();
        let from = from.into();

        assert_eq!(
            (self.dest, self.from),
            (dest, from),
            "Object types inappropriate for this relation: expected {:?}, actual {:?}",
            (self.dest, self.from),
            (dest, from),
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)] // TEMP
pub enum RelationDirection {
    Dest,
    From,
}

impl RelationType {
    pub fn types(self) -> RelationObjectTypes {
        macro_rules! t {
            ($dest:ident, $from:ident $(,)?) => {
                RelationObjectTypes {
                    dest: RelationObjectType::$dest,
                    from: RelationObjectType::$from,
                }
            };
        }

        match self {
            RelationType::SiteUser => t!(Site, User),
            RelationType::SiteBan => t!(Site, User),
            RelationType::SiteApplication => t!(Site, User),
            RelationType::SiteMember => t!(Site, User),
            RelationType::PageStar => t!(Page, User),
            RelationType::PageWatch => t!(Page, User),
            RelationType::PageAttribution => t!(Page, User),
            RelationType::UserFollow => t!(User, User),
            RelationType::UserContact => t!(User, User),
            RelationType::UserContactRequest => t!(User, User),
            RelationType::UserBlock => t!(User, User),
            RelationType::UserBotOwner => t!(User, User),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RelationObject, RelationType, relation_condition};
    use crate::models::relation;
    use sea_orm::{DatabaseBackend, EntityTrait, QueryFilter, QueryTrait};

    #[test]
    fn relation_condition_matches_legacy_and_namespaced_database_values() {
        let statement = relation::Entity::find()
            .filter(relation_condition(
                RelationType::SiteBan,
                RelationObject::Site(42),
                RelationObject::User(7),
            ))
            .build(DatabaseBackend::Postgres);

        let sql = statement.to_string();
        assert!(
            sql.contains(r#""relation"."relation_type" IN ('ban', 'site-ban')"#),
            "{sql}"
        );
    }
}
