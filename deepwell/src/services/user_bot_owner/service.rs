/*
 * services/user_bot_owner/service.rs
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

// TODO replace bot owners with a user / user relation
//      add checks like we have here, one is human one is bot, etc

use super::prelude::*;
use crate::models::sea_orm_active_enums::UserType;
use crate::models::user_bot_owner::{
    self, Entity as UserBotOwner, Model as UserBotOwnerModel,
};
use crate::services::UserService;

#[derive(Debug)]
pub struct UserBotOwnerService;

impl UserBotOwnerService {
    pub async fn get_all(
        ctx: &ServiceContext<'_>,
        bot_user_id: i64,
    ) -> Result<Vec<UserBotOwnerModel>> {
        info!("Looking up owners for bot ID {bot_user_id}");

        let txn = ctx.seaorm_transaction();
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
        debug!(
            "Retrieving user_bot_owner record for human ID {} and bot ID {}",
            human_user_id, bot_user_id,
        );

        let txn = ctx.seaorm_transaction();
        let owner = UserBotOwner::find_by_id((bot_user_id, human_user_id))
            .one(txn)
            .await?;

        Ok(owner)
    }

    /// Idempotently adds or updates a user as a bot owner.
    ///
    /// It is the responsibility of the caller to assure that
    /// `bot` is a bot user and `human` is a human user
    /// (i.e. `regular` user type).
    pub async fn add(
        ctx: &ServiceContext<'_>,
        CreateBotOwner {
            bot: bot_reference,
            human: human_reference,
            description,
        }: CreateBotOwner<'_>,
    ) -> Result<UserBotOwnerModel> {
        let (bot, human) = try_join!(
            UserService::get_with_user_type(ctx, bot_reference, UserType::Bot),
            UserService::get_with_user_type(ctx, human_reference, UserType::Regular),
        )?;

        info!(
            "Adding user ID {} as owner for bot ID {}: {}",
            human.user_id, bot.user_id, description,
        );

        // NOTE: Not using upsert (INSERT .. ON CONFLICT) because
        //       setting updated_at is a bit gnarly.

        let txn = ctx.seaorm_transaction();
        let model = match Self::get_optional(ctx, bot.user_id, human.user_id).await? {
            // Update
            Some(owner) => {
                debug!("Bot owner record exists, updating");

                let mut model = owner.into_active_model();
                model.description = Set(description);
                model.updated_at = Set(Some(now()));
                model.update(txn).await?
            }

            // Insert
            None => {
                debug!("Bot owner record is missing, inserting");

                let model = user_bot_owner::ActiveModel {
                    bot_user_id: Set(bot.user_id),
                    human_user_id: Set(human.user_id),
                    description: Set(description),
                    ..Default::default()
                };

                model.insert(txn).await?
            }
        };

        Ok(model)
    }

    /// Idempotently removes the give user / bot ownership record, if it exists.
    ///
    /// # Returns
    /// The struct contains `true` if the deletion was carried out (i.e. it used to exist),
    /// and `false` if not.
    pub async fn remove(
        ctx: &ServiceContext<'_>,
        RemoveBotOwner {
            bot: bot_reference,
            human: human_reference,
        }: RemoveBotOwner<'_>,
    ) -> Result<RemoveBotOwnerOutput> {
        let txn = ctx.seaorm_transaction();

        // We don't check user type here because we already checked it prior to insertion.
        //
        // This could also lead to an annoying circumstance where a user account is modified,
        // but because the type no longer matches, you can't edit it.

        let (bot_user_id, human_user_id) = try_join!(
            UserService::get_id(ctx, bot_reference),
            UserService::get_id(ctx, human_reference),
        )?;

        info!(
            "Deleting user ID {} as owner for bot ID {}",
            human_user_id, bot_user_id,
        );

        let DeleteResult { rows_affected } =
            UserBotOwner::delete_by_id((bot_user_id, human_user_id))
                .exec(txn)
                .await?;

        debug_assert!(
            rows_affected <= 1,
            "Rows deleted using ID was more than 1: {rows_affected}",
        );

        let was_deleted = rows_affected == 1;
        Ok(RemoveBotOwnerOutput { was_deleted })
    }
}
