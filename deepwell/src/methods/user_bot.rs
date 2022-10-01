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
use crate::services::user_bot_owner::{CreateBotOwner, UserBotOwnerService};

pub async fn user_bot_create(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    tide::log::info!("Creating new bot user");
    todo!()
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
