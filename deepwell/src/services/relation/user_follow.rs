/*
 * services/relation/user_follow.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
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

impl_relation! {
    name => UserFollow,
    dest => followed_user: User,
    from => following_user: User,
    create_fn => private,
}

impl RelationService {
    #[allow(dead_code)] // TEMP
    pub async fn create_user_follow(
        ctx: &ServiceContext<'_>,
        CreateUserFollow {
            followed_user,
            following_user,
            created_by,
        }: CreateUserFollow,
    ) -> Result<()> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to create user follow ({} is following {}), as created by {}",
                    following_user, followed_user, created_by,
                ),
                ErrorType::UserFollowRelation,
            )
        };

        // Cannot follow if blocked
        Self::check_user_block(ctx, followed_user, following_user, "follow")
            .await
            .or_raise(make_error)?;

        Self::create_user_follow_inner(
            ctx,
            CreateUserFollow {
                followed_user,
                following_user,
                created_by,
            },
        )
        .await
        .or_raise(make_error)?;

        Ok(())
    }
}
