/*
 * services/domain/service.rs
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

use super::prelude::*;
use crate::models::site_domain::{self, Entity as SiteDomain, Model as SiteDomainModel};

#[derive(Debug)]
pub struct DomainService;

impl DomainService {
    pub async fn create(
        ctx: &ServiceContext<'_>,
        CreateCustomDomain { domain, site_id }: CreateCustomDomain,
    ) -> Result<()> {
        tide::log::info!("Creating custom domain '{domain}' (site ID {site_id})");

        let txn = ctx.transaction();
        if Self::site_from_domain_exists(ctx, &domain).await? {
            tide::log::error!("Custom domain already exists, cannot create");
            return Err(Error::Conflict);
        }

        let model = site_domain::ActiveModel {
            domain: Set(domain),
            site_id: Set(site_id),
            created_at: Set(now()),
            ..Default::default()
        };
        model.insert(txn).await?;
        Ok(())
    }

    /// Delete the given custom domain.
    ///
    /// Yields `Error::NotFound` if it's missing.
    pub async fn delete(ctx: &ServiceContext<'_>, domain: String) -> Result<()> {
        tide::log::info!("Deleting custom domain '{domain}'");

        let txn = ctx.transaction();
        let DeleteResult { rows_affected, .. } =
            SiteDomain::delete_by_id(domain).exec(txn).await?;

        if rows_affected == 1 {
            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }

    /// Optional version of `site_from_domain()`.
    pub async fn site_from_domain_optional(
        ctx: &ServiceContext<'_>,
        domain: &str,
    ) -> Result<Option<SiteDomainModel>> {
        tide::log::info!("Getting site for custom domain '{domain}'");

        let txn = ctx.transaction();
        let model = SiteDomain::find()
            .filter(site_domain::Column::Domain.eq(domain))
            .one(txn)
            .await?;

        Ok(model)
    }

    /// Determines if the given custom domain is registered.
    #[inline]
    pub async fn site_from_domain_exists(
        ctx: &ServiceContext<'_>,
        domain: &str,
    ) -> Result<bool> {
        Self::site_from_domain_optional(ctx, domain)
            .await
            .map(|domain| domain.is_some())
    }

    /// Gets the custom site domain configuration for the given domain.
    #[inline]
    pub async fn site_from_domain(
        ctx: &ServiceContext<'_>,
        domain: &str,
    ) -> Result<SiteDomainModel> {
        find_or_error(Self::site_from_domain_optional(ctx, domain)).await
    }

    /// Gets all custom domains for this site.
    pub async fn domains_for_site(
        ctx: &ServiceContext<'_>,
        site_id: i64,
    ) -> Result<Vec<SiteDomainModel>> {
        tide::log::info!("Getting domains for site ID {site_id}");

        let txn = ctx.transaction();
        let models = SiteDomain::find()
            .filter(site_domain::Column::SiteId.eq(site_id))
            .all(txn)
            .await?;

        Ok(models)
    }

    /// Renews the TLS certificate for a domain, if it needs it.
    pub async fn renew_certificate(
        ctx: &ServiceContext<'_>,
        domain: &str,
    ) -> Result<bool> {
        tide::log::info!("Attempting to renew certificate for {domain}");
        // TODO
        todo!()
    }
}
