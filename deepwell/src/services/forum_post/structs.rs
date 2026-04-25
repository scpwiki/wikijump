/*
 * services/forum_post/structs.rs
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
        forum_post::Model as ForumPostModel,
        forum_post_revision::Model as ForumPostRevisionModel,
    },
    types::Maybe,
};
use ftml::parsing::ParseError;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateForumPost {
    pub forum_thread_id: i64,
    pub parent_post_id: Option<i64>,
    pub user_id: i64,
    pub title: String,
    pub wikitext: String,
    pub comments: String,
    #[serde(default)]
    pub from_wikidot: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct CreateForumPostOutput {
    pub forum_post_id: i64,
    pub forum_post_revision_id: i64,
    pub revision_number: i32,
    pub parser_errors: Vec<ParseError>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateForumPost {
    pub forum_post_id: i64,
    pub user_id: i64,
    pub comments: String,

    #[serde(flatten)]
    pub body: UpdateForumPostBody,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct UpdateForumPostBody {
    pub title: Maybe<String>,
    pub wikitext: Maybe<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct UpdateForumPostOutput {
    pub forum_post_revision_id: i64,
    pub revision_number: i32,
    pub parser_errors: Option<Vec<ParseError>>,
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct GetForumPost {
    pub forum_post_id: i64,
    #[serde(default)]
    pub include_deleted: bool,
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct GetForumPosts {
    pub forum_thread_id: i64,
    pub parent_post_id: Option<i64>,
    pub start_post_id: Option<i64>,
    #[serde(default)]
    pub include_deleted: bool,
    #[serde(default = "default_limit")]
    pub limit: u64,
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct GetStructuredForumPosts {
    pub forum_thread_id: i64,
    pub start_post_id: Option<i64>,
    #[serde(default = "default_limit")]
    pub limit: u64,
    pub max_depth: Option<u16>,
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct DeleteForumPost {
    pub forum_post_id: i64,
    pub user_id: i64,
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct RestoreForumPost {
    pub forum_post_id: i64,
    pub user_id: i64,
}

#[derive(Serialize, Debug, Clone)]
pub struct ForumPostNode {
    pub post: ForumPostModel,
    pub latest_revision: Option<ForumPostRevisionModel>,
    pub replies: Vec<ForumPostNode>,
}

#[inline]
fn default_limit() -> u64 {
    20
}
