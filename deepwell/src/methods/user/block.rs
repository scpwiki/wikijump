/*
 * methods/user/block.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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

#[derive(Serialize, Deserialize, Debug)]
struct UserBlocked {
    blocked: bool,
}

pub async fn user_client_blocked_get(req: ApiRequest) -> ApiResponse {
    // returns Vec<?>
    todo!()
}

pub async fn user_block_get(req: ApiRequest) -> ApiResponse {
    // returns UserBlocked
    todo!()
}

pub async fn user_block_put(mut req: ApiRequest) -> ApiResponse {
    // returns ()
    let UserBlocked { blocked: _ } = req.body_json().await?;
    todo!()
}
