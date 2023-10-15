/*
 * endpoints/parent.rs
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
use crate::models::page_parent::Model as PageParentModel;
use crate::services::page::GetPageReference;
use crate::services::parent::{GetParentRelationships, ParentDescription};
use serde::Serialize;

pub async fn parent_relationships_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<PageParentModel>> {
    let GetParentRelationships {
        site_id,
        page: reference,
        relationship_type,
    } = params.parse()?;

    tide::log::info!(
        "Getting all {} pages from {:?} in site ID {}",
        relationship_type.name(),
        reference,
        site_id,
    );

    ParentService::get_relationships(&ctx, site_id, reference, relationship_type).await
}

pub async fn parent_retrieve(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::from_req(&req, &txn);

    let input: ParentDescription = req.body_json().await?;

    tide::log::info!(
        "Getting parental relationship {:?} -> {:?} in site ID {}",
        input.parent,
        input.child,
        input.site_id,
    );

    let model = ParentService::get(&ctx, input).await?;

    txn.commit().await?;
    build_parent_response(&model, StatusCode::Ok)
}

pub async fn parent_put(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::from_req(&req, &txn);

    let input: ParentDescription = req.body_json().await?;

    tide::log::info!(
        "Creating parental relationship {:?} -> {:?} in site ID {}",
        input.parent,
        input.child,
        input.site_id,
    );

    let model = ParentService::create(&ctx, input).await?;

    let status = if model.is_some() {
        StatusCode::Created
    } else {
        StatusCode::NoContent
    };

    txn.commit().await?;
    Ok(Response::new(status))
}

pub async fn parent_delete(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::from_req(&req, &txn);

    let input: ParentDescription = req.body_json().await?;

    tide::log::info!(
        "Deleting parental relationship {:?} -> {:?} in site ID {}",
        input.parent,
        input.child,
        input.site_id,
    );

    let was_deleted = ParentService::remove(&ctx, input).await?;

    let status = if was_deleted {
        StatusCode::NoContent
    } else {
        StatusCode::Gone
    };

    txn.commit().await?;
    Ok(Response::new(status))
}

fn build_parent_response<T: Serialize>(data: &T, status: StatusCode) -> ApiResponse {
    let body = Body::from_json(data)?;
    let response = Response::builder(status).body(body).into();
    Ok(response)
}
