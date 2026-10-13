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

use super::structs::{CreateSite, CreateSiteOutput, SiteForumSettings, UpdateSiteBody};
use crate::constants::SYSTEM_USER_ID;
use crate::error::prelude::{Error, ErrorType, OptionExt, Result, ResultExt};
use crate::models::site::{self, Entity as Site, Model as SiteModel};
use crate::services::PageService;
use crate::services::ServiceContext;
use crate::services::alias::CreateAlias;
use crate::services::audit::{AuditEvent, AuditService, SiteFields};
use crate::services::domain::{DEFAULT_SITE_SLUG, DomainService};
use crate::services::relation::CreateSiteUser;
use crate::services::user::{CreateUser, UpdateUserBody};
use crate::services::{
    AliasService, OutdateService, PasswordService, RelationService, UserService,
};
use crate::types::{AliasType, UserType};
use crate::types::{Maybe, Reference, RerenderDepth};
use crate::utils::now;
use crate::utils::{validate_locale, validate_wikidot_site_language};
use ftml::layout::Layout;
use paste::paste;
use sea_orm::NotSet;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QuerySelect, Set,
};
use std::net::IpAddr;
use std::str::FromStr;
use wikidot_normalize::normalize;

#[derive(Debug)]
pub struct SiteService;

const RESERVED_PLATFORM_HOSTNAME_SLUGS: &[&str] = &["acme", "dns", "ech"];
const LOCAL_FILE_SOURCE_PREFIX: &str = "/local--files/";

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum SiteIconSourceKind {
    Favicon,
    Ios,
    Windows,
}

impl SiteIconSourceKind {
    fn name(self) -> &'static str {
        match self {
            Self::Favicon => "favicon",
            Self::Ios => "iOS icon",
            Self::Windows => "Windows tile",
        }
    }

    pub(super) fn parse(kind: &str) -> Option<Self> {
        match kind {
            "favicon" => Some(Self::Favicon),
            "ios" => Some(Self::Ios),
            "windows" => Some(Self::Windows),
            _ => None,
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Favicon => "favicon",
            Self::Ios => "ios",
            Self::Windows => "windows",
        }
    }

    fn wikidot_route_prefix(self) -> Option<&'static str> {
        match self {
            Self::Favicon => Some("/local--favicon/"),
            Self::Ios => Some("/local--iosicon/"),
            Self::Windows => None,
        }
    }
}

fn source_has_unsafe_text(source: &str) -> bool {
    source
        .chars()
        .any(|character| character.is_control() || character == '\\')
        || source.to_ascii_lowercase().contains("%0a")
        || source.to_ascii_lowercase().contains("%0d")
}

fn is_safe_site_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'
        })
}

fn path_has_content(path: &str, prefix: &str) -> bool {
    path.starts_with(prefix) && path.len() > prefix.len()
}

fn is_safe_local_file_source(source: &str) -> bool {
    if !source.starts_with(LOCAL_FILE_SOURCE_PREFIX) {
        return false;
    }
    let Ok(base) = reqwest::Url::parse("https://wikijump.invalid") else {
        return false;
    };
    let Ok(url) = base.join(source) else {
        return false;
    };
    path_has_content(url.path(), LOCAL_FILE_SOURCE_PREFIX)
        && url.query().is_none()
        && url.fragment().is_none()
}

fn is_site_owned_icon_source(
    site_slug: &str,
    from_wikidot: bool,
    source: &str,
    kind: SiteIconSourceKind,
) -> bool {
    if source.is_empty()
        || source.chars().count() > 2048
        || source_has_unsafe_text(source)
    {
        return false;
    }
    if is_safe_local_file_source(source) {
        return true;
    }
    if !from_wikidot || !is_safe_site_slug(site_slug) {
        return false;
    }

    let Ok(url) = reqwest::Url::parse(source) else {
        return false;
    };
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return false;
    }

    let slug = site_slug.to_ascii_lowercase();
    let Some(host) = url.host_str() else {
        return false;
    };
    if host == format!("{slug}.wdfiles.com") {
        return path_has_content(url.path(), LOCAL_FILE_SOURCE_PREFIX);
    }

    kind.wikidot_route_prefix().is_some_and(|prefix| {
        host == format!("{slug}.wikidot.com") && path_has_content(url.path(), prefix)
    })
}

