/*
 * services/relation/user_block.rs
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct UserBlockData {
    pub reason: String,
}

impl_relation! {
    name => UserBlock,
    dest => blocked_user: User,
    from => blocking_user: User,
    data => UserBlockData,
    create_fn => false,
}

impl RelationService {
    #[allow(dead_code)] // TEMP
    pub async fn create_user_block(
        ctx: &ServiceContext<'_>,
        CreateUserBlock {
            blocked_user,
            blocking_user,
            created_by,
            metadata,
        }: CreateUserBlock,
    ) -> Result<()> {
        // Never reject a block, even if already blocked the other way.

        let make_error = || {
            Error::new(
                format!(
                    "failed to create user block ({} is blocking {}), as created by {}",
                    blocking_user, blocked_user, created_by,
                ),
                ErrorType::UserBlockRelation,
            )
        };

        // Unfollow, remove contacts, etc., both ways
        let (result1, result2) = join!(
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
        );
        raise_multiple!(result1, result2; make_error);

        Self::create_user_block_inner(
            ctx,
            CreateUserBlockInner {
                blocked_user,
                blocking_user,
                created_by,
                metadata: &metadata,
            },
        )
        .await
        .or_raise(make_error)?;

        Ok(())
    }

    /// Helper method for rejecting an relation if either user in a pair has blocked the other.
    pub async fn check_user_block(
        ctx: &ServiceContext<'_>,
        user_id_1: i64,
        user_id_2: i64,
        action: &str,
    ) -> Result<()> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to check for user block between user ID {} ↔ user ID {}, cannot {}",
                    user_id_1, user_id_2, action,
                ),
                ErrorType::UserBlockRelation,
            )
        };

        macro_rules! obj {
            ($first:expr, $second:expr $(,)?) => {
                GetUserBlock {
                    blocked_user: $first,
                    blocking_user: $second,
                }
            };
        }

        macro_rules! check_user_block_exists {
            ($first:expr, $second:expr $(,)?) => {
                Self::user_block_exists(ctx, obj!($first, $second))
                    .await
                    .or_raise(make_error)?
            };
        }

        if check_user_block_exists!(user_id_1, user_id_2)
            || check_user_block_exists!(user_id_2, user_id_1)
        {
            error!(
                "User ID {user_id_1} cannot {action} user ID {user_id_2} because there is a block"
            );
            bail!(Error::new(
                format!(
                    "cannot {} because there is a block between user ID {} ↔ user ID {}",
                    action, user_id_1, user_id_2,
                ),
                ErrorType::UserBlockedUser,
            ));
        }

        Ok(())
    }
}
