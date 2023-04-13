/*
 * services/site/service.rs
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

use wikidot_normalize::normalize;

use super::prelude::*;
use crate::models::sea_orm_active_enums::AliasType;
use crate::models::site::{self, Entity as Site, Model as SiteModel};
use crate::services::alias::CreateAlias;
use crate::services::AliasService;
use crate::utils::validate_locale;
use futures::Future;

#[derive(Debug)]
pub struct SiteService;

impl SiteService {
    pub async fn create(
        ctx: &ServiceContext<'_>,
        CreateSite {
            mut slug,
            name,
            tagline,
            description,
            locale,
        }: CreateSite,
    ) -> Result<CreateSiteOutput> {
        let txn = ctx.transaction();

        // Normalize slug.
        normalize(&mut slug);

        // Check for slug conflicts.
        Self::check_conflicts(ctx, &slug, "create").await?;

        // Validate locale.
        validate_locale(&locale)?;

        let model = site::ActiveModel {
            slug: Set(slug.clone()),
            name: Set(name),
            tagline: Set(tagline),
            description: Set(description),
            locale: Set(locale),
            ..Default::default()
        };
        let site = model.insert(txn).await?;

        Ok(CreateSiteOutput {
            site_id: site.site_id,
            slug,
        })
    }

    /// Update site information.
    pub async fn update(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
        input: UpdateSiteBody,
        user_id: i64,
    ) -> Result<SiteModel> {
        let txn = ctx.transaction();
        let site = Self::get(ctx, reference).await?;
        let mut future_after = None;
        let mut model = site::ActiveModel {
            site_id: Set(site.site_id),
            ..Default::default()
        };

        if let ProvidedValue::Set(name) = input.name {
            model.name = Set(name);
        }

        if let ProvidedValue::Set(new_slug) = input.slug {
            future_after = Self::update_slug(ctx, &site, &new_slug, user_id).await?;
            model.slug = Set(new_slug);
        }

        if let ProvidedValue::Set(tagline) = input.tagline {
            model.tagline = Set(tagline);
        }

        if let ProvidedValue::Set(description) = input.description {
            model.description = Set(description);
        }

        if let ProvidedValue::Set(locale) = input.locale {
            validate_locale(&locale)?;
            model.locale = Set(locale);
        }

        // Update site
        model.updated_at = Set(Some(now()));
        let new_site = model.update(txn).await?;

        // Run future afterwards
        if let Some(future) = future_after {
            future.await?;

            try_join!(
                AliasService::verify(ctx, AliasType::Site, &site.slug),
                AliasService::verify(ctx, AliasType::Site, &new_site.slug),
            )?;
        }

        // Return
        Ok(new_site)
    }

    /// Updates the slug for a site, leaving behind an alias.
    ///
    /// If this returns a future, it must be run after the site update,
    /// since it depends on the site slug having already been changed
    /// before creating the new alias.
    async fn update_slug<'ctx>(
        ctx: &'ctx ServiceContext<'ctx>,
        site: &SiteModel,
        new_slug: &str,
        user_id: i64,
    ) -> Result<Option<impl Future<Output = Result<()>> + 'ctx>> {
        tide::log::info!("Updating slug for site {}, adding alias", site.site_id);

        let old_slug = &site.slug;
        match AliasService::get_optional(ctx, AliasType::Site, new_slug).await? {
            // Swap alias with site's current slug
            //
            // Don't return a future, nothing to do after
            Some(alias) => {
                tide::log::debug!("Swapping slug between site and alias");
                AliasService::swap(ctx, alias.alias_id, old_slug).await?;
                Ok(None)
            }

            // Return future that creates new alias at the old location
            None => {
                let old_slug = str!(old_slug);
                let site_id = site.site_id;

                Ok(Some(async move {
                    tide::log::debug!("Creating site alias for {old_slug}");

                    // Add site alias for old slug.
                    //
                    // We don't verify here because the site row hasn't been
                    // updated yet, so we instead run AliasService::verify()
                    // ourselves at the end of site updating (see above).
                    AliasService::create_no_verify(
                        ctx,
                        CreateAlias {
                            slug: old_slug,
                            alias_type: AliasType::Site,
                            target_id: site_id,
                            created_by: user_id,
                            bypass_filter: true, // sites don't have filters
                        },
                    )
                    .await?;

                    Ok(())
                }))
            }
        }
    }

    #[inline]
    pub async fn exists(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
    ) -> Result<bool> {
        Self::get_optional(ctx, reference)
            .await
            .map(|site| site.is_some())
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        mut reference: Reference<'_>,
    ) -> Result<Option<SiteModel>> {
        let txn = ctx.transaction();

        // If slug, determine if this is a site alias.
        //
        // This uses separate queries rather than a join.
        // See UserService::get_optional() for more information.
        if let Reference::Slug(ref slug) = reference {
            if let Some(alias) =
                AliasService::get_optional(ctx, AliasType::Site, slug).await?
            {
                // If present, this is the actual site. Proceed with SELECT by id.
                // Rewrite reference so in the "real" site search
                // we locate directly via site ID.
                reference = Reference::Id(alias.target_id);
            }
        }

        let site = match reference {
            Reference::Id(id) => Site::find_by_id(id).one(txn).await?,
            Reference::Slug(slug) => {
                Site::find()
                    .filter(
                        Condition::all()
                            .add(site::Column::Slug.eq(slug))
                            .add(site::Column::DeletedAt.is_null()),
                    )
                    .one(txn)
                    .await?
            }
        };

        Ok(site)
    }

    #[inline]
    pub async fn get(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
    ) -> Result<SiteModel> {
        find_or_error(Self::get_optional(ctx, reference)).await
    }

    /// Gets the site ID from a reference, looking up if necessary.
    ///
    /// Convenience method since this is much more common than the optional
    /// case, and we don't want to perform a redundant check for site existence
    /// later as part of the actual query.
    pub async fn get_id(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
    ) -> Result<i64> {
        match reference {
            Reference::Id(id) => Ok(id),
            Reference::Slug(slug) => {
                // For slugs we pass-through the call so that alias handling is done.
                let SiteModel { site_id, .. } =
                    Self::get(ctx, Reference::Slug(slug)).await?;

                Ok(site_id)
            }
        }
    }

    /// Checks to see if a site already exists at the slug specified.
    ///
    /// If so, this method fails with `Error::Conflict`. Otherwise it returns nothing.
    async fn check_conflicts(
        ctx: &ServiceContext<'_>,
        slug: &str,
        action: &str,
    ) -> Result<()> {
        let txn = ctx.transaction();

        let result = Site::find()
            .filter(
                Condition::all()
                    .add(site::Column::Slug.eq(slug))
                    .add(site::Column::DeletedAt.is_null()),
            )
            .one(txn)
            .await?;

        match result {
            None => Ok(()),
            Some(_) => {
                tide::log::error!(
                    "Site with slug '{}' already exists, cannot {}",
                    slug,
                    action,
                );

                Err(Error::Conflict)
            }
        }
    }
}
