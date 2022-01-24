/*
 * services/category.rs
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
use crate::models::page_category::{
    self, Entity as PageCategory, Model as PageCategoryModel,
};

// Service

#[derive(Debug)]
pub struct CategoryService;

impl CategoryService {
    /// Internal method to create a category.
    ///
    /// In addition to only returning the bare ID,
    /// it also does not check for conflicts before
    /// attempting to insert.
    async fn create(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        slug: &str,
    ) -> Result<PageCategoryModel> {
        let txn = ctx.transaction();
        let model = page_category::ActiveModel {
            site_id: Set(site_id),
            slug: Set(str!(slug)),
            ..Default::default()
        };

        let category = model.insert(txn).await?;
        Ok(category)
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
    ) -> Result<Option<PageCategoryModel>> {
        let txn = ctx.transaction();
        let condition = match reference {
            Reference::Id(id) => page_category::Column::CategoryId.eq(id),
            Reference::Slug(slug) => page_category::Column::Slug.eq(slug),
        };

        let category = PageCategory::find()
            .filter(
                Condition::all()
                    .add(page_category::Column::SiteId.eq(site_id))
                    .add(condition),
            )
            .one(txn)
            .await?;

        Ok(category)
    }

    pub async fn get(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
    ) -> Result<PageCategoryModel> {
        match Self::get_optional(ctx, site_id, reference).await? {
            Some(category) => Ok(category),
            None => Err(Error::NotFound),
        }
    }

    pub async fn get_or_create(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        slug: &str,
    ) -> Result<PageCategoryModel> {
        let category =
            match Self::get_optional(ctx, site_id, Reference::from(slug)).await? {
                Some(category) => category,
                None => Self::create(ctx, site_id, slug).await?,
            };

        Ok(category)
    }
}
