/*
 * endpoints/domain.rs
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
use crate::models::site::Model as SiteModel;
use crate::services::domain::CreateCustomDomain;

pub async fn site_get_from_domain(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<Option<SiteModel>> {
    let domain: String = params.one()?;
    DomainService::site_from_domain_optional(ctx, &domain).await
}

pub async fn site_custom_domain_create(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<()> {
    let input: CreateCustomDomain = params.parse()?;
    DomainService::create_custom(ctx, input).await
}

pub async fn site_custom_domain_get(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<Option<SiteModel>> {
    let domain: String = params.one()?;
    DomainService::site_from_domain_optional(ctx, &domain).await
}

// TODO rename
pub async fn site_custom_domain_delete(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<()> {
    let domain: String = params.one()?;
    DomainService::remove_custom(ctx, domain).await
}
