/*
 * services/domain/service.rs
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

//! Service for managing domains as used by Wikijump sites.
//!
//! This service has two components, management of canonical domains (e.g. `scp-wiki.wikijump.com`)
//! and custom domains (e.g. `scpwiki.com`).

// TODO disallow custom domains that are subdomains of the main domain or files domain

use super::prelude::*;
use crate::models::site::{self, Entity as Site, Model as SiteModel};
use crate::models::site_domain::{self, Entity as SiteDomain, Model as SiteDomainModel};
use crate::services::SiteService;
use std::borrow::Cow;

#[derive(Debug)]
pub struct DomainService;

impl DomainService {
    /// Creates a custom domain for a site.
    pub async fn create_custom(
        ctx: &ServiceContext<'_>,
        CreateCustomDomain { domain, site_id }: CreateCustomDomain,
    ) -> Result<()> {
        info!("Creating custom domain '{domain}' (site ID {site_id})");

        let txn = ctx.transaction();
        if Self::custom_domain_exists(ctx, &domain).await? {
            error!("Custom domain already exists, cannot create");
            return Err(Error::CustomDomainExists);
        }

        let model = site_domain::ActiveModel {
            domain: Set(domain),
            site_id: Set(site_id),
            created_at: Set(now()),
        };
        model.insert(txn).await?;
        Ok(())
    }

    /// Delete the given custom domain.
    ///
    /// Yields `Error::CustomDomainNotFound` if it's missing.
    pub async fn remove_custom(ctx: &ServiceContext<'_>, domain: String) -> Result<()> {
        info!("Deleting custom domain '{domain}'");

        let txn = ctx.transaction();
        let DeleteResult { rows_affected, .. } =
            SiteDomain::delete_by_id(domain).exec(txn).await?;

        if rows_affected == 1 {
            Ok(())
        } else {
            Err(Error::CustomDomainNotFound)
        }
    }

    pub async fn site_from_custom_domain_optional(
        ctx: &ServiceContext<'_>,
        domain: &str,
    ) -> Result<Option<SiteModel>> {
        info!("Getting site for custom domain {domain:?}");

        // Join with the site table so we can get that data, rather than just the ID.
        let txn = ctx.transaction();
        let model = Site::find()
            .join(JoinType::Join, site::Relation::SiteDomain.def())
            .filter(site_domain::Column::Domain.eq(domain))
            .one(txn)
            .await?;

        Ok(model)
    }

    #[inline]
    #[allow(dead_code)] // TODO
    pub async fn site_from_custom_domain(
        ctx: &ServiceContext<'_>,
        domain: &str,
    ) -> Result<SiteModel> {
        find_or_error!(
            Self::site_from_custom_domain_optional(ctx, domain),
            CustomDomain,
        )
    }

    /// Determines if the given custom domain is registered.
    #[inline]
    pub async fn custom_domain_exists(
        ctx: &ServiceContext<'_>,
        domain: &str,
    ) -> Result<bool> {
        Self::site_from_custom_domain_optional(ctx, domain)
            .await
            .map(|site| site.is_some())
    }

    /// Gets the site corresponding with the given domain.
    #[inline]
    #[allow(dead_code)] // TEMP
    pub async fn site_from_domain<'a>(
        ctx: &ServiceContext<'_>,
        domain: &'a str,
    ) -> Result<SiteModel> {
        find_or_error!(Self::site_from_domain_optional(ctx, domain), CustomDomain)
    }

    /// Optional version of `site_from_domain()`.
    pub async fn site_from_domain_optional<'a>(
        ctx: &ServiceContext<'_>,
        domain: &'a str,
    ) -> Result<Option<SiteModel>> {
        let result = Self::parse_site_from_domain(ctx, domain).await?;
        match result {
            SiteDomainResult::Found(site) => Ok(Some(site)),
            _ => Ok(None),
        }
    }

    /// Gets the site corresponding with the given domain.
    ///
    /// Returns one of three variants:
    /// * `Found` &mdash; Site retrieved from the domain.
    /// * `Slug` &mdash; Site does not exist. If it did, domain would be a canonical domain.
    /// * `CustomDomain` &mdash; Site does not exist. If it did, domain would be a custom domain.
    pub async fn parse_site_from_domain<'a>(
        ctx: &ServiceContext<'_>,
        domain: &'a str,
    ) -> Result<SiteDomainResult<'a>> {
        info!("Getting site for domain '{domain}'");

        match Self::parse_canonical(ctx.config(), domain) {
            // Normal canonical domain, return from site slug fetch.
            Some(subdomain) => {
                debug!("Found canonical domain with slug '{subdomain}'");

                let result =
                    SiteService::get_optional(ctx, Reference::Slug(cow!(subdomain)))
                        .await;

                match result {
                    Ok(Some(site)) => Ok(SiteDomainResult::Found(site)),
                    Ok(None) => Ok(SiteDomainResult::Slug(subdomain)),
                    Err(error) => Err(error),
                }
            }

            // Not canonical, try custom domain.
            None => {
                debug!("Not found, checking if it's a custom domain");

                let result = Self::site_from_custom_domain_optional(ctx, domain).await;
                match result {
                    Ok(Some(site)) => Ok(SiteDomainResult::Found(site)),
                    Ok(None) => Ok(SiteDomainResult::CustomDomain(domain)),
                    Err(error) => Err(error),
                }
            }
        }
    }

    /// If this domain is canonical domain, extract the site slug.
    pub fn parse_canonical<'a>(config: &Config, domain: &'a str) -> Option<&'a str> {
        let main_domain = &config.main_domain;

        // Special case, see if it's the root domain (i.e. 'wikijump.com')
        {
            // This slice is safe, we know the first character of 'main_domain'
            // is always '.', then we compare to the passed domain to see if
            // it's the root domain.
            //
            // We are not slicing 'domain' at all, which is user-provided and
            // has no guarantees about character composition.
            //
            // See config/file.rs prefix_domain()
            let root_domain = &main_domain[1..];
            if domain == root_domain {
                return Some("www");
            }
        }

        // Remove the '.wikijump.com' suffix, get slug
        match domain.strip_suffix(main_domain) {
            // Only 1-deep subdomains of the main domain are allowed.
            // For instance, foo.wikijump.com or bar.wikijump.com are valid,
            // but foo.bar.wikijump.com is not.
            Some(subdomain) if subdomain.contains('.') => {
                error!("Found domain '{domain}' is a sub-subdomain, invalid");
                None
            }

            Some(subdomain) => Some(subdomain),
            None => None,
        }
    }

    #[inline]
    pub fn get_canonical(config: &Config, site_slug: &str) -> String {
        // 'main_domain' is already prefixed with .
        format!("{}{}", site_slug, config.main_domain)
    }

    /// Gets the preferred domain for the given site.
    pub fn domain_for_site<'a>(config: &Config, site: &'a SiteModel) -> Cow<'a, str> {
        debug!(
            "Getting preferred domain for site '{}' (ID {})",
            site.slug, site.site_id,
        );

        match &site.custom_domain {
            Some(domain) => cow!(domain),
            None if site.slug == "www" => Self::www_domain(config),
            None => Cow::Owned(Self::get_canonical(config, &site.slug)),
        }
    }

    /// Return the preferred domain for the `www` site.
    ///
    /// This site is a special exception, instead of visiting `www.wikijump.com`
    /// it should instead redirect to just `wikijump.com`. The use of the `www`
    /// slug is an internal detail.
    fn www_domain(config: &Config) -> Cow<'static, str> {
        // This starts with . so we remove it and return
        let mut main_domain = str!(config.main_domain);
        debug_assert_eq!(main_domain.remove(0), '.');
        Cow::Owned(main_domain)
    }

    /// Gets all custom domains for a site.
    pub async fn list_custom(
        ctx: &ServiceContext<'_>,
        site_id: i64,
    ) -> Result<Vec<SiteDomainModel>> {
        info!("Getting domains for site ID {site_id}");

        let txn = ctx.transaction();
        let models = SiteDomain::find()
            .filter(site_domain::Column::SiteId.eq(site_id))
            .all(txn)
            .await?;

        Ok(models)
    }
}
