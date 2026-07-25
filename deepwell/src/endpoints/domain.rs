/*
 * endpoints/domain.rs
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
use crate::models::site_domain::Model as SiteDomainModel;
use crate::services::MutationAuthorization;
use crate::services::domain::{CreateCustomDomain, DomainService};
use crate::types::{Action, Permission, Reference, Resource};

async fn require_custom_domain_permission(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    action: &str,
) -> Result<i64> {
    MutationAuthorization::require_permission(
        ctx,
        site_id,
        None,
        Permission {
            resource_type: Resource::Site,
            resource_category: None,
            action: Action::Edit,
        },
        action,
    )
    .await
}

pub async fn site_get_domain(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<String> {
    let site_id: i64 = parse_one!(params, SiteSettings);
    let config = ctx.config();

    let site = SiteService::get(ctx, Reference::Id(site_id))
        .await
        .or_raise(|| Error::new("failed to get site domain", ErrorType::SiteSettings))?;

    let domain = DomainService::preferred_domain(config, &site);
    Ok(domain.into_owned())
}

pub async fn site_custom_domain_create(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let input: CreateCustomDomain = parse!(params, SiteSettings);
    require_custom_domain_permission(ctx, input.site_id, "create a custom domain")
        .await?;

    DomainService::create_custom(ctx, input).await.or_raise(|| {
        Error::new("failed to add a new custom domain", ErrorType::SiteSettings)
    })
}

// TODO rename

pub async fn site_custom_domain_remove(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let domain: String = parse_one!(params, SiteSettings);
    let site = DomainService::site_from_custom_domain_optional(ctx, &domain)
        .await
        .or_raise(|| {
            Error::new(
                "failed to authorize custom domain removal",
                ErrorType::SiteSettings,
            )
        })?
        .ok_or_raise(|| {
            Error::new(
                format!("cannot remove custom domain '{domain}', not found"),
                ErrorType::CustomDomainNotFound,
            )
        })?;
    require_custom_domain_permission(ctx, site.site_id, "remove a custom domain").await?;

    DomainService::remove_custom(ctx, domain)
        .await
        .or_raise(|| {
            Error::new("failed to remove a custom domain", ErrorType::SiteSettings)
        })
}

pub async fn site_custom_domain_list(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<SiteDomainModel>> {
    #[derive(Deserialize, Debug)]
    struct Input {
        site_id: i64,
    }

    let Input { site_id } = parse!(params, SiteSettings);

    DomainService::list_custom(ctx, site_id).await.or_raise(|| {
        Error::new(
            format!("failed to list custom domains for site ID {site_id}"),
            ErrorType::SiteSettings,
        )
    })
}
