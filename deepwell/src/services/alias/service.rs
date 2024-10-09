/*
 * services/alias/service.rs
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
use crate::models::alias::{self, Entity as Alias, Model as AliasModel};
use crate::models::sea_orm_active_enums::AliasType;
use crate::models::site::{self, Entity as Site};
use crate::models::user::{self, Entity as User};
use crate::services::filter::{FilterClass, FilterType};
use crate::services::{FilterService, SiteService, UserService};
use crate::utils::get_regular_slug;
use crate::types::Reference;

#[derive(Debug)]
pub struct AliasService;

impl AliasService {
    /// Creates a new site or user alias.
    pub async fn create(
        ctx: &ServiceContext<'_>,
        input: CreateAlias,
    ) -> Result<CreateAliasOutput> {
        Self::create2(ctx, input, true).await
    }

    /// Creates a new site or user alias, but can be instructed to not perform row checks.
    ///
    /// This method should only be invoked when the corresponding site/user
    /// row has not been updated, if in doubt use `AliasService::create()`.
    ///
    /// The caller is responsible for calling `AliasService::verify()` after
    /// all its database changes have been made.
    pub(crate) async fn create2(
        ctx: &ServiceContext<'_>,
        CreateAlias {
            slug,
            alias_type,
            target_id,
            created_by,
            bypass_filter,
        }: CreateAlias,
        verify: bool,
    ) -> Result<CreateAliasOutput> {
        let txn = ctx.transaction();
        let slug = get_regular_slug(slug);

        info!("Creating {alias_type:?} alias with slug '{slug}'");

        // Perform filter validation
        if !bypass_filter {
            Self::run_filter(ctx, alias_type, &slug).await?;
        }

        // Check for existence and conflicts
        //
        // If "target_id" does not refer to an actual object of that type,
        // we should return an error.
        //
        // Then we check that the new slug doesn't already exist.
        // This also checks aliases, though we also verify down below that
        // it actually finds conflicts properly.
        //
        // If the alias is for a user, also ensures that it is at least
        // the minimum name length in bytes.
        match alias_type {
            AliasType::Site => {
                if !SiteService::exists(ctx, Reference::Id(target_id)).await? {
                    error!(
                        "No target site with ID {target_id} exists, cannot create alias",
                    );

                    return Err(Error::SiteNotFound);
                }

                if verify && SiteService::exists(ctx, Reference::Slug(cow!(slug))).await?
                {
                    error!(
                        "Site with conflicting slug '{slug}' already exists, cannot create alias",
                    );

                    return Err(Error::SiteExists);
                }
            }
            AliasType::User => {
                if !UserService::exists(ctx, Reference::Id(target_id)).await? {
                    error!(
                        "No target user with ID {target_id} exists, cannot create alias",
                    );

                    return Err(Error::UserNotFound);
                }

                if verify && UserService::exists(ctx, Reference::Slug(cow!(slug))).await?
                {
                    error!(
                        "User with conflicting slug '{slug}' already exists, cannot create alias",
                    );

                    return Err(Error::UserExists);
                }

                if slug.len() < ctx.config().minimum_name_bytes {
                    error!(
                        "User's name is not long enough ({} < {})",
                        slug.len(),
                        ctx.config().minimum_name_bytes,
                    );

                    return Err(Error::UserNameTooShort);
                }
            }
        }

        // Insert new model
        let alias = alias::ActiveModel {
            alias_type: Set(alias_type),
            created_by: Set(created_by),
            target_id: Set(target_id),
            slug: Set(slug.clone()),
            ..Default::default()
        };

        let alias_id = Alias::insert(alias).exec(txn).await?.last_insert_id;

        // Perform verification
        if verify {
            Self::verify(ctx, alias_type, &slug).await?;
        }

        // Return
        Ok(CreateAliasOutput { alias_id, slug })
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        alias_type: AliasType,
        slug: &str,
    ) -> Result<Option<AliasModel>> {
        let txn = ctx.transaction();

        let alias = Alias::find()
            .filter(
                Condition::all()
                    .add(alias::Column::AliasType.eq(alias_type))
                    .add(alias::Column::Slug.eq(slug)),
            )
            .one(txn)
            .await?;

        Ok(alias)
    }

    #[inline]
    #[allow(dead_code)] // TEMP
    pub async fn get(
        ctx: &ServiceContext<'_>,
        alias_type: AliasType,
        slug: &str,
    ) -> Result<AliasModel> {
        find_or_error!(Self::get_optional(ctx, alias_type, slug), Alias)
    }

    #[inline]
    pub async fn exists(
        ctx: &ServiceContext<'_>,
        alias_type: AliasType,
        slug: &str,
    ) -> Result<bool> {
        Self::get_optional(ctx, alias_type, slug)
            .await
            .map(|alias| alias.is_some())
    }

    pub async fn get_all(
        ctx: &ServiceContext<'_>,
        alias_type: AliasType,
        target_id: i64,
    ) -> Result<Vec<AliasModel>> {
        info!("Finding all {alias_type:?} aliases for ID {target_id}");

        let txn = ctx.transaction();
        let aliases = Alias::find()
            .filter(
                Condition::all()
                    .add(alias::Column::AliasType.eq(alias_type))
                    .add(alias::Column::TargetId.eq(target_id)),
            )
            .all(txn)
            .await?;

        Ok(aliases)
    }

    /// Used for when a user renames to an old slug.
    ///
    /// This takes the old user alias and renames the slug in-place, without having to do
    /// `create()` / `delete()` (which runs into a dependency issue as `create()` checks
    /// `SiteService` or `UserService` to ensure that a target object doesn't already
    /// exist with that name.
    ///
    /// The database uniqueness constraint enforces that the `slug` doesn't collide with another
    /// alias of the same type.
    pub async fn swap(
        ctx: &ServiceContext<'_>,
        alias_id: i64,
        new_slug: &str,
    ) -> Result<()> {
        let txn = ctx.transaction();

        info!(
            "Swapping user alias ID {} to use slug '{}'",
            alias_id, new_slug,
        );

        let model = alias::ActiveModel {
            created_at: Set(now()), // instead of deleting and recreating, we just pretend it was
            alias_id: Set(alias_id),
            slug: Set(str!(new_slug)),
            ..Default::default()
        };

        model.update(txn).await?;
        Ok(())
    }

    /// Removes all aliases for this target.
    ///
    /// # Returns
    /// The number of deleted aliases.
    pub async fn remove_all(
        ctx: &ServiceContext<'_>,
        alias_type: AliasType,
        target_id: i64,
    ) -> Result<u64> {
        let txn = ctx.transaction();

        info!("Removing all {alias_type:?} aliases for target ID {target_id}");

        let DeleteResult { rows_affected } = Alias::delete_many()
            .filter(
                Condition::all()
                    .add(alias::Column::AliasType.eq(alias_type))
                    .add(alias::Column::TargetId.eq(target_id)),
            )
            .exec(txn)
            .await?;

        debug!(
            "{rows_affected} {alias_type:?} aliases for target ID {target_id} were removed",
        );

        Ok(rows_affected)
    }

    /// Verifies that the `user` and `user_alias` tables are consistent.
    ///
    /// These tables have a uniqueness invariant wherein a slug is only
    /// present in at most one of these two tables, but not both.
    pub async fn verify(
        ctx: &ServiceContext<'_>,
        alias_type: AliasType,
        slug: &str,
    ) -> Result<()> {
        info!("Verifying target and alias table consistency for slug '{slug}'",);

        let txn = ctx.transaction();
        let alias_fut = Alias::find()
            .filter(
                Condition::all()
                    .add(alias::Column::AliasType.eq(alias_type))
                    .add(alias::Column::Slug.eq(slug)),
            )
            .one(txn);

        // Check the target and alias result.
        //
        // If they're both present, then somewhere we have a bug,
        // since the invariant is not being upheld, so we panic.
        match alias_type {
            AliasType::Site => {
                let (site_result, alias_result) = try_join!(
                    Site::find()
                        .filter(
                            Condition::all()
                                .add(site::Column::Slug.eq(slug))
                                .add(site::Column::DeletedAt.is_null())
                        )
                        .one(txn),
                    alias_fut,
                )?;

                if let (Some(site), Some(alias)) = (site_result, alias_result) {
                    error!(
                        "Consistency error! Both site and alias tables have the slug '{}'",
                        slug,
                    );

                    panic!(
                        "Slug appears as both a site and an alias!\nSite: {site:#?}\nAlias: {alias:#?}",
                    );
                }
            }
            AliasType::User => {
                let (user_result, alias_result) = try_join!(
                    User::find()
                        .filter(
                            Condition::all()
                                .add(user::Column::Slug.eq(slug))
                                .add(user::Column::DeletedAt.is_null())
                        )
                        .one(txn),
                    alias_fut,
                )?;

                if let (Some(user), Some(alias)) = (user_result, alias_result) {
                    error!(
                        "Consistency error! Both user and alias tables have the slug '{}'",
                        slug,
                    );

                    panic!(
                        "Slug appears as both a user and an alias!\nUser: {user:#?}\nAlias: {alias:#?}",
                    );
                }
            }
        }

        Ok(())
    }

    async fn run_filter(
        ctx: &ServiceContext<'_>,
        alias_type: AliasType,
        slug: &str,
    ) -> Result<()> {
        info!("Checking user alias data against filters...");

        let filter_type = match alias_type {
            AliasType::User => FilterType::User,
            AliasType::Site => {
                // No filter with this type, skip verification
                debug!("No need to run filter verification for site alias");
                return Ok(());
            }
        };

        let filter_matcher =
            FilterService::get_matcher(ctx, FilterClass::Platform, filter_type).await?;

        filter_matcher.verify(ctx, slug).await?;
        Ok(())
    }
}
