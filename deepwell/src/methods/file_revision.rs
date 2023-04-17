/*
 * methods/file_revision.rs
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
use crate::services::file::GetFile;
use crate::services::file_revision::{
    FileRevisionCountOutput, GetFileRevision, GetFileRevisionRange, UpdateFileRevision,
};

pub async fn file_revision_count(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let GetFile {
        site_id,
        page_id,
        file: file_reference,
    } = req.body_json().await?;

    tide::log::info!(
        "Getting latest revision for file ID {page_id} in site ID {site_id}",
    );

    let file_id = FileService::get_id(&ctx, site_id, file_reference).await?;

    let revision_count = FileRevisionService::count(&ctx, page_id, file_id).await?;

    txn.commit().await?;
    let output = FileRevisionCountOutput {
        revision_count,
        first_revision: 0,
        last_revision: revision_count.get() - 1,
    };

    let body = Body::from_json(&output)?;
    let response = Response::builder(StatusCode::Ok).body(body).into();
    Ok(response)
}

pub async fn file_revision_get(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let GetFileRevision {
        page_id,
        file_id,
        revision_number,
    } = req.body_json().await?;

    tide::log::info!(
        "Getting file revision {revision_number} for file ID {file_id} on page ID {page_id}",
    );

    let revision =
        FileRevisionService::get(&ctx, page_id, file_id, revision_number).await?;

    txn.commit().await?;
    let body = Body::from_json(&revision)?;
    let response = Response::builder(StatusCode::Ok).body(body).into();
    Ok(response)
}

pub async fn file_revision_put(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: UpdateFileRevision = req.body_json().await?;

    tide::log::info!(
        "Editing file revision ID {} for file ID {} on page {}",
        input.revision_id,
        input.file_id,
        input.page_id,
    );

    FileRevisionService::update(&ctx, input).await?;

    txn.commit().await?;
    Ok(Response::new(StatusCode::NoContent))
}

pub async fn file_revision_range_get(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let input: GetFileRevisionRange = req.body_json().await?;
    let revisions = FileRevisionService::get_range(&ctx, input).await?;

    txn.commit().await?;
    let body = Body::from_json(&revisions)?;
    let response = Response::builder(StatusCode::Ok).body(body).into();
    Ok(response)
}
