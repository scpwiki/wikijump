/*
 * services/revision/service.rs
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
use crate::models::page_revision::{
    self, Entity as PageRevision, Model as PageRevisionModel,
};
use crate::services::render::RenderOutput;
use crate::services::text::Hash;
use crate::services::{RenderService, SiteService, TextService};
use crate::web::split_category;
use ftml::settings::{WikitextMode, WikitextSettings};
use ftml::{data::PageInfo, render::html::HtmlOutput};
use ref_map::*;
use std::borrow::Cow;

macro_rules! cow {
    ($s:expr) => {
        Cow::Borrowed($s.as_ref())
    };
}

macro_rules! cow_opt {
    ($s:expr) => {
        $s.ref_map(|s| cow!(s))
    };
}

#[derive(Debug)]
pub struct RevisionService;

impl RevisionService {
    /// Creates a new revision.
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
    ///
    /// If `previous` is `None` but the `input` parameter lacks any fields, this method will panic.
    pub async fn create(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        input: CreateRevision,
        previous: PageRevisionModel,
    ) -> Result<Option<CreateRevisionOutput>> {
        let txn = ctx.transaction();

        // Get the new revision number and the change tasks to process
        let (tasks, revision_number) = {
            // Check for basic consistency
            assert_eq!(
                previous.site_id, site_id,
                "Previous revision has an inconsistent site ID",
            );
            assert_eq!(
                previous.page_id, page_id,
                "Previous revision has an inconsistent page ID",
            );

            // Check to see if any fields have changed
            let tasks = RevisionTasks::determine(&previous, &input.body);
            if tasks.is_empty() {
                tide::log::info!("No changes from previous revision, returning");
                return Ok(None);
            }

            // Can proceed, increment from previous
            (tasks, previous.revision_number + 1)
        };

        // Get the site we're adding the page to
        let site = SiteService::get(ctx, Reference::from(site_id)).await?;

        /*
        if tasks.render {
            let settings = WikitextSettings::from_mode(WikitextMode::Page);
            let (category, page) = split_category(&slug);
            let page_info = PageInfo {
                page: Cow::Borrowed(page),
                category: category.map(Cow::Borrowed),
                site: Cow::Borrowed(&site.unix_name.unwrap()), // TODO fix name, no nullability
                title: Cow::Borrowed(&title),
                alt_title: alt_title.map(Cow::Borrowed),
                rating: 0.0, // TODO
                tags: tags.iter().map(|s| Cow::Borrowed(&s)).collect(),
                language: Cow::Borrowed(&site.language),
            };

            let RenderOutput {
                html_output,
                warnings,
                compiled_hash,
                compiled_generator,
            } = RenderService::render(ctx, wikitext, &page_info, &settings).await?;
        }
        */

        let _todo = (ctx, revision_number, input);
        todo!();

        // TODO: consult Outdater.php

        /*
        // Finally, insert the new revision into the table
        let model = page_revision::ActiveModel {
            revision_number: Set(revision_number),
            page_id: Set(page_id),
            site_id: Set(site_id),
            user_id: Set(user_id),
            wikitext_hash: Set(wikitext_hash),
            compiled_hash: Set(compiled_hash),
            compiled_at: Set(compiled_at),
            compiled_generator: Set(compiled_generator),
            hidden: Set(hidden),
            title: Set(title),
            alt_title: Set(alt_title),
            slug: Set(slug),
            // tags: Set(tags), TODO array
            // metadata: Set(metadata), TODO json
            ..Default::default()
        };

        let PageRevisionModel { revision_id, .. } = model.insert(txn).await?;
        Ok(Some(CreateRevisionOutput {
            revision_id,
            revision_number,
        }))
        */
    }

    /// Creates the first revision for a newly-inserted page.
    ///
    /// The first revision of a page is special.
    /// A revision change cannot be missing any fields (since there is
    /// not a previous revision to take prior data from), and always
    /// inserts, since it's not possible for it to be an empty revision
    /// (since there's no prior revision for it to be equal to).
    pub async fn create_first(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        CreateFirstRevision {
            user_id,
            comments,
            body:
                CreateRevisionBodyPresent {
                    wikitext,
                    hidden,
                    title,
                    alt_title,
                    slug,
                    tags,
                    metadata,
                },
        }: CreateFirstRevision,
    ) -> Result<CreateFirstRevisionOutput> {
        let txn = ctx.transaction();

        // Get site for page
        let site = SiteService::get(ctx, Reference::from(site_id)).await?;

        // Add wikitext
        let wikitext_hash = TextService::create(ctx, wikitext.clone()).await?;

        // Render first revision
        let settings = WikitextSettings::from_mode(WikitextMode::Page);
        let (category, page) = split_category(&slug);
        let page_info = PageInfo {
            page: cow!(page),
            category: cow_opt!(category),
            site: cow!(&site.slug),
            title: cow!(&title),
            alt_title: cow_opt!(alt_title),
            rating: 0.0, // TODO
            tags: tags.iter().map(|s| cow!(s)).collect(),
            language: cow!(&site.language),
        };

        let RenderOutput {
            html_output:
                HtmlOutput {
                    body: html_body,
                    styles: html_styles,
                    meta: html_meta,
                    backlinks,
                },
            warnings,
            compiled_hash,
            compiled_generator,
        } = RenderService::render(ctx, wikitext, &page_info, &settings).await?;

        // Update backlinks
        // TODO

        // Process navigation changes, if any
        // TODO

        // Process template changes, if any
        // TODO

        // Insert the new revision into the table
        let model = page_revision::ActiveModel {
            revision_number: Set(0),
            page_id: Set(page_id),
            site_id: Set(site_id),
            user_id: Set(user_id),
            wikitext_hash: Set(wikitext_hash.to_vec()),
            compiled_hash: Set(compiled_hash.to_vec()),
            compiled_at: Set(now()),
            compiled_generator: Set(compiled_generator),
            hidden: Set(str!("{}")), // TODO array
            title: Set(title),
            alt_title: Set(alt_title),
            slug: Set(slug),
            tags: Set(str!("{}")),     // TODO array
            metadata: Set(str!("{}")), // TODO json
            ..Default::default()
        };

        let PageRevisionModel { revision_id, .. } = model.insert(txn).await?;
        Ok(CreateFirstRevisionOutput {
            revision_id,
            parser_warnings: warnings,
        })
    }

    /// Modifies an existing revision.
    ///
    /// Normally you should think of revisions as being immutable
    /// entries in an append-only log. This however is not always
    /// true, staff of a site are able to make some classes of
    /// changes to revisions, such as overriding an offensive
    /// commit message or hiding sensitive or improper data.
    pub async fn update(
        ctx: &ServiceContext<'_>,
        revision_id: i64,
        input: UpdateRevision,
    ) -> Result<()> {
        let txn = ctx.transaction();
        let mut revision = page_revision::ActiveModel {
            revision_id: Set(revision_id),
            ..Default::default()
        };

        if let ProvidedValue::Set(comments) = input.comments {
            revision.comments = Set(comments);
            revision.comments_edited_at = Set(Some(now()));
            revision.comments_edited_by = Set(Some(input.edited_by));
        }

        // TODO add hidden edited_at and edited_by
        if let ProvidedValue::Set(hidden) = input.hidden {
            // TODO fix array conversion
            revision.hidden = Set(format!("{:#?}", hidden));
        }

        // Update and return
        revision.update(txn).await?;
        Ok(())
    }

    pub async fn get_latest(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
    ) -> Result<PageRevisionModel> {
        // NOTE: There is no optional variant of this method,
        //       since all extant pages must have at least one revision.

        let txn = ctx.transaction();
        let revision = PageRevision::find()
            .filter(
                Condition::all()
                    .add(page_revision::Column::PageId.eq(page_id))
                    .add(page_revision::Column::SiteId.eq(site_id)),
            )
            .order_by_desc(page_revision::Column::RevisionNumber)
            .one(txn)
            .await?
            .ok_or(Error::NotFound)?;

        Ok(revision)
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        revision_number: i32,
    ) -> Result<Option<PageRevisionModel>> {
        let txn = ctx.transaction();
        let revision = PageRevision::find()
            .filter(
                Condition::all()
                    .add(page_revision::Column::PageId.eq(page_id))
                    .add(page_revision::Column::SiteId.eq(site_id))
                    .add(page_revision::Column::RevisionNumber.eq(revision_number)),
            )
            .one(txn)
            .await?;

        Ok(revision)
    }

    #[inline]
    pub async fn exists(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        revision_number: i32,
    ) -> Result<bool> {
        Self::get_optional(ctx, site_id, page_id, revision_number)
            .await
            .map(|revision| revision.is_some())
    }

    #[inline]
    pub async fn get(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        revision_number: i32,
    ) -> Result<PageRevisionModel> {
        match Self::get_optional(ctx, site_id, page_id, revision_number).await? {
            Some(revision) => Ok(revision),
            None => Err(Error::NotFound),
        }
    }
}
