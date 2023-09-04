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

use super::super::prelude::*;
use super::InteractionService;

impl InteractionService {
    // User blocks

    pub async fn block_user(
        ctx: &ServiceContext<'_>,
        source_user: i64,
        target_user: i64,
        created_by: i64,
    ) -> Result<()> {
        tide::log::info!(
            "Blocking user ID {target_user} on behalf of user ID {source_user}",
        );

        // TODO: unfollow user, remove from contacts, etc. both ways

        Self::add(
            ctx,
            InteractionType::Block,
            user!(source_user),
            user!(target_user),
            created_by,
            &(),
        )
        .await?;

        Ok(())
    }

    pub async fn unblock_user(
        ctx: &ServiceContext<'_>,
        source_user: i64,
        target_user: i64,
        deleted_by: i64,
    ) -> Result<()> {
        tide::log::info!(
            "Unblocking user ID {target_user} on behalf of user ID {source_user}",
        );

        Self::remove(
            ctx,
            InteractionReference::Relationship {
                interaction_type: InteractionType::Block,
                source: user!(source_user),
                target: user!(target_user),
            },
            deleted_by,
        )
        .await
    }

    pub async fn user_blocked(
        ctx: &ServiceContext<'_>,
        source_user: i64,
        target_user: i64,
    ) -> Result<bool> {
        tide::log::info!(
            "Checking if user ID {target_user} is blocked by user ID {source_user}",
        );

        Self::exists(
            ctx,
            InteractionReference::Relationship {
                interaction_type: InteractionType::Block,
                source: user!(source_user),
                target: user!(target_user),
            },
        )
        .await
    }

    // User follow

    pub async fn follow_user(
        ctx: &ServiceContext<'_>,
        source_user: i64,
        target_user: i64,
        created_by: i64,
    ) -> Result<()> {
        tide::log::info!(
            "Following user ID {target_user} on behalf of user ID {source_user}",
        );

        if Self::user_blocked(ctx, source_user, target_user).await? {
            tide::log::error!("Cannot add follow, user is blocked");
            return Err(Error::UserBlockedUser);
        }

        Self::add(
            ctx,
            InteractionType::Watch,
            user!(source_user),
            user!(target_user),
            created_by,
            &(),
        )
        .await?;

        Ok(())
    }

    pub async fn unfollow_user(
        ctx: &ServiceContext<'_>,
        source_user: i64,
        target_user: i64,
        deleted_by: i64,
    ) -> Result<()> {
        tide::log::info!(
            "Unfollowing user ID {target_user} on behalf of user ID {source_user}",
        );

        Self::remove(
            ctx,
            InteractionReference::Relationship {
                interaction_type: InteractionType::Watch,
                source: user!(source_user),
                target: user!(target_user),
            },
            deleted_by,
        )
        .await
    }

    pub async fn user_followed(
        ctx: &ServiceContext<'_>,
        source_user: i64,
        target_user: i64,
    ) -> Result<bool> {
        tide::log::info!(
            "Checking if user ID {target_user} is followed by user ID {source_user}",
        );

        Self::exists(
            ctx,
            InteractionReference::Relationship {
                interaction_type: InteractionType::Watch,
                source: user!(source_user),
                target: user!(target_user),
            },
        )
        .await
    }
}
