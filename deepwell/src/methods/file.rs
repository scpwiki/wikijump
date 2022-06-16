/*
 * methods/file.rs
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
use crate::models::file::Model as FileModel;
use crate::models::file_revision::Model as FileRevisionModel;
use crate::services::file::GetFileOutput;
use crate::services::Result;
use crate::web::FileDetailsQuery;

pub async fn file_head(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let page_reference = Reference::try_from(&req)?;
    let site_id = req.param("site_id")?.parse()?;
    let file_id = req.param("file_id")?;
    tide::log::info!("Checking existence of file ID {file_id}");

    let page = PageService::get(&ctx, site_id, page_reference)
        .await
        .to_api()?;

    let exists = FileService::exists(&ctx, page.page_id, file_id)
        .await
        .to_api()?;

    txn.commit().await?;
    exists_status(exists)
}

pub async fn file_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let page_reference = Reference::try_from(&req)?;
    let site_id = req.param("site_id")?.parse()?;
    let file_id = req.param("file_id")?;
    let details: FileDetailsQuery = req.query()?;
    tide::log::info!("Getting file ID {file_id}");

    let page = PageService::get(&ctx, site_id, page_reference)
        .await
        .to_api()?;

    let file = FileService::get(&ctx, page.page_id, file_id)
        .await
        .to_api()?;

    let revision = FileRevisionService::get_latest(&ctx, page.page_id, &file.file_id)
        .await
        .to_api()?;

    let response = build_file_response(&ctx, &file, &revision, details, StatusCode::Ok)
        .await
        .to_api()?;

    txn.commit().await?;
    Ok(response)
}

async fn build_file_response(
    ctx: &ServiceContext<'_>,
    file: &FileModel,
    revision: &FileRevisionModel,
    details: FileDetailsQuery,
    status: StatusCode,
) -> Result<Response> {
    // Get blob data, if requested
    let data = BlobService::get_maybe(ctx, details.data, &revision.s3_hash).await?;

    // Build result struct
    let output = GetFileOutput {
        file_id: &file.file_id,
        file_created_at: file.created_at,
        file_updated_at: file.updated_at,
        file_deleted_at: file.deleted_at,
        page_id: file.page_id,
        revision_id: revision.revision_id,
        revision_type: revision.revision_type,
        revision_created_at: revision.created_at,
        revision_number: revision.revision_number,
        revision_user_id: revision.user_id,
        name: &file.name,
        data,
        mime: &revision.mime_hint,
        size: revision.size_hint,
        licensing: &revision.licensing,
        revision_comments: &revision.comments,
        hidden_fields: &revision.hidden,
    };

    let body = Body::from_json(&output)?;
    let response = Response::builder(status).body(body).into();
    Ok(response)
}
