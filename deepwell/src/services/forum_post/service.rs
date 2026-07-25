/*
 * services/forum_post/service.rs
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
    CreateForumPost, CreateForumPostOutput, DeleteForumPost, ForumPostNode, GetForumPost,
    GetForumPosts, GetStructuredForumPosts, RestoreForumPost, UpdateForumPost,
    UpdateForumPostBody, UpdateForumPostOutput,
};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::forum_post::{self, Entity as ForumPost, Model as ForumPostModel};
use crate::models::forum_post_revision::{
    self, Entity as ForumPostRevision, Model as ForumPostRevisionModel,
};
use crate::services::ServiceContext;
use crate::services::SettingsService;
use crate::services::forum_post_revision::{
    CreateFirstForumPostRevision, CreateFirstForumPostRevisionOutput,
    CreateForumPostRevision, CreateForumPostRevisionBody, ForumPostRevisionService,
};
use crate::services::forum_thread::{
    ForumThreadService, GetForumThread, TouchForumThread,
};
use crate::utils::now;
use paste::paste;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct ForumPostService;

impl ForumPostService {
    pub async fn create(
        ctx: &ServiceContext<'_>,
        CreateForumPost {
            forum_thread_id,
            mut parent_post_id,
            user_id,
            title,
            wikitext,
            comments,
            from_wikidot,
        }: CreateForumPost,
    ) -> Result<CreateForumPostOutput> {
        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!(
                    "failed to create forum post in thread ID {} by user ID {}",
                    forum_thread_id, user_id,
                ),
                ErrorType::ForumPost,
            )
        };

        let thread = ForumThreadService::get(
            ctx,
            GetForumThread {
                forum_thread_id,
                include_deleted: false,
            },
        )
        .await
        .or_raise(make_error)?;

        if let Some(original_parent_post_id) = parent_post_id {
            let parent_post = Self::get(
                ctx,
                GetForumPost {
                    forum_post_id: original_parent_post_id,
                    include_deleted: false,
                },
            )
            .await
            .or_raise(make_error)?;

            if parent_post.forum_thread_id != thread.forum_thread_id {
                bail!(Error::new(
                    format!(
                        "cannot create reply in thread ID {}, parent post ID {} belongs to thread ID {}",
                        thread.forum_thread_id,
                        original_parent_post_id,
                        parent_post.forum_thread_id,
                    ),
                    ErrorType::BadRequest,
                ));
            }

            let parent_depth = Self::get_depth(ctx, &parent_post)
                .await
                .or_raise(make_error)?;

            let max_nest_level = SettingsService::get_forum_max_nest_level(
                ctx,
                thread.site_id,
                Some(thread.forum_category_id),
            )
            .await
            .or_raise(make_error)?;

            // Wikidot's logic is to cap the depth at the max nest level,
            // so if the parent is already at the max, the reply is placed
            // at the same depth rather than failing.
            if parent_depth >= max_nest_level {
                parent_post_id = parent_post.parent_post_id;
            }
        }

        let post = forum_post::ActiveModel {
            parent_post_id: Set(parent_post_id),
            forum_thread_id: Set(thread.forum_thread_id),
            forum_category_id: Set(thread.forum_category_id),
            forum_group_id: Set(thread.forum_group_id),
            site_id: Set(thread.site_id),
            user_id: Set(user_id),
            from_wikidot: Set(from_wikidot),
            ..Default::default()
        }
        .insert(txn)
        .await
        .or_raise(make_error)?;

        let CreateFirstForumPostRevisionOutput {
            forum_post_revision_id,
            revision_number,
            parser_errors,
        } = ForumPostRevisionService::create_first(
            ctx,
            &post,
            CreateFirstForumPostRevision {
                user_id,
                comments,
                title,
                wikitext,
            },
        )
        .await
        .or_raise(make_error)?;

        forum_post::ActiveModel {
            forum_post_id: Set(post.forum_post_id),
            latest_revision_id: Set(Some(forum_post_revision_id)),
            ..Default::default()
        }
        .update(txn)
        .await
        .or_raise(make_error)?;

        ForumThreadService::touch_activity(
            ctx,
            TouchForumThread {
                forum_thread_id: thread.forum_thread_id,
                user_id: Some(user_id),
            },
        )
        .await
        .or_raise(make_error)?;

        // TODO(WJ-1340): notify people watching this thread about the new post.
        if parent_post_id.is_some() {
            // TODO(WJ-1340): notify people watching this forum post about a new child post.
        }

        Ok(CreateForumPostOutput {
            forum_post_id: post.forum_post_id,
            forum_post_revision_id,
            revision_number,
            parser_errors,
        })
    }

    pub async fn update(
        ctx: &ServiceContext<'_>,
        UpdateForumPost {
            forum_post_id,
            user_id,
            comments,
            body: UpdateForumPostBody { title, wikitext },
        }: UpdateForumPost,
    ) -> Result<Option<UpdateForumPostOutput>> {
        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!(
                    "failed to update forum post ID {} by user ID {}",
                    forum_post_id, user_id,
                ),
                ErrorType::ForumPost,
            )
        };

        let post = Self::get(
            ctx,
            GetForumPost {
                forum_post_id,
                include_deleted: false,
            },
        )
        .await
        .or_raise(make_error)?;

        let previous_revision =
            ForumPostRevisionService::get_latest(ctx, post.forum_post_id)
                .await
                .or_raise(make_error)?;

        let revision = ForumPostRevisionService::create(
            ctx,
            &post,
            CreateForumPostRevision {
                user_id,
                comments,
                body: CreateForumPostRevisionBody { title, wikitext },
            },
            previous_revision,
        )
        .await
        .or_raise(make_error)?;

        let revision = match revision {
            Some(revision) => revision,
            None => return Ok(None),
        };

        forum_post::ActiveModel {
            forum_post_id: Set(post.forum_post_id),
            latest_revision_id: Set(Some(revision.forum_post_revision_id)),
            updated_at: Set(Some(now())),
            ..Default::default()
        }
        .update(txn)
        .await
        .or_raise(make_error)?;

        ForumThreadService::touch_activity(
            ctx,
            TouchForumThread {
                forum_thread_id: post.forum_thread_id,
                user_id: Some(user_id),
            },
        )
        .await
        .or_raise(make_error)?;

        // TODO(WJ-1340): notify people watching this thread about an edited post.
        if post.parent_post_id.is_some() {
            // TODO(WJ-1340): notify people watching this forum post's parent about an edited child post.
        }

        Ok(Some(UpdateForumPostOutput {
            forum_post_revision_id: revision.forum_post_revision_id,
            revision_number: revision.revision_number,
            parser_errors: revision.parser_errors,
        }))
    }

    /// Soft-deletes a forum post without removing child posts.
    ///
    /// The post is marked as deleted via `deleted_at` / `deleted_by`,
    /// but its children remain visible so that the thread structure
    /// is preserved.
    pub async fn delete(
        ctx: &ServiceContext<'_>,
        DeleteForumPost {
            forum_post_id,
            user_id,
        }: DeleteForumPost,
    ) -> Result<ForumPostModel> {
        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!(
                    "failed to delete forum post ID {} by user ID {}",
                    forum_post_id, user_id,
                ),
                ErrorType::ForumPost,
            )
        };

        let post = Self::get(
            ctx,
            GetForumPost {
                forum_post_id,
                include_deleted: false,
            },
        )
        .await
        .or_raise(make_error)?;

        let model = forum_post::ActiveModel {
            forum_post_id: Set(post.forum_post_id),
            deleted_by: Set(Some(user_id)),
            deleted_at: Set(Some(now())),
            updated_at: Set(Some(now())),
            ..Default::default()
        };

        let post = model.update(txn).await.or_raise(make_error)?;

        ForumThreadService::touch_activity(
            ctx,
            TouchForumThread {
                forum_thread_id: post.forum_thread_id,
                user_id: Some(user_id),
            },
        )
        .await
        .or_raise(make_error)?;

        Ok(post)
    }

    pub async fn restore(
        ctx: &ServiceContext<'_>,
        RestoreForumPost {
            forum_post_id,
            user_id,
        }: RestoreForumPost,
    ) -> Result<ForumPostModel> {
        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!(
                    "failed to restore forum post ID {} by user ID {}",
                    forum_post_id, user_id,
                ),
                ErrorType::ForumPost,
            )
        };

        let post = Self::get(
            ctx,
            GetForumPost {
                forum_post_id,
                include_deleted: true,
            },
        )
        .await
        .or_raise(make_error)?;

        if post.deleted_at.is_none() {
            bail!(Error::new(
                format!(
                    "cannot restore forum post ID {} that isn't deleted",
                    forum_post_id
                ),
                ErrorType::ForumPostNotDeleted,
            ));
        }

        let model = forum_post::ActiveModel {
            forum_post_id: Set(post.forum_post_id),
            deleted_by: Set(None),
            deleted_at: Set(None),
            updated_at: Set(Some(now())),
            ..Default::default()
        };

        let post = model.update(txn).await.or_raise(make_error)?;

        ForumThreadService::touch_activity(
            ctx,
            TouchForumThread {
                forum_thread_id: post.forum_thread_id,
                user_id: Some(user_id),
            },
        )
        .await
        .or_raise(make_error)?;

        Ok(post)
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        GetForumPost {
            forum_post_id,
            include_deleted,
        }: GetForumPost,
    ) -> Result<Option<ForumPostModel>> {
        let txn = ctx.transaction();
        let post = ForumPost::find()
            .filter(
                Condition::all()
                    .add(forum_post::Column::ForumPostId.eq(forum_post_id))
                    .add_option(Self::deleted_condition(
                        include_deleted,
                        forum_post::Column::DeletedAt,
                    )),
            )
            .one(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!("failed to get forum post ID {}", forum_post_id),
                    ErrorType::ForumPost,
                )
            })?;

        Ok(post)
    }

    pub async fn get(
        ctx: &ServiceContext<'_>,
        key: GetForumPost,
    ) -> Result<ForumPostModel> {
        find_or_error!(Self::get_optional(ctx, key), "forum post", ForumPost)
    }

    pub async fn list(
        ctx: &ServiceContext<'_>,
        GetForumPosts {
            forum_thread_id,
            parent_post_id,
            start_post_id,
            include_deleted,
            limit,
        }: GetForumPosts,
    ) -> Result<Vec<ForumPostModel>> {
        let txn = ctx.transaction();
        let posts = ForumPost::find()
            .filter(
                Condition::all()
                    .add(forum_post::Column::ForumThreadId.eq(forum_thread_id))
                    .add_option(
                        parent_post_id.map(|id| forum_post::Column::ParentPostId.eq(id)),
                    )
                    .add_option(
                        start_post_id.map(|id| forum_post::Column::ForumPostId.gt(id)),
                    )
                    .add_option(Self::deleted_condition(
                        include_deleted,
                        forum_post::Column::DeletedAt,
                    )),
            )
            .order_by_asc(forum_post::Column::CreatedAt)
            .order_by_asc(forum_post::Column::ForumPostId)
            .limit(limit)
            .all(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to list forum posts in thread ID {}",
                        forum_thread_id,
                    ),
                    ErrorType::ForumPost,
                )
            })?;

        Ok(posts)
    }

    pub async fn list_structured(
        ctx: &ServiceContext<'_>,
        GetStructuredForumPosts {
            forum_thread_id,
            start_post_id,
            limit,
            max_depth,
        }: GetStructuredForumPosts,
    ) -> Result<Vec<ForumPostNode>> {
        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!(
                    "failed to get structured forum posts for thread ID {}",
                    forum_thread_id,
                ),
                ErrorType::ForumPost,
            )
        };

        // Always include deleted posts so that the tree structure is
        // preserved. If a middle post is deleted, its children must
        // still appear under it. The caller checks `deleted_at` to
        // decide how to render the placeholder.
        let posts = Self::list(
            ctx,
            GetForumPosts {
                forum_thread_id,
                parent_post_id: None,
                start_post_id,
                include_deleted: true,
                limit,
            },
        )
        .await
        .or_raise(make_error)?;

        let revision_ids: Vec<i64> = posts
            .iter()
            .filter_map(|post| post.latest_revision_id)
            .collect();

        let revisions = if revision_ids.is_empty() {
            Vec::new()
        } else {
            ForumPostRevision::find()
                .filter(
                    forum_post_revision::Column::ForumPostRevisionId.is_in(revision_ids),
                )
                .all(txn)
                .await
                .or_raise(make_error)?
        };

        let mut revisions_by_id: BTreeMap<i64, ForumPostRevisionModel> = BTreeMap::new();
        for revision in revisions {
            revisions_by_id.insert(revision.forum_post_revision_id, revision);
        }

        let mut posts_by_parent: BTreeMap<Option<i64>, Vec<ForumPostModel>> =
            BTreeMap::new();
        for post in posts {
            posts_by_parent
                .entry(post.parent_post_id)
                .or_default()
                .push(post);
        }

        let nodes = Self::build_post_nodes(
            None,
            0,
            max_depth.unwrap_or(u16::MAX),
            &mut posts_by_parent,
            &revisions_by_id,
        );

        Ok(nodes)
    }

    async fn get_depth(ctx: &ServiceContext<'_>, post: &ForumPostModel) -> Result<i16> {
        let mut depth = 0i16;
        let mut parent_post_id = post.parent_post_id;

        while let Some(parent_id) = parent_post_id {
            depth += 1;
            if depth > 128 {
                bail!(Error::new(
                    format!(
                        "forum post depth exceeded safety threshold while traversing from post ID {}",
                        post.forum_post_id,
                    ),
                    ErrorType::BadRequest,
                ));
            }

            let parent_post = Self::get(
                ctx,
                GetForumPost {
                    forum_post_id: parent_id,
                    include_deleted: false,
                },
            )
            .await?;

            parent_post_id = parent_post.parent_post_id;
        }

        Ok(depth)
    }

    fn build_post_nodes(
        parent_post_id: Option<i64>,
        depth: u16,
        max_depth: u16,
        posts_by_parent: &mut BTreeMap<Option<i64>, Vec<ForumPostModel>>,
        revisions_by_id: &BTreeMap<i64, ForumPostRevisionModel>,
    ) -> Vec<ForumPostNode> {
        let posts = posts_by_parent.remove(&parent_post_id).unwrap_or_default();
        let mut nodes = Vec::new();

        for post in posts {
            // Don't expose revision content for deleted posts.
            let latest_revision = if post.deleted_at.is_none() {
                post.latest_revision_id
                    .and_then(|id| revisions_by_id.get(&id).cloned())
            } else {
                None
            };

            let replies = if depth < max_depth {
                Self::build_post_nodes(
                    Some(post.forum_post_id),
                    depth + 1,
                    max_depth,
                    posts_by_parent,
                    revisions_by_id,
                )
            } else {
                Vec::new()
            };

            nodes.push(ForumPostNode {
                post,
                latest_revision,
                replies,
            });
        }

        nodes
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
}
