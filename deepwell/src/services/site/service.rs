/*
 * services/site/service.rs
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
use crate::models::site::{self, Entity as Site, Model as SiteModel};

#[derive(Debug)]
pub struct SiteService;

impl SiteService {
    // TODO Temporary method to create a new site, for testing.
    pub async fn create_temp(
        ctx: &ServiceContext<'_>,
        name: String,
        slug: String,
        description: String,
        language: String,
    ) -> Result<i64> {
        let txn = ctx.transaction();
        let model = site::ActiveModel {
            name: Set(Some(name)),
            slug: Set(slug),
            description: Set(Some(description)),
            language: Set(language),
            date_created: Set(Some(now_naive())),
            visible: Set(true),
            default_page: Set(str!("start")),
            private: Set(false),
            deleted: Set(false),
            ..Default::default()
        };

        let site = model.insert(txn).await?;
        Ok(site.site_id)
    }

    #[inline]
    pub async fn exists(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
    ) -> Result<bool> {
        Self::get_optional(ctx, reference)
            .await
            .map(|user| user.is_some())
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
    ) -> Result<Option<SiteModel>> {
        let txn = ctx.transaction();
        let site = match reference {
            Reference::Id(id) => Site::find_by_id(id).one(txn).await?,
            Reference::Slug(slug) => {
                Site::find()
                    .filter(
                        Condition::all()
                            .add(site::Column::Slug.eq(slug))
                            .add(site::Column::Deleted.eq(false)),
                    )
                    .one(txn)
                    .await?
            }
        };

        Ok(site)
    }

    pub async fn get(
        ctx: &ServiceContext<'_>,
        reference: Reference<'_>,
    ) -> Result<SiteModel> {
        match Self::get_optional(ctx, reference).await? {
            Some(site) => Ok(site),
            None => Err(Error::NotFound),
        }
    }
}
