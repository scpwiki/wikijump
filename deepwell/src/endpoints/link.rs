/*
 * endpoints/link.rs
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
use crate::services::link::{
    GetLinksExternalFrom, GetLinksExternalFromOutput, GetLinksExternalTo,
    GetLinksExternalToOutput, GetLinksFrom, GetLinksFromOutput, GetLinksTo,
    GetLinksToMissing, GetLinksToMissingOutput, GetLinksToOutput,
};

pub async fn page_links_from_get(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<GetLinksFromOutput> {
    let GetLinksFrom {
        site_id,
        page: reference,
    } = params.parse()?;

    info!("Getting page links for page {reference:?} in site ID {site_id}");
    let page_id = PageService::get_id(ctx, site_id, reference).await?;
    LinkService::get_from(ctx, page_id).await
}

pub async fn page_links_to_get(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<GetLinksToOutput> {
    let GetLinksTo {
        site_id,
        page: reference,
    } = params.parse()?;

    info!("Getting page links from page {reference:?} in site ID {site_id}");
    let page_id = PageService::get_id(ctx, site_id, reference).await?;
    LinkService::get_to(ctx, page_id, None).await
}

pub async fn page_links_to_missing_get(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<GetLinksToMissingOutput> {
    let GetLinksToMissing { site_id, page_slug } = params.parse()?;
    info!("Getting missing page links from page slug {page_slug} in site ID {site_id}",);

    LinkService::get_to_missing(ctx, site_id, &page_slug, None).await
}

pub async fn page_links_external_from(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<GetLinksExternalFromOutput> {
    let GetLinksExternalFrom {
        site_id,
        page: reference,
    } = params.parse()?;

    info!("Getting external links from page {reference:?} in site ID {site_id}",);

    let page_id = PageService::get_id(ctx, site_id, reference).await?;
    LinkService::get_external_from(ctx, page_id).await
}

pub async fn page_links_external_to(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<GetLinksExternalToOutput> {
    let GetLinksExternalTo { site_id, url } = params.parse()?;
    info!("Getting external links to URL {url} in site ID {site_id}");
    LinkService::get_external_to(ctx, site_id, &url).await
}
