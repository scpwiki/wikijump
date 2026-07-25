/*
 * services/category/service.rs
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

use super::structs::UpdateCategoryBody;
use crate::error::prelude::{Error, ErrorType, OptionExt, Result, ResultExt};
use crate::license::validate_wikidot_license_override;
use crate::models::page;
use crate::models::page_category::{
    self, Entity as PageCategory, Model as PageCategoryModel,
};
use crate::services::OutdateService;
use crate::services::ServiceContext;
use crate::services::audit::{AuditEvent, AuditService, PageCategoryFields};
use crate::types::RerenderDepth;
use crate::types::{Maybe, Reference};
use crate::utils::get_category_name;
use crate::utils::now;
use paste::paste;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, Set,
};
use sea_query::{Expr, ExprTrait, Func, Query};
use std::net::IpAddr;

#[derive(Debug)]
pub struct CategoryService;

impl CategoryService {
    /// Internal method to create a category.
    ///
    /// In addition to only returning the bare ID,
    /// it also does not check for conflicts before
    /// attempting to insert.
    async fn create(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        slug: &str,
    ) -> Result<PageCategoryModel> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to create new page category '{}' in site ID {}",
                    slug, site_id,
                ),
                ErrorType::PageCategory,
            )
        };

        let txn = ctx.transaction();
        let model = page_category::ActiveModel {
            site_id: Set(site_id),
            slug: Set(str!(slug)),
            ..Default::default()
        };

        let category = model.insert(txn).await.or_raise(make_error)?;
        Ok(category)
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
    ) -> Result<Option<PageCategoryModel>> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to get page category {:?} in site ID {}",
                    reference, site_id,
                ),
                ErrorType::PageCategory,
            )
        };

        let txn = ctx.transaction();
        let condition = match reference.borrow() {
            Reference::Id(id) => page_category::Column::CategoryId.eq(id),
            Reference::Slug(slug) => page_category::Column::Slug.eq(slug),
        };

        let category = PageCategory::find()
            .filter(
                Condition::all()
                    .add(page_category::Column::SiteId.eq(site_id))
                    .add(condition),
            )
            .one(txn)
            .await
            .or_raise(make_error)?;

        Ok(category)
    }

    #[inline]
    pub async fn get(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
    ) -> Result<PageCategoryModel> {
        find_or_error!(
            Self::get_optional(ctx, site_id, reference),
            "page category",
            PageCategory,
        )
    }

    pub async fn get_or_create(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        slug: &str,
    ) -> Result<PageCategoryModel> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to get-or-create page category '{}' in site ID {}",
                    slug, site_id,
                ),
                ErrorType::PageCategory,
            )
        };

        let category = match Self::get_optional(ctx, site_id, Reference::from(slug))
            .await
            .or_raise(make_error)?
        {
            Some(category) => category,
            None => Self::create(ctx, site_id, slug)
                .await
                .or_raise(make_error)?,
        };

        Ok(category)
    }

    pub async fn get_all(
        ctx: &ServiceContext<'_>,
        site_id: i64,
    ) -> Result<Vec<PageCategoryModel>> {
        let make_error = || {
            Error::new(
                format!("failed to get all categories in site ID {}", site_id),
                ErrorType::PageCategory,
            )
        };

        let txn = ctx.transaction();
        let categories = PageCategory::find()
            .filter(page_category::Column::SiteId.eq(site_id))
            .order_by_asc(page_category::Column::Slug)
            .all(txn)
            .await
            .or_raise(make_error)?;

        Ok(categories)
    }

    pub async fn update(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        reference: Reference<'_>,
        input: UpdateCategoryBody,
        updating_user_id: i64,
        ip_address: IpAddr,
    ) -> Result<PageCategoryModel> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to update page category {reference:?} in site ID {site_id}"
                ),
                ErrorType::PageCategory,
            )
        };

        let category = Self::get(ctx, site_id, reference.clone())
            .await
            .or_raise(make_error)?;
        if let Maybe::Set(Some(template_page_id)) = &input.template_page_id {
            let template = page::Entity::find_by_id(*template_page_id)
                .filter(page::Column::SiteId.eq(site_id))
                .filter(page::Column::DeletedAt.is_null())
                .one(ctx.transaction())
                .await
                .or_raise(make_error)?
                .ok_or_raise(|| {
                    Error::new(
                        "page template must reference a live page in the same site",
                        ErrorType::PageCategory,
                    )
                })?;
            if get_category_name(&template.slug) != "template" {
                return Err(Error::new(
                    "page template must reference a page in the template category",
                    ErrorType::PageCategory,
                )
                .into());
            }
        }
        let normalized_license = match (&input.license, &input.license_other) {
            (Maybe::Set(license), Maybe::Set(license_other)) => Some(
                validate_wikidot_license_override(
                    license.as_deref(),
                    license_other.as_deref(),
                )
                .or_raise(make_error)?,
            ),
            (Maybe::Set(license), Maybe::Unset) => Some(
                validate_wikidot_license_override(license.as_deref(), None)
                    .or_raise(make_error)?,
            ),
            (Maybe::Unset, Maybe::Set(_)) => bail!(Error::new(
                "license_other cannot be updated without license",
                ErrorType::PageCategory,
            )),
            (Maybe::Unset, Maybe::Unset) => None,
        };
        let navigation_changed = input
            .top_bar_page
            .to_option()
            .is_some_and(|value| value != &category.top_bar_page)
            || input
                .side_bar_page
                .to_option()
                .is_some_and(|value| value != &category.side_bar_page);

        let previous_fields = PageCategoryFields {
            top_bar_page: match &input.top_bar_page {
                Maybe::Set(_) => Maybe::Set(category.top_bar_page.as_deref()),
                Maybe::Unset => Maybe::Unset,
            },
            side_bar_page: match &input.side_bar_page {
                Maybe::Set(_) => Maybe::Set(category.side_bar_page.as_deref()),
                Maybe::Unset => Maybe::Unset,
            },
            template_page_id: match &input.template_page_id {
                Maybe::Set(_) => Maybe::Set(category.template_page_id),
                Maybe::Unset => Maybe::Unset,
            },
            license: match &input.license {
                Maybe::Set(_) => Maybe::Set(category.license.as_deref()),
                Maybe::Unset => Maybe::Unset,
            },
            license_other: match &input.license {
                Maybe::Set(_) => Maybe::Set(category.license_other.as_deref()),
                Maybe::Unset => Maybe::Unset,
            },
            rating_enabled: match &input.rating_enabled {
                Maybe::Set(_) => Maybe::Set(category.rating_enabled),
                Maybe::Unset => Maybe::Unset,
            },
            rating_permission: match &input.rating_permission {
                Maybe::Set(_) => Maybe::Set(category.rating_permission.as_deref()),
                Maybe::Unset => Maybe::Unset,
            },
            rating_visibility: match &input.rating_visibility {
                Maybe::Set(_) => Maybe::Set(category.rating_visibility.as_deref()),
                Maybe::Unset => Maybe::Unset,
            },
            rating_type: match &input.rating_type {
                Maybe::Set(_) => Maybe::Set(category.rating_type.as_deref()),
                Maybe::Unset => Maybe::Unset,
            },
            per_page_discussion: match &input.per_page_discussion {
                Maybe::Set(_) => Maybe::Set(category.per_page_discussion),
                Maybe::Unset => Maybe::Unset,
            },
        };
        let changed_fields = PageCategoryFields {
            top_bar_page: match &input.top_bar_page {
                Maybe::Set(value) => Maybe::Set(value.as_deref()),
                Maybe::Unset => Maybe::Unset,
            },
            side_bar_page: match &input.side_bar_page {
                Maybe::Set(value) => Maybe::Set(value.as_deref()),
                Maybe::Unset => Maybe::Unset,
            },
            template_page_id: input.template_page_id.clone(),
            license: normalized_license
                .as_ref()
                .map_or(Maybe::Unset, |(license, _)| Maybe::Set(license.as_deref())),
            license_other: normalized_license
                .as_ref()
                .map_or(Maybe::Unset, |(_, license_other)| {
                    Maybe::Set(license_other.as_deref())
                }),
            rating_enabled: input.rating_enabled.clone(),
            rating_permission: match &input.rating_permission {
                Maybe::Set(value) => Maybe::Set(value.map(|value| value.as_storage())),
                Maybe::Unset => Maybe::Unset,
            },
            rating_visibility: match &input.rating_visibility {
                Maybe::Set(value) => Maybe::Set(value.map(|value| value.as_storage())),
                Maybe::Unset => Maybe::Unset,
            },
            rating_type: match &input.rating_type {
                Maybe::Set(value) => Maybe::Set(value.map(|value| value.as_storage())),
                Maybe::Unset => Maybe::Unset,
            },
            per_page_discussion: input.per_page_discussion.clone(),
        };

        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::PageCategoryUpdate {
                site_id,
                category_id: category.category_id,
                user_id: updating_user_id,
                previous_fields,
                changed_fields,
            },
        )
        .await
        .or_raise(make_error)?;

        let category_id = category.category_id;
        let mut model = category.into_active_model();
        if let Maybe::Set(top_bar_page) = input.top_bar_page {
            model.top_bar_page = Set(top_bar_page);
        }
        if let Maybe::Set(side_bar_page) = input.side_bar_page {
            model.side_bar_page = Set(side_bar_page);
        }
        if let Maybe::Set(template_page_id) = input.template_page_id {
            model.template_page_id = Set(template_page_id);
        }
        if let Some((license, license_other)) = normalized_license {
            model.license = Set(license);
            model.license_other = Set(license_other);
        }
        if let Maybe::Set(rating_enabled) = input.rating_enabled {
            model.rating_enabled = Set(rating_enabled);
        }
        if let Maybe::Set(rating_permission) = input.rating_permission {
            model.rating_permission =
                Set(rating_permission.map(|value| str!(value.as_storage())));
        }
        if let Maybe::Set(rating_visibility) = input.rating_visibility {
            model.rating_visibility =
                Set(rating_visibility.map(|value| str!(value.as_storage())));
        }
        if let Maybe::Set(rating_type) = input.rating_type {
            model.rating_type = Set(rating_type.map(|value| str!(value.as_storage())));
        }
        if let Maybe::Set(per_page_discussion) = input.per_page_discussion {
            model.per_page_discussion = Set(per_page_discussion);
        }
        model.updated_at = Set(Some(now()));

        ctx.defer_public_content_cache_invalidate_site(site_id)
            .or_raise(make_error)?;
        let category = model.update(ctx.transaction()).await.or_raise(make_error)?;

        if navigation_changed {
            OutdateService::outdate_nav_category(
                ctx,
                site_id,
                category_id,
                RerenderDepth::default(),
            )
            .await
            .or_raise(make_error)?;
        }

        Ok(category)
    }

    /// Gets all page categories which have non-deleted pages in them.
    pub async fn get_all_active(
        ctx: &ServiceContext<'_>,
        site_id: i64,
    ) -> Result<Vec<PageCategoryModel>> {
        let make_error = || {
            Error::new(
                format!("failed to get all active categories in site ID {}", site_id),
                ErrorType::PageCategory,
            )
        };

        // Raw SQL query
        //
        // SELECT * FROM page_category
        // WHERE category_id IN (
        //     SELECT page_category_id
        //     FROM page
        //     WHERE site_id = ?
        //     GROUP BY page_category_id, deleted_at
        //     HAVING coalesce(deleted_at) IS NULL
        // );

        let txn = ctx.transaction();
        let categories = PageCategory::find()
            .filter(
                page_category::Column::CategoryId.in_subquery(
                    Query::select()
                        .column(page::Column::PageCategoryId)
                        .from(page::Entity)
                        .and_where(Expr::column(page::Column::SiteId).eq(site_id))
                        .group_by_columns([
                            page::Column::PageCategoryId,
                            page::Column::DeletedAt,
                        ])
                        .and_having(
                            Func::coalesce([Expr::column(page::Column::DeletedAt)])
                                .is_null(),
                        )
                        .to_owned(),
                ),
            )
            .all(txn)
            .await
            .or_raise(make_error)?;

        Ok(categories)
    }
}
