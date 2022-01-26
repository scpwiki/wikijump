/*
 * services/page/service.rs
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
use crate::models::page::{self, Entity as Page, Model as PageModel};
use crate::models::page_category::Model as PageCategoryModel;
use crate::services::revision::{
    CreateFirstRevision, CreateFirstRevisionOutput, CreateRevision, CreateRevisionBody,
    CreateRevisionBodyPresent, CreateRevisionOutput,
};
use crate::services::{CategoryService, RevisionService};
use crate::web::trim_default;
use wikidot_normalize::normalize;

#[derive(Debug)]
pub struct PageService;

impl PageService {
    pub async fn create(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        CreatePage {
            wikitext,
            title,
            alt_title,
            mut slug,
            revision_comments: comments,
            user_id,
        }: CreatePage,
    ) -> Result<CreatePageOutput> {
        let txn = ctx.transaction();
        normalize(&mut slug);

        // Check for conflicts
        let result = Page::find()
            .filter(
                Condition::all()
                    .add(page::Column::SiteId.eq(site_id))
                    .add(page::Column::Slug.eq(slug.as_str()))
                    .add(page::Column::DeletedAt.is_null()),
            )
            .one(txn)
            .await?;

        if result.is_some() {
            return Err(Error::Conflict);
        }

        // Create category if not already present
        let category = CategoryService::get_or_create(ctx, site_id, &slug).await?;

        // Insert page
        let model = page::ActiveModel {
            site_id: Set(site_id),
            page_category_id: Set(category.category_id),
            slug: Set(slug.clone()),
            ..Default::default()
        };
        let page = model.insert(txn).await?;

        // Commit first revision
        let revision_input = CreateFirstRevision {
            user_id,
            comments,
            body: CreateRevisionBodyPresent {
                wikitext,
                title,
                alt_title,
                slug: slug.clone(),
                hidden: Vec::new(),
                tags: Vec::new(),
                metadata: serde_json::json!({}),
            },
        };

        let CreateFirstRevisionOutput {
            revision_id,
            parser_warnings,
        } = RevisionService::create_first(ctx, site_id, page.page_id, revision_input)
            .await?;

        // Build and return
        Ok(CreatePageOutput {
            page_id: page.page_id,
            slug,
            revision_id,
            parser_warnings,
        })
    }

    pub async fn edit(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
        EditPage {
            wikitext,
            title,
            alt_title,
            tags,
            revision_comments: comments,
            user_id,
        }: EditPage,
    ) -> Result<Option<EditPageOutput>> {
        let txn = ctx.transaction();
        let page = Self::get(ctx, site_id, reference).await?;

        // Get latest revision
        let last_revision =
            RevisionService::get_latest(ctx, site_id, page.page_id).await?;

        // Create new revision
        //
        // A response of None means no revision was created
        // because none of the data actually changed.

        let revision_input = CreateRevision {
            user_id,
            comments,
            body: CreateRevisionBody {
                wikitext,
                title,
                alt_title,
                tags,
                ..Default::default()
            },
        };

        let revision_output = RevisionService::create(
            ctx,
            site_id,
            page.page_id,
            revision_input,
            last_revision,
        )
        .await?;

        // Only mark the page as updated if a revision was committed
        if revision_output.is_some() {
            let model = page::ActiveModel {
                page_id: Set(page.page_id),
                updated_at: Set(Some(now())),
                ..Default::default()
            };

            model.update(txn).await?;
        }

        // Build and return
        Ok(revision_output.map(|data| data.into()))
    }

    pub async fn rename(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
        new_slug: &str,
    ) -> Result<()> {
        todo!()
    }

    pub async fn delete(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
        input: DeletePage,
    ) -> Result<DeletePageOutput> {
        let txn = ctx.transaction();
        let PageModel { page_id, .. } = Self::get(ctx, site_id, reference).await?;

        // Create tombstone revision
        // This also updates backlinks, includes, etc
        let output = RevisionService::create_tombstone(
            ctx,
            site_id,
            page_id,
            input.user_id,
            input.revision_comments,
            true,
        )
        .await?;

        // Set deletion flag
        let model = page::ActiveModel {
            page_id: Set(page_id),
            deleted_at: Set(Some(now())),
            ..Default::default()
        };

        // Update and return
        model.update(txn).await?;
        Ok((output, page_id).into())
    }

    #[inline]
    pub async fn exists(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
    ) -> Result<bool> {
        Self::get_optional(ctx, site_id, reference)
            .await
            .map(|page| page.is_some())
    }

    pub async fn get(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
    ) -> Result<PageModel> {
        match Self::get_optional(ctx, site_id, reference).await? {
            Some(page) => Ok(page),
            None => Err(Error::NotFound),
        }
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
    ) -> Result<Option<PageModel>> {
        let txn = ctx.transaction();
        let page = {
            let condition = match reference {
                Reference::Id(id) => page::Column::PageId.eq(id),
                Reference::Slug(slug) => {
                    // Trim off _default category if present
                    page::Column::Slug.eq(trim_default(slug))
                }
            };

            Page::find()
                .filter(
                    Condition::all()
                        .add(condition)
                        .add(page::Column::SiteId.eq(site_id))
                        .add(page::Column::DeletedAt.is_null()),
                )
                .one(txn)
                .await?
        };

        Ok(page)
    }

    #[inline]
    pub async fn exists_direct(ctx: &ServiceContext<'_>, page_id: i64) -> Result<bool> {
        Self::get_direct_optional(ctx, page_id)
            .await
            .map(|page| page.is_some())
    }

    pub async fn get_direct(ctx: &ServiceContext<'_>, page_id: i64) -> Result<PageModel> {
        match Self::get_direct_optional(ctx, page_id).await? {
            Some(page) => Ok(page),
            None => Err(Error::NotFound),
        }
    }

    pub async fn get_direct_optional(
        ctx: &ServiceContext<'_>,
        page_id: i64,
    ) -> Result<Option<PageModel>> {
        let txn = ctx.transaction();
        let page = Page::find_by_id(page_id).one(txn).await?;
        Ok(page)
    }

    /// Get all pages in a site, with potential conditions.
    ///
    /// The `category` argument:
    /// * If it is `Some(_)`, then it specifies a reference to the category
    ///   to select from.
    /// * If it is `None`, then all pages on the site are selected.
    ///
    /// The `deleted` argument:
    /// * If it is `Some(true)`, then it only returns pages which have been deleted.
    /// * If it is `Some(false)`, then it only returns pages which are extant.
    /// * If it is `None`, then it all pages regardless of deletion status are selected.
    pub async fn get_all(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        category: Option<Reference<'_>>,
        deleted: Option<bool>,
    ) -> Result<Vec<PageModel>> {
        let txn = ctx.transaction();

        let category_condition = match category {
            None => None,
            Some(category_reference) => {
                let PageCategoryModel { category_id, .. } =
                    CategoryService::get(ctx, site_id, category_reference).await?;

                Some(page::Column::PageCategoryId.eq(category_id))
            }
        };

        let deleted_condition = match deleted {
            Some(true) => Some(page::Column::DeletedAt.is_not_null()),
            Some(false) => Some(page::Column::DeletedAt.is_null()),
            None => None,
        };

        let pages = Page::find()
            .filter(
                Condition::all()
                    .add(page::Column::SiteId.eq(site_id))
                    .add_option(category_condition)
                    .add_option(deleted_condition),
            )
            .all(txn)
            .await?;

        Ok(pages)
    }
}
