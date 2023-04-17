/*
 * endpoints/file.rs
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
use crate::models::file::Model as FileModel;
use crate::models::file_revision::Model as FileRevisionModel;
use crate::services::file::{GetFile, GetFileOutput};
use crate::services::Result;
use crate::web::FileDetailsQuery;

pub async fn file_get(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let details: FileDetailsQuery = req.query()?;
    let GetFile {
        site_id,
        page_id,
        file: file_reference,
    } = req.body_json().await?;

    tide::log::info!(
        "Getting file {file_reference:?} from page ID {page_id} in site ID {site_id}",
    );

    // We cannot use get_id() because we need File for build_file_response().
    let file = FileService::get(&ctx, page_id, file_reference).await?;

    let revision = FileRevisionService::get_latest(&ctx, page_id, file.file_id).await?;

    let response =
        build_file_response(&ctx, &file, &revision, details, StatusCode::Ok).await?;

    txn.commit().await?;
    Ok(response)
}

pub async fn file_create(_req: ApiRequest) -> ApiResponse {
    todo!()
}

pub async fn file_edit(_req: ApiRequest) -> ApiResponse {
    todo!()
}

pub async fn file_delete(_req: ApiRequest) -> ApiResponse {
    todo!()
}

pub async fn file_move(_req: ApiRequest) -> ApiResponse {
    todo!()
}

pub async fn file_restore(_req: ApiRequest) -> ApiResponse {
    todo!()
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
        file_id: file.file_id,
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
