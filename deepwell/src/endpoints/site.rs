/*
 * endpoints/site.rs
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

use super::prelude::*;
use crate::models::sea_orm_active_enums::AliasType;
use crate::models::site::Model as SiteModel;
use crate::services::site::{
    CreateSite, CreateSiteOutput, GetSite, GetSiteOutput, UpdateSite,
};

pub async fn site_create(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<CreateSiteOutput> {
    let input: CreateSite = params.parse()?;
    SiteService::create(ctx, input).await
}

pub async fn site_get(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<Option<GetSiteOutput>> {
    let GetSite { site } = params.parse()?;
    info!("Getting site {:?}", site);
    match SiteService::get_optional(ctx, site).await? {
        None => Ok(None),
        Some(site) => {
            let (aliases, domains) = try_join!(
                AliasService::get_all(ctx, AliasType::Site, site.site_id),
                DomainService::list_custom(ctx, site.site_id),
            )?;

            Ok(Some(GetSiteOutput {
                site,
                aliases,
                domains,
            }))
        }
    }
}

pub async fn site_update(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<SiteModel> {
    let UpdateSite {
        site,
        body,
        user_id,
    } = params.parse()?;

    info!("Updating site {:?}", site);
    SiteService::update(ctx, site, body, user_id).await
}
