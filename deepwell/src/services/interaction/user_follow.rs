/*
 * services/interaction/user_follow.rs
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

use super::prelude::*;

impl_interaction!(UserFollow, user_follow, User, followed_user, User, following_user, false);

impl InteractionService {
    pub async fn add_user_follow(
        ctx: &ServiceContext<'_>,
        dest: i64,
        from: i64,
        created_by: i64,
    ) -> Result<()> {
        // Cannot follow if blocked
        Self::check_user_block(ctx, dest, from, "follow").await?;

        add_operation!(UserFollow, followed_user, following_user, created_by, metadata)?;
        Ok(())
    }
}
