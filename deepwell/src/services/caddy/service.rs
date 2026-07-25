/*
 * services/caddy/service.rs
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

//! Service to handle the [Caddy webserver](https://caddyserver.com/docs/).
//!
//! This is primarily concerned with generating the `Caddyfile` that
//! powers the server, which is where host → site mapping is performed.

use super::structs::{CaddyfileOptions, CustomDomainData, SiteData, SiteDomainData};
use crate::config::Config;
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::alias::Model as AliasModel;
use crate::models::site::{self, Entity as Site};
use crate::models::site_domain::{self, Entity as SiteDomain};
use crate::services::ServiceContext;
use crate::services::domain::DEFAULT_SITE_SLUG;
use crate::services::{AliasService, DomainService};
use crate::types::AliasType;
use askama::Template;
use sea_orm::{ColumnTrait, QueryFilter, QueryOrder};
use sea_orm::{EntityTrait, QuerySelect};
use std::borrow::Cow;
use std::collections::HashMap;

// Askama template for generating the Caddyfile

#[derive(Template, Debug)]
#[template(path = "caddyfile.j2", escape = "none")]
struct CaddyTemplate<'a> {
    // Basic options
    debug: bool,
    local: bool,
    http_port: Option<u16>,
    https_port: Option<u16>,

    // TLS
    wildcard_cert: Option<&'a str>,
    should_use_wildcard: bool,

    // Reverse proxy destinations
    deploy_host: Option<&'a str>,
    framerail_host: &'a str,
    wws_host: &'a str,

    // Instance configuration
    config: &'a Config,
    files_domain: &'a str,
    files_domain_no_dot: &'a str,
    main_domain: &'a str,

    // Site and domain data
    sites: &'a [(i64, String, Option<String>)],
    domains: &'a HashMap<i64, SiteDomainData>,
}

// Actual service

#[derive(Debug)]
pub struct CaddyService;

impl CaddyService {
    pub async fn generate(
        ctx: &ServiceContext<'_>,
        options: &CaddyfileOptions<'_>,
    ) -> Result<String> {
        let config = ctx.config();
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!("failed to generate Caddyfile with options {:#?}", options),
                ErrorType::Caddyfile,
            )
        };

        let sites: Vec<(i64, String, Option<String>)> = Site::find()
            .select_only()
            .column(site::Column::SiteId)
            .column(site::Column::Slug)
            .column(site::Column::PreferredDomain)
            .order_by_asc(site::Column::SiteId)
            .into_tuple()
            .all(txn)
            .await
            .or_raise(make_error)?;

        let domains = {
            let mut domains = HashMap::with_capacity(sites.len());

            for &(site_id, _, _) in &sites {
                let aliases = AliasService::get_all(ctx, AliasType::Site, site_id)
                    .await
                    .or_raise(make_error)?
                    .into_iter()
                    .map(|AliasModel { slug, .. }| slug)
                    .collect();

                let custom_domains = SiteDomain::find()
                    .order_by_asc(site_domain::Column::Domain)
                    .select_only()
                    .column(site_domain::Column::Domain)
                    .column(site_domain::Column::WwwRedirect)
                    .filter(site_domain::Column::SiteId.eq(site_id))
                    .into_model::<CustomDomainData>()
                    .all(txn)
                    .await
                    .or_raise(make_error)?;

                domains.insert(
                    site_id,
                    SiteDomainData {
                        aliases,
                        custom_domains,
                    },
                );
            }

            domains
        };

        let caddyfile =
            Self::generate_with_data(config, options, &SiteData { sites, domains })
                .or_raise(make_error)?;

        Ok(caddyfile)
    }

    pub fn generate_with_data(
        config: &Config,
        CaddyfileOptions {
            debug,
            local,
            http_port,
            https_port,
            wildcard_cert,
            deploy_host,
            framerail_host,
            wws_host,
        }: &CaddyfileOptions<'_>,
        SiteData { sites, domains }: &SiteData,
    ) -> Result<String> {
        info!("Generating Caddyfile for {} sites", sites.len());

        let template = CaddyTemplate {
            debug: *debug,
            local: *local,
            http_port: *http_port,
            https_port: *https_port,
            wildcard_cert: wildcard_cert.as_deref(),
            should_use_wildcard: wildcard_cert.is_some() || *local,
            deploy_host: deploy_host.as_deref(),
            framerail_host,
            wws_host,
            config,
            files_domain: &config.files_domain,
            files_domain_no_dot: &config.files_domain_no_dot,
            main_domain: &config.main_domain,
            sites,
            domains,
        };

        let caddyfile = template.render().or_raise(|| {
            Error::new(
                format!("failed to generate Caddyfile for {} sites", sites.len()),
                ErrorType::Caddyfile,
            )
        })?;

        Ok(caddyfile)
    }
}

// Helper functions for rendering

fn get_canonical_domain<'s>(config: &'s Config, site_slug: &'s str) -> Cow<'s, str> {
    if site_slug == DEFAULT_SITE_SLUG {
        Cow::Borrowed(&config.main_domain_no_dot)
    } else {
        Cow::Owned(DomainService::get_canonical(config, site_slug))
    }
}

fn get_preferred_domain<'s>(
    custom_domains: &[CustomDomainData],
    preferred_domain: &'s Option<String>,
    canonical_domain: &'s str,
) -> (&'s str, bool) {
    match preferred_domain {
        None => (canonical_domain, true),
        Some(preferred_domain) => {
            let preferred_domain = preferred_domain.as_str();

            custom_domains
                .iter()
                .find(|custom| custom.domain == preferred_domain)
                .map(|custom| (preferred_domain, custom.www_redirect))
                // Consistency error: Preferred custom domain should be in the list of the site's custom domains
                .expect("No matching custom domain for preferred domain")
        }
    }
}

/// Determines the index to give to Caddy to get the site slug from a domain.
///
/// Caddy enables us to extract parts of a domain via indices, with 0 being
/// the top-level domain (TLD).
///
/// So if the files domain (with dot) is `.wjfiles.com`, there are 2 periods.
/// Any site slugs would be before that first dot, such as in `foo.wjfiles.com`,
/// which would be index 2 using Caddy's domain addressing system:
///
/// 0 - "com"
/// 1 - "wjfiles"
/// 2 - "foo"      <-- what we want
///
/// An additional example, say the domain is `.host.wikijump.example.com`,
/// then there are 4 dots in the domain, and thus the zero-based index is 4:
///
/// 0 - "com"
/// 1 - "example"
/// 2 - "wikijump"
/// 3 - "host"
/// 4 - "foo"      <-- what we want
fn site_slug_split_index(domain: &str) -> usize {
    debug_assert_eq!(
        domain.chars().next(),
        Some('.'),
        "Didn't pass a dot-prefixed domain",
    );
    domain.chars().filter(|&c| c == '.').count()
}

#[test]
fn test_site_slug_split_index() {
    const TEST_SITE_SLUG: &str = "XYZ";

    macro_rules! check {
        ($domain:expr, $expected_index:expr) => {{
            // Get and validate index
            let index = site_slug_split_index($domain);
            assert_eq!(
                index, $expected_index,
                "Caddy site slug extraction index does not match",
            );

            // Test it on a real string
            let domain = format!("{}{}", TEST_SITE_SLUG, $domain);
            let parts = domain.split('.').rev().collect::<Vec<_>>();
            assert_eq!(parts[index], TEST_SITE_SLUG, "Wrong substring extracted");
        }};
    }

    check!(".wikijump.com", 2);
    check!(".foo.example.org", 3);
    check!(".bar.foo.example.org", 4);
    check!(".alpha.beta.gamma.delta.epsilon.zeta.eta.wjfiles.com", 9);
}
