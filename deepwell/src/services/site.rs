/*
 * services/site.rs
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
use crate::models::site::{self, Entity as Site, Model as SiteModel};

// Helper structs
// TODO

// Service

#[derive(Debug)]
pub struct SiteService;

impl SiteService {
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
                            // TODO: rename after migration
                            .add(site::Column::UnixName.eq(slug))
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
