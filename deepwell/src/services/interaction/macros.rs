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

/// Helper macro for generating types. Do not use directly.
macro_rules! __generate_add_type {
    // Doesn't have data
    (
        $interaction_type:ident,
        $dest_name:ident,
        $from_name:ident,
        () $(,)?
    ) => {
        paste! {
            #[derive(Deserialize, Debug, Copy, Clone)]
            pub struct [<Create $interaction_type>] {
                pub $dest_name: i64,
                pub $from_name: i64,
            }
        }
    };

    // Has data
    (
        $interaction_type:ident,
        $dest_name:ident,
        $from_name:ident,
        $data_type:ty $(,)?
    ) => {
        paste! {
            #[derive(Deserialize, Debug, Clone)]
            pub struct [<Create $interaction_type>] {
                pub $dest_name: i64,
                pub $from_name: i64,
                pub metadata: $data_type,
            }
        }
    };
}

/// Helper macro for generating the add method. Do not use directly.
macro_rules! __generate_add_method {
    // Don't implement
    (
        $interaction_type:ident,
        $add_method_name:ident,
        $dest_name:ident,
        $from_name:ident,
        $data_type:ty,
        false $(,)?
    ) => {};

    // Implement method, doesn't have data
    (
        $interaction_type:ident,
        $add_method_name:ident,
        $dest_name:ident,
        $from_name:ident,
        (),
        true $(,)?
    ) => {
        paste! {
            pub async fn $add_method_name(
                ctx: &ServiceContext<'_>,
                [<Add $interaction_type>] {
                    $dest_name,
                    $from_name,
                    created_by,
                }: [<Add $interaction_type>],
            ) -> Result<()> {
                add_operation!($interaction_type, $dest_name, $from_name, created_by)
            }
        }
    };

    // Implement method, has data
    (
        $interaction_type:ident,
        $add_method_name:ident,
        $dest_name:ident,
        $from_name:ident,
        $data_type:ty,
        true $(,)?
    ) => {
        paste! {
            pub async fn $add_name(
                ctx: &ServiceContext<'_>,
                [<Add $interaction_type>] {
                    $dest_name,
                    $from_name,
                    created_by,
                    metadata,
                }: [<Add $interaction_type>],
            ) -> Result<()> {
                add_operation!($interaction_type, $dest_name, $from_name, created_by, metadata)
            }
        }
    };
}

/// Implements the types and all non-add methods for an interaction.
macro_rules! impl_interaction {
    (
        $interaction_type:ident,
        $name:ident,
        $dest_type:ident,
        $dest_name:ident,
        $from_type:ident,
        $from_name:ident,
        $impl_add_method:expr,
        $data_type:ty $(,)?
    ) => {
        paste! {
            // Methods
            impl InteractionService {
                pub async fn [<get_ $name>](
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

                pub async fn [<remove_ $name>](
                    ctx: &ServiceContext<'_>,
                    [<Delete $interaction_type>] {
                        $dest_name,
                        $from_name,
                        deleted_by,
                    }: [<Delete $interaction_type>],
                ) -> Result<()> {
                    Self::remove(
                        ctx,
                        InteractionReference::Relationship {
                            interaction_type: InteractionType::$interaction_type,
                            dest: InteractionObject::$dest_type($dest_name),
                            from: InteractionObject::$from_type($from_name),
                        },
                        deleted_by,
                    ).await
                }

                // TODO paginate
                pub async fn [<get_ $name _history>](
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
                pub async fn [<get_ $name _entries>](
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

                // Separate macro so we can conditionally implement the add method
                __generate_add_method!([<add_ $name>], $dest_name, $from_name, $data_type, $impl_add_method);
            }

            // Data types
            #[derive(Deserialize, Debug, Copy, Clone)]
            pub struct [<Get $name>] {
                pub $dest_name: i64,
                pub $from_name: i64,
            }

            #[derive(Deserialize, Debug, Copy, Clone)]
            pub struct [<Delete $name>] {
                pub $dest_name: i64,
                pub $from_name: i64,
                pub deleted_by: i64,
            }

            // Separate macro so we can differentiate based on $data_type
            __generate_add_type!($name, $dest_name, $from_name, $data_type);
        }
    };

    (
        $interaction_type:ident,
        $name:ident,
        $dest_type:ident,
        $dest_name:ident,
        $from_type:ident,
        $from_name:ident,
        $impl_add_method:expr $(,)?
    ) => {
        impl_interaction!(
            $interaction_type,
            $name,
            $dest_type,
            $dest_name,
            $from_type,
            $from_name,
            $impl_add_method,
            (),
        );
    };
}

macro_rules! add_operation {
    (
        $interaction_type:ident,
        $dest_type:ident,
        $from_type:ident,
        $created_by:expr,
        $metadata:expr $(,)?
    ) => {{
        Self::add(
            ctx,
            InteractionType::$interaction_type,
            InteractionObject::$dest_type(dest),
            InteractionObject::$from_type(from),
            $created_by,
            $metadata,
        )
        .await?

        Ok(())
    }};

    (
        $interaction_type:ident,
        $dest_type:ident,
        $from_type:ident,
        $created_by:expr $(,)?
    ) => {
        add_operation!($interaction_type, $dest_type, $from_type, $created_by, ())
    };
}
