/*
 * permission/category.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
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

use crate::error::Result;
use crate::services::{CategoryService, ServiceContext};
use crate::types::{Reference, Resource};
use std::borrow::Cow;

async fn resolve_page_category_reference(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    reference: &Reference<'_>,
) -> Result<Option<i64>> {
    match reference {
        Reference::Id(id) => Ok(Some(*id)),
        Reference::Slug(slug) => {
            let category =
                CategoryService::get_optional(ctx, site_id, Reference::Slug(cow!(&slug)))
                    .await?;
            Ok(category.map(|category| category.category_id))
        }
    }
}

async fn resolve_page_category_slug<'slug>(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    reference: &Reference<'slug>,
) -> Result<Option<Cow<'slug, str>>> {
    match reference {
        Reference::Id(id) => {
            let category =
                CategoryService::get_optional(ctx, site_id, Reference::Id(*id)).await?;
            Ok(category.map(|category| Cow::Owned(category.slug)))
        }
        Reference::Slug(slug) => Ok(Some(slug.clone())),
    }
}

/// Helper function to resolve a category reference to an ID based on resource type.
pub async fn resolve_category_reference(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    resource_type: Resource,
    reference: &Reference<'_>,
) -> Result<Option<i64>> {
    match resource_type {
        Resource::Page => resolve_page_category_reference(ctx, site_id, reference).await,
        _ => Ok(None),
    }
}

pub async fn resolve_category_slug<'slug>(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    resource_type: Resource,
    reference: &Reference<'slug>,
) -> Result<Option<Cow<'slug, str>>> {
    match resource_type {
        Resource::Page => resolve_page_category_slug(ctx, site_id, reference).await,
        _ => Ok(None),
    }
}
