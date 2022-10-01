/*
 * services/user_bot_owner/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2022 Wikijump Team
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

use super::prelude::*;
use crate::models::user_bot_owner::{
    self, Entity as UserBotOwner, Model as UserBotOwnerModel,
};

#[derive(Debug)]
pub struct UserBotOwnerService;

impl UserBotOwnerService {
    #[allow(dead_code)] // TODO
    pub async fn get_all(
        ctx: &ServiceContext<'_>,
        bot_user_id: i64,
    ) -> Result<Vec<UserBotOwnerModel>> {
        tide::log::info!("Looking up owners for bot ID {bot_user_id}");

        let txn = ctx.transaction();
        let owners = UserBotOwner::find()
            .filter(user_bot_owner::Column::BotUserId.eq(bot_user_id))
            .all(txn)
            .await?;

        Ok(owners)
    }

    async fn get_optional(
        ctx: &ServiceContext<'_>,
        bot_user_id: i64,
        human_user_id: i64,
    ) -> Result<Option<UserBotOwnerModel>> {
        tide::log::debug!(
            "Retrieving user_bot_owner record for human ID {} and bot ID {}",
            human_user_id,
            bot_user_id,
        );

        let txn = ctx.transaction();
        let owner = UserBotOwner::find_by_id((bot_user_id, human_user_id))
            .one(txn)
            .await?;

        Ok(owner)
    }

    /// Idempotently adds or updates a user as a bot owner.
    ///
    /// It is the responsibility of the caller to assure
    /// that the `bot_user_id` is a bot, and
    /// that `human_user_id` is a human (i.e. `regular` user).
    pub async fn add(
        ctx: &ServiceContext<'_>,
        CreateBotOwner {
            bot_user_id,
            human_user_id,
            description,
        }: CreateBotOwner,
    ) -> Result<()> {
        tide::log::info!(
            "Adding user ID {} as owner for bot ID {}: {}",
            human_user_id,
            bot_user_id,
            description,
        );

        // NOTE: Not using upsert (INSERT .. ON CONFLICT) because
        //       setting updated_at is a bit gnarly.

        let txn = ctx.transaction();
        match Self::get_optional(ctx, bot_user_id, human_user_id).await? {
            // Update
            Some(owner) => {
                tide::log::debug!("Bot owner record exists, updating");

                let mut model = owner.into_active_model();
                model.description = Set(description);
                model.updated_at = Set(Some(now()));
                model.update(txn).await?;
            }

            // Insert
            None => {
                tide::log::debug!("Bot owner record is missing, inserting");

                let model = user_bot_owner::ActiveModel {
                    bot_user_id: Set(bot_user_id),
                    human_user_id: Set(human_user_id),
                    description: Set(description),
                    ..Default::default()
                };

                model.insert(txn).await?;
            }
        }

        Ok(())
    }

    /// Idempotently deletes the give user / bot ownership record, if it exists.
    ///
    /// Returns `true` if the deletion was carried out (i.e. it used to exist),
    /// and `false` if not.
    #[allow(dead_code)] // TODO
    pub async fn delete(
        ctx: &ServiceContext<'_>,
        DeleteBotOwner {
            bot_user_id,
            human_user_id,
        }: DeleteBotOwner,
    ) -> Result<bool> {
        tide::log::info!(
            "Deleting user ID {} as owner for bot ID {}",
            human_user_id,
            bot_user_id,
        );

        let txn = ctx.transaction();
        let DeleteResult { rows_affected } =
            UserBotOwner::delete_by_id((bot_user_id, human_user_id))
                .exec(txn)
                .await?;

        debug_assert!(
            rows_affected <= 1,
            "Rows deleted using ID was more than 1: {}",
            rows_affected,
        );

        Ok(rows_affected == 1)
    }
}
