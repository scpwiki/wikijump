/*
 * methods/parent.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2022 Wikijump Team
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
use crate::services::{parent::ParentalRelationshipType, ParentService};
use serde::Serialize;

pub async fn parent_relationships_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let reference = Reference::try_from(&req)?;
    let site_id = req.param("site_id")?.parse()?;
    let relationship_type: ParentalRelationshipType =
        req.param("relationship_type")?.parse()?;

    tide::log::info!(
        "Getting all {} pages from {:?} in site ID {}",
        relationship_type.name(),
        reference,
        site_id,
    );

    let models =
        ParentService::get_relationships(&ctx, site_id, reference, relationship_type)
            .await
            .to_api()?;

    txn.commit().await?;
    build_parent_response(&models, StatusCode::Ok)
}

pub async fn parent_head(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    let parent_reference =
        Reference::try_from_fields_key(&req, "parent_type", "parent_id_or_slug")?;
    let child_reference =
        Reference::try_from_fields_key(&req, "child_type", "child_id_or_slug")?;

    tide::log::info!(
        "Checking existence of parental relationship {:?} -> {:?} in site ID {}",
        parent_reference,
        child_reference,
        site_id,
    );

    let exists = ParentService::exists(&ctx, site_id, parent_reference, child_reference)
        .await
        .to_api()?;

    txn.commit().await?;
    exists_status(exists)
}

pub async fn parent_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    let parent_reference =
        Reference::try_from_fields_key(&req, "parent_type", "parent_id_or_slug")?;
    let child_reference =
        Reference::try_from_fields_key(&req, "child_type", "child_id_or_slug")?;

    tide::log::info!(
        "Getting parental relationship {:?} -> {:?} in site ID {}",
        parent_reference,
        child_reference,
        site_id,
    );

    let model = ParentService::get(&ctx, site_id, parent_reference, child_reference)
        .await
        .to_api()?;

    txn.commit().await?;
    build_parent_response(&model, StatusCode::Ok)
}

pub async fn parent_put(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    let parent_reference =
        Reference::try_from_fields_key(&req, "parent_type", "parent_id_or_slug")?;
    let child_reference =
        Reference::try_from_fields_key(&req, "child_type", "child_id_or_slug")?;

    tide::log::info!(
        "Creating parental relationship {:?} -> {:?} in site ID {}",
        parent_reference,
        child_reference,
        site_id,
    );

    let created = ParentService::create(&ctx, site_id, parent_reference, child_reference)
        .await
        .to_api()?;

    let status = if created {
        StatusCode::Created
    } else {
        StatusCode::NoContent
    };

    txn.commit().await?;
    Ok(Response::new(status))
}

pub async fn parent_delete(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    let parent_reference =
        Reference::try_from_fields_key(&req, "parent_type", "parent_id_or_slug")?;
    let child_reference =
        Reference::try_from_fields_key(&req, "child_type", "child_id_or_slug")?;

    tide::log::info!(
        "Deleting parental relationship {:?} -> {:?} in site ID {}",
        parent_reference,
        child_reference,
        site_id,
    );

    ParentService::remove(&ctx, site_id, parent_reference, child_reference)
        .await
        .to_api()?;

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

fn build_parent_response<T: Serialize>(data: &T, status: StatusCode) -> ApiResponse {
    let body = Body::from_json(data)?;
    let response = Response::builder(status).body(body).into();
    Ok(response)
}
