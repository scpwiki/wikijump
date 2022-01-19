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
    _user_id: i64,
    _comments: String,

    #[serde(flatten)]
    _body: CreateRevisionBody,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct CreateRevisionBody {
    _wikitext: ProvidedValue<String>,
    _hidden: ProvidedValue<Vec<String>>,
    _title: ProvidedValue<String>,
    _alt_title: ProvidedValue<Option<String>>,
    _slug: ProvidedValue<String>,
    _tags: ProvidedValue<Vec<String>>,
    _metadata: ProvidedValue<serde_json::Value>,
}

#[derive(Serialize, Debug)]
pub struct CreateRevisionOutput {
    revision_id: i64,
    revision_number: i32,
}

// Service

#[derive(Debug)]
pub struct RevisionService;

impl RevisionService {
    pub async fn create(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        input: CreateRevision,
        previous: Option<&PageRevisionModel>,
    ) -> Result<Option<CreateRevisionOutput>> {
        let _revision_number = match previous {
            Some(revision) => {
                assert_eq!(
                    revision.site_id, site_id,
                    "Previous revision has an inconsistent site ID",
                );
                assert_eq!(
                    revision.page_id, page_id,
                    "Previous revision has an inconsistent page ID",
                );

                revision.revision_number
            }
            None => 0,
        };

        let _todo = (ctx, input);

        todo!()
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
