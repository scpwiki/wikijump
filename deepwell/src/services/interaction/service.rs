/*
 * services/interaction/service.rs
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

use paste::paste;

// Macros

macro_rules! impl_methods {
    (
        $name:ident,
        $interaction_type:ident,
        $dest_type:ident,
        $from_type:ident,
        $data_type:ty,
        $before_add:ident $(,)?
    ) => {
        paste! {
            pub async fn [<get_ $name>](
                ctx: &ServiceContext<'_>,
                dest: i64,
                from: i64,
            ) -> Result<bool> {
                Self::exists(
                    ctx,
                    InteractionReference::Relationship {
                        interaction_type: InteractionType::$interaction_type,
                        dest: InteractionObject::$dest_type(dest),
                        from: InteractionObject::$from_type(from),
                    },
                )
                .await
            }

            pub async fn [<add_ $name>]<M>(
                ctx: &ServiceContext<'_>,
                dest: i64,
                from: i64,
                created_by: i64,
                metadata: M,
            ) -> Result<InteractionModel>
                where M: AsRef<$data_type>,
            {
                Self::$before_add(ctx, dest, from, created_by).await?;

                Self::add(
                    ctx,
                    InteractionType::$interaction_type,
                    InteractionObject::$dest_type(dest),
                    InteractionObject::$from_type(from),
                    created_by,
                    metadata.as_ref(),
                )
                .await
            }

            pub async fn [<remove_ $name>](
                ctx: &ServiceContext<'_>,
                dest: i64,
                from: i64,
                deleted_by: i64,
            ) -> Result<()> {
                Self::remove(
                    ctx,
                    InteractionReference::Relationship {
                        interaction_type: InteractionType::$interaction_type,
                        dest: InteractionObject::$dest_type(dest),
                        from: InteractionObject::$from_type(dest),
                    },
                    deleted_by,
                ).await
            }
        }
    };

    (
        $name:ident,
        $interaction_type:ident,
        $dest_type:ident,
        $from_type:ident,
        $data_type:ty,
        $before_add:ident $(,)?
    ) => {
        impl_methods!(
            $name,
            $interaction_type,
            $dest_type,
            $from_type,
            $data_type,
            $before_add,
        );
    };

    (
        $name:ident,
        $interaction_type:ident,
        $dest_type:ident,
        $from_type:ident,
        $data_type:ty $(,)?
    ) => {
        impl_methods!(
            $name,
            $interaction_type,
            $dest_type,
            $from_type,
            $data_type,
            null_hook,
        );
    };

    (
        $name:ident,
        $interaction_type:ident,
        $dest_type:ident,
        $from_type:ident $(,)?
    ) => {
        impl_methods!(
            $name,
            $interaction_type,
            $dest_type,
            $from_type,
            (),
            null_hook,
        );
    };
}

// Service

use super::prelude::*;
use crate::models::interaction::{
    self, Entity as Interaction, Model as InteractionModel,
};
use serde::Serialize;

#[derive(Debug)]
pub struct InteractionService;

impl InteractionService {
    pub async fn add<M: Serialize>(
        ctx: &ServiceContext<'_>,
        interaction_type: InteractionType,
        dest: InteractionObject,
        from: InteractionObject,
        created_by: i64,
        metadata: &M,
    ) -> Result<InteractionModel> {
        tide::log::debug!(
            "Adding interaction for {dest:?} ← {interaction_type:?} ← {from:?}",
        );

        // Delete previous interaction, if present
        if let Some(interaction) = Self::get_optional(
            ctx,
            InteractionReference::Relationship {
                interaction_type,
                dest,
                from,
            },
        )
        .await?
        {
            Self::remove(
                ctx,
                InteractionReference::Id(interaction.interaction_id),
                created_by,
            )
            .await?;
        }

        // Insert new interaction
        let (dest_type, dest_id) = dest.into();
        let (from_type, from_id) = from.into();
        interaction_type.types().check(dest_type, from_type);

        let metadata = serde_json::to_value(metadata)?;
        let model = interaction::ActiveModel {
            interaction_type: Set(str!(interaction_type.value())),
            dest_type: Set(dest_type),
            dest_id: Set(dest_id),
            from_type: Set(from_type),
            from_id: Set(from_id),
            metadata: Set(metadata),
            created_by: Set(created_by),
            ..Default::default()
        };

        let txn = ctx.transaction();
        let interaction = model.insert(txn).await?;
        Ok(interaction)
    }

    pub async fn remove(
        ctx: &ServiceContext<'_>,
        reference: InteractionReference,
        deleted_by: i64,
    ) -> Result<()> {
        tide::log::debug!("Removing interaction for {reference:?}");

        let txn = ctx.transaction();
        let interaction_id = Self::get_id(ctx, reference).await?;
        let model = interaction::ActiveModel {
            interaction_id: Set(interaction_id),
            deleted_at: Set(Some(now())),
            deleted_by: Set(Some(deleted_by)),
            ..Default::default()
        };

        model.update(txn).await?;
        Ok(())
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        reference: InteractionReference,
    ) -> Result<Option<InteractionModel>> {
        tide::log::debug!("Getting interaction for {reference:?}");

        let txn = ctx.transaction();
        let interaction = Interaction::find()
            .filter(
                Condition::all()
                    .add(reference.condition())
                    .add(interaction::Column::DeletedAt.is_null()),
            )
            .one(txn)
            .await?;

        Ok(interaction)
    }

    /// Gets the interaction ID from a reference, looking up if necessary.
    pub async fn get_id(
        ctx: &ServiceContext<'_>,
        reference: InteractionReference,
    ) -> Result<i64> {
        match reference {
            InteractionReference::Id(interaction_id) => Ok(interaction_id),
            InteractionReference::Relationship { .. } => {
                let InteractionModel { interaction_id, .. } =
                    Self::get(ctx, reference).await?;
                Ok(interaction_id)
            }
        }
    }

    pub async fn get(
        ctx: &ServiceContext<'_>,
        reference: InteractionReference,
    ) -> Result<InteractionModel> {
        find_or_error(Self::get_optional(ctx, reference)).await
    }

    pub async fn exists(
        ctx: &ServiceContext<'_>,
        reference: InteractionReference,
    ) -> Result<bool> {
        Self::get_optional(ctx, reference)
            .await
            .map(|interaction| interaction.is_some())
    }

    // TODO paginate
    pub async fn get_history(
        ctx: &ServiceContext<'_>,
        interaction_type: InteractionType,
        source: InteractionObject,
        target: InteractionObject,
    ) -> Result<Vec<InteractionModel>> {
        tide::log::debug!(
            "Getting history of interactions for {source:?} / {interaction_type:?} / {target:?}",
        );

        let txn = ctx.transaction();
        let interactions = Interaction::find()
            .filter(interaction_condition(interaction_type, source, target))
            .order_by_asc(interaction::Column::CreatedAt)
            .all(txn)
            .await?;

        Ok(interactions)
    }

    // Methods

    impl_methods!(site_ban, SiteBan, Site, User, SiteBanData, pre_add_site_ban);

    async fn pre_add_site_ban(
        ctx: &ServiceContext<'_>,
        dest: i64,
        from: i64,
        created_by: i64,
    ) -> Result<()> {
        Self::remove_site_member(ctx, dest, from, created_by).await?;
        // TODO: remove roles?

        Ok(())
    }

    impl_methods!(
        site_member,
        SiteMember,
        Site,
        User,
        SiteMemberData,
        pre_add_site_member,
    );

    async fn pre_add_site_member(
        ctx: &ServiceContext<'_>,
        dest: i64,
        from: i64,
        created_by: i64,
    ) -> Result<()> {
        // Cannot join if banned
        Self::check_site_ban(ctx, dest, from, "join").await
    }

    impl_methods!(page_watch, PageWatch, Page, User);
    impl_methods!(user_follow, UserFollow, User, User, (), pre_add_user_follow);

    async fn pre_add_user_follow(
        ctx: &ServiceContext<'_>,
        dest: i64,
        from: i64,
        created_by: i64,
    ) -> Result<()> {
        // Cannot follow if blocked
        Self::check_user_block(ctx, dest, from, "follow").await
    }

    impl_methods!(
        user_contact,
        UserContact,
        User,
        User,
        (),
        pre_add_user_contact,
    );

    async fn pre_add_user_contact(
        ctx: &ServiceContext<'_>,
        dest: i64,
        from: i64,
        created_by: i64,
    ) -> Result<()> {
        // Cannot follow if blocked
        Self::check_user_block(ctx, dest, from, "contact").await
    }

    impl_methods!(user_block, UserBlock, User, User, (), pre_add_user_block);

    async fn pre_add_user_block(
        ctx: &ServiceContext<'_>,
        dest: i64,
        from: i64,
        created_by: i64,
    ) -> Result<()> {
        // Unfollow, remove contacts, etc., both ways

        Self::remove_user_follow(ctx, dest, from, created_by).await?;
        Self::remove_user_follow(ctx, from, dest, created_by).await?;

        Self::remove_user_contact(ctx, dest, from, created_by).await?;
        Self::remove_user_contact(ctx, from, dest, created_by).await?;

        Ok(())
    }

    #[inline]
    async fn null_hook(_: &ServiceContext<'_>, _: i64, _: i64, _: i64) -> Result<()> {
        Ok(())
    }

    async fn check_site_ban(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        user_id: i64,
        action: &str,
    ) -> Result<()> {
        if Self::get_site_ban(ctx, site_id, user_id).await? {
            tide::log::error!("User ID {user_id} cannot {action} site ID {site_id} because they are banned");
            return Err(Error::SiteBlockedUser);
        }

        Ok(())
    }

    async fn check_user_block(
        ctx: &ServiceContext<'_>,
        user_id_1: i64,
        user_id_2: i64,
        action: &str,
    ) -> Result<()> {
        if Self::get_user_block(ctx, user_id_1, user_id_2).await?
            || Self::get_user_block(ctx, user_id_2, user_id_1).await?
        {
            tide::log::error!("User ID {user_id_1} cannot {action} user ID {user_id_2} because there is a block");
            return Err(Error::UserBlockedUser);
        }

        Ok(())
    }
}
