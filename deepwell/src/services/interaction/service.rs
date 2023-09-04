/*
 * services/interaction/service.rs
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

// Macros

macro_rules! site {
    ($id:expr $(,)?) => {
        InteractionObject::Site($id)
    };
}

macro_rules! user {
    ($id:expr $(,)?) => {
        InteractionObject::User($id)
    };
}

macro_rules! page {
    ($id:expr $(,)?) => {
        InteractionObject::Page($id)
    };
}

// Adding an "i" (for "interaction") because file!() itself conflicts with logging.
macro_rules! ifile {
    ($id:expr $(,)?) => {
        InteractionObject::File($id)
    };
}

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
        dest: InteractionObject,
        from: InteractionObject,
        created_by: i64,
        metadata: &M,
    ) -> Result<InteractionModel> {
        tide::log::debug!(
            "Adding interaction for {dest:?} ← {interaction_type:?} ← {from:?}",
        );

        // Delete previous interaction, if present
        if let Some(interaction) = Self::get_optional(
            ctx,
            InteractionReference::Relationship {
                interaction_type,
                dest,
                from,
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
        let (dest_type, dest_id) = dest.into();
        let (from_type, from_id) = from.into();
        interaction_type.types().check(dest_type, from_type);

        let metadata = serde_json::to_value(metadata)?;
        let model = interaction::ActiveModel {
            interaction_type: Set(str!(interaction_type.value())),
            dest_type: Set(dest_type),
            dest_id: Set(dest_id),
            from_type: Set(from_type),
            from_id: Set(from_id),
            metadata: Set(metadata),
            created_by: Set(created_by),
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
            deleted_at: Set(Some(now())),
            deleted_by: Set(Some(deleted_by)),
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

    // User blocks

    pub async fn block_user(
        ctx: &ServiceContext<'_>,
        dest_user: i64,
        from_user: i64,
        created_by: i64,
    ) -> Result<()> {
        tide::log::info!("Blocking user ID {dest_user} on behalf of user ID {from_user}");

        // TODO: unfollow user, remove from contacts, etc. both ways

        Self::add(
            ctx,
            InteractionType::UserBlock,
            user!(dest_user),
            user!(from_user),
            created_by,
            &(),
        )
        .await?;

        Ok(())
    }

    pub async fn unblock_user(
        ctx: &ServiceContext<'_>,
        dest_user: i64,
        from_user: i64,
        deleted_by: i64,
    ) -> Result<()> {
        tide::log::info!(
            "Unblocking user ID {dest_user} on behalf of user ID {from_user}",
        );

        Self::remove(
            ctx,
            InteractionReference::Relationship {
                interaction_type: InteractionType::UserBlock,
                dest: user!(dest_user),
                from: user!(from_user),
            },
            deleted_by,
        )
        .await
    }

    pub async fn user_blocked(
        ctx: &ServiceContext<'_>,
        dest_user: i64,
        from_user: i64,
    ) -> Result<bool> {
        tide::log::info!(
            "Checking if user ID {dest_user} is blocked by user ID {from_user}",
        );

        Self::exists(
            ctx,
            InteractionReference::Relationship {
                interaction_type: InteractionType::UserBlock,
                dest: user!(dest_user),
                from: user!(from_user),
            },
        )
        .await
    }
}
