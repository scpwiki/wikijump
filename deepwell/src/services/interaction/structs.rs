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
use crate::models::sea_orm_active_enums::{InteractionObjectType, InteractionType};
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InteractionReference {
    Id(i64),
    Relationship {
        interaction_type: InteractionType,
        source: InteractionObject,
        target: InteractionObject,
    },
}

impl InteractionReference {
    pub fn condition(self) -> Condition {
        match self {
            InteractionReference::Id(id) => {
                // Needs wrapping Condition due to type ALL or ANY both work
                Condition::all().add(interaction::Column::InteractionId.eq(id))
            }
            InteractionReference::Relationship {
                interaction_type,
                source,
                target,
            } => interaction_condition(interaction_type, source, target),
        }
    }
}

pub fn interaction_condition(
    interaction_type: InteractionType,
    source: InteractionObject,
    target: InteractionObject,
) -> Condition {
    let (source_type, source_id) = source.into();
    let (target_type, target_id) = target.into();

    Condition::all()
        .add(interaction::Column::InteractionType.eq(interaction_type))
        .add(interaction::Column::SourceType.eq(source_type))
        .add(interaction::Column::SourceId.eq(source_id))
        .add(interaction::Column::TargetType.eq(target_type))
        .add(interaction::Column::TargetId.eq(target_id))
}
