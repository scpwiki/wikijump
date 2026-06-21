/*
 * services/alias/service.rs
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
use crate::models::alias::{self, Entity as Alias, Model as AliasModel};
use crate::models::site::{self, Entity as Site};
use crate::models::user::{self, Entity as User};
use crate::services::audit::{AuditEvent, AuditService, ObjectScope};
use crate::services::filter::{FilterClass, FilterType};
use crate::services::{FilterService, SiteService, UserService};
use crate::types::{AliasType, Reference};
use crate::utils::get_regular_slug;
use std::net::IpAddr;

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
            ip_address,
        }: CreateAlias,
        verify: bool,
    ) -> Result<CreateAliasOutput> {
        let txn = ctx.transaction();
        let slug = get_regular_slug(slug);

        info!("Creating {alias_type:?} alias with slug '{slug}'");

        let make_error = || {
            Error::new(
                format!(
                    "failed to create {:?} alias '{}' to target ID {}, created by user ID {}",
                    alias_type, slug, target_id, created_by,
                ),
                ErrorType::Alias,
            )
        };

        // Perform filter validation
        if !bypass_filter {
            Self::run_filter(ctx, alias_type, &slug, target_id, ip_address)
                .await
                .or_raise(make_error)?;
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
        // the minimum name length in bytes and chars.
        match alias_type {
            AliasType::Site => {
                if !SiteService::exists(ctx, Reference::Id(target_id))
                    .await
                    .or_raise(make_error)?
                {
                    error!(
                        "No target site with ID {} exists, cannot create alias",
                        target_id,
                    );
                    bail!(Error::new(
                        format!(
                            "cannot create alias '{}' to site ID {}, since that site does not exist",
                            slug, target_id,
                        ),
                        ErrorType::SiteNotFound
                    ));
                }

                if verify
                    && SiteService::exists(ctx, Reference::Slug(cow!(slug)))
                        .await
                        .or_raise(make_error)?
                {
                    error!(
                        "Site with conflicting slug '{}' already exists, cannot create alias",
                        slug,
                    );
                    bail!(Error::new(
                        format!(
                            "cannot create site alias '{}', as another site with that slug already exists",
                            target_id,
                        ),
                        ErrorType::SiteExists
                    ));
                }
            }
            AliasType::User => {
                if !UserService::exists(ctx, Reference::Id(target_id))
                    .await
                    .or_raise(make_error)?
                {
                    error!(
                        "No target user with ID {} exists, cannot create alias",
                        target_id,
                    );
                    bail!(Error::new(
                        format!(
                            "cannot create alias '{}' to user ID {}, since that user does not exist",
                            slug, target_id,
                        ),
                        ErrorType::UserNotFound,
                    ));
                }

                if verify
                    && UserService::exists(ctx, Reference::Slug(cow!(slug)))
                        .await
                        .or_raise(make_error)?
                {
                    error!(
                        "User with conflicting slug '{}' already exists, cannot create alias",
                        slug,
                    );
                    bail!(Error::new(
                        format!(
                            "cannot create suser alias '{}', as another user with that slug already exists",
                            target_id,
                        ),
                        ErrorType::SiteExists,
                    ));
                }

                // Do user name checks

                let config = ctx.config();
                if slug.len() < config.minimum_name_bytes {
                    error!(
                        "User's name is not long enough ({} < {} bytes)",
                        slug.len(),
                        config.minimum_name_bytes,
                    );
                    bail!(Error::new(
                        format!(
                            "cannot create user alias '{}', name is not long enough ({} < {} bytes)",
                            slug,
                            slug.len(),
                            config.minimum_name_bytes,
                        ),
                        ErrorType::UserNameTooShort
                    ));
                }

                let slug_chars = slug.chars().count();
                if slug_chars < config.minimum_name_chars {
                    error!(
                        "User's name is not long enough ({} < {} chars)",
                        slug_chars, config.minimum_name_chars,
                    );
                    bail!(Error::new(
                        format!(
                            "cannot create user alias '{}', name is not long enough ({} < {} chars)",
                            slug, slug_chars, config.minimum_name_chars
                        ),
                        ErrorType::UserNameTooShort,
                    ));
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

        let alias_id = Alias::insert(alias)
            .exec(txn)
            .await
            .or_raise(make_error)?
            .last_insert_id;

        // Perform verification
        if verify {
            Self::verify(ctx, alias_type, &slug)
                .await
                .or_raise(make_error)?;
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

        let make_error = || {
            Error::new(
                format!("failed to get {:?} alias '{}'", alias_type, slug),
                ErrorType::Alias,
            )
        };

        let alias = Alias::find()
            .filter(
                Condition::all()
                    .add(alias::Column::AliasType.eq(alias_type))
                    .add(alias::Column::Slug.eq(slug)),
            )
            .one(txn)
            .await
            .or_raise(make_error)?;

        Ok(alias)
    }

    #[inline]
    #[allow(dead_code)] // TEMP
    pub async fn get(
        ctx: &ServiceContext<'_>,
        alias_type: AliasType,
        slug: &str,
    ) -> Result<AliasModel> {
        find_or_error!(Self::get_optional(ctx, alias_type, slug), "alias", Alias)
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

        let make_error = || {
            Error::new(
                format!(
                    "failed to get all {:?} aliases for ID {}",
                    alias_type, target_id,
                ),
                ErrorType::Alias,
            )
        };

        let txn = ctx.transaction();
        let aliases = Alias::find()
            .filter(
                Condition::all()
                    .add(alias::Column::AliasType.eq(alias_type))
                    .add(alias::Column::TargetId.eq(target_id)),
            )
            .all(txn)
            .await
            .or_raise(make_error)?;

        Ok(aliases)
    }

    /// Used for when a site or user renames to an old slug.
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
        info!("Swapping alias ID {alias_id} to use new slug '{new_slug}'");

        let make_error = || {
            Error::new(
                format!(
                    "failed to swap alias ID {} with new slug '{}'",
                    alias_id, new_slug,
                ),
                ErrorType::Alias,
            )
        };

        let txn = ctx.transaction();
        let model = alias::ActiveModel {
            created_at: Set(now()), // instead of deleting and recreating, we just pretend it was
            alias_id: Set(alias_id),
            slug: Set(str!(new_slug)),
            ..Default::default()
        };

        model.update(txn).await.or_raise(make_error)?;
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
        info!("Removing all {alias_type:?} aliases for target ID {target_id}");

        let make_error = || {
            Error::new(
                format!(
                    "failed to remove all {:?} aliases pointing to ID {}",
                    alias_type, target_id,
                ),
                ErrorType::Alias,
            )
        };

        let txn = ctx.transaction();
        let DeleteResult { rows_affected } = Alias::delete_many()
            .filter(
                Condition::all()
                    .add(alias::Column::AliasType.eq(alias_type))
                    .add(alias::Column::TargetId.eq(target_id)),
            )
            .exec(txn)
            .await
            .or_raise(make_error)?;

        debug!(
            "{} {:?} aliases for target ID {} were removed",
            rows_affected, alias_type, target_id,
        );

        Ok(rows_affected)
    }

    /// Verifies that the main and alias tables are consistent.
    ///
    /// These tables have a uniqueness invariant wherein a slug is only
    /// present in at most one of these two tables, but not both.
    pub async fn verify(
        ctx: &ServiceContext<'_>,
        alias_type: AliasType,
        slug: &str,
    ) -> Result<()> {
        info!(
            "Verifying {alias_type:?} alias and target table consistency for slug '{slug}'"
        );

        let make_error = || {
            Error::new(
                format!(
                    "failed to verify {:?} alias and target table are consistent for slug '{}'",
                    alias_type, slug,
                ),
                ErrorType::Alias,
            )
        };

        let txn = ctx.transaction();
        let alias_opt = Alias::find()
            .filter(
                Condition::all()
                    .add(alias::Column::AliasType.eq(alias_type))
                    .add(alias::Column::Slug.eq(slug)),
            )
            .one(txn)
            .await
            .or_raise(make_error)?;

        // Check the target and alias result.
        //
        // If they're both present, then somewhere we have a bug,
        // since the invariant is not being upheld, so we panic.
        match alias_type {
            AliasType::Site => {
                let site_opt = Site::find()
                    .filter(
                        Condition::all()
                            .add(site::Column::Slug.eq(slug))
                            .add(site::Column::DeletedAt.is_null()),
                    )
                    .one(txn)
                    .await
                    .or_raise(make_error)?;

                if let (Some(site), Some(alias)) = (site_opt, alias_opt) {
                    error!(
                        "Consistency error! Both site and alias tables have the slug '{slug}'"
                    );

                    panic!(
                        "Slug appears as both a site and an alias!\nSite: {site:#?}\nAlias: {alias:#?}",
                    );
                }
            }
            AliasType::User => {
                let user_opt = User::find()
                    .filter(
                        Condition::all()
                            .add(user::Column::Slug.eq(slug))
                            .add(user::Column::DeletedAt.is_null()),
                    )
                    .one(txn)
                    .await
                    .or_raise(make_error)?;

                if let (Some(user), Some(alias)) = (user_opt, alias_opt) {
                    error!(
                        "Consistency error! Both user and alias tables have the slug '{slug}'"
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
        target_id: i64,
        ip_address: IpAddr,
    ) -> Result<()> {
        info!("Checking alias name against filters...");

        let make_error = || Error::new("failed to run filters", ErrorType::Alias);

        let (filter_type, target_object) = match alias_type {
            AliasType::User => (FilterType::User, ObjectScope::User(target_id)),
            AliasType::Site => {
                // No filter with this type, skip verification
                debug!("No need to run filter verification for site alias");
                return Ok(());
            }
        };

        let filter_matcher =
            FilterService::get_matcher(ctx, FilterClass::Platform, filter_type)
                .await
                .or_raise(make_error)?;

        filter_matcher
            .verify(ctx, "slug", slug, target_object, ip_address)
            .await
            .or_raise(make_error)?;

        Ok(())
    }
}
