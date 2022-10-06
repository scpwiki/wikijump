/*
 * methods/user_bot.rs
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
use crate::models::sea_orm_active_enums::UserType;
use crate::models::user_bot_owner::Model as UserBotOwnerModel;
use crate::services::user::{CreateUser, UpdateUser, UserProfileOutput};
use crate::services::user_bot_owner::{
    BotOwner, BotUserOutput, CreateBotOwner, CreateBotUser, UserBotOwnerService,
};
use crate::web::ProvidedValue;

pub async fn user_bot_create(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    tide::log::info!("Creating new bot user");
    let CreateBotUser {
        name,
        email,
        locale,
        purpose,
        owners,
        authorization_token,
    } = req.body_json().await?;

    // TODO verify auth token
    let _ = authorization_token;

    // Create bot user
    let output = UserService::create(
        &ctx,
        UserType::Bot,
        CreateUser {
            name,
            email,
            locale,
            password: String::new(),
        },
    )
    .await
    .to_api()?;

    let bot_user_id = output.user_id;

    // Set description
    UserService::update(
        &ctx,
        Reference::Id(bot_user_id),
        UpdateUser {
            biography: ProvidedValue::Set(Some(purpose)),
            ..Default::default()
        },
    )
    .await
    .to_api()?;

    // Add bot owners
    tide::log::debug!("Adding human owners for bot user ID {}", bot_user_id);
    for owner in owners {
        let BotOwner {
            user_id: human_user_id,
            description,
        } = owner;

        let human = UserService::get_with_user_type(
            &ctx,
            Reference::Id(human_user_id),
            UserType::Regular,
        )
        .await
        .to_api()?;

        tide::log::debug!("Adding human user ID {} as bot owner", human_user_id);
        UserBotOwnerService::add(
            &ctx,
            CreateBotOwner {
                human_user_id,
                bot_user_id,
                description,
            },
        )
        .await
        .to_api()?;
    }

    // Build and return response
    let body = Body::from_json(&output)?;
    txn.commit().await?;

    let response = Response::builder(StatusCode::Created).body(body).into();
    Ok(response)
}

pub async fn user_bot_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let reference = Reference::try_from_fields_key(&req, "bot_type", "bot_id_or_slug")?;
    tide::log::info!("Getting bot user {:?}", reference);

    let user = UserService::get(&ctx, reference).await.to_api()?;
    let owners = UserBotOwnerService::get_all(&ctx, user.user_id)
        .await
        .to_api()?;

    let output = BotUserOutput {
        user: UserProfileOutput::from(&user),
        owners: owners
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
            .collect(),
    };

    let body = Body::from_json(&output)?;
    txn.commit().await?;

    Ok(body.into())
}

pub async fn user_bot_owner_put(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let bot_reference =
        Reference::try_from_fields_key(&req, "bot_type", "bot_id_or_slug")?;
    let human_reference =
        Reference::try_from_fields_key(&req, "human_type", "human_id_or_slug")?;

    tide::log::info!(
        "Adding or updating bot owner ({:?} <- {:?})",
        bot_reference,
        human_reference,
    );

    let (bot, human) = try_join!(
        UserService::get_with_user_type(&ctx, bot_reference, UserType::Bot),
        UserService::get_with_user_type(&ctx, human_reference, UserType::Regular),
    )
    .to_api()?;

    todo!()
}

pub async fn user_bot_owner_delete(req: ApiRequest) -> ApiResponse {
    todo!()
}
