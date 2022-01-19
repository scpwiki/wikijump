/*
 * services/revision.rs
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

// Helper structs

#[derive(Deserialize, Debug)]
pub struct CreateRevision {
    pub user_id: i64,
    pub comments: String,

    #[serde(flatten)]
    pub body: CreateRevisionBody,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct CreateRevisionBody {
    pub wikitext: ProvidedValue<String>,
    pub hidden: ProvidedValue<Vec<String>>,
    pub title: ProvidedValue<String>,
    pub alt_title: ProvidedValue<Option<String>>,
    pub slug: ProvidedValue<String>,
    pub tags: ProvidedValue<Vec<String>>,
    pub metadata: ProvidedValue<serde_json::Value>,
}

impl CreateRevisionBody {
    #[inline]
    pub fn any_set(&self) -> bool {
        self.wikitext.is_set()
            || self.hidden.is_set()
            || self.title.is_set()
            || self.alt_title.is_set()
            || self.slug.is_set()
            || self.tags.is_set()
            || self.metadata.is_set()
    }
}

#[derive(Serialize, Debug)]
pub struct CreateRevisionOutput {
    pub revision_id: i64,
    pub revision_number: i32,
}

#[derive(Deserialize, Debug)]
pub struct UpdateRevision {
    pub comments: ProvidedValue<String>,
    pub hidden: ProvidedValue<Vec<String>>,
    pub edited_by: i64,
}

// Service

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
    pub async fn create(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        input: CreateRevision,
        previous: Option<&PageRevisionModel>,
    ) -> Result<Option<CreateRevisionOutput>> {
        // Get the number for the new revision
        let revision_number = match previous {
            None => 0,
            Some(revision) => {
                // Check for basic consistency
                assert_eq!(
                    revision.site_id, site_id,
                    "Previous revision has an inconsistent site ID",
                );
                assert_eq!(
                    revision.page_id, page_id,
                    "Previous revision has an inconsistent page ID",
                );

                // Check to see if any fields have changed
                if !input.body.any_set() {
                    tide::log::info!("No changes from previous revision, returning");
                    return Ok(None);
                }

                // Can proceed, increment from previous
                revision.revision_number + 1
            }
        };

        let _todo = (ctx, revision_number, input);

        // TODO: consult Outdater.php

        todo!()
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
