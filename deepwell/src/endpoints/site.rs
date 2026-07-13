/*
 * endpoints/site.rs
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
use crate::models::site::Model as SiteModel;
use crate::services::MutationAuthorization;
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::site::{
    CreateSite, CreateSiteOutput, GetSite, GetSiteOutput, UpdateSite,
};
use crate::types::{Action, AliasType, Permission, Reference, Resource};

pub async fn site_create(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<CreateSiteOutput> {
    let input: CreateSite = parse!(params, Site);
    MutationAuthorization::require_authenticated(ctx, "create a site")?;

    SiteService::create(ctx, input)
        .await
        .or_raise(|| Error::new("failed to create site", ErrorType::Site))
}

pub async fn site_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<GetSiteOutput>> {
    let GetSite { site } = parse!(params, Site);
    info!("Getting site {site:?}");

    let make_error = || Error::new("failed to get site", ErrorType::Site);

    let site = SiteService::get_optional(ctx, site)
        .await
        .or_raise(make_error)?;

    match site {
        None => Ok(None),
        Some(site) => {
            let (aliases, domains) = join!(
                AliasService::get_all(ctx, AliasType::Site, site.site_id),
                DomainService::list_custom(ctx, site.site_id),
            );
            let (aliases, domains) = raise_multiple!(aliases, domains; make_error);

            Ok(Some(GetSiteOutput {
                site,
                aliases,
                domains,
            }))
        }
    }
}

pub async fn site_update(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<SiteModel> {
    let UpdateSite {
        site,
        body,
        user_id: _submitted_user_id,
        ip_address,
    } = parse!(params, Site);

    info!("Updating site {site:?}");

    let actor_user_id = ctx.request().user_id().or_raise(|| {
        Error::new(
            "user does not have permission to edit this site",
            ErrorType::PermissionDenied,
        )
    })?;

    let site_id = match site {
        Reference::Id(site_id) => site_id,
        Reference::Slug(_) => {
            return Err(Error::new(
                "site_update requires a numeric site ID",
                ErrorType::BadRequest,
            )
            .into());
        }
    };

    let can_edit = PermissionService::check_user_can(
        ctx,
        &CheckPermissionContext {
            user_id: Some(actor_user_id),
            site_id,
            page_reference: None,
        },
        Permission {
            resource_type: Resource::Site,
            resource_category: None,
            action: Action::Edit,
        },
    )
    .await
    .or_raise(|| Error::new("failed to check site edit permission", ErrorType::Site))?;

    if !can_edit {
        return Err(Error::new(
            "user does not have permission to edit this site",
            ErrorType::PermissionDenied,
        )
        .into());
    }

    SiteService::update(ctx, Reference::Id(site_id), body, actor_user_id, ip_address)
        .await
        .or_raise(|| Error::new("failed to update site data", ErrorType::Site))
}
