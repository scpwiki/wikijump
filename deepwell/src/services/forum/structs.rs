/*
 * services/forum/structs.rs
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

use crate::{
    models::{
        forum_category::Model as ForumCategoryModel,
        forum_group::Model as ForumGroupModel,
    },
    types::Maybe,
};

#[derive(Deserialize, Debug, Clone)]
pub struct CreateForumGroup {
    pub site_id: i64,
    pub user_id: i64,
    pub name: String,
    pub description: String,
    #[serde(default = "default_true")]
    pub visible: bool,
    pub sort_index: Option<i32>,
    #[serde(default)]
    pub from_wikidot: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateForumGroup {
    pub forum_group_id: i64,
    pub user_id: i64,

    #[serde(flatten)]
    pub body: UpdateForumGroupBody,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct UpdateForumGroupBody {
    pub name: Maybe<String>,
    pub description: Maybe<String>,
    pub visible: Maybe<bool>,
    pub sort_index: Maybe<i32>,
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct GetForumGroup {
    pub site_id: i64,
    pub forum_group_id: i64,
    #[serde(default)]
    pub include_deleted: bool,
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct GetForumGroups {
    pub site_id: i64,
    #[serde(default)]
    pub include_deleted: bool,
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct DeleteForumGroup {
    pub forum_group_id: i64,
    pub user_id: i64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CreateForumCategory {
    pub forum_group_id: i64,
    pub user_id: i64,
    pub name: String,
    pub description: String,
    pub sort_index: Option<i32>,
    pub max_nest_level: Option<i16>,
    pub per_page_discussion: Option<bool>,
    pub layout: Option<String>,
    #[serde(default)]
    pub from_wikidot: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateForumCategory {
    pub forum_category_id: i64,
    pub user_id: i64,

    #[serde(flatten)]
    pub body: UpdateForumCategoryBody,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct UpdateForumCategoryBody {
    pub forum_group_id: Maybe<i64>,
    pub name: Maybe<String>,
    pub description: Maybe<String>,
    pub sort_index: Maybe<i32>,
    pub max_nest_level: Maybe<Option<i16>>,
    pub per_page_discussion: Maybe<Option<bool>>,
    pub layout: Maybe<Option<String>>,
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct GetForumCategory {
    pub site_id: i64,
    pub forum_category_id: i64,
    #[serde(default)]
    pub include_deleted: bool,
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct GetForumCategories {
    pub site_id: i64,
    pub forum_group_id: Option<i64>,
    #[serde(default)]
    pub include_deleted: bool,
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct DeleteForumCategory {
    pub forum_category_id: i64,
    pub user_id: i64,
}

#[derive(Serialize, Debug, Clone)]
pub struct ForumGroupStructure {
    pub group: ForumGroupModel,
    pub categories: Vec<ForumCategoryModel>,
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct GetForumStructure {
    pub site_id: i64,
    #[serde(default)]
    pub include_deleted: bool,
}

#[inline]
fn default_true() -> bool {
    true
}
