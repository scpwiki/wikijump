/*
 * services/relation/user_follow.rs
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

impl_relation!(
    UserFollow,
    User,
    followed_user,
    User,
    following_user,
    (),
    NO_CREATE_IMPL,
);

impl RelationService {
    #[allow(dead_code)] // TEMP
    pub async fn create_user_follow(
        ctx: &ServiceContext,
        CreateUserFollow {
            followed_user,
            following_user,
            created_by,
            metadata: (),
        }: CreateUserFollow,
    ) -> Result<()> {
        // Cannot follow if blocked
        Self::check_user_block(ctx, followed_user, following_user, "follow").await?;

        create_operation!(
            ctx,
            UserFollow,
            User,
            followed_user,
            User,
            following_user,
            created_by,
        )
    }
}
