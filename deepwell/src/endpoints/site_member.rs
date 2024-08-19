/*
 * endpoints/site_member.rs
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
use crate::models::relation::Model as RelationModel;
use crate::services::relation::{CreateSiteMember, GetSiteMember, RemoveSiteMember};

pub async fn membership_get(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<Option<RelationModel>> {
    let input: GetSiteMember = params.parse()?;
    RelationService::get_optional_site_member(ctx, input).await
}

pub async fn membership_set(ctx: &ServiceContext, params: Params<'static>) -> Result<()> {
    let input: CreateSiteMember = params.parse()?;
    RelationService::create_site_member(ctx, input).await
}

pub async fn membership_delete(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<RelationModel> {
    let input: RemoveSiteMember = params.parse()?;
    RelationService::remove_site_member(ctx, input).await
}
