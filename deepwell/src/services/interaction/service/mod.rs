/*
 * services/interaction/service/mod.rs
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

#[macro_use]
mod macros;
mod site;
mod user;

use super::prelude::*;
use crate::models::interaction::{
    self, Entity as Interaction, Model as InteractionModel,
};
use serde::Serialize;

#[derive(Debug)]
pub struct InteractionService;

impl InteractionService {
    pub async fn add<M: Serialize>(
        ctx: &ServiceContext<'_>,
        interaction_type: InteractionType,
        source: InteractionObject,
        target: InteractionObject,
        created_by: i64,
        metadata: &M,
    ) -> Result<InteractionModel> {
        tide::log::debug!(
            "Adding interaction for {source:?} / {interaction_type:?} / {target:?}",
        );

        // Delete previous interaction, if present
        if let Some(interaction) = Self::get_optional(
            ctx,
            InteractionReference::Relationship {
                interaction_type,
                source,
                target,
            },
        )
        .await?
        {
            Self::remove(
                ctx,
                InteractionReference::Id(interaction.interaction_id),
                created_by,
            )
            .await?;
        }

        // Insert new interaction
        let (source_type, source_id) = source.into();
        let (target_type, target_id) = target.into();
        let metadata = serde_json::to_value(metadata)?;
        let model = interaction::ActiveModel {
            source_type: Set(source_type),
            source_id: Set(source_id),
            interaction_type: Set(interaction_type),
            target_type: Set(target_type),
            target_id: Set(target_id),
            created_by: Set(created_by),
            metadata: Set(metadata),
            ..Default::default()
        };

        let txn = ctx.transaction();
        let interaction = model.insert(txn).await?;
        Ok(interaction)
    }

    pub async fn remove(
        ctx: &ServiceContext<'_>,
        reference: InteractionReference,
        deleted_by: i64,
    ) -> Result<()> {
        tide::log::debug!("Removing interaction for {reference:?}");

        let txn = ctx.transaction();
        let interaction_id = Self::get_id(ctx, reference).await?;
        let model = interaction::ActiveModel {
            interaction_id: Set(interaction_id),
            deleted_by: Set(Some(deleted_by)),
            deleted_at: Set(Some(now())),
            ..Default::default()
        };

        model.update(txn).await?;
        Ok(())
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        reference: InteractionReference,
    ) -> Result<Option<InteractionModel>> {
        tide::log::debug!("Getting interaction for {reference:?}");

        let txn = ctx.transaction();
        let interaction = Interaction::find()
            .filter(
                Condition::all()
                    .add(reference.condition())
                    .add(interaction::Column::DeletedAt.is_null()),
            )
            .one(txn)
            .await?;

        Ok(interaction)
    }

    /// Gets the interaction ID from a reference, looking up if necessary.
    pub async fn get_id(
        ctx: &ServiceContext<'_>,
        reference: InteractionReference,
    ) -> Result<i64> {
        match reference {
            InteractionReference::Id(interaction_id) => Ok(interaction_id),
            InteractionReference::Relationship { .. } => {
                let InteractionModel { interaction_id, .. } =
                    Self::get(ctx, reference).await?;
                Ok(interaction_id)
            }
        }
    }

    pub async fn get(
        ctx: &ServiceContext<'_>,
        reference: InteractionReference,
    ) -> Result<InteractionModel> {
        find_or_error(Self::get_optional(ctx, reference)).await
    }

    pub async fn exists(
        ctx: &ServiceContext<'_>,
        reference: InteractionReference,
    ) -> Result<bool> {
        Self::get_optional(ctx, reference)
            .await
            .map(|interaction| interaction.is_some())
    }

    // TODO paginate
    pub async fn get_history(
        ctx: &ServiceContext<'_>,
        interaction_type: InteractionType,
        source: InteractionObject,
        target: InteractionObject,
    ) -> Result<Vec<InteractionModel>> {
        tide::log::debug!(
            "Getting history of interactions for {source:?} / {interaction_type:?} / {target:?}",
        );

        let txn = ctx.transaction();
        let interactions = Interaction::find()
            .filter(interaction_condition(interaction_type, source, target))
            .order_by_asc(interaction::Column::CreatedAt)
            .all(txn)
            .await?;

        Ok(interactions)
    }
}
