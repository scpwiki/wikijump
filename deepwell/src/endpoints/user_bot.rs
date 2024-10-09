/*
 * endpoints/user_bot.rs
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

use super::prelude::*;
use crate::models::sea_orm_active_enums::UserType;
use crate::models::user_bot_owner::Model as UserBotOwnerModel;
use crate::services::user::{CreateUser, CreateUserOutput, GetUser, UpdateUserBody};
use crate::services::user_bot_owner::{
    BotOwner, BotUserOutput, CreateBotOwner, CreateBotUser, RemoveBotOwner,
    RemoveBotOwnerOutput, UserBotOwnerService,
};
use crate::types::{Maybe, Reference};

pub async fn bot_user_create(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<CreateUserOutput> {
    let CreateBotUser {
        name,
        email,
        locales,
        purpose,
        owners,
        authorization_token,
        bypass_filter,
        bypass_email_verification,
    } = params.parse()?;

    info!("Creating new bot user with name '{}'", name);

    // TODO verify auth token
    let _ = authorization_token;

    // Create bot user
    let output = UserService::create(
        ctx,
        CreateUser {
            user_type: UserType::Bot,
            name,
            email,
            locales,
            password: String::new(), // TODO configure user-bot password
            bypass_filter,
            bypass_email_verification,
        },
    )
    .await?;

    let bot_user_id = output.user_id;

    // Set description
    UserService::update(
        ctx,
        Reference::Id(bot_user_id),
        UpdateUserBody {
            biography: Maybe::Set(Some(purpose)),
            ..Default::default()
        },
    )
    .await?;

    // Add bot owners
    debug!("Adding human owners for bot user ID {}", bot_user_id);
    for owner in owners {
        let BotOwner {
            user_id: human_user_id,
            description,
        } = owner;

        debug!("Adding human user ID {} as bot owner", human_user_id);
        UserBotOwnerService::add(
            ctx,
            CreateBotOwner {
                human: Reference::Id(human_user_id),
                bot: Reference::Id(bot_user_id),
                description,
            },
        )
        .await?;
    }

    // Return
    Ok(output)
}

pub async fn bot_user_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<BotUserOutput>> {
    let GetUser { user: reference } = params.parse()?;
    info!("Getting bot user {reference:?}");
    match UserService::get_optional(ctx, reference).await? {
        None => Ok(None),
        Some(user) => {
            let owners = UserBotOwnerService::get_all(ctx, user.user_id).await?;
            let owners = owners
                .into_iter()
                .map(
                    |UserBotOwnerModel {
                         human_user_id: user_id,
                         description,
                         ..
                     }| BotOwner {
                        user_id,
                        description,
                    },
                )
                .collect();

            Ok(Some(BotUserOutput { user, owners }))
        }
    }
}

pub async fn bot_user_owner_set(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<UserBotOwnerModel> {
    let input: CreateBotOwner = params.parse()?;

    info!(
        "Adding or updating bot owner ({:?} <- {:?})",
        input.bot, input.human,
    );

    UserBotOwnerService::add(ctx, input).await
}

pub async fn bot_user_owner_remove(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<RemoveBotOwnerOutput> {
    let input: RemoveBotOwner = params.parse()?;
    info!("Remove bot owner ({:?} <- {:?})", input.bot, input.human,);
    UserBotOwnerService::remove(ctx, input).await
}
