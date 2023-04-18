/*
 * endpoints/category.rs
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
use crate::models::page_category::Model as PageCategoryModel;
use crate::services::category::{CategoryOutput, GetCategory};
use crate::services::site::GetSite;

pub async fn category_get(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let GetCategory { site, category } = req.body_json().await?;
    let site_id = SiteService::get_id(&ctx, site).await?;
    tide::log::info!("Getting page category {category:?} in site ID {site_id}");

    let category = CategoryService::get(&ctx, site_id, category).await?;

    let output: CategoryOutput = category.into();
    let body = Body::from_json(&output)?;
    Ok(body.into())
}

pub async fn category_all_get(mut req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let GetSite { site } = req.body_json().await?;
    let site_id = SiteService::get_id(&ctx, site).await?;
    tide::log::info!("Getting all page categories in site ID {site_id}");

    let categories: Vec<CategoryOutput> = CategoryService::get_all(&ctx, site_id)
        .await?
        .into_iter()
        .map(PageCategoryModel::into)
        .collect();

    let body = Body::from_json(&categories)?;
    Ok(body.into())
}
