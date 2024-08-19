/*
 * endpoints/text.rs
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
use crate::web::Bytes;

pub async fn text_create(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<Bytes<'static>> {
    let contents: String = params.one()?;
    info!("Inserting new stored text (bytes {})", contents.len());
    let hash = TextService::create(ctx, contents).await?;
    Ok(Bytes::from(hash))
}

pub async fn text_get(ctx: &ServiceContext, params: Params<'static>) -> Result<String> {
    info!("Getting stored text");
    let hash: Bytes = params.one()?;
    TextService::get(ctx, hash.as_ref()).await
}
