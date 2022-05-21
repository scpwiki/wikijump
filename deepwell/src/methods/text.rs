/*
 * methods/text.rs
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
use crate::hash::Hash;

pub async fn text_put(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let contents = req.body_string().await?;
    tide::log::info!("Inserting new stored text (bytes {})", contents.len());

    let hash = TextService::create(&ctx, contents).await.to_api()?;
    let hash_hex = hex::encode(hash);
    let body = Body::from_string(hash_hex);
    txn.commit().await?;

    Ok(body.into())
}

pub async fn text_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    tide::log::info!("Getting stored text");
    let hash = read_hash(&req)?;
    let contents = TextService::get(&ctx, &hash).await.to_api()?;
    let body = Body::from_string(contents);
    txn.commit().await?;

    Ok(body.into())
}

pub async fn text_head(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    tide::log::info!("Checking existence of stored text");
    let hash = read_hash(&req)?;
    let exists = TextService::exists(&ctx, &hash).await.to_api()?;
    txn.commit().await?;

    if exists {
        Ok(Response::new(StatusCode::NoContent))
    } else {
        Ok(Response::new(StatusCode::NotFound))
    }
}

fn read_hash(req: &ApiRequest) -> Result<Hash, TideError> {
    let hash_hex = req.param("hash")?;
    tide::log::debug!("Text hash: {hash_hex}");

    let mut hash = [0; 64];

    hex::decode_to_slice(hash_hex, &mut hash)
        .map_err(|error| TideError::new(StatusCode::UnprocessableEntity, error))?;

    Ok(hash)
}