pub(super) fn validate_site_icon_source(
    site_slug: &str,
    from_wikidot: bool,
    source: &Option<String>,
    kind: SiteIconSourceKind,
) -> Result<()> {
    if source.as_deref().is_none_or(|source| {
        is_site_owned_icon_source(site_slug, from_wikidot, source, kind)
    }) {
        return Ok(());
    }

    bail!(Error::new(
        format!(
            "{} source must be a site-local file or a site-owned imported resource",
            kind.name(),
        ),
        ErrorType::BadRequest,
    ));
}

#[allow(dead_code)] // TODO
const DEFAULT_FORUM_PER_PAGE_DISCUSSION: bool = false;

impl SiteService {
    pub(crate) fn is_reserved_platform_hostname_slug(slug: &str) -> bool {
        is_reserved_platform_hostname_slug(slug)
    }

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
        actor_user_id: Option<i64>,
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
            master_admin_user_id: Set(actor_user_id),
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
                user_id: actor_user_id,
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
        mut input: UpdateSiteBody,
        expected_settings_revision: Option<i64>,
        updating_user_id: i64,
        ip_address: IpAddr,
    ) -> Result<SiteModel> {
        let txn = ctx.transaction();

        // Site slugs use the same canonical representation on create and
        // update. This must happen before audit capture and hostname policy.
        if let Maybe::Set(slug) = &mut input.slug {
            normalize(slug);
        }

        let site = Self::get_for_update(ctx, reference)
            .await
            .or_raise(|| Error::new("failed to update site data", ErrorType::Site))?;

        if let Some(expected) = expected_settings_revision
            && expected != site.settings_revision
        {
            bail!(Error::new(
                format!(
                    "site settings changed since revision {expected}; current revision is {}",
                    site.settings_revision,
                ),
                ErrorType::BadRequest,
            ));
        }

        if let Maybe::Set(upgrade) = &input.educational_upgrade {
            if site.master_admin_user_id != Some(updating_user_id) {
                bail!(Error::new(
                    "only the site's Master Administrator can apply the educational upgrade",
                    ErrorType::PermissionDenied,
                ));
            }
            if site.educational {
                bail!(Error::new(
                    "site already has educational status",
                    ErrorType::BadRequest,
                ));
            }
            if upgrade.organization.trim().is_empty() || upgrade.purpose.trim().is_empty()
            {
                bail!(Error::new(
                    "educational upgrade requires organization and purpose",
                    ErrorType::BadRequest,
                ));
            }
        }

        if let Maybe::Set(analytics) = &input.google_analytics {
            analytics.validate()?;
        }
        let membership_password_hash = match &input.membership {
            Maybe::Set(membership) => {
                if membership.password_enabled
                    && membership.password.is_none()
                    && site.membership_password_hash.is_none()
                {
                    bail!(Error::new(
                        "membership password must be configured before password membership is enabled",
                        ErrorType::BadRequest,
                    ));
                }
                membership
                    .password
                    .as_deref()
                    .map(PasswordService::new_hash)
                    .transpose()?
            }
            Maybe::Unset => None,
        };
        if let Maybe::Set(page_slug) = &input.default_page
            && page_slug != &site.default_page
        {
            validate_settings_page(ctx, site.site_id, page_slug, "default page").await?;
        }
        if let Maybe::Set(page_slug) = &input.welcome_page
            && page_slug != &site.welcome_page
        {
            validate_settings_page(ctx, site.site_id, page_slug, "welcome page").await?;
        }

        let icon_site_slug = match &input.slug {
            Maybe::Set(slug) => slug.as_str(),
            Maybe::Unset => site.slug.as_str(),
        };
        if let Maybe::Set(source) = &input.favicon_source {
            validate_site_icon_source(
                icon_site_slug,
                site.from_wikidot,
                source,
                SiteIconSourceKind::Favicon,
            )?;
        }
        if let Maybe::Set(source) = &input.ios_icon_source {
            validate_site_icon_source(
                icon_site_slug,
                site.from_wikidot,
                source,
                SiteIconSourceKind::Ios,
            )?;
        }
        if let Maybe::Set(source) = &input.windows_tile_source {
            validate_site_icon_source(
                icon_site_slug,
                site.from_wikidot,
                source,
                SiteIconSourceKind::Windows,
            )?;
        }

        if let Maybe::Set(max_nest_level) = input.forum_max_nest_level
            && !(0..=10).contains(&max_nest_level)
        {
            bail!(Error::new(
                format!(
                    "forum max_nest_level must be between 0 and 10, got {max_nest_level}"
                ),
                ErrorType::BadRequest,
            ));
        }

        let top_bar_changed = input
            .top_bar_page
            .to_option()
            .is_some_and(|value| value != &site.top_bar_page);
        let side_bar_changed = input
            .side_bar_page
            .to_option()
            .is_some_and(|value| value != &site.side_bar_page);

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
            add_changed_field!(welcome_page);
            add_changed_field!(top_bar_page);
            add_changed_field!(side_bar_page);
            add_changed_field!(ref preferred_domain);
            add_changed_field!(ref favicon_source);
            add_changed_field!(ref ios_icon_source);
            add_changed_field!(ref windows_tile_source);

            if let Maybe::Set(analytics) = &input.google_analytics {
                previous_fields.google_analytics_enabled =
                    Maybe::Set(site.google_analytics_enabled);
                previous_fields.google_analytics_profile =
                    Maybe::Set(site.google_analytics_profile.as_deref());
                changed_fields.google_analytics_enabled = Maybe::Set(analytics.enabled);
                changed_fields.google_analytics_profile =
                    Maybe::Set(analytics.validated_profile()?);
            }
            if let Maybe::Set(toolbars) = input.toolbars {
                previous_fields.show_top_toolbar = Maybe::Set(site.show_top_toolbar);
                previous_fields.show_bottom_toolbar =
                    Maybe::Set(site.show_bottom_toolbar);
                previous_fields.promote_on_other_sites =
                    Maybe::Set(site.promote_on_other_sites);
                changed_fields.show_top_toolbar = Maybe::Set(toolbars.top);
                changed_fields.show_bottom_toolbar = Maybe::Set(toolbars.bottom);
                changed_fields.promote_on_other_sites = Maybe::Set(toolbars.promote);
            }
            if let Maybe::Set(membership) = &input.membership {
                previous_fields.membership_by_application =
                    Maybe::Set(site.membership_by_application);
                previous_fields.membership_by_password =
                    Maybe::Set(site.membership_by_password);
                previous_fields.membership_password_configured =
                    Maybe::Set(site.membership_password_hash.is_some());
                changed_fields.membership_by_application =
                    Maybe::Set(membership.application_enabled);
                changed_fields.membership_by_password =
                    Maybe::Set(membership.password_enabled);
                changed_fields.membership_password_configured = Maybe::Set(
                    membership.password.is_some()
                        || site.membership_password_hash.is_some(),
                );
            }

            if let Maybe::Set(value) = input.forum_max_nest_level {
                previous_fields.forum_max_nest_level =
                    Maybe::Set(site.forum_max_nest_level);
                changed_fields.forum_max_nest_level = Maybe::Set(value);
            }

            if matches!(&input.educational_upgrade, Maybe::Set(_)) {
                previous_fields.educational = Maybe::Set(site.educational);
                changed_fields.educational = Maybe::Set(true);
            }

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

        if let Maybe::Set(new_slug) = input.slug
            && new_slug != site.slug
        {
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
            validate_wikidot_site_language(&locale)?;
            model.locale = Set(locale.clone());
            site_user_body.locales = Maybe::Set(vec![locale]);
        }

        if let Maybe::Set(default_page) = input.default_page {
            model.default_page = Set(default_page);
        }

        if let Maybe::Set(welcome_page) = input.welcome_page {
            model.welcome_page = Set(welcome_page);
        }

        if let Maybe::Set(top_bar_page) = input.top_bar_page {
            model.top_bar_page = Set(top_bar_page);
        }

        if let Maybe::Set(side_bar_page) = input.side_bar_page {
            model.side_bar_page = Set(side_bar_page);
        }

        if let Maybe::Set(analytics) = input.google_analytics {
            model.google_analytics_enabled = Set(analytics.enabled);
            model.google_analytics_profile =
                Set(analytics.validated_profile()?.map(ToOwned::to_owned));
        }

        if let Maybe::Set(toolbars) = input.toolbars {
            model.show_top_toolbar = Set(toolbars.top);
            model.show_bottom_toolbar = Set(toolbars.bottom);
            model.promote_on_other_sites = Set(toolbars.promote);
        }

        if let Maybe::Set(membership) = input.membership {
            model.membership_by_application = Set(membership.application_enabled);
            model.membership_by_password = Set(membership.password_enabled);
            if let Some(password_hash) = membership_password_hash {
                model.membership_password_hash = Set(Some(password_hash));
            }
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

        if let Maybe::Set(forum_max_nest_level) = input.forum_max_nest_level {
            model.forum_max_nest_level = Set(forum_max_nest_level);
        }

        if let Maybe::Set(favicon_source) = input.favicon_source {
            model.favicon_source = Set(favicon_source);
        }

        if let Maybe::Set(ios_icon_source) = input.ios_icon_source {
            model.ios_icon_source = Set(ios_icon_source);
        }

        if let Maybe::Set(windows_tile_source) = input.windows_tile_source {
            model.windows_tile_source = Set(windows_tile_source);
        }

        if let Maybe::Set(upgrade) = input.educational_upgrade {
            model.educational = Set(true);
            model.educational_organization = Set(Some(upgrade.organization));
            model.educational_purpose = Set(Some(upgrade.purpose));
        }

        ctx.defer_public_content_cache_invalidate_site(site.site_id)
            .or_raise(make_error)?;

        // Update site
        model.updated_at = Set(Some(now()));
        model.settings_revision = Set(site.settings_revision + 1);
        let new_site = model.update(txn).await.or_raise(make_error)?;

        OutdateService::outdate_changed_site_navigation(
            ctx,
            site.site_id,
            top_bar_changed,
            side_bar_changed,
            RerenderDepth::default(),
        )
        .await
        .or_raise(make_error)?;

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

        if is_reserved_platform_hostname_slug(new_slug) {
            error!(
                "Cannot update site with reserved platform hostname slug '{new_slug}'"
            );
            bail!(Error::new(
                format!(
                    "cannot update site, site slug '{}' is reserved by the platform",
                    new_slug
                ),
                ErrorType::BadRequest
            ));
        }

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
            Some(_) if is_reserved_platform_hostname_slug(old_slug) => {
                error!(
                    "Cannot release reserved site slug '{old_slug}' while renaming to existing alias '{new_slug}'"
                );
                bail!(Error::new(
                    format!(
                        "cannot update site, an alias with slug '{}' already exists",
                        new_slug
                    ),
                    ErrorType::SiteExists
                ));
            }

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
            None if is_reserved_platform_hostname_slug(old_slug) => {
                // Legacy data may predate this reservation. Release the
                // infrastructure hostname instead of retaining it as an alias.
                info!(
                    "Releasing reserved platform hostname slug '{old_slug}' during site rename"
                );
            }

            None => {
                debug!("Creating site alias for {old_slug}");

                // Add site alias for old slug.
                //
                // We don't verify here because the site row hasn't been
                // updated yet, so we instead run AliasService::verify()
                // ourselves at the end of site updating (see above).
                AliasService::create_for_pending_target_rename(
                    ctx,
                    CreateAlias {
                        slug: str!(old_slug),
                        alias_type: AliasType::Site,
                        target_id: site.site_id,
                        created_by: user_id,
                        bypass_filter: true, // sites don't have filters
                        ip_address,
                    },
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

    pub(super) async fn get_for_update(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
    ) -> Result<SiteModel> {
        let site_id = match reference {
            Reference::Id(site_id) => site_id,
            Reference::Slug(slug) => Self::get(ctx, Reference::Slug(slug)).await?.site_id,
        };
        Site::find_by_id(site_id)
            .lock_exclusive()
            .one(ctx.transaction())
            .await
            .or_raise(|| Error::new("failed to lock site settings", ErrorType::Site))?
            .ok_or_raise(|| Error::new("site does not exist", ErrorType::SiteNotFound))
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
    pub async fn get_forum_settings(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
    ) -> Result<SiteForumSettings> {
        let SiteModel {
            site_id,
            forum_max_nest_level,
            ..
        } = Self::get(ctx, reference).await.or_raise(|| {
            Error::new("failed to get site forum settings", ErrorType::Forum)
        })?;

        debug!("Using stored forum settings for site ID {site_id}");
        Ok(SiteForumSettings {
            max_nest_level: forum_max_nest_level,
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

        if is_reserved_platform_hostname_slug(slug) {
            error!("Cannot {action} site with reserved platform hostname slug '{slug}'");
            bail!(Error::new(
                format!(
                    "cannot {}, site slug '{}' is reserved by the platform",
                    action, slug
                ),
                ErrorType::BadRequest
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

async fn validate_settings_page(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_slug: &str,
    field_name: &str,
) -> Result<()> {
    if !page_slug.is_empty()
        && PageService::get_optional(ctx, site_id, Reference::from(page_slug))
            .await?
            .is_some()
    {
        Ok(())
    } else {
        Err(Error::new(
            format!("{field_name} must reference a live page in the same site"),
            ErrorType::BadRequest,
        )
        .into())
    }
}

pub(crate) fn is_reserved_platform_hostname_slug(slug: &str) -> bool {
    let mut canonical = slug.to_owned();
    normalize(&mut canonical);
    let canonical = canonical.trim_end_matches('.');
    RESERVED_PLATFORM_HOSTNAME_SLUGS
        .iter()
        .any(|reserved| canonical.eq_ignore_ascii_case(reserved))
}

#[cfg(test)]
mod tests {
    use super::is_reserved_platform_hostname_slug;
    use wikidot_normalize::normalize;

    #[test]
    fn dns_related_platform_hostnames_are_reserved() {
        for slug in ["acme", "dns", "ech", "ACME", "Dns", "ECh"] {
            assert!(
                is_reserved_platform_hostname_slug(slug),
                "{slug} should be reserved",
            );
        }
    }

    #[test]
    fn unrelated_site_slugs_are_not_reserved_hostnames() {
        for slug in ["example", "secure", "static", "service"] {
            assert!(
                !is_reserved_platform_hostname_slug(slug),
                "{slug} should not be reserved by the DNS hostname guard",
            );
        }
    }

    #[test]
    fn canonical_dns_hostname_variants_are_reserved_after_slug_normalization() {
        let mut slug = String::from("ｄｎｓ");
        normalize(&mut slug);

        assert_eq!(slug, "dns");
        assert!(is_reserved_platform_hostname_slug(&slug));
    }
}
