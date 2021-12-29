/*
 * services/page/mod.rs
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

mod links;

use super::prelude::*;
use crate::models::page::{self, Entity as Page, Model as PageModel};
use wikidot_normalize::normalize;

// Helper structs

#[derive(Deserialize, Debug)]
pub struct CreatePage {
    site_id: i64,
    category_id: i64,
    slug: String,
    vote_type: (), // TODO
    revision_comments: String,
}

#[derive(Serialize, Debug)]
pub struct CreatePageOutput {
    page_id: i64,
    slug: String,
}

// Service

#[derive(Debug)]
pub struct PageService;

impl PageService {
    pub async fn create(
        ctx: &ServiceContext<'_>,
        mut input: CreatePage,
    ) -> Result<CreatePageOutput> {
        let txn = ctx.transaction();
        normalize(&mut input.slug);

        // Check for conflicts
        // TODO

        let _todo = (txn, input);

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

                    page::Column::UnixName.eq(slug) // TODO rename to Slug
                }
            };

            Page::find()
                .filter(
                    Condition::all()
                        .add(condition)
                        .add(page::Column::SiteId.eq(site_id)),
                    // TODO: re-add .add(page::Column::DeletedAt.is_null()),
                )
                .one(txn)
                .await?
        };

        Ok(page)
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

    pub async fn delete(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
    ) -> Result<PageModel> {
        let txn = ctx.transaction();
        let model = Self::get(ctx, site_id, reference).await?;
        let page: page::ActiveModel = model.clone().into();

        /*
        TODO: soft deletion

        // Set deletion flag
        page.deleted_at = Set(Some(now()));

        // Update and return
        page.update(txn).await?;
        */

        page.delete(txn).await?;
        Ok(model)
    }
}
