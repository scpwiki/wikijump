/*
 * endpoints/parent.rs
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
use crate::models::page_parent::Model as PageParentModel;
use crate::services::parent::{
    GetParentRelationships, ParentDescription, RemoveParentOutput,
};

pub async fn parent_relationships_get(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<Vec<PageParentModel>> {
    let GetParentRelationships {
        site_id,
        page: reference,
        relationship_type,
    } = params.parse()?;

    info!(
        "Getting all {} pages from {:?} in site ID {}",
        relationship_type.name(),
        reference,
        site_id,
    );

    ParentService::get_relationships(ctx, site_id, reference, relationship_type).await
}

pub async fn parent_get(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<Option<PageParentModel>> {
    let input: ParentDescription = params.parse()?;

    info!(
        "Getting parental relationship {:?} -> {:?} in site ID {}",
        input.parent, input.child, input.site_id,
    );

    ParentService::get_optional(ctx, input).await
}

pub async fn parent_set(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<Option<PageParentModel>> {
    let input: ParentDescription = params.parse()?;

    info!(
        "Creating parental relationship {:?} -> {:?} in site ID {}",
        input.parent, input.child, input.site_id,
    );

    ParentService::create(ctx, input).await
}

pub async fn parent_remove(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<RemoveParentOutput> {
    let input: ParentDescription = params.parse()?;

    info!(
        "Removing parental relationship {:?} -> {:?} in site ID {}",
        input.parent, input.child, input.site_id,
    );

    ParentService::remove(ctx, input).await
}
