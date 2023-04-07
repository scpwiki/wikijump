/*
 * utils/id.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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

//! Extract the ID from a reference.

use crate::services::{PageService, Result, ServiceContext, SiteService, UserService};
use crate::web::Reference;

pub async fn get_site_id(
    ctx: &ServiceContext<'_>,
    reference: Reference<'_>,
) -> Result<i64> {
    match reference {
        Reference::Id(id) => Ok(id),
        Reference::Slug(slug) => {
            let site = SiteService::get(ctx, reference).await?;
            Ok(site.site_id)
        }
    }
}

pub async fn get_user_id(
    ctx: &ServiceContext<'_>,
    reference: Reference<'_>,
) -> Result<i64> {
    match reference {
        Reference::Id(id) => Ok(id),
        Reference::Slug(slug) => {
            let user = UserService::get(ctx, reference).await?;
            Ok(user.user_id)
        }
    }
}

pub async fn get_page_id(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    reference: Reference<'_>,
) -> Result<i64> {
    match reference {
        Reference::Id(id) => Ok(id),
        Reference::Slug(slug) => {
            let page = PageService::get(ctx, site_id, reference).await?;
            Ok(page.page_id)
        }
    }
}
