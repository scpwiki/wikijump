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
use crate::services::user::{CreateUser, UpdateUser};
use crate::services::user_bot_owner::{
    BotOwner, CreateBotOwner, CreateBotUser, UserBotOwnerService,
};
use crate::services::{Error as ServiceError, Result as ServiceResult};
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

    // Set description
    UserService::update(
        &ctx,
        Reference::Id(output.user_id),
        UpdateUser {
            biography: ProvidedValue::Set(Some(purpose)),
            ..Default::default()
        },
    )
    .await
    .to_api()?;

    // Add bot owners
    add_bot_owners(&ctx, output.user_id, owners)
        .await
        .to_api()?;

    // Build and return response
    let body = Body::from_json(&output)?;
    txn.commit().await?;

    let response = Response::builder(StatusCode::Created).body(body).into();
    Ok(response)
}

async fn add_bot_owners(
    ctx: &ServiceContext<'_>,
    bot_user_id: i64,
    owners: Vec<BotOwner>,
) -> ServiceResult<()> {
    tide::log::debug!("Adding human owners for bot user ID {}", bot_user_id);

    for owner in owners {
        let BotOwner {
            user_id: human_user_id,
            description,
        } = owner;

        let human = UserService::get(&ctx, Reference::Id(human_user_id))
            .await
            .to_api()?;

        if human.user_type != UserType::Regular {
            tide::log::error!(
                "Bot owner user ID {} does not refer to a regular user",
                human_user_id,
            );

            return Err(ServiceError::BadRequest);
        }

        tide::log::debug!("Adding human user ID {} as bot owner", human_user_id);
        UserBotOwnerService::add(
            &ctx,
            CreateBotOwner {
                human_user_id,
                bot_user_id,
                description,
            },
        )
        .await?;
    }

    Ok(())
}

pub async fn user_bot_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    tide::log::info!("Getting bot user information");
    todo!()
}

pub async fn user_bot_owner_put(req: ApiRequest) -> ApiResponse {
    todo!()
}

pub async fn user_bot_owner_delete(req: ApiRequest) -> ApiResponse {
    todo!()
}
