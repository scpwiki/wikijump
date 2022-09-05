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

use wikidot_normalize::normalize;

use super::prelude::*;
use crate::models::site::{self, Entity as Site, Model as SiteModel};

#[derive(Debug)]
pub struct SiteService;

impl SiteService {
    pub async fn create(
        ctx: &ServiceContext<'_>,
        CreateSite {
            mut slug,
            name,
            subtitle,
            description,
            locale,
        }: CreateSite,
    ) -> Result<CreateSiteOutput> {
        let txn = ctx.transaction();

        normalize(&mut slug);
        Self::check_conflicts(ctx, &slug, "create").await?;

        let model = site::ActiveModel {
            slug: Set(slug.clone()),
            name: Set(name),
            subtitle: Set(subtitle),
            description: Set(description),
            locale: Set(locale),         
            ..Default::default()
        };
        let site = model.insert(txn).await?;

        Ok(CreateSiteOutput {
            site_id: site.site_id,
            slug
        })
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
                            .add(site::Column::DeletedAt.is_null()),
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
        Self::get_optional(ctx, reference)
            .await?
            .ok_or(Error::NotFound)
    }

    /// Checks to see if a site already exists at the slug specified.
    ///
    /// If so, this method fails with `Error::Conflict`. Otherwise it returns nothing.
    async fn check_conflicts(
        ctx: &ServiceContext<'_>,
        slug: &str,
        action: &str,
    ) -> Result<()> {
        let txn = ctx.transaction();

        let result = Site::find()
            .filter(
                Condition::all()
                    .add(site::Column::Slug.eq(slug))
                    .add(site::Column::DeletedAt.is_null()),
            )
            .one(txn)
            .await?;

        match result {
            None => Ok(()),
            Some(_) => {
                tide::log::error!(
                    "Site with slug '{}' already exists, cannot {}",
                    slug,
                    action,
                );

                Err(Error::Conflict)
            }
        }
    }
}
