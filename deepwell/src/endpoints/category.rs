/*
 * endpoints/category.rs
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

use super::prelude::*;
use crate::models::page_category::Model as PageCategoryModel;
use crate::services::category::{GetCategory, UpdateCategory};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::site::GetSite;
use crate::types::{Action, Permission, Reference, Resource};

pub async fn category_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<PageCategoryModel>> {
    let GetCategory { site, category } = parse!(params, PageCategory);
    let make_error =
        || Error::new("failed to get page category", ErrorType::PageCategory);

    let site_id = SiteService::get_id(ctx, site).await.or_raise(make_error)?;
    info!("Getting page category {category:?} in site ID {site_id}");
    CategoryService::get_optional(ctx, site_id, category)
        .await
        .or_raise(make_error)
}

pub async fn category_get_all(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<PageCategoryModel>> {
    let GetSite { site } = parse!(params, PageCategory);

    let make_error = || {
        Error::new(
            "failed to get all page categories for a site",
            ErrorType::PageCategory,
        )
    };

    let site_id = SiteService::get_id(ctx, site).await.or_raise(make_error)?;
    info!("Getting all page categories in site ID {site_id}");
    CategoryService::get_all(ctx, site_id)
        .await
        .or_raise(make_error)
}

pub async fn category_get_all_active(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<PageCategoryModel>> {
    let GetSite { site } = parse!(params, PageCategory);

    let make_error = || {
        Error::new(
            "failed to get all active page categories for a site",
            ErrorType::PageCategory,
        )
    };

    let site_id = SiteService::get_id(ctx, site).await.or_raise(make_error)?;
    info!("Getting all active page categories in site ID {site_id}");
    CategoryService::get_all_active(ctx, site_id)
        .await
        .or_raise(make_error)
}

pub async fn category_update(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<PageCategoryModel> {
    let UpdateCategory {
        site,
        category,
        user_id: _submitted_user_id,
        body,
        ip_address,
    } = parse!(params, PageCategory);

    let actor_user_id = ctx.request().user_id().or_raise(|| {
        Error::new(
            "user does not have permission to edit this site's categories",
            ErrorType::PermissionDenied,
        )
    })?;
    let site_id = match site {
        Reference::Id(site_id) => site_id,
        Reference::Slug(_) => {
            return Err(Error::new(
                "category_update requires a numeric site ID",
                ErrorType::BadRequest,
            )
            .into());
        }
    };

    let can_edit = PermissionService::check_user_can(
        ctx,
        &CheckPermissionContext {
            user_id: Some(actor_user_id),
            site_id,
            page_reference: None,
        },
        Permission {
            resource_type: Resource::Site,
            resource_category: None,
            action: Action::Edit,
        },
    )
    .await
    .or_raise(|| {
        Error::new(
            "failed to check site edit permission for category update",
            ErrorType::PageCategory,
        )
    })?;

    if !can_edit {
        return Err(Error::new(
            "user does not have permission to edit this site's categories",
            ErrorType::PermissionDenied,
        )
        .into());
    }

    CategoryService::update(ctx, site_id, category, body, actor_user_id, ip_address)
        .await
        .or_raise(|| {
            Error::new("failed to update page category", ErrorType::PageCategory)
        })
}
