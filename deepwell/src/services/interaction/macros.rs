/*
 * services/interaction/macros.rs
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

/// Implements the types and all non-add methods for an interaction.
macro_rules! impl_interaction {
    // Don't add create() method impl
    (
        $interaction_type:ident,
        $dest_type:ident,
        $dest_name:ident,
        $from_type:ident,
        $from_name:ident,
        $data_type:ty,
        NO_CREATE_IMPL $(,)?
    ) => {
        paste! {
            // Methods
            impl InteractionService {
                pub async fn [<get_ $interaction_type:snake>](
                    ctx: &ServiceContext<'_>,
                    [<Get $interaction_type>] {
                        $dest_name,
                        $from_name,
                    }: [<Get $interaction_type>],
                ) -> Result<bool> {
                    Self::exists(
                        ctx,
                        InteractionReference::Relationship {
                            interaction_type: InteractionType::$interaction_type,
                            dest: InteractionObject::$dest_type($dest_name),
                            from: InteractionObject::$from_type($from_name),
                        },
                    )
                    .await
                }

                pub async fn [<remove_ $interaction_type:snake>](
                    ctx: &ServiceContext<'_>,
                    [<Remove $interaction_type>] {
                        $dest_name,
                        $from_name,
                        removed_by,
                    }: [<Remove $interaction_type>],
                ) -> Result<()> {
                    Self::remove(
                        ctx,
                        InteractionReference::Relationship {
                            interaction_type: InteractionType::$interaction_type,
                            dest: InteractionObject::$dest_type($dest_name),
                            from: InteractionObject::$from_type($from_name),
                        },
                        removed_by,
                    ).await
                }

                // TODO paginate
                pub async fn [<get_ $interaction_type:snake _history>](
                    ctx: &ServiceContext<'_>,
                    [<Get $interaction_type>] {
                        $dest_name,
                        $from_name,
                    }: [<Get $interaction_type>],
                ) -> Result<Vec<InteractionModel>> {
                    Self::get_history(
                        ctx,
                        InteractionType::$interaction_type,
                        InteractionObject::$dest_type($dest_name),
                        InteractionObject::$from_type($from_name),
                    )
                    .await
                }

                // TODO paginate
                pub async fn [<get_ $interaction_type:snake _entries>](
                    ctx: &ServiceContext<'_>,
                    object: InteractionObject,
                    direction: InteractionDirection,
                ) -> Result<Vec<InteractionModel>> {
                    Self::get_entries(
                        ctx,
                        InteractionType::$interaction_type,
                        object,
                        direction,
                    )
                    .await
                }
            }

            // Data types

            // TODO: Unfortunately, currently, despite my best efforts, we are not able to
            //       differentiate in the macro between () and other types, thus allowing us
            //       to exclude the metadata field if it's nothing.
            //
            //       Properly fixing this will likely require a proc-macro. Which is annoying.
            #[derive(Deserialize, Debug, Clone)]
            #[serde(rename_all = "camelCase")]
            pub struct [<Create $interaction_type>] {
                pub $dest_name: i64,
                pub $from_name: i64,
                pub metadata: $data_type,
                pub created_by: i64,
            }

            #[derive(Deserialize, Debug, Copy, Clone)]
            #[serde(rename_all = "camelCase")]
            pub struct [<Get $interaction_type>] {
                pub $dest_name: i64,
                pub $from_name: i64,
            }

            #[derive(Deserialize, Debug, Copy, Clone)]
            #[serde(rename_all = "camelCase")]
            pub struct [<Remove $interaction_type>] {
                pub $dest_name: i64,
                pub $from_name: i64,
                pub removed_by: i64,
            }
        }
    };

    // Add create() method impl
    (
        $interaction_type:ident,
        $dest_type:ident,
        $dest_name:ident,
        $from_type:ident,
        $from_name:ident,
        $data_type:ty $(,)?
    ) => {
        impl_interaction!(
            $interaction_type,
            $dest_type,
            $dest_name,
            $from_type,
            $from_name,
            $data_type,
            NO_CREATE_IMPL,
        );

        paste! {
            impl InteractionService {
                pub async fn [<create_ $interaction_type:snake>](
                    ctx: &ServiceContext<'_>,
                    [<Create $interaction_type>] {
                        $dest_name,
                        $from_name,
                        created_by,
                        metadata,
                    }: [<Create $interaction_type>],
                ) -> Result<()> {
                    create_operation!(
                        ctx,
                        $interaction_type,
                        $dest_type,
                        $dest_name,
                        $from_type,
                        $from_name,
                        created_by,
                        &metadata,
                    )
                }
            }
        }
    };
}

/// Macro which runs the actual `create()` call for the interaction.
macro_rules! create_operation {
    (
        $ctx:expr,
        $interaction_type:ident,
        $dest_type:ident,
        $dest_name:ident,
        $from_type:ident,
        $from_name:ident,
        $created_by:expr,
        $metadata:expr $(,)?
    ) => {{
        Self::create(
            $ctx,
            InteractionType::$interaction_type,
            InteractionObject::$dest_type($dest_name),
            InteractionObject::$from_type($from_name),
            $created_by,
            $metadata,
        )
        .await?;
        Ok(())
    }};

    (
        $ctx:expr,
        $interaction_type:ident,
        $dest_type:ident,
        $dest_name:ident,
        $from_type:ident,
        $from_name:ident,
        $created_by:expr $(,)?
    ) => {
        create_operation!(
            $ctx,
            $interaction_type,
            $dest_type,
            $dest_name,
            $from_type,
            $from_name,
            $created_by,
            &(),
        )
    };
}
