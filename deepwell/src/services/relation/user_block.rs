/*
 * services/relation/user_block.rs
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct UserBlockData {
    pub reason: String,
}

impl_relation!(
    UserBlock,
    User,
    blocked_user,
    User,
    blocking_user,
    UserBlockData,
    NO_CREATE_IMPL,
);

impl RelationService {
    #[allow(dead_code)] // TEMP
    pub async fn create_user_block(
        ctx: &ServiceContext,
        CreateUserBlock {
            blocked_user,
            blocking_user,
            created_by,
            metadata,
        }: CreateUserBlock,
    ) -> Result<()> {
        // Never reject a block, even if already blocked the other way.

        // Unfollow, remove contacts, etc., both ways
        try_join!(
            Self::remove_user_follow(
                ctx,
                RemoveUserFollow {
                    followed_user: blocked_user,
                    following_user: blocking_user,
                    removed_by: created_by,
                },
            ),
            Self::remove_user_follow(
                ctx,
                RemoveUserFollow {
                    followed_user: blocking_user,
                    following_user: blocked_user,
                    removed_by: created_by,
                },
            ),
            // TODO add user_contact
            // TODO add user_contact_request
        )?;

        create_operation!(
            ctx,
            UserBlock,
            User,
            blocked_user,
            User,
            blocking_user,
            created_by,
            &metadata,
        )
    }

    /// Helper method for rejecting an relation if either user in a pair has blocked the other.
    pub async fn check_user_block(
        ctx: &ServiceContext,
        user_id_1: i64,
        user_id_2: i64,
        action: &str,
    ) -> Result<()> {
        macro_rules! obj {
            ($first:expr, $second:expr $(,)?) => {
                GetUserBlock {
                    blocked_user: $first,
                    blocking_user: $second,
                }
            };
        }

        if Self::user_block_exists(ctx, obj!(user_id_1, user_id_2)).await?
            || Self::user_block_exists(ctx, obj!(user_id_2, user_id_1)).await?
        {
            error!("User ID {user_id_1} cannot {action} user ID {user_id_2} because there is a block");
            return Err(Error::UserBlockedUser);
        }

        Ok(())
    }
}
