/*
 * services/interaction/service/user.rs
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

use super::InteractionService;
use super::super::prelude::*;

impl InteractionService {
    #[inline]
    pub async fn block_user(ctx: &ServiceContext<'_>, source_user: i64, target_user: i64) -> Result<()> {
        tide::log::info!("Blocking user ID {target_user} on behalf of user ID {source_user}");
        Self::add(ctx, InteractionType::Block, source_user, target_user).await
    }

    #[inline]
    pub async fn unblock_user(ctx: &ServiceContext<'_>, source_user: i64, target_user: i64) -> Result<()> {
        tide::log::info!("Unblocking user ID {target_user} on behalf of user ID {source_user}");
        Self::remove(ctx, InteractionType::Block, source_user, target_user).await
    }
}
