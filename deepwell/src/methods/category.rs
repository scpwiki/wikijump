/*
 * methods/category.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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
use crate::services::category::CategoryOutput;

pub async fn category_head_direct(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let category_id = req.param("category_id")?.parse()?;
    tide::log::info!("Checking existence of category ID {category_id}");

    let exists = CategoryService::exists_direct(&ctx, category_id)
        .await
        .to_api()?;

    txn.commit().await?;
    exists_status(exists)
}

pub async fn category_get_direct(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let category_id = req.param("category_id")?.parse()?;
    tide::log::info!("Getting category ID {category_id}");

    let category = CategoryService::get_direct(&ctx, category_id)
        .await
        .to_api()?;

    let output: CategoryOutput = category.into();
    let body = Body::from_json(&output)?;
    Ok(body.into())
}

pub async fn category_head(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    let reference = Reference::try_from(&req)?;
    tide::log::info!(
        "Checking existence of page category {reference:?} in site ID {site_id}",
    );

    let exists = CategoryService::exists(&ctx, site_id, reference)
        .await
        .to_api()?;

    txn.commit().await?;
    exists_status(exists)
}

pub async fn category_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    let reference = Reference::try_from(&req)?;
    tide::log::info!("Getting page category {reference:?} in site ID {site_id}");

    let category = CategoryService::get(&ctx, site_id, reference)
        .await
        .to_api()?;

    let output: CategoryOutput = category.into();
    let body = Body::from_json(&output)?;
    Ok(body.into())
}

pub async fn category_all_get(req: ApiRequest) -> ApiResponse {
    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);

    let site_id = req.param("site_id")?.parse()?;
    tide::log::info!("Getting all page categories in site ID {site_id}");

    let categories: Vec<CategoryOutput> = CategoryService::get_all(&ctx, site_id)
        .await
        .to_api()?
        .into_iter()
        .map(PageCategoryModel::into)
        .collect();

    let body = Body::from_json(&categories)?;
    Ok(body.into())
}
