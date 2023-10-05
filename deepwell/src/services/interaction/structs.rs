/*
 * services/interaction/structs.rs
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

use crate::models::interaction::{
    self, Entity as Interaction, Model as InteractionModel,
};
use crate::models::sea_orm_active_enums::InteractionObjectType;
use sea_orm::{ColumnTrait, Condition};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InteractionObject {
    Site(i64),
    User(i64),
    Page(i64),
    File(i64),
}

impl From<InteractionObject> for (InteractionObjectType, i64) {
    fn from(object: InteractionObject) -> (InteractionObjectType, i64) {
        match object {
            InteractionObject::Site(id) => (InteractionObjectType::Site, id),
            InteractionObject::User(id) => (InteractionObjectType::User, id),
            InteractionObject::Page(id) => (InteractionObjectType::Page, id),
            InteractionObject::File(id) => (InteractionObjectType::File, id),
        }
    }
}

impl From<InteractionObject> for InteractionObjectType {
    fn from(object: InteractionObject) -> InteractionObjectType {
        let (otype, _): (InteractionObjectType, i64) = object.into();
        otype
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InteractionReference {
    Id(i64),
    Relationship {
        interaction_type: InteractionType,
        dest: InteractionObject,
        from: InteractionObject,
    },
}

impl InteractionReference {
    pub fn condition(self) -> Condition {
        match self {
            InteractionReference::Id(id) => {
                // Needs wrapping Condition due to type; ALL or ANY both work
                Condition::all().add(interaction::Column::InteractionId.eq(id))
            }
            InteractionReference::Relationship {
                interaction_type,
                dest,
                from,
            } => interaction_condition(interaction_type, dest, from),
        }
    }
}

pub fn interaction_condition(
    interaction_type: InteractionType,
    dest: InteractionObject,
    from: InteractionObject,
) -> Condition {
    let (dest_type, dest_id) = dest.into();
    let (from_type, from_id) = from.into();

    Condition::all()
        .add(interaction::Column::InteractionType.eq(interaction_type.value()))
        .add(interaction::Column::DestType.eq(dest_type))
        .add(interaction::Column::DestId.eq(dest_id))
        .add(interaction::Column::FromType.eq(from_type))
        .add(interaction::Column::FromId.eq(from_id))
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InteractionObjectTypes {
    pub dest: InteractionObjectType,
    pub from: InteractionObjectType,
}

impl InteractionObjectTypes {
    /// Asserts that the given object types match for interaction type.
    pub fn check<O1, O2>(self, dest: O1, from: O2)
    where
        O1: Into<InteractionObjectType>,
        O2: Into<InteractionObjectType>,
    {
        let dest = dest.into();
        let from = from.into();

        assert_eq!(
            (self.dest, self.from),
            (dest, from),
            "Object types inappropriate for this interaction: expected {:?}, actual {:?}",
            (self.dest, self.from),
            (dest, from),
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InteractionDirection {
    Dest,
    From,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InteractionType {
    SiteBan,
    SiteMember,
    PageStar,
    PageWatch,
    UserFollow,
    UserContact,
    UserContactRequest,
    UserBlock,
}

impl InteractionType {
    /// Get the constant string value used to represent this interaction in the database.
    ///
    /// It need not be unique among all interactions, but it must be unique for each triplet
    /// of `(dest_type, from_type, interaction_value)`.
    pub fn value(self) -> &'static str {
        match self {
            InteractionType::SiteBan => "ban",
            InteractionType::SiteMember => "member",
            InteractionType::PageStar => "star",
            InteractionType::PageWatch => "watch",
            InteractionType::UserFollow => "follow",
            InteractionType::UserContact => "contact",
            InteractionType::UserContactRequest => "contact-request",
            InteractionType::UserBlock => "block",
        }
    }

    pub fn types(self) -> InteractionObjectTypes {
        macro_rules! t {
            ($dest:ident, $from:ident $(,)?) => {
                InteractionObjectTypes {
                    dest: InteractionObjectType::$dest,
                    from: InteractionObjectType::$from,
                }
            };
        }

        match self {
            InteractionType::SiteBan => t!(Site, User),
            InteractionType::SiteMember => t!(Site, User),
            InteractionType::PageStar => t!(Page, User),
            InteractionType::PageWatch => t!(Page, User),
            InteractionType::UserFollow => t!(User, User),
            InteractionType::UserContact => t!(User, User),
            InteractionType::UserContactRequest => t!(User, User),
            InteractionType::UserBlock => t!(User, User),
        }
    }
}
