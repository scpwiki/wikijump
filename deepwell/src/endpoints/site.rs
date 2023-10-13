/*
 * endpoints/site.rs
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
use crate::models::alias::Model as AliasModel;
use crate::models::sea_orm_active_enums::AliasType;
use crate::models::site::Model as SiteModel;
use crate::models::site_domain::Model as SiteDomainModel;
use crate::services::site::{
    CreateSite, CreateSiteOutput, GetSite, GetSiteOutput, UpdateSite,
};

pub async fn site_create(
    state: ServerState,
    params: Params<'static>,
) -> Result<CreateSiteOutput> {
    let txn = state.database.begin().await?;
    let ctx = ServiceContext::new(&state, &txn);
    let input: CreateSite = params.parse()?;
    let output = SiteService::create(&ctx, input).await?;
    txn.commit().await?;
    Ok(output)
}

pub async fn site_get(
    state: ServerState,
    params: Params<'static>,
) -> Result<GetSiteOutput> {
    let txn = state.database.begin().await?;
    let ctx = ServiceContext::new(&state, &txn);
    let GetSite { site } = params.parse()?;

    tide::log::info!("Getting site {:?}", site);
    let site = SiteService::get(&ctx, site).await?;
    let (aliases, domains) = try_join!(
        AliasService::get_all(&ctx, AliasType::Site, site.site_id),
        DomainService::list_custom(&ctx, site.site_id),
    )?;

    Ok(GetSiteOutput {
        site,
        aliases,
        domains,
    })
}

pub async fn site_update(state: ServerState, params: Params<'static>) -> Result<()> {
    let txn = state.database.begin().await?;
    let ctx = ServiceContext::new(&state, &txn);
    let UpdateSite {
        site,
        body,
        user_id,
    } = params.parse()?;

    tide::log::info!("Updating site {:?}", site);
    SiteService::update(&ctx, site, body, user_id).await?;
    txn.commit().await?;
    Ok(())
}

fn build_site_response(
    site: SiteModel,
    aliases: Vec<AliasModel>,
    domains: Vec<SiteDomainModel>,
    status: StatusCode,
) -> ApiResponse {
    let output = GetSiteOutput {
        site,
        aliases,
        domains,
    };

    let body = Body::from_json(&output)?;
    let response = Response::builder(status).body(body).into();
    Ok(response)
}
