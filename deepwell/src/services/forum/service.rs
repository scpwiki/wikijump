/*
 * services/forum/service.rs
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

#![allow(dead_code)] // TEMP

use super::structs::{
    CreateForumCategory, CreateForumGroup, DeleteForumCategory, DeleteForumGroup,
    ForumGroupStructure, GetForumCategories, GetForumCategory, GetForumGroup,
    GetForumGroups, GetForumStructure, UpdateForumCategory, UpdateForumCategoryBody,
    UpdateForumGroup, UpdateForumGroupBody,
};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::forum_category::{
    self, Entity as ForumCategory, Model as ForumCategoryModel,
};
use crate::models::forum_group::{self, Entity as ForumGroup, Model as ForumGroupModel};
use crate::models::{forum_post, forum_post_revision, forum_thread};
use crate::services::ServiceContext;
use crate::types::Maybe;
use crate::utils::now;
use paste::paste;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, Set,
};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct ForumService;

impl ForumService {
    pub async fn create_group(
        ctx: &ServiceContext<'_>,
        CreateForumGroup {
            site_id,
            user_id,
            name,
            description,
            visible,
            sort_index,
            from_wikidot,
        }: CreateForumGroup,
    ) -> Result<ForumGroupModel> {
        let sort_index = match sort_index {
            Some(sort_index) => {
                Self::check_sort_index(sort_index, "group sort index")?;
                sort_index
            }
            None => Self::next_group_sort_index(ctx, site_id).await?,
        };

        let make_error = || {
            Error::new(
                format!(
                    "failed to create forum group in site ID {} by user ID {}",
                    site_id, user_id,
                ),
                ErrorType::ForumGroup,
            )
        };

        let txn = ctx.transaction();
        let model = forum_group::ActiveModel {
            site_id: Set(site_id),
            created_by: Set(user_id),
            name: Set(name),
            description: Set(description),
            visible: Set(visible),
            sort_index: Set(sort_index),
            from_wikidot: Set(from_wikidot),
            ..Default::default()
        };

        let group = model.insert(txn).await.or_raise(make_error)?;
        Ok(group)
    }

    pub async fn update_group(
        ctx: &ServiceContext<'_>,
        UpdateForumGroup {
            forum_group_id,
            user_id,
            body:
                UpdateForumGroupBody {
                    name,
                    description,
                    visible,
                    sort_index,
                },
        }: UpdateForumGroup,
    ) -> Result<ForumGroupModel> {
        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!(
                    "failed to update forum group ID {} by user ID {}",
                    forum_group_id, user_id,
                ),
                ErrorType::ForumGroup,
            )
        };

        let group = Self::get_group_direct(ctx, forum_group_id, false)
            .await
            .or_raise(make_error)?;

        let mut model = forum_group::ActiveModel {
            forum_group_id: Set(group.forum_group_id),
            updated_by: Set(Some(user_id)),
            updated_at: Set(Some(now())),
            ..Default::default()
        };

        if let Maybe::Set(name) = name {
            model.name = Set(name);
        }

        if let Maybe::Set(description) = description {
            model.description = Set(description);
        }

        if let Maybe::Set(visible) = visible {
            model.visible = Set(visible);
        }

        if let Maybe::Set(sort_index) = sort_index {
            Self::check_sort_index(sort_index, "group sort index")?;
            model.sort_index = Set(sort_index);
        }

        let group = model.update(txn).await.or_raise(make_error)?;
        Ok(group)
    }

    pub async fn delete_group(
        ctx: &ServiceContext<'_>,
        DeleteForumGroup {
            forum_group_id,
            user_id,
        }: DeleteForumGroup,
    ) -> Result<ForumGroupModel> {
        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!(
                    "failed to delete forum group ID {} by user ID {}",
                    forum_group_id, user_id,
                ),
                ErrorType::ForumGroup,
            )
        };

        let group = Self::get_group_direct(ctx, forum_group_id, false)
            .await
            .or_raise(make_error)?;

        let model = forum_group::ActiveModel {
            forum_group_id: Set(group.forum_group_id),
            deleted_by: Set(Some(user_id)),
            deleted_at: Set(Some(now())),
            ..Default::default()
        };

        let group = model.update(txn).await.or_raise(make_error)?;
        Ok(group)
    }

    pub async fn get_group_optional(
        ctx: &ServiceContext<'_>,
        GetForumGroup {
            site_id,
            forum_group_id,
            include_deleted,
        }: GetForumGroup,
    ) -> Result<Option<ForumGroupModel>> {
        let txn = ctx.transaction();
        let group = ForumGroup::find()
            .filter(
                Condition::all()
                    .add(forum_group::Column::SiteId.eq(site_id))
                    .add(forum_group::Column::ForumGroupId.eq(forum_group_id))
                    .add_option(Self::deleted_condition(
                        include_deleted,
                        forum_group::Column::DeletedAt,
                    )),
            )
            .one(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to get forum group ID {} in site ID {}",
                        forum_group_id, site_id,
                    ),
                    ErrorType::ForumGroup,
                )
            })?;

        Ok(group)
    }

    pub async fn get_group(
        ctx: &ServiceContext<'_>,
        key: GetForumGroup,
    ) -> Result<ForumGroupModel> {
        find_or_error!(
            Self::get_group_optional(ctx, key),
            "forum group",
            ForumGroup
        )
    }

    pub async fn get_group_direct_optional(
        ctx: &ServiceContext<'_>,
        forum_group_id: i64,
        include_deleted: bool,
    ) -> Result<Option<ForumGroupModel>> {
        let txn = ctx.transaction();
        let group = ForumGroup::find()
            .filter(
                Condition::all()
                    .add(forum_group::Column::ForumGroupId.eq(forum_group_id))
                    .add_option(Self::deleted_condition(
                        include_deleted,
                        forum_group::Column::DeletedAt,
                    )),
            )
            .one(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!("failed to get forum group ID {} directly", forum_group_id),
                    ErrorType::ForumGroup,
                )
            })?;

        Ok(group)
    }

    pub async fn get_group_direct(
        ctx: &ServiceContext<'_>,
        forum_group_id: i64,
        include_deleted: bool,
    ) -> Result<ForumGroupModel> {
        find_or_error!(
            Self::get_group_direct_optional(ctx, forum_group_id, include_deleted),
            "forum group",
            ForumGroup,
        )
    }

    pub async fn list_groups(
        ctx: &ServiceContext<'_>,
        GetForumGroups {
            site_id,
            include_deleted,
        }: GetForumGroups,
    ) -> Result<Vec<ForumGroupModel>> {
        let txn = ctx.transaction();
        let groups = ForumGroup::find()
            .filter(
                Condition::all()
                    .add(forum_group::Column::SiteId.eq(site_id))
                    .add_option(Self::deleted_condition(
                        include_deleted,
                        forum_group::Column::DeletedAt,
                    )),
            )
            .order_by_asc(forum_group::Column::SortIndex)
            .all(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!("failed to list forum groups in site ID {}", site_id),
                    ErrorType::ForumGroup,
                )
            })?;

        Ok(groups)
    }

    pub async fn create_category(
        ctx: &ServiceContext<'_>,
        CreateForumCategory {
            forum_group_id,
            user_id,
            name,
            description,
            sort_index,
            max_nest_level,
            per_page_discussion,
            layout,
            from_wikidot,
        }: CreateForumCategory,
    ) -> Result<ForumCategoryModel> {
        let group = Self::get_group_direct(ctx, forum_group_id, false).await?;

        let sort_index = match sort_index {
            Some(sort_index) => {
                Self::check_sort_index(sort_index, "category sort index")?;
                sort_index
            }
            None => Self::next_category_sort_index(ctx, forum_group_id).await?,
        };

        if let Some(max_nest_level) = max_nest_level {
            Self::check_max_nest_level(max_nest_level)?;
        }

        let make_error = || {
            Error::new(
                format!(
                    "failed to create forum category in group ID {} by user ID {}",
                    forum_group_id, user_id,
                ),
                ErrorType::ForumCategory,
            )
        };

        let txn = ctx.transaction();
        let model = forum_category::ActiveModel {
            forum_group_id: Set(group.forum_group_id),
            site_id: Set(group.site_id),
            created_by: Set(user_id),
            name: Set(name),
            description: Set(description),
            sort_index: Set(sort_index),
            max_nest_level: Set(max_nest_level),
            per_page_discussion: Set(per_page_discussion),
            layout: Set(layout),
            from_wikidot: Set(from_wikidot),
            ..Default::default()
        };

        let category = model.insert(txn).await.or_raise(make_error)?;
        Ok(category)
    }

    pub async fn update_category(
        ctx: &ServiceContext<'_>,
        UpdateForumCategory {
            forum_category_id,
            user_id,
            body:
                UpdateForumCategoryBody {
                    forum_group_id,
                    name,
                    description,
                    sort_index,
                    max_nest_level,
                    per_page_discussion,
                    layout,
                },
        }: UpdateForumCategory,
    ) -> Result<ForumCategoryModel> {
        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!(
                    "failed to update forum category ID {} by user ID {}",
                    forum_category_id, user_id,
                ),
                ErrorType::ForumCategory,
            )
        };

        let category = Self::get_category_direct(ctx, forum_category_id, false)
            .await
            .or_raise(make_error)?;

        let mut new_group_id = category.forum_group_id;
        let mut new_site_id = category.site_id;

        if let Maybe::Set(forum_group_id) = forum_group_id {
            let group = Self::get_group_direct(ctx, forum_group_id, false)
                .await
                .or_raise(make_error)?;

            // Categories are site-local. We keep this strict at the service layer.
            if group.site_id != category.site_id {
                bail!(Error::new(
                    format!(
                        "cannot move forum category ID {} to group ID {} in a different site",
                        forum_category_id, forum_group_id,
                    ),
                    ErrorType::BadRequest,
                ));
            }

            new_group_id = group.forum_group_id;
            new_site_id = group.site_id;
        }

        if let Maybe::Set(sort_index) = sort_index {
            Self::check_sort_index(sort_index, "category sort index")?;
        }

        if let Maybe::Set(Some(max_nest_level)) = max_nest_level {
            Self::check_max_nest_level(max_nest_level)?;
        }

        let moved_group =
            category.forum_group_id != new_group_id || category.site_id != new_site_id;

        let mut model = forum_category::ActiveModel {
            forum_category_id: Set(category.forum_category_id),
            forum_group_id: Set(new_group_id),
            site_id: Set(new_site_id),
            updated_by: Set(Some(user_id)),
            updated_at: Set(Some(now())),
            ..Default::default()
        };

        if let Maybe::Set(name) = name {
            model.name = Set(name);
        }

        if let Maybe::Set(description) = description {
            model.description = Set(description);
        }

        if let Maybe::Set(sort_index) = sort_index {
            model.sort_index = Set(sort_index);
        }

        if let Maybe::Set(max_nest_level) = max_nest_level {
            model.max_nest_level = Set(max_nest_level);
        }

        if let Maybe::Set(per_page_discussion) = per_page_discussion {
            model.per_page_discussion = Set(per_page_discussion);
        }

        if let Maybe::Set(layout) = layout {
            model.layout = Set(layout);
        }

        let category = model.update(txn).await.or_raise(make_error)?;

        if moved_group {
            // Keep denormalized group/site fields in sync for all descendants.
            let thread_model = forum_thread::ActiveModel {
                forum_group_id: Set(new_group_id),
                site_id: Set(new_site_id),
                ..Default::default()
            };

            forum_thread::Entity::update_many()
                .set(thread_model)
                .filter(
                    forum_thread::Column::ForumCategoryId.eq(category.forum_category_id),
                )
                .exec(txn)
                .await
                .or_raise(make_error)?;

            let post_model = forum_post::ActiveModel {
                forum_group_id: Set(new_group_id),
                site_id: Set(new_site_id),
                ..Default::default()
            };

            forum_post::Entity::update_many()
                .set(post_model)
                .filter(
                    forum_post::Column::ForumCategoryId.eq(category.forum_category_id),
                )
                .exec(txn)
                .await
                .or_raise(make_error)?;

            let revision_model = forum_post_revision::ActiveModel {
                forum_group_id: Set(new_group_id),
                site_id: Set(new_site_id),
                ..Default::default()
            };

            forum_post_revision::Entity::update_many()
                .set(revision_model)
                .filter(
                    forum_post_revision::Column::ForumCategoryId
                        .eq(category.forum_category_id),
                )
                .exec(txn)
                .await
                .or_raise(make_error)?;
        }

        Ok(category)
    }

    pub async fn delete_category(
        ctx: &ServiceContext<'_>,
        DeleteForumCategory {
            forum_category_id,
            user_id,
        }: DeleteForumCategory,
    ) -> Result<ForumCategoryModel> {
        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!(
                    "failed to delete forum category ID {} by user ID {}",
                    forum_category_id, user_id,
                ),
                ErrorType::ForumCategory,
            )
        };

        let category = Self::get_category_direct(ctx, forum_category_id, false)
            .await
            .or_raise(make_error)?;

        let model = forum_category::ActiveModel {
            forum_category_id: Set(category.forum_category_id),
            deleted_by: Set(Some(user_id)),
            deleted_at: Set(Some(now())),
            ..Default::default()
        };

        let category = model.update(txn).await.or_raise(make_error)?;
        Ok(category)
    }

    pub async fn get_category_optional(
        ctx: &ServiceContext<'_>,
        GetForumCategory {
            site_id,
            forum_category_id,
            include_deleted,
        }: GetForumCategory,
    ) -> Result<Option<ForumCategoryModel>> {
        let txn = ctx.transaction();
        let category = ForumCategory::find()
            .filter(
                Condition::all()
                    .add(forum_category::Column::SiteId.eq(site_id))
                    .add(forum_category::Column::ForumCategoryId.eq(forum_category_id))
                    .add_option(Self::deleted_condition(
                        include_deleted,
                        forum_category::Column::DeletedAt,
                    )),
            )
            .one(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to get forum category ID {} in site ID {}",
                        forum_category_id, site_id,
                    ),
                    ErrorType::ForumCategory,
                )
            })?;

        Ok(category)
    }

    pub async fn get_category(
        ctx: &ServiceContext<'_>,
        key: GetForumCategory,
    ) -> Result<ForumCategoryModel> {
        find_or_error!(
            Self::get_category_optional(ctx, key),
            "forum category",
            ForumCategory,
        )
    }

    pub async fn get_category_direct_optional(
        ctx: &ServiceContext<'_>,
        forum_category_id: i64,
        include_deleted: bool,
    ) -> Result<Option<ForumCategoryModel>> {
        let txn = ctx.transaction();
        let category = ForumCategory::find()
            .filter(
                Condition::all()
                    .add(forum_category::Column::ForumCategoryId.eq(forum_category_id))
                    .add_option(Self::deleted_condition(
                        include_deleted,
                        forum_category::Column::DeletedAt,
                    )),
            )
            .one(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to get forum category ID {} directly",
                        forum_category_id,
                    ),
                    ErrorType::ForumCategory,
                )
            })?;

        Ok(category)
    }

    pub async fn get_category_direct(
        ctx: &ServiceContext<'_>,
        forum_category_id: i64,
        include_deleted: bool,
    ) -> Result<ForumCategoryModel> {
        find_or_error!(
            Self::get_category_direct_optional(ctx, forum_category_id, include_deleted),
            "forum category",
            ForumCategory,
        )
    }

    pub async fn list_categories(
        ctx: &ServiceContext<'_>,
        GetForumCategories {
            site_id,
            forum_group_id,
            include_deleted,
        }: GetForumCategories,
    ) -> Result<Vec<ForumCategoryModel>> {
        let txn = ctx.transaction();
        let categories = ForumCategory::find()
            .filter(
                Condition::all()
                    .add(forum_category::Column::SiteId.eq(site_id))
                    .add_option(
                        forum_group_id
                            .map(|id| forum_category::Column::ForumGroupId.eq(id)),
                    )
                    .add_option(Self::deleted_condition(
                        include_deleted,
                        forum_category::Column::DeletedAt,
                    )),
            )
            .order_by_asc(forum_category::Column::SortIndex)
            .all(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!("failed to list forum categories in site ID {}", site_id),
                    ErrorType::ForumCategory,
                )
            })?;

        Ok(categories)
    }

    pub async fn get_structure(
        ctx: &ServiceContext<'_>,
        GetForumStructure {
            site_id,
            include_deleted,
        }: GetForumStructure,
    ) -> Result<Vec<ForumGroupStructure>> {
        let make_error = || {
            Error::new(
                format!("failed to get forum structure in site ID {}", site_id),
                ErrorType::Forum,
            )
        };

        let (groups_result, categories_result) = join!(
            Self::list_groups(
                ctx,
                GetForumGroups {
                    site_id,
                    include_deleted,
                },
            ),
            Self::list_categories(
                ctx,
                GetForumCategories {
                    site_id,
                    forum_group_id: None,
                    include_deleted,
                },
            ),
        );

        let (groups, categories) =
            raise_multiple!(groups_result, categories_result; make_error);
        let mut grouped_categories: BTreeMap<i64, Vec<ForumCategoryModel>> =
            BTreeMap::new();

        for category in categories {
            grouped_categories
                .entry(category.forum_group_id)
                .or_default()
                .push(category);
        }

        let mut structure = Vec::new();
        for group in groups {
            let categories = grouped_categories
                .remove(&group.forum_group_id)
                .unwrap_or_default();

            structure.push(ForumGroupStructure { group, categories });
        }

        Ok(structure)
    }

    async fn next_group_sort_index(
        ctx: &ServiceContext<'_>,
        site_id: i64,
    ) -> Result<i32> {
        let txn = ctx.transaction();
        let group = ForumGroup::find()
            .filter(
                Condition::all()
                    .add(forum_group::Column::SiteId.eq(site_id))
                    .add(forum_group::Column::DeletedAt.is_null()),
            )
            .order_by_desc(forum_group::Column::SortIndex)
            .one(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to determine next forum group sort index in site ID {}",
                        site_id,
                    ),
                    ErrorType::ForumGroup,
                )
            })?;

        Ok(group.map(|group| group.sort_index + 1).unwrap_or(0))
    }

    async fn next_category_sort_index(
        ctx: &ServiceContext<'_>,
        forum_group_id: i64,
    ) -> Result<i32> {
        let txn = ctx.transaction();
        let category = ForumCategory::find()
            .filter(
                Condition::all()
                    .add(forum_category::Column::ForumGroupId.eq(forum_group_id))
                    .add(forum_category::Column::DeletedAt.is_null()),
            )
            .order_by_desc(forum_category::Column::SortIndex)
            .one(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to determine next forum category sort index in group ID {}",
                        forum_group_id,
                    ),
                    ErrorType::ForumCategory,
                )
            })?;

        Ok(category
            .map(|category| category.sort_index + 1)
            .unwrap_or(0))
    }

    fn deleted_condition(
        include_deleted: bool,
        column: impl ColumnTrait,
    ) -> Option<sea_orm::sea_query::SimpleExpr> {
        if include_deleted {
            None
        } else {
            Some(column.is_null())
        }
    }

    fn check_sort_index(sort_index: i32, name: &str) -> Result<()> {
        if sort_index < 0 {
            bail!(Error::new(
                format!("{name} cannot be negative, got {sort_index}"),
                ErrorType::BadRequest,
            ));
        }

        Ok(())
    }

    fn check_max_nest_level(max_nest_level: i16) -> Result<()> {
        if !(0..=10).contains(&max_nest_level) {
            bail!(Error::new(
                format!(
                    "forum max_nest_level must be between 0 and 10, got {}",
                    max_nest_level,
                ),
                ErrorType::BadRequest,
            ));
        }

        Ok(())
    }
}
