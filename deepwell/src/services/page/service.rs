/*
 * services/page/service.rs
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
use crate::json_utils::json_to_string_list;
use crate::models::page::{self, Entity as Page, Model as PageModel};
use crate::models::page_category::Model as PageCategoryModel;
use crate::services::revision::{
    CreateFirstRevision, CreateFirstRevisionOutput, CreateResurrectionRevision,
    CreateRevision, CreateRevisionBody, CreateRevisionOutput,
};
use crate::services::{CategoryService, RevisionService, TextService};
use crate::web::{get_category_name, trim_default};
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
        Self::check_conflicts(ctx, site_id, &slug).await?;

        // Create category if not already present
        let PageCategoryModel { category_id, .. } =
            CategoryService::get_or_create(ctx, site_id, get_category_name(&slug))
                .await?;

        // Insert page
        let model = page::ActiveModel {
            site_id: Set(site_id),
            page_category_id: Set(category_id),
            slug: Set(slug.clone()),
            ..Default::default()
        };
        let page = model.insert(txn).await?;

        // Commit first revision
        let revision_input = CreateFirstRevision {
            user_id,
            comments,
            wikitext,
            title,
            alt_title,
            slug: slug.clone(),
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
        let PageModel { page_id, .. } = Self::get(ctx, site_id, reference).await?;

        // Get latest revision
        let last_revision = RevisionService::get_latest(ctx, site_id, page_id).await?;

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

        let revision_output =
            RevisionService::create(ctx, site_id, page_id, revision_input, last_revision)
                .await?;

        // Set page updated_at column.
        //
        // Previously this was conditional on whether a revision was actually created.
        // But since this rerenders regardless, we need to update the page row.
        let model = page::ActiveModel {
            page_id: Set(page_id),
            updated_at: Set(Some(now())),
            ..Default::default()
        };

        model.update(txn).await?;

        // Build and return
        Ok(revision_output.map(|data| data.into()))
    }

    /// Moves a page from from one slug to another.
    pub async fn r#move(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
        MovePage {
            revision_comments: comments,
            user_id,
        }: MovePage,
        mut new_slug: String,
    ) -> Result<MovePageOutput> {
        let txn = ctx.transaction();

        let PageModel {
            page_id,
            slug: old_slug,
            ..
        } = Self::get(ctx, site_id, reference).await?;

        // Check that a move is actually taking place,
        // and that a page with that slug doesn't already exist.
        normalize(&mut new_slug);
        if old_slug == new_slug {
            tide::log::error!("Source and destination slugs are the same: {}", old_slug);
            return Err(Error::BadRequest);
        }

        Self::check_conflicts(ctx, site_id, &new_slug).await?;

        // Create category if not already present
        let PageCategoryModel { category_id, .. } =
            CategoryService::get_or_create(ctx, site_id, get_category_name(&new_slug))
                .await?;

        // Get latest revision
        let last_revision = RevisionService::get_latest(ctx, site_id, page_id).await?;

        // Create revision for move
        let revision_input = CreateRevision {
            user_id,
            comments,
            body: CreateRevisionBody {
                slug: ProvidedValue::Set(new_slug.clone()),
                ..Default::default()
            },
        };

        let revision_output =
            RevisionService::create(ctx, site_id, page_id, revision_input, last_revision)
                .await?;

        // Update page after move. This changes:
        // * slug             -- New slug for the page
        // * page_category_id -- In case the category also changed
        // * updated_at       -- This is updated every time a page is changed
        let model = page::ActiveModel {
            page_id: Set(page_id),
            slug: Set(new_slug.clone()),
            page_category_id: Set(category_id),
            updated_at: Set(Some(now())),
            ..Default::default()
        };

        model.update(txn).await?;

        // Build and return

        match revision_output {
            Some(CreateRevisionOutput {
                revision_id,
                revision_number,
                parser_warnings,
            }) => Ok(MovePageOutput {
                old_slug,
                new_slug,
                revision_id,
                revision_number,
                parser_warnings,
            }),
            None => {
                tide::log::error!("Page move did not create new revision");
                Err(Error::BadRequest)
            }
        }
    }

    pub async fn delete(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
        input: DeletePage,
    ) -> Result<DeletePageOutput> {
        let txn = ctx.transaction();
        let PageModel { page_id, .. } = Self::get(ctx, site_id, reference).await?;

        // Get latest revision
        let last_revision = RevisionService::get_latest(ctx, site_id, page_id).await?;

        // Create tombstone revision
        // This also updates backlinks, includes, etc
        let output = RevisionService::create_tombstone(
            ctx,
            site_id,
            page_id,
            input.user_id,
            input.revision_comments,
            last_revision,
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

    /// Restore a deleted page, causing it to be undeleted.
    pub async fn restore(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        input: RestorePage,
    ) -> Result<RestorePageOutput> {
        let txn = ctx.transaction();
        let page = Self::get_direct(ctx, page_id).await?;
        let slug = input.slug.unwrap_or(page.slug);

        // Do page checks:
        // - Site is correct
        // - Page is deleted
        // - Slug doesn't already exist

        if page.site_id != site_id {
            tide::log::warn!("Page's site ID and passed site ID do not match");
            return Err(Error::NotFound);
        }

        if page.deleted_at.is_none() {
            tide::log::warn!("Page requested to be restored is not currently deleted");
            return Err(Error::BadRequest);
        }

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
            tide::log::error!("Page with slug '{slug}' already exists on site ID {site_id}, cannot restore");
            return Err(Error::Conflict);
        }

        // Create category if not already present
        let category =
            CategoryService::get_or_create(ctx, site_id, get_category_name(&slug))
                .await?;

        // Get latest revision
        let last_revision = RevisionService::get_latest(ctx, site_id, page_id).await?;

        // Create resurrection revision
        // This also updates backlinks, includes, etc.
        let output = RevisionService::create_resurrection(
            ctx,
            site_id,
            page_id,
            CreateResurrectionRevision {
                user_id: input.user_id,
                comments: input.revision_comments,
                new_slug: slug.clone(),
            },
            last_revision,
        )
        .await?;

        // Set deletion flag
        let model = page::ActiveModel {
            page_id: Set(page_id),
            page_category_id: Set(category.category_id),
            deleted_at: Set(None),
            ..Default::default()
        };

        // Update and return
        model.update(txn).await?;
        Ok((output, slug).into())
    }

    /// Rolls back a page to be the same as it was in a previous revision.
    /// Also called "page reset".
    ///
    /// It changes the page to have the exact state it had in a previous
    /// revision, regardless of any changes since.
    ///
    /// This is equivalent to Wikidot's concept of a "revert".
    pub async fn rollback(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        revision_number: i32,
        RollbackPage {
            revision_comments: comments,
            user_id,
        }: RollbackPage,
    ) -> Result<Option<EditPageOutput>> {
        let txn = ctx.transaction();

        // Get target revision and latest revision
        let (target_revision, last_revision) = try_join!(
            RevisionService::get(ctx, site_id, page_id, revision_number),
            RevisionService::get_latest(ctx, site_id, page_id),
        )?;

        // Note: we can't just copy the wikitext_hash because we
        //       need its actual value for rendering.
        //       This isn't run here, but in RevisionService::create().
        let wikitext = TextService::get(ctx, &target_revision.wikitext_hash).await?;

        // TODO annoying JSON/array workaround
        let tags = json_to_string_list(target_revision.tags)?;

        // Create new revision
        //
        // Copy the body of the target revision

        let revision_input = CreateRevision {
            user_id,
            comments,
            body: CreateRevisionBody {
                wikitext: ProvidedValue::Set(wikitext),
                title: ProvidedValue::Set(target_revision.title),
                alt_title: ProvidedValue::Set(target_revision.alt_title),
                tags: ProvidedValue::Set(tags),
                slug: ProvidedValue::Unset, // rollbacks should never move a page
            },
        };

        let revision_output =
            RevisionService::create(ctx, site_id, page_id, revision_input, last_revision)
                .await?;

        // Set page updated_at column.
        let model = page::ActiveModel {
            page_id: Set(page_id),
            updated_at: Set(Some(now())),
            ..Default::default()
        };

        model.update(txn).await?;

        // Build and return
        Ok(revision_output.map(|data| data.into()))
    }

    /// Undoes a past revision, applying the inverse of its changes.
    ///
    /// It looks at the changes made in that revision, and does the
    /// inverse there specifically. It is contextual, and preserves
    /// all other changes made since.
    ///
    /// However, this can cause it to conflict, which will occur if
    /// the reversed changes interfere with other changes made since.
    ///
    /// This is equivalent to git's concept of a "revert".
    #[allow(dead_code)]
    pub async fn undo(
        _ctx: &ServiceContext<'_>,
        _site_id: i64,
        _page_id: i64,
        _revision_number: i32,
    ) -> Result<EditPageOutput> {
        todo!()
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
    /// * If it is `None`, then it returns all pages regardless of deletion status are selected.
    // TODO add pagination
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

    /// Checks to see if a page already exists at the slug specified.
    ///
    /// If so, this method fails with `Error::Conflict`. Otherwise it returns nothing.
    async fn check_conflicts(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        slug: &str,
    ) -> Result<()> {
        let txn = ctx.transaction();

        let result = Page::find()
            .filter(
                Condition::all()
                    .add(page::Column::SiteId.eq(site_id))
                    .add(page::Column::Slug.eq(slug))
                    .add(page::Column::DeletedAt.is_null()),
            )
            .one(txn)
            .await?;

        match result {
            None => Ok(()),
            Some(page) => {
                tide::log::error!(
                    "Page {} with slug '{}' already exists on site ID {}, cannot create",
                    page.page_id,
                    slug,
                    site_id,
                );

                Err(Error::Conflict)
            }
        }
    }
}
