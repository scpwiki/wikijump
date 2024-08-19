/*
 * services/page_revision/service.rs
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
use crate::models::page_revision::{
    self, Entity as PageRevision, Model as PageRevisionModel,
};
use crate::models::sea_orm_active_enums::PageRevisionType;
use crate::services::render::RenderOutput;
use crate::services::score::ScoreValue;
use crate::services::{
    LinkService, OutdateService, PageService, ParentService, RenderService, ScoreService,
    SiteService, TextService,
};
use crate::utils::{split_category, split_category_name};
use crate::web::FetchDirection;
use ftml::data::PageInfo;
use ftml::layout::Layout;
use ftml::settings::{WikitextMode, WikitextSettings};
use once_cell::sync::Lazy;
use ref_map::*;
use std::num::NonZeroI32;

/// The changes for the first revision.
/// The first revision is always considered to have changed everything.
///
/// See `services/file_revision/service.rs`.
static ALL_CHANGES: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        str!("wikitext"),
        str!("title"),
        str!("alt_title"),
        str!("slug"),
        str!("tags"),
    ]
});

macro_rules! conditional_future {
    ($conditional:expr, $future:expr $(,)?) => {
        async move {
            if $conditional {
                $future.await
            } else {
                Ok(())
            }
        }
    };
}

#[derive(Debug)]
pub struct PageRevisionService;

impl PageRevisionService {
    /// Creates a new revision on an existing page.
    ///
    /// For the given page, look at the changes to make. If there are none,
    /// or they are all equivalent to the previous revision's, then no
    /// revision is committed and `Ok(None)` is returned.
    ///
    /// If there are changes, then the new revision is created and all the
    /// appropriate updating is done. For instance, recompiling the page
    /// or updating backlinks.
    ///
    /// For page renames, this does not explicitly check if the target slug
    /// already exists. If so, the database will fail with a uniqueness error.
    /// This is checked in `PageService::rename()`, where renames should be done from.
    ///
    /// The revision number is subject to an invariant:
    /// * For a new page, then the value must be `0` (this corresponds with a `previous` of `None`).
    /// * For an existing page, then the value must be precisely one greater than the previous
    ///   revision's number. No holes are permitted in the revision count, for some maximum
    ///   revision number `n`, there must be revisions for each revision number from `0` to `n`
    ///   inclusive.
    ///
    /// This is enforced by requiring the previous revision be passed in during creation.
    ///
    /// # Panics
    /// If the given previous revision is for a different page or site, this method will panic.
    pub async fn create(
        ctx: &ServiceContext,
        site_id: i64,
        page_id: i64,
        CreatePageRevision {
            user_id,
            comments,
            body,
        }: CreatePageRevision,
        previous: PageRevisionModel,
    ) -> Result<Option<CreatePageRevisionOutput>> {
        let txn = ctx.seaorm_transaction();
        let revision_number = next_revision_number(&previous, site_id, page_id);

        // Fields to create in the revision
        let mut parser_errors = None;
        let mut old_slug = None;
        let mut changes = Vec::new();
        let PageRevisionModel {
            mut wikitext_hash,
            mut compiled_hash,
            mut compiled_at,
            mut compiled_generator,
            hidden,
            mut title,
            mut alt_title,
            mut slug,
            mut tags,
            ..
        } = previous;

        // Update fields from input
        //
        // We check the values so that the only listed "changes"
        // are those that actually are different.

        if let ProvidedValue::Set(new_title) = body.title {
            if title != new_title {
                changes.push(str!("title"));
                title = new_title;
            }
        }

        if let ProvidedValue::Set(new_alt_title) = body.alt_title {
            if alt_title != new_alt_title {
                changes.push(str!("alt_title"));
                alt_title = new_alt_title;
            }
        }

        if let ProvidedValue::Set(new_slug) = body.slug {
            if slug != new_slug {
                changes.push(str!("slug"));
                old_slug = Some(slug);
                slug = new_slug;
            }
        }

        if let ProvidedValue::Set(new_tags) = body.tags {
            if tags != new_tags {
                changes.push(str!("tags"));
                tags = new_tags;
            }
        }

        // Get slug strings for the new location
        let (category_slug, page_slug) = split_category_name(&slug);

        // Get wikitext, set wikitext hash
        let wikitext = match body.wikitext {
            // Insert new wikitext and update hash
            ProvidedValue::Set(new_wikitext) => {
                let new_hash = TextService::create(ctx, new_wikitext.clone()).await?;

                if wikitext_hash != new_hash {
                    changes.push(str!("wikitext"));
                    replace_hash(&mut wikitext_hash, &new_hash);
                }

                new_wikitext
            }

            // Use previous revision's wikitext
            ProvidedValue::Unset => TextService::get(ctx, &wikitext_hash).await?,
        };

        // If nothing has changed, then don't create a new revision
        if changes.is_empty() {
            Self::rerender(ctx, site_id, page_id, 0).await?;
            return Ok(None);
        }

        // Get ancillary page data
        let (score, layout) = try_join!(
            ScoreService::score(ctx, page_id),
            PageService::get_layout(ctx, site_id, page_id),
        )?;

        // Run tasks based on changes:
        // See PageRevisionTasks struct for more information.
        let tasks = PageRevisionTasks::determine(&changes);

        if tasks.render_and_update_links {
            // This is necessary until we are able to replace the
            // 'tags' column with TEXT[] instead of JSON.
            let render_input = RenderPageInfo {
                layout,
                slug: &slug,
                title: &title,
                alt_title: alt_title.ref_map(|s| s.as_str()),
                score,
                tags: &tags,
            };

            // Run renderer and related tasks
            //
            // Since outdating depends on scope (see PageRevisionTasks),
            // we don't do that right after here.
            //
            // TODO: use html_output
            let render_output = Self::render_and_update_links(
                ctx,
                site_id,
                page_id,
                wikitext,
                render_input,
            )
            .await?;

            // Update fields
            parser_errors = Some(render_output.errors);
            replace_hash(&mut compiled_hash, &render_output.compiled_hash);
            compiled_generator = render_output.compiled_generator;
            compiled_at = now();
        }

        // Perform outdating based on changes made.
        //
        // Also, determine the revision type.
        // If the slug changes it's "move", otherwise "regular".
        let revision_type = match old_slug {
            Some(ref old_slug) => {
                // If there's an "old slug" set, then this is a page rename / move.
                // Thus we should invoke the OutdateService for both the source
                // and destination.
                //
                // This is equivalent to the three outdate calls below, but for
                // the source and destination slugs, which is why we don't
                // also run those again.

                OutdateService::process_page_move(
                    ctx, site_id, page_id, old_slug, &slug, 0,
                )
                .await?;

                PageRevisionType::Move
            }
            None => {
                // Run all outdating tasks in parallel.
                //
                // This macro runs the given method (second value) if the condition (first value)
                // is true, otherwise does nothing.

                try_join!(
                    conditional_future!(
                        tasks.rerender_incoming_links,
                        OutdateService::outdate_incoming_links(ctx, page_id, 0),
                    ),
                    conditional_future!(
                        tasks.rerender_outgoing_includes,
                        OutdateService::outdate_outgoing_includes(ctx, page_id, 0),
                    ),
                    conditional_future!(
                        tasks.rerender_templates,
                        OutdateService::outdate_templates(
                            ctx,
                            site_id,
                            category_slug,
                            page_slug,
                            0,
                        ),
                    ),
                )?;

                PageRevisionType::Regular
            }
        };

        // Insert the new revision into the table
        let model = page_revision::ActiveModel {
            revision_type: Set(revision_type),
            revision_number: Set(revision_number),
            page_id: Set(page_id),
            site_id: Set(site_id),
            user_id: Set(user_id),
            changes: Set(changes),
            wikitext_hash: Set(wikitext_hash),
            compiled_hash: Set(compiled_hash),
            compiled_at: Set(compiled_at),
            compiled_generator: Set(compiled_generator),
            comments: Set(comments),
            hidden: Set(hidden),
            title: Set(title),
            alt_title: Set(alt_title),
            slug: Set(slug),
            tags: Set(tags),
            ..Default::default()
        };

        let PageRevisionModel { revision_id, .. } = model.insert(txn).await?;
        Ok(Some(CreatePageRevisionOutput {
            revision_id,
            revision_number,
            parser_errors,
        }))
    }

    /// Creates the first revision for a newly-inserted page.
    ///
    /// The first revision of a page is special.
    /// A revision change cannot be missing any fields (since there is
    /// not a previous revision to take prior data from), and always
    /// inserts, since it's not possible for it to be an empty revision
    /// (since there's no prior revision for it to be equal to).
    pub async fn create_first(
        ctx: &ServiceContext,
        site_id: i64,
        page_id: i64,
        CreateFirstPageRevision {
            user_id,
            comments,
            wikitext,
            title,
            alt_title,
            slug,
            layout,
        }: CreateFirstPageRevision,
    ) -> Result<CreateFirstPageRevisionOutput> {
        let txn = ctx.seaorm_transaction();

        // If the page creation doesn't specify a preferred layout,
        // use the default for the site.
        let layout = match layout {
            None => SiteService::get_layout(ctx, site_id).await?,
            Some(layout) => layout,
        };

        // Get ancillary page data
        let (wikitext_hash, score) = try_join!(
            TextService::create(ctx, wikitext.clone()),
            ScoreService::score(ctx, page_id),
        )?;

        // Render first revision
        let render_input = RenderPageInfo {
            layout,
            slug: &slug,
            title: &title,
            alt_title: alt_title.ref_map(|s| s.as_str()),
            score,
            tags: &[], // Initial revision always has empty tags
        };

        let RenderOutput {
            // TODO: use html_output
            html_output: _,
            errors,
            compiled_hash,
            compiled_at,
            compiled_generator,
        } = Self::render_and_update_links(ctx, site_id, page_id, wikitext, render_input)
            .await?;

        // Run outdater
        OutdateService::process_page_displace(ctx, site_id, page_id, &slug, 0).await?;

        // Insert the first revision into the table
        let model = page_revision::ActiveModel {
            revision_type: Set(PageRevisionType::Create),
            revision_number: Set(0),
            page_id: Set(page_id),
            site_id: Set(site_id),
            user_id: Set(user_id),
            changes: Set(ALL_CHANGES.clone()),
            wikitext_hash: Set(wikitext_hash.to_vec()),
            compiled_hash: Set(compiled_hash.to_vec()),
            compiled_at: Set(compiled_at),
            compiled_generator: Set(compiled_generator),
            comments: Set(comments),
            hidden: Set(vec![]),
            title: Set(title),
            alt_title: Set(alt_title),
            slug: Set(slug),
            tags: Set(vec![]),
            ..Default::default()
        };

        let PageRevisionModel { revision_id, .. } = model.insert(txn).await?;
        Ok(CreateFirstPageRevisionOutput {
            revision_id,
            parser_errors: errors,
        })
    }

    /// Creates a revision marking a page as deleted.
    ///
    /// This revision is called a "tombstone" in that
    /// its only purpose is to mark that the page has been deleted.
    ///
    /// # Panics
    /// If the given previous revision is for a different page or site, this method will panic.
    pub async fn create_tombstone(
        ctx: &ServiceContext,
        CreateTombstonePageRevision {
            site_id,
            page_id,
            user_id,
            comments,
        }: CreateTombstonePageRevision,
        previous: PageRevisionModel,
    ) -> Result<CreatePageRevisionOutput> {
        let txn = ctx.seaorm_transaction();
        let revision_number = next_revision_number(&previous, site_id, page_id);

        let PageRevisionModel {
            wikitext_hash,
            compiled_hash,
            compiled_at,
            compiled_generator,
            title,
            alt_title,
            slug,
            tags,
            ..
        } = previous;

        // Run outdater
        OutdateService::process_page_displace(ctx, site_id, page_id, &slug, 0).await?;

        // Delete parent-child relationships, if any
        ParentService::remove_all(ctx, page_id).await?;

        // Insert the tombstone revision into the table
        let model = page_revision::ActiveModel {
            revision_type: Set(PageRevisionType::Delete),
            revision_number: Set(revision_number),
            page_id: Set(page_id),
            site_id: Set(site_id),
            user_id: Set(user_id),
            changes: Set(vec![]),
            wikitext_hash: Set(wikitext_hash),
            compiled_hash: Set(compiled_hash),
            compiled_at: Set(compiled_at),
            compiled_generator: Set(compiled_generator),
            comments: Set(comments),
            hidden: Set(vec![]),
            title: Set(title),
            alt_title: Set(alt_title),
            slug: Set(slug),
            tags: Set(tags),
            ..Default::default()
        };

        let PageRevisionModel { revision_id, .. } = model.insert(txn).await?;
        Ok(CreatePageRevisionOutput {
            revision_id,
            revision_number,
            parser_errors: None,
        })
    }

    /// Creates a revision marking a pages as restored (i.e., undeleted).
    ///
    /// Similar to `create_tombstone`, this method creates
    /// a revision whose only purpose is to mark that the page
    /// has been restored.
    ///
    /// Note that page parenting information is removed during deletion
    /// and is not restored here.
    ///
    /// Remember that, like `create_first()`, this method assumes
    /// the caller has already verified that undeleting the page here
    /// will not cause conflicts.
    ///
    /// # Panics
    /// If the given previous revision is for a different page or site, this method will panic.
    pub async fn create_resurrection(
        ctx: &ServiceContext,
        CreateResurrectionPageRevision {
            site_id,
            page_id,
            user_id,
            comments,
            new_slug,
        }: CreateResurrectionPageRevision,
        previous: PageRevisionModel,
    ) -> Result<CreatePageRevisionOutput> {
        let txn = ctx.seaorm_transaction();
        let revision_number = next_revision_number(&previous, site_id, page_id);

        let PageRevisionModel {
            wikitext_hash,
            mut compiled_hash,
            hidden,
            title,
            alt_title,
            slug: old_slug,
            tags,
            ..
        } = previous;

        let changes = if old_slug == new_slug {
            vec![]
        } else {
            vec![str!("slug")]
        };

        // Get ancillary page data
        let (score, layout) = try_join!(
            ScoreService::score(ctx, page_id),
            PageService::get_layout(ctx, site_id, page_id),
        )?;

        // Re-render page
        let render_input = RenderPageInfo {
            layout,
            slug: &new_slug,
            title: &title,
            alt_title: alt_title.ref_map(|s| s.as_str()),
            score,
            tags: &tags,
        };

        let wikitext = TextService::get(ctx, &wikitext_hash).await?;
        let RenderOutput {
            // TODO: use html_output
            html_output: _,
            errors,
            compiled_hash: new_compiled_hash,
            compiled_at,
            compiled_generator,
        } = Self::render_and_update_links(ctx, site_id, page_id, wikitext, render_input)
            .await?;

        replace_hash(&mut compiled_hash, &new_compiled_hash);

        // Run outdater
        OutdateService::process_page_displace(ctx, site_id, page_id, &new_slug, 0)
            .await?;

        // Insert the resurrection revision into the table
        let model = page_revision::ActiveModel {
            revision_type: Set(PageRevisionType::Undelete),
            revision_number: Set(revision_number),
            page_id: Set(page_id),
            site_id: Set(site_id),
            user_id: Set(user_id),
            changes: Set(changes),
            wikitext_hash: Set(wikitext_hash),
            compiled_hash: Set(compiled_hash),
            compiled_at: Set(compiled_at),
            compiled_generator: Set(compiled_generator),
            comments: Set(comments),
            hidden: Set(hidden),
            title: Set(title),
            alt_title: Set(alt_title),
            slug: Set(new_slug),
            tags: Set(tags),
            ..Default::default()
        };

        let PageRevisionModel { revision_id, .. } = model.insert(txn).await?;
        Ok(CreatePageRevisionOutput {
            revision_id,
            revision_number,
            parser_errors: Some(errors),
        })
    }

    /// Helper method for performing rendering for a revision.
    ///
    /// Makes all the changes associated with rendering, such as
    /// committing the new wikitext, calling ftml, and updating
    /// backlinks.
    async fn render_and_update_links(
        ctx: &ServiceContext,
        site_id: i64,
        page_id: i64,
        wikitext: String,
        RenderPageInfo {
            layout,
            slug,
            title,
            alt_title,
            score,
            tags,
        }: RenderPageInfo<'_>,
    ) -> Result<RenderOutput> {
        // Get site
        let site = SiteService::get(ctx, Reference::from(site_id)).await?;

        // Set up parse context
        let settings = WikitextSettings::from_mode(WikitextMode::Page, layout);
        let (category_slug, page_slug) = split_category(slug);
        let page_info = PageInfo {
            page: cow!(page_slug),
            category: cow_opt!(category_slug),
            site: cow!(&site.slug),
            title: cow!(title),
            alt_title: cow_opt!(alt_title),
            score,
            tags: tags.iter().map(|s| cow!(s)).collect(),
            language: cow!(&site.locale),
        };

        // Parse and render
        let output = RenderService::render(ctx, wikitext, &page_info, &settings).await?;

        // Update backlinks
        LinkService::update(ctx, site_id, page_id, &output.html_output.backlinks).await?;

        Ok(output)
    }

    /// Re-renders a page.
    ///
    /// This fetches the latest revision for a page, and re-renders it.
    ///
    /// The `depth` parameter describes the number of layers of prior rerendering
    /// automatically leading to other updates. For a manual rerender this value
    /// should be 0.
    pub async fn rerender(
        ctx: &ServiceContext,
        site_id: i64,
        page_id: i64,
        depth: u32,
    ) -> Result<()> {
        let txn = ctx.seaorm_transaction();
        let revision = Self::get_latest(ctx, site_id, page_id).await?;
        info!(
            "Re-rendering revision: site ID {} page ID {} revision ID {} (depth {})",
            site_id, page_id, revision.revision_id, depth,
        );

        // Check that this rerender request / job is not blocked by the
        // specified anti-loop/excessive rerender rules.
        macro_rules! updated_recently {
            ($offset:expr) => {
                match ($offset, revision.updated_at) {
                    (None, _) => true, // no update offset, skip check
                    (_, None) => true, // revision has never been updated before, check is irrelevant

                    // check that at least [duration] time since [updated_at] has elapsed
                    (Some(duration), Some(updated_at)) => {
                        now() > updated_at + duration
                    }
                }
            };
        }

        for &(check_depth, update_offset) in &ctx.config().rerender_skip {
            debug!("Checking rerender-skip rule: depth {check_depth}, updated offset {update_offset:?}");
            if depth >= check_depth && updated_recently!(update_offset) {
                warn!("Skipping rerender job, too deep and updated too recently");
                return Ok(());
            }
        }

        // Get data for page
        let (wikitext, score, layout) = try_join!(
            TextService::get(ctx, &revision.wikitext_hash),
            ScoreService::score(ctx, page_id),
            PageService::get_layout(ctx, site_id, page_id),
        )?;

        // This is necessary until we are able to replace the
        // 'tags' column with TEXT[] instead of JSON.
        let render_input = RenderPageInfo {
            layout,
            slug: &revision.slug,
            title: &revision.title,
            alt_title: revision.alt_title.ref_map(|s| s.as_str()),
            score,
            tags: &revision.tags,
        };

        // TODO use html_output
        let RenderOutput {
            compiled_hash,
            compiled_generator,
            ..
        } = Self::render_and_update_links(ctx, site_id, page_id, wikitext, render_input)
            .await?;

        // Update descendents
        OutdateService::process_page_edit(ctx, site_id, page_id, &revision.slug, depth)
            .await?;

        let model = page_revision::ActiveModel {
            updated_at: Set(Some(now())),
            revision_id: Set(revision.revision_id),
            compiled_hash: Set(compiled_hash.to_vec()),
            compiled_generator: Set(compiled_generator),
            ..Default::default()
        };

        model.update(txn).await?;
        Ok(())
    }

    /// Modifies an existing revision.
    ///
    /// Normally you should think of revisions as being immutable
    /// entries in an append-only log. This however is not always
    /// true. In addition to `rerender()`, staff are able to change
    /// the `hidden` column, causing some fields of the revision to be hidden,
    /// for instance, if it contains spam, abuse, or harassment.
    pub async fn update(
        ctx: &ServiceContext,
        UpdatePageRevision {
            site_id,
            page_id,
            revision_id,
            user_id,
            hidden,
        }: UpdatePageRevision,
    ) -> Result<()> {
        let txn = ctx.seaorm_transaction();

        // Unfortunately, we cannot do .contains() on Vec<String> because
        // it wans to compare with &String, not &str.
        #[inline]
        fn contains(items: &[String], query: &str) -> bool {
            for item in items {
                if item == query {
                    return true;
                }
            }

            false
        }

        // The wikitext changes to a page are visible even if that part
        // of the revision is hidden, so current revisions are not allowed
        // to have that field hidden. It should be reverted first, and then
        // the diff can be hidden like any other.

        let latest = Self::get_latest(ctx, site_id, page_id).await?;
        if revision_id == latest.revision_id && contains(&hidden, "wikitext") {
            return Err(Error::CannotHideLatestRevision);
        }

        // TODO: record revision edit in audit log
        let _ = user_id;

        // Update the revision

        let model = page_revision::ActiveModel {
            updated_at: Set(Some(now())),
            revision_id: Set(revision_id),
            hidden: Set(hidden),
            ..Default::default()
        };

        // Update and return
        model.update(txn).await?;
        Ok(())
    }

    pub async fn get_latest(
        ctx: &ServiceContext,
        site_id: i64,
        page_id: i64,
    ) -> Result<PageRevisionModel> {
        // NOTE: There is no optional variant of this method,
        //       since all extant pages must have at least one revision.

        let txn = ctx.seaorm_transaction();
        let revision = PageRevision::find()
            .filter(
                Condition::all()
                    .add(page_revision::Column::SiteId.eq(site_id))
                    .add(page_revision::Column::PageId.eq(page_id)),
            )
            .order_by_desc(page_revision::Column::RevisionNumber)
            .one(txn)
            .await?
            .ok_or(Error::PageRevisionNotFound)?;

        Ok(revision)
    }

    pub async fn get_optional(
        ctx: &ServiceContext,
        site_id: i64,
        page_id: i64,
        revision_number: i32,
    ) -> Result<Option<PageRevisionModel>> {
        let txn = ctx.seaorm_transaction();
        let revision = PageRevision::find()
            .filter(
                Condition::all()
                    .add(page_revision::Column::SiteId.eq(site_id))
                    .add(page_revision::Column::PageId.eq(page_id))
                    .add(page_revision::Column::RevisionNumber.eq(revision_number)),
            )
            .one(txn)
            .await?;

        Ok(revision)
    }

    #[inline]
    pub async fn get(
        ctx: &ServiceContext,
        site_id: i64,
        page_id: i64,
        revision_number: i32,
    ) -> Result<PageRevisionModel> {
        find_or_error!(
            Self::get_optional(ctx, site_id, page_id, revision_number),
            PageRevision,
        )
    }

    pub async fn get_direct(
        ctx: &ServiceContext,
        revision_id: i64,
    ) -> Result<PageRevisionModel> {
        find_or_error!(Self::get_direct_optional(ctx, revision_id), PageRevision)
    }

    pub async fn get_direct_optional(
        ctx: &ServiceContext,
        revision_id: i64,
    ) -> Result<Option<PageRevisionModel>> {
        let txn = ctx.seaorm_transaction();
        let revision = PageRevision::find_by_id(revision_id).one(txn).await?;
        Ok(revision)
    }

    pub async fn count(
        ctx: &ServiceContext,
        site_id: i64,
        page_id: i64,
    ) -> Result<NonZeroI32> {
        let txn = ctx.seaorm_transaction();
        let row_count = PageRevision::find()
            .filter(
                Condition::all()
                    .add(page_revision::Column::SiteId.eq(site_id))
                    .add(page_revision::Column::PageId.eq(page_id)),
            )
            .count(txn)
            .await?;

        // We store revision_number in INT, which is i32.
        // So even though this row count is usize, it
        // should always fit inside an i32.
        let row_count = i32::try_from(row_count)
            .expect("Revision row count greater than revision_number integer size");

        // All pages have at least one revision, so if there are none
        // that means this page does not exist, and we should return an error.
        match NonZeroI32::new(row_count) {
            Some(count) => Ok(count),
            None => Err(Error::PageNotFound),
        }
    }

    pub async fn get_range(
        ctx: &ServiceContext,
        GetPageRevisionRange {
            site_id,
            page_id,
            revision_number,
            revision_direction,
            limit,
        }: GetPageRevisionRange,
    ) -> Result<Vec<PageRevisionModel>> {
        let revision_condition = {
            use page_revision::Column::RevisionNumber;

            // Allow specifying "-1" to mean "the most recent revision",
            // otherwise keep as-is.
            let revision_number = if revision_number >= 0 {
                revision_number
            } else {
                i32::MAX
            };

            // Get correct database condition based on requested ordering
            match revision_direction {
                FetchDirection::Before => RevisionNumber.lte(revision_number),
                FetchDirection::After => RevisionNumber.gte(revision_number),
            }
        };

        let txn = ctx.seaorm_transaction();
        let revisions = PageRevision::find()
            .filter(
                Condition::all()
                    .add(page_revision::Column::SiteId.eq(site_id))
                    .add(page_revision::Column::PageId.eq(page_id))
                    .add(revision_condition),
            )
            .order_by_asc(page_revision::Column::RevisionNumber)
            .limit(limit)
            .all(txn)
            .await?;

        Ok(revisions)
    }
}

#[derive(Debug, Copy, Clone)]
struct RenderPageInfo<'a> {
    layout: Layout,
    slug: &'a str,
    title: &'a str,
    alt_title: Option<&'a str>,
    score: ScoreValue,
    tags: &'a [String],
}

#[inline]
fn replace_hash(dest: &mut Vec<u8>, src: &[u8]) {
    debug_assert_eq!(
        dest.len(),
        src.len(),
        "Lengths of hash buffers are not equal",
    );

    dest.as_mut_slice().copy_from_slice(src);
}

fn next_revision_number(previous: &PageRevisionModel, site_id: i64, page_id: i64) -> i32 {
    // Check for basic consistency
    assert_eq!(
        previous.site_id, site_id,
        "Previous revision has an inconsistent site ID",
    );
    assert_eq!(
        previous.page_id, page_id,
        "Previous revision has an inconsistent page ID",
    );

    // Get the new revision number
    previous.revision_number + 1
}
