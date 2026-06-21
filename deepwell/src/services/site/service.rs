/*
 * services/site/service.rs
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
use crate::constants::SYSTEM_USER_ID;
use crate::error::prelude::*;
use crate::models::site::{self, Entity as Site, Model as SiteModel};
use crate::services::alias::CreateAlias;
use crate::services::audit::{AuditEvent, AuditService, SiteFields};
use crate::services::domain::{DEFAULT_SITE_SLUG, DomainService};
use crate::services::relation::CreateSiteUser;
use crate::services::user::{CreateUser, UpdateUserBody};
use crate::services::{AliasService, RelationService, UserService};
use crate::types::{AliasType, UserType};
use crate::utils::validate_locale;
use ftml::layout::Layout;
use ref_map::*;
use sea_orm::NotSet;
use std::borrow::Cow;
use std::net::IpAddr;
use std::str::FromStr;
use wikidot_normalize::normalize;

#[derive(Debug)]
pub struct SiteService;

#[allow(dead_code)] // TODO
const DEFAULT_FORUM_MAX_NEST_LEVEL: i16 = 10;
#[allow(dead_code)] // TODO
const DEFAULT_FORUM_PER_PAGE_DISCUSSION: bool = false;

impl SiteService {
    pub async fn create(
        ctx: &ServiceContext<'_>,
        CreateSite {
            mut slug,
            name,
            tagline,
            description,
            default_page,
            layout,
            license,
            locale,
            ip_address,
        }: CreateSite,
    ) -> Result<CreateSiteOutput> {
        let txn = ctx.transaction();

        // Normalize slug.
        normalize(&mut slug);

        let make_error =
            || Error::new(format!("failed to create site '{}'", slug), ErrorType::Site);

        // Check for slug conflicts.
        Self::check_conflicts(ctx, &slug, "create")
            .await
            .or_raise(make_error)?;

        // Validate locale.
        validate_locale(&locale)?;

        // Insert into database
        let model = site::ActiveModel {
            slug: Set(slug.clone()),
            name: Set(name),
            tagline: Set(tagline),
            description: Set(description.clone()),
            default_page: match default_page {
                Some(slug) => Set(slug),
                None => NotSet,
            },
            layout: Set(layout.map(|l| str!(l.value()))),
            license: Set(license),
            locale: Set(locale.clone()),
            ..Default::default()
        };
        let site = model.insert(txn).await.or_raise(make_error)?;

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
                bypass_email_verification: true,
                override_user_id: None,
                ip_address,
            },
        )
        .await
        .or_raise(make_error)?;

        // Some fields can only be set in update after creation
        UserService::update(
            ctx,
            Reference::Id(user.user_id),
            ip_address,
            UpdateUserBody {
                biography: Maybe::Set(Some(description)),
                ..Default::default()
            },
        )
        .await
        .or_raise(make_error)?;

        RelationService::create_site_user(
            ctx,
            CreateSiteUser {
                site_id: site.site_id,
                user_id: user.user_id,
                metadata: (),
                created_by: SYSTEM_USER_ID,
            },
        )
        .await
        .or_raise(make_error)?;

        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::SiteCreate {
                site_id: site.site_id,
            },
        )
        .await
        .or_raise(make_error)?;

        // Build and return
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
        ip_address: IpAddr,
    ) -> Result<SiteModel> {
        let txn = ctx.transaction();
        let site = Self::get(ctx, reference)
            .await
            .or_raise(|| Error::new("failed to update site data", ErrorType::Site))?;

        let mut model = site::ActiveModel {
            site_id: Set(site.site_id),
            ..Default::default()
        };

        let make_error = || {
            Error::new(
                format!(
                    "failed to update site ID {}, changed by user ID {}",
                    site.site_id, updating_user_id,
                ),
                ErrorType::Site,
            )
        };

        // Gather data for audit log entry
        {
            let mut previous_fields = SiteFields::default();
            let mut changed_fields = SiteFields::default();

            macro_rules! add_changed_field {
                ($field:ident) => {{
                    if let Maybe::Set(value) = &input.$field {
                        previous_fields.$field = Maybe::Set(&site.$field);
                        changed_fields.$field = Maybe::Set(value);
                    }
                }};
                (ref $field:ident) => {{
                    if let Maybe::Set(value) = &input.$field {
                        previous_fields.$field = Maybe::Set(site.$field.as_deref());
                        changed_fields.$field = Maybe::Set(value.as_deref());
                    }
                }};
                (move $field:ident) => {{
                    if let Maybe::Set(value) = input.$field {
                        previous_fields.$field = Maybe::Set(site.$field);
                        changed_fields.$field = Maybe::Set(value);
                    }
                }};
            }

            add_changed_field!(name);
            add_changed_field!(slug);
            add_changed_field!(tagline);
            add_changed_field!(description);
            add_changed_field!(move license);
            add_changed_field!(locale);
            add_changed_field!(default_page);
            add_changed_field!(top_bar_page);
            add_changed_field!(side_bar_page);
            add_changed_field!(ref preferred_domain);

            if let Maybe::Set(layout) = input.layout {
                let old_layout = site.layout.as_ref().map(|value| {
                    Layout::from_str(value)
                        .expect("Invalid layout value found in database")
                });
                previous_fields.layout = Maybe::Set(old_layout);
                changed_fields.layout = Maybe::Set(layout);
            }

            AuditService::log(
                ctx,
                ip_address,
                AuditEvent::SiteUpdate {
                    site_id: site.site_id,
                    user_id: updating_user_id,
                    previous_fields,
                    changed_fields,
                },
            )
            .await?;
        }

        // For updating the corresponding site user
        let mut site_user_body = UpdateUserBody::default();
        let site_user_id = RelationService::get_site_user_id_for_site(ctx, site.site_id)
            .await
            .or_raise(make_error)?;

        if let Maybe::Set(name) = input.name {
            model.name = Set(name);
        }

        if let Maybe::Set(new_slug) = input.slug {
            Self::update_slug(ctx, &site, &new_slug, updating_user_id, ip_address)
                .await
                .or_raise(make_error)?;

            site_user_body.name = Maybe::Set(format!("site:{new_slug}"));
            model.slug = Set(new_slug);
        }

        if let Maybe::Set(tagline) = input.tagline {
            model.tagline = Set(tagline);
        }

        if let Maybe::Set(description) = input.description {
            model.description = Set(description.clone());
            site_user_body.biography = Maybe::Set(Some(description))
        }

        if let Maybe::Set(locale) = input.locale {
            validate_locale(&locale)?;
            model.locale = Set(locale.clone());
            site_user_body.locales = Maybe::Set(vec![locale]);
        }

        if let Maybe::Set(default_page) = input.default_page {
            model.default_page = Set(default_page);
        }

        if let Maybe::Set(preferred_domain) = input.preferred_domain {
            // Disallow preferred domains for the default site (www)
            if site.slug == DEFAULT_SITE_SLUG && preferred_domain.is_some() {
                error!("Cannot set a preferred domain for the default site");
                bail!(Error::new(
                    "cannot set a preferred domain for the default site",
                    ErrorType::BadRequest
                ));
            }

            // TODO expire redis cache on change to domains

            // Ensure that the custom domain exists and belongs to this site
            if let Some(domain) = &preferred_domain {
                match DomainService::site_from_custom_domain_optional(ctx, domain)
                    .await
                    .or_raise(make_error)?
                {
                    Some(found_site) if found_site.site_id == site.site_id => (),
                    Some(found_site) => {
                        error!(
                            "Attempting to set preferred domain for site ID {} '{}' to '{}', but the custom domain belongs to site ID {} '{}'!",
                            site.site_id,
                            site.slug,
                            domain,
                            found_site.site_id,
                            found_site.slug,
                        );
                        bail!(Error::new(
                            format!(
                                "cannot set preferred domain for site '{}' (ID {}) to '{}', because the custom domain belongs to site '{}' (ID {})",
                                site.slug,
                                site.site_id,
                                domain,
                                found_site.slug,
                                found_site.site_id,
                            ),
                            ErrorType::CustomDomainWrongSite,
                        ));
                    }
                    None => {
                        error!(
                            "Attempting to set preferred domain to '{domain}', but this is not a known custom domain!"
                        );
                        bail!(Error::new(
                            format!(
                                "cannot set preferred domain for site '{}' (ID {}) to '{}', because this is not a known custom domain",
                                site.slug, site.site_id, domain,
                            ),
                            ErrorType::CustomDomainNotFound,
                        ));
                    }
                }
            }

            model.preferred_domain = Set(preferred_domain);
        }

        if let Maybe::Set(layout) = input.layout {
            model.layout = Set(layout.map(|l| str!(l.value())));
        }

        if let Maybe::Set(license) = input.license {
            model.license = Set(license);
        }

        // Update site
        model.updated_at = Set(Some(now()));
        let new_site = model.update(txn).await.or_raise(make_error)?;

        // Update site user
        UserService::update(ctx, Reference::Id(site_user_id), ip_address, site_user_body)
            .await
            .or_raise(make_error)?;

        // Run verification afterwards if the slug changed
        if site.slug != new_site.slug {
            let (result1, result2) = join!(
                AliasService::verify(ctx, AliasType::Site, &site.slug),
                AliasService::verify(ctx, AliasType::Site, &new_site.slug),
            );
            raise_multiple!(result1, result2; make_error);
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
        ip_address: IpAddr,
    ) -> Result<()> {
        info!("Updating slug for site {}, adding alias", site.site_id);
        let old_slug = &site.slug;

        let make_error = || {
            Error::new(
                format!(
                    "failed to update slug from '{}' -> '{}' for site ID {}, done by user ID {}",
                    old_slug, new_slug, site.site_id, user_id,
                ),
                ErrorType::Site,
            )
        };

        match AliasService::get_optional(ctx, AliasType::Site, new_slug)
            .await
            .or_raise(make_error)?
        {
            // Swap alias with site's current slug
            //
            // Don't return a future, nothing to do after
            Some(alias) => {
                debug!("Swapping slug between site and alias");
                AliasService::swap(ctx, alias.alias_id, old_slug)
                    .await
                    .or_raise(make_error)?;
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
                        ip_address,
                    },
                    false,
                )
                .await
                .or_raise(make_error)?;
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
        let txn = ctx.transaction();

        let make_error = || Error::new("failed to get site", ErrorType::Site);

        // If slug, determine if this is a site alias.
        //
        // This uses separate queries rather than a join.
        // See UserService::get_optional() for more information.
        if let Reference::Slug(ref slug) = reference
            && let Some(alias) = AliasService::get_optional(ctx, AliasType::Site, slug)
                .await
                .or_raise(make_error)?
        {
            // If present, this is the actual site. Proceed with SELECT by id.
            // Rewrite reference so in the "real" site search
            // we locate directly via site ID.
            reference = Reference::Id(alias.target_id);
        }

        let site = match reference {
            Reference::Id(id) => {
                Site::find_by_id(id).one(txn).await.or_raise(make_error)?
            }
            Reference::Slug(slug) => Site::find()
                .filter(
                    Condition::all()
                        .add(site::Column::Slug.eq(slug))
                        .add(site::Column::DeletedAt.is_null()),
                )
                .one(txn)
                .await
                .or_raise(make_error)?,
        };

        Ok(site)
    }

    #[inline]
    pub async fn get(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
    ) -> Result<SiteModel> {
        find_or_error!(Self::get_optional(ctx, reference), "site", Site)
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
        let make_error = || Error::new("failed to get ID for site", ErrorType::File);
        match reference {
            Reference::Id(id) => Ok(id),
            Reference::Slug(slug) => {
                // For slugs we pass-through the call so that alias handling is done.
                let SiteModel { site_id, .. } = Self::get(ctx, Reference::Slug(slug))
                    .await
                    .or_raise(make_error)?;

                Ok(site_id)
            }
        }
    }

    /// Gets site-wide forum settings.
    ///
    /// At present this is sourced from service defaults; the site row itself
    /// does not yet carry dedicated forum configuration columns.
    #[allow(dead_code)] // TODO
    pub async fn get_forum_settings(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
    ) -> Result<SiteForumSettings> {
        let SiteModel { site_id, .. } =
            Self::get(ctx, reference).await.or_raise(|| {
                Error::new("failed to get site forum settings", ErrorType::Forum)
            })?;

        debug!("Using default forum settings for site ID {site_id}");
        Ok(SiteForumSettings {
            max_nest_level: DEFAULT_FORUM_MAX_NEST_LEVEL,
            per_page_discussion: DEFAULT_FORUM_PER_PAGE_DISCUSSION,
        })
    }

    /// Checks to see if a site already exists at the slug specified.
    ///
    /// If so, this method fails with `ErrorType::SiteExists`. Otherwise it returns nothing.
    async fn check_conflicts(
        ctx: &ServiceContext<'_>,
        slug: &str,
        action: &str,
    ) -> Result<()> {
        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!(
                    "cannot {}, failed to conflict checks for '{}'",
                    action, slug,
                ),
                ErrorType::Site,
            )
        };

        if slug.is_empty() {
            error!("Cannot create site with empty slug");
            bail!(Error::new(
                "empty site slugs are not allowed",
                ErrorType::SiteSlugEmpty
            ));
        }

        let result = Site::find()
            .filter(
                Condition::all()
                    .add(site::Column::Slug.eq(slug))
                    .add(site::Column::DeletedAt.is_null()),
            )
            .one(txn)
            .await
            .or_raise(make_error)?;

        match result {
            None => Ok(()),
            Some(_) => {
                error!("Site with slug '{slug}' already exists, cannot {action}");
                bail!(Error::new(
                    format!(
                        "cannot {}, a site with slug '{}' already exists",
                        action, slug
                    ),
                    ErrorType::SiteExists
                ));
            }
        }
    }
}
