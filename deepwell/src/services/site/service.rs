/*
 * services/site/service.rs
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

use wikidot_normalize::normalize;

use super::prelude::*;
use crate::constants::SYSTEM_USER_ID;
use crate::models::sea_orm_active_enums::{AliasType, UserType};
use crate::models::site::{self, Entity as Site, Model as SiteModel};
use crate::services::alias::CreateAlias;
use crate::services::relation::CreateSiteUser;
use crate::services::user::{CreateUser, UpdateUserBody};
use crate::services::{AliasService, Error, RelationService, UserService};
use crate::utils::validate_locale;
use ftml::layout::Layout;
use ref_map::*;
use std::borrow::Cow;

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
            layout,
            locale,
        }: CreateSite,
    ) -> Result<CreateSiteOutput> {
        let txn = ctx.seaorm_transaction();

        // Normalize slug.
        normalize(&mut slug);

        // Check for slug conflicts.
        Self::check_conflicts(ctx, &slug, "create").await?;

        // Validate locale.
        validate_locale(&locale)?;

        // Insert into database
        let model = site::ActiveModel {
            slug: Set(slug.clone()),
            name: Set(name),
            tagline: Set(tagline),
            description: Set(description.clone()),
            layout: Set(layout.map(|l| str!(l.value()))),
            locale: Set(locale.clone()),
            ..Default::default()
        };
        let site = model.insert(txn).await?;

        // Create site user, and add relation

        let user = UserService::create(
            ctx,
            CreateUser {
                user_type: UserType::Site,
                name: format!("site:{slug}"),
                email: String::new(),
                locales: vec![locale],
                password: String::new(),
                bypass_filter: false,
                bypass_email_verification: false,
            },
        )
        .await?;

        // Some fields can only be set in update after creation
        UserService::update(
            ctx,
            Reference::Id(user.user_id),
            UpdateUserBody {
                biography: ProvidedValue::Set(Some(description)),
                ..Default::default()
            },
        )
        .await?;

        RelationService::create_site_user(
            ctx,
            CreateSiteUser {
                site_id: site.site_id,
                user_id: user.user_id,
                metadata: (),
                created_by: SYSTEM_USER_ID,
            },
        )
        .await?;

        // Return
        Ok(CreateSiteOutput {
            site_id: site.site_id,
            site_user_id: user.user_id,
            slug,
        })
    }

    /// Update site information.
    pub async fn update(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
        input: UpdateSiteBody,
        updating_user_id: i64,
    ) -> Result<SiteModel> {
        let txn = ctx.seaorm_transaction();
        let site = Self::get(ctx, reference).await?;
        let mut model = site::ActiveModel {
            site_id: Set(site.site_id),
            ..Default::default()
        };

        // For updating the corresponding site user
        let site_user_id =
            RelationService::get_site_user_id_for_site(ctx, site.site_id).await?;
        let mut site_user_body = UpdateUserBody::default();

        if let ProvidedValue::Set(name) = input.name {
            model.name = Set(name);
        }

        if let ProvidedValue::Set(new_slug) = input.slug {
            Self::update_slug(ctx, &site, &new_slug, updating_user_id).await?;
            site_user_body.name = ProvidedValue::Set(format!("site:{new_slug}"));
            model.slug = Set(new_slug);
        }

        if let ProvidedValue::Set(tagline) = input.tagline {
            model.tagline = Set(tagline);
        }

        if let ProvidedValue::Set(description) = input.description {
            model.description = Set(description.clone());
            site_user_body.biography = ProvidedValue::Set(Some(description))
        }

        if let ProvidedValue::Set(locale) = input.locale {
            validate_locale(&locale)?;
            model.locale = Set(locale.clone());
            site_user_body.locales = ProvidedValue::Set(vec![locale]);
        }

        if let ProvidedValue::Set(layout) = input.layout {
            model.layout = Set(layout.map(|l| str!(l.value())));
        }

        // Update site
        model.updated_at = Set(Some(now()));
        let new_site = model.update(txn).await?;

        // Update site user
        UserService::update(ctx, Reference::Id(site_user_id), site_user_body).await?;

        // Run verification afterwards if the slug changed
        if site.slug != new_site.slug {
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
    /// No alias row checks are performed because of a dependency order requiring
    /// the user's slug to have been updated before aliases can be added.
    /// Instead, alias row verification occurs manually afterwards.
    async fn update_slug(
        ctx: &ServiceContext<'_>,
        site: &SiteModel,
        new_slug: &str,
        user_id: i64,
    ) -> Result<()> {
        info!("Updating slug for site {}, adding alias", site.site_id);

        let old_slug = &site.slug;
        match AliasService::get_optional(ctx, AliasType::Site, new_slug).await? {
            // Swap alias with site's current slug
            //
            // Don't return a future, nothing to do after
            Some(alias) => {
                debug!("Swapping slug between site and alias");
                AliasService::swap(ctx, alias.alias_id, old_slug).await?;
            }

            // Return future that creates new alias at the old location
            None => {
                debug!("Creating site alias for {old_slug}");

                // Add site alias for old slug.
                //
                // We don't verify here because the site row hasn't been
                // updated yet, so we instead run AliasService::verify()
                // ourselves at the end of site updating (see above).
                AliasService::create2(
                    ctx,
                    CreateAlias {
                        slug: str!(old_slug),
                        alias_type: AliasType::Site,
                        target_id: site.site_id,
                        created_by: user_id,
                        bypass_filter: true, // sites don't have filters
                    },
                    false,
                )
                .await?;
            }
        }

        Ok(())
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
        let txn = ctx.seaorm_transaction();

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
        find_or_error!(Self::get_optional(ctx, reference), Site)
    }

    /// Get the default page layout for this site.
    /// If the site has not set a page layout, then the platform default is used.
    ///
    /// Since this is the only field needed most of the time, and
    /// is fairly commonly needed, we have a separate method for it.
    pub async fn get_layout(ctx: &ServiceContext<'_>, site_id: i64) -> Result<Layout> {
        debug!("Getting page layout for site ID {site_id}");

        /*
            TODO: Temporary workaround, see set_layout()
                  See https://scuttle.atlassian.net/browse/WJ-1270

        #[derive(Debug)]
        struct Row {
            layout: Option<String>,
        }

        let mut txn = ctx.make_sqlx_transaction().await?;
        let row =
            sqlx::query_as!(Row, r"SELECT layout FROM site WHERE site_id = $1", site_id)
                .fetch_one(&mut *txn)
                .await?;

        txn.commit().await?;
        */

        let site = Self::get(ctx, Reference::Id(site_id)).await?;
        match site.layout {
            // Parse layout from string in site table
            Some(layout) => match layout.parse() {
                Ok(layout) => Ok(layout),
                Err(_) => Err(Error::InvalidEnumValue),
            },

            // Fallback to default platform layout
            None => Ok(ctx.config().default_page_layout),
        }
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
    /// If so, this method fails with `Error::SiteExists`. Otherwise it returns nothing.
    async fn check_conflicts(
        ctx: &ServiceContext<'_>,
        slug: &str,
        action: &str,
    ) -> Result<()> {
        let txn = ctx.seaorm_transaction();

        if slug.is_empty() {
            error!("Cannot create site with empty slug");
            return Err(Error::SiteSlugEmpty);
        }

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
                error!(
                    "Site with slug '{}' already exists, cannot {}",
                    slug, action,
                );

                Err(Error::SiteExists)
            }
        }
    }
}
