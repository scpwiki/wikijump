/*
 * services/domain/service.rs
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

//! Service for managing domains as used by Wikijump sites.
//!
//! This service has two components, management of canonical domains (e.g. `scp-wiki.wikijump.com`)
//! and custom domains (e.g. `scpwiki.com`).

use super::structs::CreateCustomDomain;
use crate::config::Config;
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::site::{self, Entity as Site, Model as SiteModel};
use crate::models::site_domain::{self, Entity as SiteDomain, Model as SiteDomainModel};
use crate::services::ServiceContext;
use crate::utils::now;
use regex::Regex;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DeleteResult, EntityTrait, JoinType, QueryFilter,
    QuerySelect, RelationTrait, Set,
};
use std::borrow::Cow;
use std::sync::LazyLock;

pub const DEFAULT_SITE_SLUG: &str = "www";

#[derive(Debug)]
pub struct DomainService;

impl DomainService {
    /// Creates a custom domain for a site.
    pub async fn create_custom(
        ctx: &ServiceContext<'_>,
        CreateCustomDomain {
            domain,
            site_id,
            www_redirect,
        }: CreateCustomDomain,
    ) -> Result<()> {
        info!("Creating custom domain '{domain}' (site ID {site_id})");

        let make_error = || {
            Error::new(
                format!("failed to create new custom domain for site ID {}", site_id),
                ErrorType::CustomDomain,
            )
        };

        // Correctness checks

        if Self::custom_domain_exists(ctx, &domain)
            .await
            .or_raise(make_error)?
        {
            error!("Custom domain '{domain}' already exists, cannot create");
            bail!(Error::new(
                format!("cannot create domain '{}', already exists", domain),
                ErrorType::CustomDomainExists,
            ));
        }

        validate_domain(&domain).or_raise(make_error)?;

        let config = ctx.config();
        if domain.ends_with(&config.main_domain) || domain.ends_with(&config.files_domain)
        {
            error!(
                "Custom domains cannot be subdomains of the Wikijump main domain ('{}') or files domain ('{}'): {}",
                config.main_domain, config.files_domain, domain,
            );
            bail!(Error::new(
                format!(
                    "cannot create domain '{}', must not be subdomain of the Wikijump main ('{}') or files domains ('{}')",
                    domain, config.main_domain, config.files_domain
                ),
                ErrorType::CustomDomainSubdomain,
            ));
        }

        // Seems good, insert data

        let txn = ctx.transaction();
        let model = site_domain::ActiveModel {
            domain: Set(domain),
            site_id: Set(site_id),
            created_at: Set(now()),
            www_redirect: Set(www_redirect),
        };
        model.insert(txn).await.or_raise(make_error)?;
        Ok(())
    }

    /// Delete the given custom domain.
    ///
    /// Yields `ErrorType::CustomDomainNotFound` if it's missing.
    pub async fn remove_custom(ctx: &ServiceContext<'_>, domain: String) -> Result<()> {
        info!("Deleting custom domain '{domain}'");

        let make_error = || {
            Error::new(
                format!("failed to remove custom domain '{}'", domain),
                ErrorType::CustomDomain,
            )
        };

        let txn = ctx.transaction();

        let DeleteResult { rows_affected, .. } = SiteDomain::delete_by_id(&domain)
            .exec(txn)
            .await
            .or_raise(make_error)?;

        if rows_affected == 1 {
            Ok(())
        } else {
            bail!(Error::new(
                format!("cannot remove custom domain '{}', not found", domain),
                ErrorType::CustomDomainNotFound,
            ));
        }
    }

    pub async fn site_from_custom_domain_optional(
        ctx: &ServiceContext<'_>,
        domain: &str,
    ) -> Result<Option<SiteModel>> {
        info!("Getting site for custom domain {domain:?}");

        let make_error = || {
            Error::new(
                format!("failed to get site model from custom domain '{}'", domain),
                ErrorType::CustomDomain,
            )
        };

        // Join with the site table so we can get that data, rather than just the ID.
        let txn = ctx.transaction();
        let model = Site::find()
            .join(JoinType::Join, site::Relation::SiteDomain.def())
            .filter(site_domain::Column::Domain.eq(domain))
            .one(txn)
            .await
            .or_raise(make_error)?;

        Ok(model)
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

    #[inline]
    pub fn get_canonical(config: &Config, site_slug: &str) -> String {
        // 'main_domain' is already prefixed with .
        format!("{}{}", site_slug, config.main_domain)
    }

    #[inline]
    pub fn get_files(config: &Config, site_slug: &str) -> String {
        // 'files_domain' is already prefixed with .
        format!("{}{}", site_slug, config.files_domain)
    }

    /// Gets the preferred domain for the given site.
    pub fn preferred_domain<'a>(config: &'a Config, site: &'a SiteModel) -> Cow<'a, str> {
        debug!(
            "Getting preferred domain for site '{}' (ID {})",
            site.slug, site.site_id,
        );

        match &site.preferred_domain {
            Some(domain) => cow!(domain),
            None if site.slug == DEFAULT_SITE_SLUG => Self::www_domain(config),
            None => Cow::Owned(Self::get_canonical(config, &site.slug)),
        }
    }

    /// Return the preferred domain for the `www` site.
    ///
    /// This site is a special exception, instead of visiting `www.wikijump.com`
    /// it should instead redirect to just `wikijump.com`. The use of the `www`
    /// slug is an internal detail.
    #[inline]
    fn www_domain(config: &Config) -> Cow<'_, str> {
        Cow::Borrowed(&config.main_domain_no_dot)
    }

    /// Gets all custom domains for a site.
    pub async fn list_custom(
        ctx: &ServiceContext<'_>,
        site_id: i64,
    ) -> Result<Vec<SiteDomainModel>> {
        info!("Getting domains for site ID {site_id}");

        let make_error = || {
            Error::new(
                format!("failed to list all custom domains for site ID {}", site_id),
                ErrorType::CustomDomain,
            )
        };

        let txn = ctx.transaction();
        let models = SiteDomain::find()
            .filter(site_domain::Column::SiteId.eq(site_id))
            .all(txn)
            .await
            .or_raise(make_error)?;

        Ok(models)
    }
}

fn validate_domain(domain: &str) -> Result<()> {
    const PUNYCODE_PREFIX: &str = "xn--";
    const DOMAIN_MAX_BYTES: usize = 253;

    static DOMAIN_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[A-Za-z0-9\-\.]{1,253}$").unwrap());

    // First, more user-friendly check for unicode characters
    let has_unicode = !domain.is_ascii();
    if !domain.starts_with(PUNYCODE_PREFIX) && has_unicode {
        error!(
            "Custom domain '{}' has non-ASCII characters but is not using punycode",
            domain,
        );
        bail!(Error::new(
            format!(
                "domain should use punycode to convey non-ASCII characters: {}",
                domain,
            ),
            ErrorType::CustomDomainUsePunycode {
                domain: str!(domain),
            }
        ));
    }

    // Now do more specific domain validation checks
    //
    // * Domain cannot be an empty string
    // * Isn't ASCII (but we aren't giving the punycode-specific error) (reusing this prior result)
    // * Domains can only be at most 253 bytes long (excludes the trailing null byte / dot)
    // * Domains may only be composed of the limited subset of characters allowed by DNS

    macro_rules! raise_error {
        ($($arg:tt)*) => {{
            let message = format!($($arg)*);
            error!("Custom domain verification failed, {message}: {domain}");
            bail!(Error::new(message, ErrorType::InvalidDomainValue { domain: str!(domain) }));
        }};
    }

    if domain.is_empty() {
        raise_error!("domain cannot be empty");
    }

    if has_unicode {
        raise_error!("punycode domain '{}' has non-ASCII characters", domain);
    }

    if domain.len() > DOMAIN_MAX_BYTES {
        raise_error!(
            "domain '{}' is too long ({} > {})",
            domain,
            domain.len(),
            DOMAIN_MAX_BYTES,
        );
    }

    if !DOMAIN_REGEX.is_match(domain) {
        raise_error!("domain '{}' contains invalid characters", domain);
    }

    Ok(())
}

#[test]
fn test_validate_domain() {
    macro_rules! test_ok {
        ($domain:expr $(,)?) => {
            validate_domain($domain).expect("domain was detected as invalid")
        };
    }

    macro_rules! test_err {
        ($domain:expr $(,)?) => {
            validate_domain($domain).expect_err("domain was detected as valid")
        };
    }

    test_err!("");
    test_ok!("example.com");
    test_ok!("ftp.example.com");
    test_ok!("foo.bar.baz.example.com");
    test_err!(
        "this.domain.is.far.far.too.long.and.not.permitted.by.the.dns.standard.xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.example.net",
    );
    test_ok!("alpha1.beta2.my-host.wikijump.dev");
    test_ok!("ALLUPPERCASEANDABITLONG.COM");
    test_ok!("foo.bar.somesite.co.uk");
    test_err!("this has whitespace in it");
    test_err!(" this-has-whitespace-too ");
    test_err!("site1\n.com");
    test_err!("ascii$but^not_valid.net");

    // Punycode tests
    test_err!("bücher.tld");
    test_err!("xn--bücher.tld");
    test_ok!("xn--bcher-kva.tld");
    test_err!("例.tld");
    test_ok!("xn--fsq.tld");
    test_err!("🦘.tld");
    test_ok!("xn--ot9h.tld");
    test_err!("правда.ru");
    test_ok!("xn--80aafi6cg.ru");
    test_err!("ความสงบสุข.th");
    test_ok!("xn--22cdj0f7a9awc1e5c.th");
    test_err!("도메인.or.kr");
    test_ok!("xn--hq1bm8jm9l.or.kr");
    test_err!("ドメイン名例.co.jp");
    test_ok!("xn--eckwd4c7cu47r2wf.co.jp");
    test_err!("MajiでKoiする5秒前.co.jp");
    test_ok!("xn--MajiKoi5-783gue6qz075azm5e.co.jp");
}
