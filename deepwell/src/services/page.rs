/*
 * services/page.rs
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
use crate::services::category::CategoryService;
use crate::services::revision::{
    CreateRevision, CreateRevisionBody, CreateRevisionOutput, RevisionService,
};
use wikidot_normalize::normalize;

// Helper structs

#[derive(Deserialize, Debug)]
pub struct CreatePage {
    wikitext: String,
    title: String,
    alt_title: Option<String>,
    slug: String,
    revision_comments: String,
    user_id: i64,
}

#[derive(Serialize, Debug)]
pub struct CreatePageOutput {
    page_id: i64,
    slug: String,
    revision_id: i64,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct EditPage {
    wikitext: ProvidedValue<String>,
    title: ProvidedValue<String>,
    alt_title: ProvidedValue<Option<String>>,
    tags: ProvidedValue<Vec<String>>,
    revision_comments: String,
    user_id: i64,
}

#[derive(Serialize, Debug)]
pub struct EditPageOutput {
    revision_id: i64,
    revision_number: i32,
}

// Service

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
        let revision_input = CreateRevision {
            user_id,
            comments,
            body: CreateRevisionBody {
                wikitext: ProvidedValue::Set(wikitext),
                title: ProvidedValue::Set(title),
                alt_title: ProvidedValue::Set(alt_title),
                slug: ProvidedValue::Set(slug.clone()),
                ..Default::default()
            },
        };

        let revision_id = match RevisionService::create(
            ctx,
            site_id,
            page.page_id,
            revision_input,
            None,
        )
        .await?
        {
            None => panic!("No revision created, but page is new"),
            Some(CreateRevisionOutput {
                revision_id,
                revision_number,
            }) => {
                assert_eq!(revision_number, 0, "Created revision has a nonzero number");
                revision_id
            }
        };

        // Build and return
        Ok(CreatePageOutput {
            page_id: page.page_id,
            slug,
            revision_id,
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
            Some(&last_revision),
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
        Ok(revision_output.map(
            |CreateRevisionOutput {
                 revision_id,
                 revision_number,
             }| EditPageOutput {
                revision_id,
                revision_number,
            },
        ))
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
    ) -> Result<()> {
        let txn = ctx.transaction();
        let page = Self::get(ctx, site_id, reference).await?;
        let mut model: page::ActiveModel = page.into();

        // Set deletion flag
        model.deleted_at = Set(Some(now()));

        // Update and return
        model.update(txn).await?;
        Ok(())
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
                    let slug = slug.strip_prefix("_default:").unwrap_or(slug);

                    page::Column::Slug.eq(slug)
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
}

/// Retrieves the category portion of a normalized slug.
///
/// This finds the first `:` in the full slug and returns everything
/// up to that as the category slug.
///
/// Normal slugs do not have an explicit `_default`, so they
/// should lack a `:` entirely.
fn get_category(slug: &str) -> &str {
    match slug.find(':') {
        None => "_default",
        Some(idx) => {
            let (category, page) = slug.split_at(idx);
            category
        }
    }
}
