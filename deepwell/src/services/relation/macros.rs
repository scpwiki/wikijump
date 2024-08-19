/*
 * services/relation/macros.rs
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

/// Implements the types and all non-add methods for a relation.
macro_rules! impl_relation {
    // Don't add create() method impl
    (
        $relation_type:ident,
        $dest_type:ident,
        $dest_name:ident,
        $from_type:ident,
        $from_name:ident,
        $data_type:ty,
        NO_CREATE_IMPL $(,)?
    ) => {
        paste! {
            // Methods
            impl RelationService {
                #[allow(dead_code)] // TEMP
                pub async fn [<get_ $relation_type:snake>](
                    ctx: &ServiceContext,
                    [<Get $relation_type>] {
                        $dest_name,
                        $from_name,
                    }: [<Get $relation_type>],
                ) -> Result<RelationModel> {
                    Self::get(
                        ctx,
                        RelationReference::Relationship {
                            relation_type: RelationType::$relation_type,
                            dest: RelationObject::$dest_type($dest_name),
                            from: RelationObject::$from_type($from_name),
                        },
                    )
                    .await
                }

                #[allow(dead_code)] // TEMP
                pub async fn [<get_optional_ $relation_type:snake>](
                    ctx: &ServiceContext,
                    [<Get $relation_type>] {
                        $dest_name,
                        $from_name,
                    }: [<Get $relation_type>],
                ) -> Result<Option<RelationModel>> {
                    Self::get_optional(
                        ctx,
                        RelationReference::Relationship {
                            relation_type: RelationType::$relation_type,
                            dest: RelationObject::$dest_type($dest_name),
                            from: RelationObject::$from_type($from_name),
                        },
                    )
                    .await
                }

                #[allow(dead_code)] // TEMP
                pub async fn [<$relation_type:snake _exists>](
                    ctx: &ServiceContext,
                    [<Get $relation_type>] {
                        $dest_name,
                        $from_name,
                    }: [<Get $relation_type>],
                ) -> Result<bool> {
                    Self::exists(
                        ctx,
                        RelationReference::Relationship {
                            relation_type: RelationType::$relation_type,
                            dest: RelationObject::$dest_type($dest_name),
                            from: RelationObject::$from_type($from_name),
                        },
                    )
                    .await
                }

                #[allow(dead_code)] // TEMP
                pub async fn [<remove_ $relation_type:snake>](
                    ctx: &ServiceContext,
                    [<Remove $relation_type>] {
                        $dest_name,
                        $from_name,
                        removed_by,
                    }: [<Remove $relation_type>],
                ) -> Result<RelationModel> {
                    Self::remove(
                        ctx,
                        RelationReference::Relationship {
                            relation_type: RelationType::$relation_type,
                            dest: RelationObject::$dest_type($dest_name),
                            from: RelationObject::$from_type($from_name),
                        },
                        removed_by,
                    ).await
                }

                // TODO paginate
                #[allow(dead_code)] // TEMP
                pub async fn [<get_ $relation_type:snake _history>](
                    ctx: &ServiceContext,
                    [<Get $relation_type>] {
                        $dest_name,
                        $from_name,
                    }: [<Get $relation_type>],
                ) -> Result<Vec<RelationModel>> {
                    Self::get_history(
                        ctx,
                        RelationType::$relation_type,
                        RelationObject::$dest_type($dest_name),
                        RelationObject::$from_type($from_name),
                    )
                    .await
                }

                // TODO paginate
                #[allow(dead_code)] // TEMP
                pub async fn [<get_ $relation_type:snake _entries>](
                    ctx: &ServiceContext,
                    object: RelationObject,
                    direction: RelationDirection,
                ) -> Result<Vec<RelationModel>> {
                    Self::get_entries(
                        ctx,
                        RelationType::$relation_type,
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
            pub struct [<Create $relation_type>] {
                pub $dest_name: i64,
                pub $from_name: i64,
                pub metadata: $data_type,
                pub created_by: i64,
            }

            #[derive(Deserialize, Debug, Copy, Clone)]
            pub struct [<Get $relation_type>] {
                pub $dest_name: i64,
                pub $from_name: i64,
            }

            #[derive(Deserialize, Debug, Copy, Clone)]
            pub struct [<Remove $relation_type>] {
                pub $dest_name: i64,
                pub $from_name: i64,
                pub removed_by: i64,
            }
        }
    };

    // Add create() method impl
    (
        $relation_type:ident,
        $dest_type:ident,
        $dest_name:ident,
        $from_type:ident,
        $from_name:ident,
        $data_type:ty $(,)?
    ) => {
        impl_relation!(
            $relation_type,
            $dest_type,
            $dest_name,
            $from_type,
            $from_name,
            $data_type,
            NO_CREATE_IMPL,
        );

        paste! {
            impl RelationService {
                #[allow(dead_code)] // TEMP
                pub async fn [<create_ $relation_type:snake>](
                    ctx: &ServiceContext,
                    [<Create $relation_type>] {
                        $dest_name,
                        $from_name,
                        created_by,
                        metadata,
                    }: [<Create $relation_type>],
                ) -> Result<()> {
                    create_operation!(
                        ctx,
                        $relation_type,
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

// TODO: change to create-or-edit kind of thing?
/// Macro which runs the actual `create()` call for the relation.
macro_rules! create_operation {
    (
        $ctx:expr,
        $relation_type:ident,
        $dest_type:ident,
        $dest_name:ident,
        $from_type:ident,
        $from_name:ident,
        $created_by:expr,
        $metadata:expr $(,)?
    ) => {{
        Self::create(
            $ctx,
            RelationType::$relation_type,
            RelationObject::$dest_type($dest_name),
            RelationObject::$from_type($from_name),
            $created_by,
            $metadata,
        )
        .await?;
        Ok(())
    }};

    (
        $ctx:expr,
        $relation_type:ident,
        $dest_type:ident,
        $dest_name:ident,
        $from_type:ident,
        $from_name:ident,
        $created_by:expr $(,)?
    ) => {
        create_operation!(
            $ctx,
            $relation_type,
            $dest_type,
            $dest_name,
            $from_type,
            $from_name,
            $created_by,
            &(),
        )
    };
}
