/*
 * services/page/service.rs
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

use super::prelude::*;
use crate::models::page::{self, Entity as Page, Model as PageModel};
use crate::models::page_category::Model as PageCategoryModel;
use crate::models::page_revision::Model as PageRevisionModel;
use crate::services::audit::{AuditEvent, AuditService, ObjectScope};
use crate::services::filter::{FilterClass, FilterType};
use crate::services::page_revision::{
    CreateFirstPageRevision, CreateFirstPageRevisionOutput, CreatePageRevision,
    CreatePageRevisionBody, CreatePageRevisionOutput, CreateResurrectionPageRevision,
    CreateTombstonePageRevision,
};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::{
    CategoryService, FilterService, PageRevisionService, SiteService, TextBlockService,
    TextService,
};
use crate::types::{
    Action, PageId, PageOrder, PageRevisionType, Permission, Reference, Resource,
};
use crate::utils::{get_category_name, trim_default};
use ftml::layout::Layout;
use ref_map::*;
use sea_orm::ActiveValue;
use std::net::IpAddr;
use wikidot_normalize::normalize;

#[derive(Debug)]
pub struct PageService;

impl PageService {
    pub async fn create(
        ctx: &ServiceContext<'_>,
        CreatePage {
            site_id,
            wikitext,
            title,
            alt_title,
            mut slug,
            layout,
            revision_comments: comments,
            user_id,
            bypass_filter,
            ip_address,
        }: CreatePage,
    ) -> Result<CreatePageOutput> {
        let txn = ctx.transaction();

        // Ensure slug is normalized
        normalize(&mut slug);

        let make_error = || {
            Error::new(
                format!(
                    "failed to create page '{}' in site ID {}, performed by user ID {}",
                    slug, site_id, user_id,
                ),
                ErrorType::Page,
            )
        };

        // Ensure row consistency
        Self::check_conflicts(ctx, site_id, &slug, "create")
            .await
            .or_raise(make_error)?;

        // Perform filter validation
        if !bypass_filter {
            Self::run_filter(
                ctx,
                site_id,
                None,
                Some(&wikitext),
                Some(&title),
                alt_title.as_ref(),
                ip_address,
            )
            .await
            .or_raise(make_error)?;
        }

        // Create category if not already present
        let PageCategoryModel { category_id, .. } =
            CategoryService::get_or_create(ctx, site_id, get_category_name(&slug))
                .await
                .or_raise(make_error)?;

        // Insert page
        let model = page::ActiveModel {
            site_id: Set(site_id),
            page_category_id: Set(category_id),
            slug: Set(slug.clone()),
            ..Default::default()
        };
        let PageModel { page_id, .. } = model.insert(txn).await.or_raise(make_error)?;

        // Commit first revision
        let revision_input = CreateFirstPageRevision {
            user_id,
            comments,
            wikitext,
            title,
            alt_title,
            slug: slug.clone(),
            layout,
        };

        let CreateFirstPageRevisionOutput {
            revision_id,
            parser_errors,
        } = PageRevisionService::create_first(
            ctx,
            PageId {
                site_id,
                category_id,
                page_id,
            },
            revision_input,
        )
        .await
        .or_raise(make_error)?;

        // Update latest revision
        let model = page::ActiveModel {
            page_id: Set(page_id),
            latest_revision_id: Set(Some(revision_id)),
            ..Default::default()
        };
        let page = model.update(txn).await.or_raise(make_error)?;
        assert_latest_revision(&page);

        // Audit log
        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::PageCreate {
                site_id,
                page_id,
                user_id,
                revision_id,
                category_id,
            },
        )
        .await
        .or_raise(make_error)?;

        // Build and return
        Ok(CreatePageOutput {
            page_id,
            slug,
            revision_id,
            parser_errors,
        })
    }

    pub async fn edit(
        ctx: &ServiceContext<'_>,
        EditPage {
            site_id,
            page: reference,
            last_revision_id,
            revision_comments: comments,
            user_id,
            body:
                EditPageBody {
                    wikitext,
                    title,
                    alt_title,
                    tags,
                },
            ip_address,
        }: EditPage<'_>,
    ) -> Result<Option<EditPageOutput>> {
        let txn = ctx.transaction();

        let PageModel {
            page_id,
            page_category_id: category_id,
            latest_revision_id,
            slug,
            ..
        } = Self::get(ctx, site_id, reference)
            .await
            .or_raise(|| Error::new("failed to edit page", ErrorType::Page))?;

        let id = PageId {
            site_id,
            category_id,
            page_id,
        };

        let make_error = || {
            Error::new(
                format!(
                    "failed to edit page '{}' (ID {}) in site ID {}, performed by user ID {}",
                    slug, page_id, site_id, user_id,
                ),
                ErrorType::Page,
            )
        };

        // Perform filter validation
        Self::run_filter(
            ctx,
            site_id,
            Some(page_id),
            wikitext.to_option(),
            title.to_option(),
            // Flatten what is essentially Option<Option<_>>
            match alt_title {
                Maybe::Set(Some(ref alt_title)) => Some(alt_title),
                _ => None,
            },
            ip_address,
        )
        .await
        .or_raise(make_error)?;

        // Get and check latest revision
        let last_revision = PageRevisionService::get_latest(ctx, site_id, page_id)
            .await
            .or_raise(make_error)?;

        check_last_revision(Some(&last_revision), latest_revision_id, last_revision_id)
            .or_raise(make_error)?;

        // Create new revision
        //
        // A response of None means no revision was created
        // because none of the data actually changed.

        let revision_input = CreatePageRevision {
            user_id,
            comments,
            revision_type: PageRevisionType::Regular,
            body: CreatePageRevisionBody {
                wikitext,
                title,
                alt_title,
                tags,
                ..Default::default()
            },
        };

        let revision_output =
            PageRevisionService::create(ctx, id, revision_input, last_revision)
                .await
                .or_raise(make_error)?;

        let revision_id = revision_output.ref_map(|output| output.revision_id);

        // Set page updated_at and latest_revision_id columns.
        //
        // Previously this was conditional on whether a revision was actually created.
        // But since this rerenders regardless, we need to update the page row.
        let model = page::ActiveModel {
            page_id: Set(page_id),
            latest_revision_id: match revision_id {
                Some(revision_id) => ActiveValue::Set(Some(revision_id)),
                None => ActiveValue::NotSet,
            },
            updated_at: Set(Some(now())),
            ..Default::default()
        };

        let page = model.update(txn).await.or_raise(make_error)?;
        assert_latest_revision(&page);

        // Audit log
        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::PageEdit {
                site_id,
                page_id,
                user_id,
                revision_id,
            },
        )
        .await
        .or_raise(make_error)?;

        // Build and return
        Ok(revision_output)
    }

    /// Moves a page from from one slug to another.
    pub async fn r#move(
        ctx: &ServiceContext<'_>,
        MovePage {
            site_id,
            page: reference,
            mut new_slug,
            last_revision_id,
            revision_comments: comments,
            user_id,
            ip_address,
        }: MovePage<'_>,
    ) -> Result<MovePageOutput> {
        let txn = ctx.transaction();
        normalize(&mut new_slug);

        let PageModel {
            page_id,
            page_category_id: category_id,
            slug: old_slug,
            latest_revision_id,
            ..
        } = Self::get(ctx, site_id, reference)
            .await
            .or_raise(|| Error::new("failed to move page", ErrorType::Page))?;

        let make_error = || {
            Error::new(
                format!(
                    "failed to move page '{}' to '{}' (ID {}) in site ID {}, performed by user ID {}",
                    old_slug, new_slug, page_id, site_id, user_id,
                ),
                ErrorType::Page,
            )
        };

        let id = PageId {
            site_id,
            category_id,
            page_id,
        };

        // Check last revision ID argument
        check_last_revision(None, latest_revision_id, last_revision_id)
            .or_raise(make_error)?;

        // Check that a move is actually taking place,
        // and that a page with that slug doesn't already exist.
        if old_slug == new_slug {
            bail!(Error::new(
                format!(
                    "cannot move page, source and destination slugs are the same: '{}'",
                    old_slug
                ),
                ErrorType::PageSlugExists
            ));
        }

        Self::check_conflicts(ctx, site_id, &new_slug, "move")
            .await
            .or_raise(make_error)?;

        // Create category if not already present
        let PageCategoryModel { category_id, .. } =
            CategoryService::get_or_create(ctx, site_id, get_category_name(&new_slug))
                .await
                .or_raise(make_error)?;

        // Get latest revision
        let last_revision = PageRevisionService::get_latest(ctx, site_id, page_id)
            .await
            .or_raise(make_error)?;

        // Create revision for move
        let revision_input = CreatePageRevision {
            user_id,
            comments,
            revision_type: PageRevisionType::Move,
            body: CreatePageRevisionBody {
                slug: Maybe::Set(new_slug.clone()),
                ..Default::default()
            },
        };

        let revision_output =
            PageRevisionService::create(ctx, id, revision_input, last_revision)
                .await
                .or_raise(make_error)?;

        let latest_revision_id = match revision_output {
            Some(ref output) => ActiveValue::Set(Some(output.revision_id)),
            None => ActiveValue::NotSet,
        };

        // Update page after move. This changes:
        // * slug               -- New slug for the page
        // * page_category_id   -- In case the category also changed
        // * latest_revision_id -- In case a new revision was created
        // * updated_at         -- This is updated every time a page is changed
        let model = page::ActiveModel {
            page_id: Set(page_id),
            slug: Set(new_slug.clone()),
            page_category_id: Set(category_id),
            latest_revision_id,
            updated_at: Set(Some(now())),
            ..Default::default()
        };

        let page = model.update(txn).await.or_raise(make_error)?;
        assert_latest_revision(&page);

        // Build and return
        match revision_output {
            Some(CreatePageRevisionOutput {
                revision_id,
                revision_number,
                parser_errors,
            }) => {
                // Audit log
                // Only in this path since the other is an error
                AuditService::log(
                    ctx,
                    ip_address,
                    AuditEvent::PageMove {
                        site_id,
                        page_id,
                        user_id,
                        revision_id,
                        old_slug: &old_slug,
                        new_slug: &new_slug,
                    },
                )
                .await
                .or_raise(make_error)?;

                Ok(MovePageOutput {
                    old_slug,
                    new_slug,
                    revision_id,
                    revision_number,
                    parser_errors,
                })
            }
            None => {
                error!("Page move did not create new revision");
                bail!(Error::new(
                    "page move did not create a new revision",
                    ErrorType::BadRequest
                ));
            }
        }
    }

    pub async fn delete(
        ctx: &ServiceContext<'_>,
        DeletePage {
            site_id,
            page: reference,
            last_revision_id,
            user_id,
            revision_comments: comments,
            ip_address,
        }: DeletePage<'_>,
    ) -> Result<DeletePageOutput> {
        let txn = ctx.transaction();

        let PageModel {
            page_id,
            latest_revision_id,
            slug,
            site_id,
            ..
        } = Self::get(ctx, site_id, reference)
            .await
            .or_raise(|| Error::new("failed to delete page", ErrorType::Page))?;

        let make_error = || {
            Error::new(
                format!(
                    "failed to delete page '{}' (ID {}) in site ID {}",
                    slug, page_id, site_id
                ),
                ErrorType::Page,
            )
        };

        // Get and check latest revision
        let last_revision = PageRevisionService::get_latest(ctx, site_id, page_id)
            .await
            .or_raise(make_error)?;

        check_last_revision(Some(&last_revision), latest_revision_id, last_revision_id)
            .or_raise(make_error)
            .or_raise(make_error)?;

        // Create tombstone revision
        // This also updates backlinks, includes, etc
        let output = PageRevisionService::create_tombstone(
            ctx,
            CreateTombstonePageRevision {
                site_id,
                page_id,
                user_id,
                comments,
            },
            last_revision,
        )
        .await
        .or_raise(make_error)?;

        // Set deletion flag
        let model = page::ActiveModel {
            page_id: Set(page_id),
            latest_revision_id: Set(Some(output.revision_id)),
            deleted_at: Set(Some(now())),
            ..Default::default()
        };
        let page = model.update(txn).await.or_raise(make_error)?;
        assert_latest_revision(&page);

        // Audit log
        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::PageDelete {
                site_id,
                page_id,
                user_id,
                revision_id: output.revision_id,
                page_slug: &page.slug,
            },
        )
        .await
        .or_raise(make_error)?;

        // Finally, clear out any hosted text blocks
        //
        // We do this last because this involves
        // removing from S3, which is not reversible
        // in a database transaction rollback.
        TextBlockService::delete_blocks(ctx, page_id)
            .await
            .or_raise(make_error)?;

        // Return deletion information
        Ok((output, page_id).into())
    }

    /// Restore a deleted page, causing it to be undeleted.
    pub async fn restore(
        ctx: &ServiceContext<'_>,
        RestorePage {
            site_id,
            page_id,
            user_id,
            slug,
            revision_comments: comments,
            ip_address,
        }: RestorePage,
    ) -> Result<RestorePageOutput> {
        let txn = ctx.transaction();

        let page = Self::get_direct(ctx, page_id, true)
            .await
            .or_raise(|| Error::new("failed to restore page", ErrorType::Page))?;

        let id = PageId {
            site_id,
            category_id: page.page_category_id,
            page_id,
        };

        let slug = slug.unwrap_or(page.slug);

        let make_error = || {
            Error::new(
                format!(
                    "failed to restore page '{}' (ID {}) to site ID {}",
                    slug, page_id, site_id,
                ),
                ErrorType::Page,
            )
        };

        // Do page checks:
        // - Site is correct
        // - Page is deleted
        // - Slug doesn't already exist

        if page.site_id != site_id {
            warn!(
                "Page's site ID ({}) and passed site ID ({}) do not match",
                page.site_id, site_id,
            );
            bail!(Error::new(
                "cannot restore page to a different site",
                ErrorType::PageNotFound,
            ));
        }

        if page.deleted_at.is_none() {
            warn!("Page requested to be restored is not currently deleted");
            bail!(Error::new(
                "cannot restore a page that isn't deleted",
                ErrorType::PageNotDeleted,
            ));
        }

        Self::check_conflicts(ctx, site_id, &slug, "restore")
            .await
            .or_raise(make_error)?;

        // Create category if not already present
        let category =
            CategoryService::get_or_create(ctx, site_id, get_category_name(&slug))
                .await
                .or_raise(make_error)?;

        // Get latest revision
        let last_revision = PageRevisionService::get_latest(ctx, site_id, page_id)
            .await
            .or_raise(make_error)?;

        // Create resurrection revision
        // This also updates backlinks, includes, etc.
        let output = PageRevisionService::create_resurrection(
            ctx,
            CreateResurrectionPageRevision {
                id,
                user_id,
                comments,
                new_slug: slug.clone(),
            },
            last_revision,
        )
        .await
        .or_raise(make_error)?;

        // Set deletion flag
        let model = page::ActiveModel {
            page_id: Set(page_id),
            page_category_id: Set(category.category_id),
            latest_revision_id: Set(Some(output.revision_id)),
            deleted_at: Set(None),
            ..Default::default()
        };

        let page = model.update(txn).await.or_raise(make_error)?;
        assert_latest_revision(&page);

        // Audit log
        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::PageUndelete {
                site_id,
                page_id,
                user_id,
                revision_id: output.revision_id,
                category_id: category.category_id,
                page_slug: &slug,
            },
        )
        .await
        .or_raise(make_error)?;

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
        RollbackPage {
            site_id,
            page: reference,
            last_revision_id,
            revision_number,
            revision_comments: comments,
            user_id,
            ip_address,
        }: RollbackPage<'_>,
    ) -> Result<Option<EditPageOutput>> {
        let txn = ctx.transaction();

        let PageModel {
            page_id,
            page_category_id: category_id,
            latest_revision_id,
            slug,
            ..
        } = Self::get(ctx, site_id, reference)
            .await
            .or_raise(|| Error::new("failed to roll back page", ErrorType::Page))?;

        let id = PageId {
            site_id,
            category_id,
            page_id,
        };

        let make_error = || {
            Error::new(
                format!(
                    "failed to roll back page '{}' (ID {}) on site ID {}, performed by user ID {}",
                    slug, page_id, site_id, user_id,
                ),
                ErrorType::Page,
            )
        };

        // Get target revision and latest revision
        let (target_revision, last_revision) = join!(
            PageRevisionService::get(ctx, site_id, page_id, revision_number),
            PageRevisionService::get_latest(ctx, site_id, page_id),
        );
        let (target_revision, last_revision) =
            raise_multiple!(target_revision, last_revision; make_error);

        // Check last revision ID
        check_last_revision(Some(&last_revision), latest_revision_id, last_revision_id)
            .or_raise(make_error)?;

        let PageRevisionModel {
            wikitext_hash,
            title,
            alt_title,
            tags,
            hidden,
            ..
        } = target_revision;

        let hide_wikitext = hidden.iter().any(|field| field == "wikitext");
        let hide_title = hidden.iter().any(|field| field == "title");
        let hide_alt_title = hidden.iter().any(|field| field == "alt_title");
        let hide_tags = hidden.iter().any(|field| field == "tags");

        // NOTE: we can't just copy the wikitext_hash because we
        //       need its actual value for rendering unless it has been hidden.
        //       This isn't run here, but in PageRevisionService::create().
        let wikitext = if hide_wikitext {
            Maybe::Unset
        } else {
            let text = TextService::get(ctx, &wikitext_hash)
                .await
                .or_raise(make_error)?;
            Maybe::Set(text)
        };

        // Create new revision
        //
        // Copy the body of the target revision

        let revision_input = CreatePageRevision {
            user_id,
            revision_type: PageRevisionType::Rollback,
            comments,
            body: CreatePageRevisionBody {
                wikitext,
                title: if hide_title {
                    Maybe::Unset
                } else {
                    Maybe::Set(title)
                },
                alt_title: if hide_alt_title {
                    Maybe::Unset
                } else {
                    Maybe::Set(alt_title)
                },
                tags: if hide_tags {
                    Maybe::Unset
                } else {
                    Maybe::Set(tags)
                },
                slug: Maybe::Unset, // rollbacks should never move a page
            },
        };

        let revision_output =
            PageRevisionService::create(ctx, id, revision_input, last_revision)
                .await
                .or_raise(make_error)?;

        let latest_revision_id = match revision_output {
            Some(ref output) => ActiveValue::Set(Some(output.revision_id)),
            None => ActiveValue::NotSet,
        };

        // Set page updated_at column.
        let model = page::ActiveModel {
            page_id: Set(page_id),
            updated_at: Set(Some(now())),
            latest_revision_id,
            ..Default::default()
        };

        model.update(txn).await.or_raise(make_error)?;

        // Audit log
        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::PageRollback {
                site_id,
                page_id,
                user_id,
                revision_id: revision_output.ref_map(|output| output.revision_id),
                revision_number,
            },
        )
        .await
        .or_raise(make_error)?;

        // Build and return
        Ok(revision_output)
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
        // TODO update audit-log.md
        todo!()
    }

    /// Sets the layout override for a page.
    pub async fn set_layout(
        ctx: &ServiceContext<'_>,
        SetPageLayout {
            site_id,
            page_id,
            layout,
            user_id,
            ip_address,
        }: SetPageLayout,
    ) -> Result<()> {
        debug!("Setting page layout for site ID {site_id} page ID {page_id}");

        let make_error = || {
            Error::new(
                format!(
                    "failed to set layout for page ID {} in site ID {}, performed by user ID {}",
                    page_id, site_id, user_id,
                ),
                ErrorType::Page,
            )
        };

        // Update in database
        let txn = ctx.transaction();
        let model = page::ActiveModel {
            page_id: Set(page_id),
            layout: Set(layout.map(|l| str!(l.value()))),
            ..Default::default()
        };
        model.update(txn).await.or_raise(make_error)?;

        // Audit log
        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::PageLayoutUpdate {
                user_id,
                site_id,
                page_id,
                layout,
            },
        )
        .await
        .or_raise(make_error)?;

        Ok(())
    }

    #[inline]
    pub async fn get(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
    ) -> Result<PageModel> {
        find_or_error!(Self::get_optional(ctx, site_id, reference), "page", Page)
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
                    page::Column::Slug.eq(trim_default(&slug))
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
                .await
                .or_raise(|| Error::new("failed to get page", ErrorType::Page))?
        };

        Ok(page)
    }

    /// Gets all deleted pages that match the provided slug.
    pub async fn get_deleted_by_slug(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        slug: &str,
    ) -> Result<Vec<PageModel>> {
        let txn = ctx.transaction();
        let pages = {
            Page::find()
                .filter(
                    Condition::all()
                        .add(page::Column::Slug.eq(trim_default(slug)))
                        .add(page::Column::SiteId.eq(site_id))
                        .add(page::Column::DeletedAt.is_not_null()),
                )
                .order_by_desc(page::Column::CreatedAt)
                .all(txn)
                .await
                .or_raise(|| {
                    Error::new(
                        format!(
                            "failed to get deleted page '{}' in site ID {}",
                            slug, site_id,
                        ),
                        ErrorType::Page,
                    )
                })?
        };

        Ok(pages)
    }

    /// Gets the page ID from a reference, looking up if necessary.
    ///
    /// Convenience method since this is much more common than the optional
    /// case, and we don't want to perform a redundant check for site existence
    /// later as part of the actual query.
    pub async fn get_id(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
    ) -> Result<i64> {
        match reference {
            Reference::Id(page_id) => Ok(page_id),
            Reference::Slug(slug) => {
                // For slugs we pass-through the call so that slug-handling is done consistently.
                let slug: &str = slug.as_ref();
                let PageModel { page_id, .. } =
                    Self::get(ctx, site_id, Reference::Slug(cow!(slug)))
                        .await
                        .or_raise(|| {
                            Error::new(
                                format!(
                                    "failed to get ID of page '{}' in site ID {}",
                                    slug, site_id,
                                ),
                                ErrorType::Page,
                            )
                        })?;

                Ok(page_id)
            }
        }
    }

    #[inline]
    pub async fn get_direct(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        allow_deleted: bool,
    ) -> Result<PageModel> {
        find_or_error!(
            Self::get_direct_optional(ctx, page_id, allow_deleted),
            "page",
            Page,
        )
    }

    pub async fn get_direct_optional(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        allow_deleted: bool,
    ) -> Result<Option<PageModel>> {
        let txn = ctx.transaction();
        let page = Page::find_by_id(page_id).one(txn).await.or_raise(|| {
            Error::new(
                format!(
                    "failed to get page ID {} directly (accept deleted {})",
                    page_id, allow_deleted,
                ),
                ErrorType::Page,
            )
        })?;

        if let Some(ref page) = page
            && !allow_deleted
            && page.deleted_at.is_some()
        {
            // If we're not looking for deleted pages, then
            // return nothing if the page whose ID match is.
            return Ok(None);
        }

        Ok(page)
    }

    /// Gets all pages which match the given page references.
    ///
    /// The result list is not in the same order as the input, it
    /// is up to the caller to order it if they wish.
    pub async fn get_pages(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        references: &[Reference<'_>],
    ) -> Result<Vec<PageModel>> {
        info!(
            "Getting {} pages from references in site ID {}",
            references.len(),
            site_id,
        );

        let mut filter_ids = Vec::new();
        let mut filter_slugs = Vec::new();

        for reference in references {
            match reference {
                Reference::Id(id) => filter_ids.push(*id),
                Reference::Slug(slug) => filter_slugs.push(slug.as_ref()),
            }
        }

        let txn = ctx.transaction();
        let models = Page::find()
            .filter(
                Condition::all()
                    .add(page::Column::SiteId.eq(site_id))
                    .add(page::Column::DeletedAt.is_null())
                    .add(
                        Condition::any()
                            .add(page::Column::PageId.is_in(filter_ids))
                            .add(page::Column::Slug.is_in(filter_slugs)),
                    ),
            )
            .all(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!("failed to get a series of {} pages", references.len()),
                    ErrorType::Page,
                )
            })?;

        Ok(models)
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
    /// * If it is `None`, then it returns all pages regardless of deletion status.
    ///
    /// For the `order` argument, see documentation on `PageOrder`.
    // TODO add pagination
    pub async fn get_all(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        category: Option<Reference<'_>>,
        deleted: Option<bool>,
        order: PageOrder,
    ) -> Result<Vec<PageModel>> {
        let txn = ctx.transaction();

        let make_error =
            || Error::new("failed to perform an all-site pages query", ErrorType::Page);

        let category_condition = match category {
            None => None,
            Some(category_reference) => {
                let PageCategoryModel { category_id, .. } =
                    CategoryService::get(ctx, site_id, category_reference)
                        .await
                        .or_raise(make_error)?;

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
            .order_by(order.column.into_column(), order.direction)
            .all(txn)
            .await
            .or_raise(make_error)?;

        Ok(pages)
    }

    /// Checks to see if a page already exists at the slug specified.
    ///
    /// If so, this method fails with `ErrorType::PageExists`. Otherwise it returns nothing.
    async fn check_conflicts(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        slug: &str,
        action: &str,
    ) -> Result<()> {
        let txn = ctx.transaction();

        if slug.is_empty() {
            bail!(Error::new(
                "cannot create page with empty slug",
                ErrorType::PageSlugEmpty,
            ));
        }

        let result = Page::find()
            .filter(
                Condition::all()
                    .add(page::Column::SiteId.eq(site_id))
                    .add(page::Column::Slug.eq(slug))
                    .add(page::Column::DeletedAt.is_null()),
            )
            .one(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "checking page conflicts for page '{}' on site ID {} failed",
                        slug, site_id,
                    ),
                    ErrorType::Page,
                )
            })?;

        match result {
            None => Ok(()),
            Some(page) => {
                error!(
                    "Page {} with slug '{}' already exists on site ID {}, cannot {}",
                    page.page_id, slug, site_id, action,
                );
                bail!(Error::new(
                    format!(
                        "cannot {} page '{}' on site ID {}, another page (ID {}) already exists",
                        action, slug, site_id, page.page_id,
                    ),
                    ErrorType::PageExists,
                ));
            }
        }
    }

    async fn run_filter<S: AsRef<str>>(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: Option<i64>,
        wikitext: Option<S>,
        title: Option<S>,
        alt_title: Option<S>,
        ip_address: IpAddr,
    ) -> Result<()> {
        info!("Checking page data against filters...");

        let make_error = || Error::new("filter verification failed", ErrorType::Page);

        let filter_matcher = FilterService::get_matcher(
            ctx,
            FilterClass::PlatformAndSite(site_id),
            FilterType::Page,
        )
        .await
        .or_raise(make_error)?;

        macro_rules! verify_optional {
            ($field:expr) => {
                async {
                    match $field {
                        None => Ok(()),
                        Some(value) => {
                            let field = stringify!($field);
                            let value = value.as_ref();
                            let object = match page_id {
                                Some(id) => ObjectScope::Page(id),
                                None => ObjectScope::Other,
                            };

                            filter_matcher
                                .verify(ctx, field, value, object, ip_address)
                                .await
                                .or_raise(make_error)
                        }
                    }
                }
            };
        }

        let (result1, result2, result3) = join!(
            verify_optional!(title),
            verify_optional!(alt_title),
            verify_optional!(wikitext),
        );
        raise_multiple!(result1, result2, result3; make_error);

        Ok(())
    }

    pub async fn check_user_permission(
        ctx: &ServiceContext<'_>,
        _permission_context: &CheckPermissionContext<'_>,
        action: Action,
    ) -> Result<bool> {
        let make_error =
            || Error::new("failed to check user permissions for page", ErrorType::Page);

        let page_ref = ctx.request().page_reference().or_raise(make_error)?;
        let site_id = ctx.request().site_id().or_raise(make_error)?;

        info!(
            "Checking edit permission for page {:?} in site ID {:?}",
            page_ref, site_id
        );

        let page_model = Self::get(ctx, site_id, page_ref.clone())
            .await
            .or_raise(make_error)?;

        ctx.user_has_permission(Permission {
            resource_type: Resource::Page,
            resource_category: Some(Reference::Id(page_model.page_category_id)),
            action,
        })
        .await
        .or_raise(make_error)
    }
}

/// Verifies that a `last_revision_id` passed into this function is actually the latest.
///
/// This is to avoid issues wherein a user edits overs a more recently-updated page
/// without realizing it, since attempting to make this edit would cause the backend
/// to produce an error saying that the request had too old of a revision ID and thus
/// the page would need to be refreshed.
///
/// This check is intended for before an operation has run.
fn check_last_revision(
    last_revision_model: Option<&PageRevisionModel>,
    page_latest_revision_id: Option<i64>,
    arg_last_revision_id: i64,
) -> Result<()> {
    // Only check if we have this model fetched anyways
    if let Some(model) = last_revision_model {
        assert_eq!(
            model.revision_id,
            page_latest_revision_id.expect("Page row has NULL latest_revision_id"),
            "Page table has an inconsistent last_revision_id column value",
        );
    }

    // Perform main check, ensure that the argument matches the latest
    if page_latest_revision_id != Some(arg_last_revision_id) {
        error!(
            "Latest revision ID in page struct is {}, but user argument has ID {}",
            page_latest_revision_id.unwrap(),
            arg_last_revision_id,
        );
        bail!(Error::new(
            format!(
                "last revision verification failed, actual last revision ID {}, but user passed in ID {}",
                page_latest_revision_id.unwrap(),
                arg_last_revision_id,
            ),
            ErrorType::NotLatestRevisionId,
        ));
    }

    Ok(())
}

/// Ensure that the page has a properly-set `latest_revision_id` column.
///
/// This check is intended for after an operation has run.
fn assert_latest_revision(page: &PageModel) {
    // Even in production, we want to assert that this invariant holds.
    //
    // We cannot set the column itself to NOT NULL because of cyclic update
    // requirements. However when using PageService, at no point should a method
    // quit with this value being null.

    assert!(
        page.latest_revision_id.is_some(),
        "Page ID {} (slug '{}', site ID {}) has a NULL latest_revision_id column!",
        page.page_id,
        page.slug,
        page.site_id,
    );
}
