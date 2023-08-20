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

#[derive(Debug)]
pub struct InteractionService;

impl InteractionService {
    pub async fn add(
        ctx: &ServiceContext<'_>,
        interaction_type: InteractionType,
        source: InteractionObject,
        target: InteractionObject,
    ) -> Result<()> {
        tide::log::debug!(
            "Adding interaction for {source:?} / {interaction_type:?} / {target:?}",
        );

        let txn = ctx.transaction();
        let model = Self::active_model(interaction_type, source, target);

        if !Self::exists(ctx, interaction_type, source, target).await? {
            model.insert(txn).await?;
        }

        Ok(())
    }

    pub async fn remove(
        ctx: &ServiceContext<'_>,
        interaction_type: InteractionType,
        source: InteractionObject,
        target: InteractionObject,
    ) -> Result<()> {
        tide::log::debug!(
            "Removing interaction for {source:?} / {interaction_type:?} / {target:?}",
        );

        let txn = ctx.transaction();
        let model = Self::active_model(interaction_type, source, target);
        model.delete(txn).await?;
        Ok(())
    }

    pub async fn exists(
        ctx: &ServiceContext<'_>,
        interaction_type: InteractionType,
        source: InteractionObject,
        target: InteractionObject,
    ) -> Result<bool> {
        tide::log::debug!(
            "Getting interaction for {source:?} / {interaction_type:?} / {target:?}",
        );

        let txn = ctx.transaction();
        let (source_type, source_id) = source.into();
        let (target_type, target_id) = target.into();

        let exists = Interaction::find_by_id((
            source_type,
            source_id,
            interaction_type,
            target_type,
            target_id,
        ))
        .one(txn)
        .await?
        .is_some();

        Ok(exists)
    }

    fn active_model(
        interaction_type: InteractionType,
        source: InteractionObject,
        target: InteractionObject,
    ) -> interaction::ActiveModel {
        let (source_type, source_id) = source.into();
        let (target_type, target_id) = target.into();

        interaction::ActiveModel {
            source_type: Set(source_type),
            source_id: Set(source_id),
            interaction_type: Set(interaction_type),
            target_type: Set(target_type),
            target_id: Set(target_id),
            ..Default::default()
        }
    }
}
