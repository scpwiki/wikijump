/*
 * services/parent/service.rs
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
use crate::models::page_parent::{self, Entity as PageParent, Model as PageParentModel};
use crate::services::PageService;

#[derive(Debug)]
pub struct ParentService;

impl ParentService {
    /// Adds a parental relationship with the two given pages.
    ///
    /// Both pages must be extant and on the same site.
    ///
    /// # Returns
    /// Returns `true` if the relationship was created, and
    /// `false` if it already existed.
    pub async fn create(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        parent_page_ref: Reference<'_>,
        child_page_ref: Reference<'_>,
    ) -> Result<bool> {
        let txn = ctx.transaction();

        let (parent_page, child_page) = try_join!(
            PageService::get(ctx, site_id, parent_page_ref),
            PageService::get(ctx, site_id, child_page_ref),
        )?;

        // Check if the two pages are the same
        if parent_page.page_id == child_page.page_id {
            return Err(Error::Conflict);
        }

        // Check if this relationship already exists
        let relationship =
            PageParent::find_by_id((parent_page.page_id, child_page.page_id))
                .one(txn)
                .await?;

        match relationship {
            // Create new parent relationship
            None => {
                let model = page_parent::ActiveModel {
                    parent_page_id: Set(parent_page.page_id),
                    child_page_id: Set(child_page.page_id),
                    ..Default::default()
                };

                model.insert(txn).await?;
                Ok(true)
            }

            // Parent relationship already exists
            Some(_) => Ok(false),
        }
    }

    /// Removes the parental relationship with the two given pages.
    ///
    /// # Returns
    /// Returns `true` if the relationship was deleted, and
    /// `false` if it was already absent.
    pub async fn remove(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        parent_page_ref: Reference<'_>,
        child_page_ref: Reference<'_>,
    ) -> Result<bool> {
        let txn = ctx.transaction();

        let (parent_page, child_page) = try_join!(
            PageService::get(ctx, site_id, parent_page_ref),
            PageService::get(ctx, site_id, child_page_ref),
        )?;

        let rows_deleted =
            PageParent::delete_by_id((parent_page.page_id, child_page.page_id))
                .exec(txn)
                .await?
                .rows_affected;

        Ok(rows_deleted == 1)
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        parent_page_ref: Reference<'_>,
        child_page_ref: Reference<'_>,
    ) -> Result<Option<PageParentModel>> {
        let txn = ctx.transaction();

        let (parent_page, child_page) = try_join!(
            PageService::get(ctx, site_id, parent_page_ref),
            PageService::get(ctx, site_id, child_page_ref),
        )?;

        let model = PageParent::find_by_id((parent_page.page_id, child_page.page_id))
            .one(txn)
            .await?;

        Ok(model)
    }

    #[inline]
    pub async fn exists(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        parent_page_ref: Reference<'_>,
        child_page_ref: Reference<'_>,
    ) -> Result<bool> {
        Self::get_optional(ctx, site_id, parent_page_ref, child_page_ref)
            .await
            .map(|model| model.is_some())
    }

    #[inline]
    pub async fn get(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        parent_page_ref: Reference<'_>,
        child_page_ref: Reference<'_>,
    ) -> Result<PageParentModel> {
       Self::get_optional(ctx, site_id, parent_page_ref, child_page_ref).await?.ok_or(Error::NotFound)
    }

    /// Gets all relationships of the given type.
    pub async fn get_relationships(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
        relationship_type: ParentalRelationshipType,
    ) -> Result<Vec<PageParentModel>> {
        let txn = ctx.transaction();
        let page = PageService::get(ctx, site_id, reference).await?;
        let column = match relationship_type {
            ParentalRelationshipType::Parent => page_parent::Column::ParentPageId,
            ParentalRelationshipType::Child => page_parent::Column::ChildPageId,
        };

        let models = PageParent::find()
            .filter(column.eq(page.page_id))
            .all(txn)
            .await?;

        Ok(models)
    }

    /// Removes all parent relationships involving this page.
    ///
    /// Whether this page is a parent or a child, this method
    /// will remove all those relationships.
    ///
    /// # Returns
    /// Returns the number of relationships deleted.
    pub async fn remove_all(ctx: &ServiceContext<'_>, page_id: i64) -> Result<u64> {
        let txn = ctx.transaction();

        let rows_deleted = PageParent::delete_many()
            .filter(
                Condition::any()
                    .add(page_parent::Column::ParentPageId.eq(page_id))
                    .add(page_parent::Column::ChildPageId.eq(page_id)),
            )
            .exec(txn)
            .await?
            .rows_affected;

        Ok(rows_deleted)
    }
}
