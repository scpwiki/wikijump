/*
 * services/parent/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
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

//! Page parenting is a specialized relation backed by `page_parent`.
//!
//! It remains separate from generic user and site relations because it enforces
//! page-specific same-site and cycle invariants.

use super::structs::{ParentDescription, ParentalRelationshipType, RemoveParentOutput};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::page::Model as PageModel;
use crate::models::page_parent::{self, Entity as PageParent, Model as PageParentModel};
use crate::services::{OutdateService, PageService, ServiceContext};
use crate::types::{Reference, RerenderDepth};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DeleteResult, EntityTrait, QueryFilter,
    QuerySelect, Set,
};

#[derive(Debug)]
pub struct ParentService;

impl ParentService {
    /// Adds a parental relationship with the two given pages.
    ///
    /// Both pages must be extant and on the same site.
    ///
    /// # Returns
    /// Returns `Some` with a model if the relationship was created,
    /// and `None` if it already existed.
    pub async fn create(
        ctx: &ServiceContext<'_>,
        ParentDescription {
            site_id,
            parent: parent_reference,
            child: child_reference,
        }: ParentDescription<'_>,
    ) -> Result<Option<PageParentModel>> {
        let txn = ctx.transaction();

        let ParentAndChild {
            parent_page,
            child_page,
        } = Self::get_parent_child(ctx, site_id, parent_reference, child_reference)
            .await
            .or_raise(|| {
                Error::new("failed to create page parents", ErrorType::PageParent)
            })?;

        let make_error = || {
            Error::new(
                format!(
                    "failed to create page parents for parent '{}' (ID {}) to child '{}' (ID {})",
                    parent_page.slug,
                    parent_page.page_id,
                    child_page.slug,
                    child_page.page_id,
                ),
                ErrorType::PageParent,
            )
        };

        // Check if the two pages are the same
        //
        // When we move to relations, this check can be REMOVED,
        // as this will be verified for us by RelationService itself.
        if parent_page.page_id == child_page.page_id {
            error!(
                "Cannot parent a page to itself (ID {})",
                parent_page.page_id,
            );
            bail!(Error::new(
                format!(
                    "cannot parent a page to itself (page '{}', ID {})",
                    parent_page.slug, parent_page.page_id
                ),
                ErrorType::BadRequest
            ));
        }

        // Check if this relationship already exists
        let relationship =
            PageParent::find_by_id((parent_page.page_id, child_page.page_id))
                .one(txn)
                .await
                .or_raise(make_error)?;

        match relationship {
            // Create new parent relationship
            None => {
                let model = page_parent::ActiveModel {
                    parent_page_id: Set(parent_page.page_id),
                    child_page_id: Set(child_page.page_id),
                    ..Default::default()
                };

                let relationship = model.insert(txn).await.or_raise(make_error)?;
                OutdateService::outdate(
                    ctx,
                    parent_page.page_id,
                    RerenderDepth::default(),
                )
                .await
                .or_raise(make_error)?;
                Ok(Some(relationship))
            }

            // Parent relationship already exists
            Some(_) => Ok(None),
        }
    }

    /// Removes the parental relationship with the two given pages.
    ///
    /// # Returns
    /// The struct contains `true` if the relationship was deleted, and
    /// `false` if it was already absent.
    pub async fn remove(
        ctx: &ServiceContext<'_>,
        ParentDescription {
            site_id,
            parent: parent_reference,
            child: child_reference,
        }: ParentDescription<'_>,
    ) -> Result<RemoveParentOutput> {
        let txn = ctx.transaction();

        let ParentAndChild {
            parent_page,
            child_page,
        } = Self::get_parent_child(ctx, site_id, parent_reference, child_reference)
            .await
            .or_raise(|| {
                Error::new("failed to remove page parents", ErrorType::PageParent)
            })?;

        let make_error = || {
            Error::new(
                format!(
                    "failed to remove page parents for parent '{}' (ID {}) to child '{}' (ID {})",
                    parent_page.slug,
                    parent_page.page_id,
                    child_page.slug,
                    child_page.page_id,
                ),
                ErrorType::PageParent,
            )
        };

        let DeleteResult { rows_affected, .. } =
            PageParent::delete_by_id((parent_page.page_id, child_page.page_id))
                .exec(txn)
                .await
                .or_raise(make_error)?;

        debug_assert!(
            rows_affected <= 1,
            "Rows deleted using ID was more than 1: {rows_affected}",
        );

        let was_deleted = rows_affected == 1;
        if was_deleted {
            OutdateService::outdate(ctx, parent_page.page_id, RerenderDepth::default())
                .await
                .or_raise(make_error)?;
        }
        Ok(RemoveParentOutput { was_deleted })
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        ParentDescription {
            site_id,
            parent: parent_reference,
            child: child_reference,
        }: ParentDescription<'_>,
    ) -> Result<Option<PageParentModel>> {
        let txn = ctx.transaction();

        let ParentAndChild {
            parent_page,
            child_page,
        } = Self::get_parent_child(ctx, site_id, parent_reference, child_reference)
            .await
            .or_raise(|| {
                Error::new("failed to get page parents", ErrorType::PageParent)
            })?;

        let make_error = || {
            Error::new(
                format!(
                    "failed to get page parents for parent '{}' (ID {}) to child '{}' (ID {})",
                    parent_page.slug,
                    parent_page.page_id,
                    child_page.slug,
                    child_page.page_id,
                ),
                ErrorType::PageParent,
            )
        };

        let model = PageParent::find_by_id((parent_page.page_id, child_page.page_id))
            .one(txn)
            .await
            .or_raise(make_error)?;

        Ok(model)
    }

    /// Gets all relationships of the given type.
    pub async fn get_relationships(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
        relationship_type: ParentalRelationshipType,
    ) -> Result<Vec<PageParentModel>> {
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!(
                    "failed to get page parents relations in site ID {} for {:?} {:?}",
                    site_id, reference, relationship_type,
                ),
                ErrorType::PageParent,
            )
        };

        let page_id = PageService::get_id(ctx, site_id, reference.borrow())
            .await
            .or_raise(make_error)?;

        let column = match relationship_type {
            ParentalRelationshipType::Parent => page_parent::Column::ChildPageId,
            ParentalRelationshipType::Child => page_parent::Column::ParentPageId,
        };

        let models = PageParent::find()
            .filter(column.eq(page_id))
            .all(txn)
            .await
            .or_raise(make_error)?;

        Ok(models)
    }

    /// Gets all parents of the given page.
    pub async fn get_parents(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
    ) -> Result<Vec<PageParentModel>> {
        Self::get_relationships(
            ctx,
            site_id,
            reference.borrow(),
            ParentalRelationshipType::Parent,
        )
        .await
        .or_raise(|| {
            Error::new(
                format!(
                    "failed to get parents of page {:?} in site ID {}",
                    reference, site_id,
                ),
                ErrorType::PageParent,
            )
        })
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

        let make_error = || {
            Error::new(
                format!(
                    "failed to remove all parent/child relationships for page ID {}",
                    page_id,
                ),
                ErrorType::PageParent,
            )
        };

        let former_parent_ids = PageParent::find()
            .select_only()
            .column(page_parent::Column::ParentPageId)
            .filter(page_parent::Column::ChildPageId.eq(page_id))
            .into_tuple()
            .all(txn)
            .await
            .or_raise(make_error)?;

        let rows_deleted = PageParent::delete_many()
            .filter(
                Condition::any()
                    .add(page_parent::Column::ParentPageId.eq(page_id))
                    .add(page_parent::Column::ChildPageId.eq(page_id)),
            )
            .exec(txn)
            .await
            .or_raise(make_error)?
            .rows_affected;

        for parent_id in former_parent_ids {
            OutdateService::outdate(ctx, parent_id, RerenderDepth::default())
                .await
                .or_raise(make_error)?;
        }

        Ok(rows_deleted)
    }
}

#[derive(Debug)]
struct ParentAndChild {
    parent_page: PageModel,
    child_page: PageModel,
}

impl ParentService {
    async fn get_parent_child(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        parent_reference: Reference<'_>,
        child_reference: Reference<'_>,
    ) -> Result<ParentAndChild> {
        let make_error = || {
            Error::new(
                "failed to get parent and child from reference",
                ErrorType::PageParent,
            )
        };

        let (parent_page_result, child_page_result) = join!(
            PageService::get(ctx, site_id, parent_reference),
            PageService::get(ctx, site_id, child_reference),
        );

        let (parent_page, child_page) =
            raise_multiple!(parent_page_result, child_page_result; make_error);

        Ok(ParentAndChild {
            parent_page,
            child_page,
        })
    }
}
