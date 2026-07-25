/*
 * endpoints/link.rs
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
use crate::services::link::{
    GetLinksExternalFrom, GetLinksExternalFromOutput, GetLinksExternalTo,
    GetLinksExternalToOutput, GetLinksFrom, GetLinksFromOutput, GetLinksTo,
    GetLinksToMissing, GetLinksToMissingOutput, GetLinksToOutput,
};

pub async fn page_links_from_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<GetLinksFromOutput> {
    let GetLinksFrom {
        site_id,
        page: reference,
    } = parse!(params, Page);

    let make_error = || Error::new("failed to get page links from page", ErrorType::Page);

    let page_id = PageService::get_id(ctx, site_id, reference)
        .await
        .or_raise(make_error)?;

    LinkService::get_from(ctx, page_id)
        .await
        .or_raise(make_error)
}

pub async fn page_links_to_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<GetLinksToOutput> {
    let GetLinksTo {
        site_id,
        page: reference,
    } = parse!(params, Page);

    let make_error = || Error::new("failed to get page links to page", ErrorType::Page);

    let page_id = PageService::get_id(ctx, site_id, reference)
        .await
        .or_raise(make_error)?;

    LinkService::get_to(ctx, page_id, None)
        .await
        .or_raise(make_error)
}

pub async fn page_links_to_missing_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<GetLinksToMissingOutput> {
    let GetLinksToMissing { site_id, page_slug } = parse!(params, Page);

    LinkService::get_to_missing(ctx, site_id, &page_slug, None)
        .await
        .or_raise(|| Error::new("failed to get links to missing page", ErrorType::Page))
}

pub async fn page_links_external_from(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<GetLinksExternalFromOutput> {
    let GetLinksExternalFrom {
        site_id,
        page: reference,
    } = parse!(params);

    let make_error =
        || Error::new("failed to get external links from page", ErrorType::Page);

    let page_id = PageService::get_id(ctx, site_id, reference)
        .await
        .or_raise(make_error)?;

    LinkService::get_external_from(ctx, page_id)
        .await
        .or_raise(make_error)
}

pub async fn page_links_external_to(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<GetLinksExternalToOutput> {
    let GetLinksExternalTo { site_id, url } = parse!(params);
    LinkService::get_external_to(ctx, site_id, &url)
        .await
        .or_raise(|| Error::new("failed to get external links to URL", ErrorType::Page))
}
