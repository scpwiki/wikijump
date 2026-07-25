/*
 * services/forum_post/compat_query.rs
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

use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::{
    forum_post, forum_post::Entity as ForumPost, forum_post::Model as ForumPostModel,
    forum_post_revision::Entity as ForumPostRevision, forum_thread,
    forum_thread::Entity as ForumThread, forum_thread::Model as ForumThreadModel, page,
    page::Model as PageModel,
};
use crate::services::ServiceContext;
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::{PageService, TextService, UserService};
use crate::types::{Action, Permission, Reference, Resource};
use sea_orm::prelude::TimeDateTimeWithTimeZone;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct ForumPostSelectInput {
    site_id: i64,
    page: Option<String>,
    reply_to: Option<String>,
    created_by: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ForumPostGetInput {
    site_id: i64,
    posts: Vec<ForumPostIdInput>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ForumPostIdInput {
    Number(i64),
    String(String),
}

#[derive(Deserialize, Debug)]
pub struct ForumPostPageSummaryInput {
    site_id: i64,
    page: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct WikidotForumPost {
    id: i64,
    fullname: String,
    reply_to: Option<i64>,
    title: String,
    content: String,
    html: String,
    created_by: String,
    #[serde(with = "time::serde::rfc3339")]
    created_at: TimeDateTimeWithTimeZone,
}

#[derive(Serialize, Debug, Clone)]
pub struct ForumPostPageSummary {
    comments: i64,
    #[serde(with = "time::serde::rfc3339::option")]
    commented_at: Option<TimeDateTimeWithTimeZone>,
    commented_by: Option<String>,
}

enum ParentPostFilter {
    All,
    TopLevel,
    Direct(i64),
}

pub async fn forum_post_select(
    ctx: &ServiceContext<'_>,
    ForumPostSelectInput {
        site_id,
        page,
        reply_to,
        created_by,
    }: ForumPostSelectInput,
) -> Result<Vec<i64>> {
    let thread = if let Some(page) = page.as_deref() {
        let Some((page, thread)) = find_page_thread(ctx, site_id, page).await? else {
            return Ok(Vec::new());
        };
        if !can_view_page(ctx, site_id, &page).await? {
            return Ok(Vec::new());
        }
        Some(thread)
    } else {
        None
    };

    let Some(created_by_user_id) =
        resolve_optional_user_filter(ctx, created_by.as_deref()).await?
    else {
        return Ok(Vec::new());
    };

    let parent_filter = parse_parent_post_filter(reply_to.as_deref())?;
    let mut condition = Condition::all()
        .add(forum_post::Column::SiteId.eq(site_id))
        .add(forum_post::Column::DeletedAt.is_null());

    if let Some(thread) = thread {
        condition =
            condition.add(forum_post::Column::ForumThreadId.eq(thread.forum_thread_id));
    }
    if let Some(user_id) = created_by_user_id {
        condition = condition.add(forum_post::Column::UserId.eq(user_id));
    }

    match parent_filter {
        ParentPostFilter::All => {}
        ParentPostFilter::TopLevel => {
            condition = condition.add(forum_post::Column::ParentPostId.is_null());
        }
        ParentPostFilter::Direct(parent_post_id) => {
            condition =
                condition.add(forum_post::Column::ParentPostId.eq(parent_post_id));
        }
    }

    let posts = ForumPost::find()
        .filter(condition)
        .order_by_asc(forum_post::Column::CreatedAt)
        .order_by_asc(forum_post::Column::ForumPostId)
        .all(ctx.transaction())
        .await
        .or_raise(|| Error::new("failed to select forum posts", ErrorType::ForumPost))?;

    let mut output = Vec::with_capacity(posts.len());
    for post in posts {
        if can_view_forum_post(ctx, &post).await? {
            output.push(post.forum_post_id);
        }
    }

    Ok(output)
}

pub async fn forum_post_get(
    ctx: &ServiceContext<'_>,
    ForumPostGetInput { site_id, posts }: ForumPostGetInput,
) -> Result<Vec<WikidotForumPost>> {
    if posts.len() > 10 {
        return Err(Error::new(
            "forum_post_get posts is limited to 10 entries",
            ErrorType::BadRequest,
        )
        .into());
    }

    let post_ids = parse_post_ids(&posts)?;
    if post_ids.is_empty() {
        return Ok(Vec::new());
    }

    let models = ForumPost::find()
        .filter(
            Condition::all()
                .add(forum_post::Column::SiteId.eq(site_id))
                .add(forum_post::Column::ForumPostId.is_in(post_ids.clone()))
                .add(forum_post::Column::DeletedAt.is_null()),
        )
        .all(ctx.transaction())
        .await
        .or_raise(|| Error::new("failed to get forum posts", ErrorType::ForumPost))?;

    let models_by_id: HashMap<i64, ForumPostModel> = models
        .into_iter()
        .map(|post| (post.forum_post_id, post))
        .collect();
    let mut output = Vec::with_capacity(post_ids.len());
    for post_id in post_ids {
        if let Some(post) = models_by_id.get(&post_id)
            && can_view_forum_post(ctx, post).await?
        {
            output.push(build_wikidot_forum_post(ctx, post.clone()).await?);
        }
    }

    Ok(output)
}

pub async fn forum_post_page_summary(
    ctx: &ServiceContext<'_>,
    ForumPostPageSummaryInput { site_id, page }: ForumPostPageSummaryInput,
) -> Result<ForumPostPageSummary> {
    let Some((page_model, thread)) = find_page_thread(ctx, site_id, &page).await? else {
        return Ok(empty_page_summary());
    };
    if !can_view_page(ctx, site_id, &page_model).await? {
        return Ok(empty_page_summary());
    }

    let condition = Condition::all()
        .add(forum_post::Column::SiteId.eq(site_id))
        .add(forum_post::Column::ForumThreadId.eq(thread.forum_thread_id))
        .add(forum_post::Column::DeletedAt.is_null());

    let comments = ForumPost::find()
        .filter(condition.clone())
        .count(ctx.transaction())
        .await
        .or_raise(|| Error::new("failed to count forum posts", ErrorType::ForumPost))?;

    if comments == 0 {
        return Ok(empty_page_summary());
    }

    let latest = ForumPost::find()
        .filter(condition)
        .order_by_desc(forum_post::Column::CreatedAt)
        .order_by_desc(forum_post::Column::ForumPostId)
        .one(ctx.transaction())
        .await
        .or_raise(|| {
            Error::new("failed to get latest forum post", ErrorType::ForumPost)
        })?;

    let Some(latest) = latest else {
        return Ok(empty_page_summary());
    };
    let commented_by = user_slug(ctx, latest.user_id).await?;

    Ok(ForumPostPageSummary {
        comments: i64::try_from(comments).unwrap_or(i64::MAX),
        commented_at: Some(latest.created_at),
        commented_by: Some(commented_by),
    })
}

async fn build_wikidot_forum_post(
    ctx: &ServiceContext<'_>,
    post: ForumPostModel,
) -> Result<WikidotForumPost> {
    let Some(revision_id) = post.latest_revision_id else {
        return Err(Error::new(
            format!(
                "forum post {} is missing its latest revision",
                post.forum_post_id
            ),
            ErrorType::ForumPost,
        )
        .into());
    };
    let make_error =
        || Error::new("failed to build forum post output", ErrorType::ForumPost);

    let Some(revision) = ForumPostRevision::find_by_id(revision_id)
        .one(ctx.transaction())
        .await
        .or_raise(make_error)?
    else {
        return Err(Error::new(
            format!("forum post revision {revision_id} does not exist"),
            ErrorType::ForumPostRevisionNotFound,
        )
        .into());
    };
    let Some(thread) = ForumThread::find_by_id(post.forum_thread_id)
        .filter(
            Condition::all()
                .add(forum_thread::Column::SiteId.eq(post.site_id))
                .add(forum_thread::Column::DeletedAt.is_null()),
        )
        .one(ctx.transaction())
        .await
        .or_raise(make_error)?
    else {
        return Err(Error::new(
            format!("forum thread {} does not exist", post.forum_thread_id),
            ErrorType::ForumPost,
        )
        .into());
    };
    let Some(page_id) = thread.page_id else {
        return Err(Error::new(
            format!(
                "forum thread {} is not associated with a page",
                thread.forum_thread_id
            ),
            ErrorType::ForumPost,
        )
        .into());
    };
    let Some(page) = page::Entity::find_by_id(page_id)
        .filter(
            Condition::all()
                .add(page::Column::SiteId.eq(post.site_id))
                .add(page::Column::DeletedAt.is_null()),
        )
        .one(ctx.transaction())
        .await
        .or_raise(make_error)?
    else {
        return Err(Error::new(
            format!("forum post page {page_id} does not exist"),
            ErrorType::Page,
        )
        .into());
    };

    if !can_view_page(ctx, post.site_id, &page).await? {
        return Err(Error::new(
            "forum post page is not viewable",
            ErrorType::PermissionDenied,
        )
        .into());
    }

    let content = TextService::get(ctx, &revision.wikitext_hash)
        .await
        .or_raise(make_error)?;
    let html = TextService::get(ctx, &revision.compiled_html_hash)
        .await
        .or_raise(make_error)?;
    let created_by = user_slug(ctx, post.user_id).await?;

    Ok(WikidotForumPost {
        id: post.forum_post_id,
        fullname: page.slug,
        reply_to: post.parent_post_id,
        title: revision.title,
        content,
        html,
        created_by,
        created_at: post.created_at,
    })
}

async fn find_page_thread(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_reference: &str,
) -> Result<Option<(PageModel, ForumThreadModel)>> {
    let make_error = || {
        Error::new(
            format!("failed to find forum thread for page '{page_reference}'"),
            ErrorType::ForumPost,
        )
    };

    let Some(page) =
        PageService::get_optional(ctx, site_id, Reference::from(page_reference))
            .await
            .or_raise(make_error)?
    else {
        return Ok(None);
    };

    if let Some(thread_id) = page.discussion_thread_id {
        let thread = ForumThread::find_by_id(thread_id)
            .filter(
                Condition::all()
                    .add(forum_thread::Column::SiteId.eq(site_id))
                    .add(forum_thread::Column::PageId.eq(page.page_id))
                    .add(forum_thread::Column::DeletedAt.is_null()),
            )
            .one(ctx.transaction())
            .await
            .or_raise(make_error)?;

        if let Some(thread) = thread {
            return Ok(Some((page, thread)));
        }
    }

    let thread = ForumThread::find()
        .filter(
            Condition::all()
                .add(forum_thread::Column::SiteId.eq(site_id))
                .add(forum_thread::Column::PageId.eq(page.page_id))
                .add(forum_thread::Column::DeletedAt.is_null()),
        )
        .one(ctx.transaction())
        .await
        .or_raise(make_error)?;

    Ok(thread.map(|thread| (page, thread)))
}

async fn can_view_forum_post(
    ctx: &ServiceContext<'_>,
    post: &ForumPostModel,
) -> Result<bool> {
    let make_error =
        || Error::new("failed to check forum post page", ErrorType::ForumPost);

    let Some(thread) = ForumThread::find_by_id(post.forum_thread_id)
        .filter(
            Condition::all()
                .add(forum_thread::Column::SiteId.eq(post.site_id))
                .add(forum_thread::Column::DeletedAt.is_null()),
        )
        .one(ctx.transaction())
        .await
        .or_raise(make_error)?
    else {
        return Ok(false);
    };
    let Some(page_id) = thread.page_id else {
        return Ok(false);
    };
    let Some(page) = page::Entity::find_by_id(page_id)
        .filter(
            Condition::all()
                .add(page::Column::SiteId.eq(post.site_id))
                .add(page::Column::DeletedAt.is_null()),
        )
        .one(ctx.transaction())
        .await
        .or_raise(make_error)?
    else {
        return Ok(false);
    };

    can_view_page(ctx, post.site_id, &page).await
}

async fn can_view_page(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page: &PageModel,
) -> Result<bool> {
    PermissionService::check_user_can(
        ctx,
        &CheckPermissionContext {
            user_id: ctx.request().user_id,
            site_id,
            page_reference: Some(Reference::Id(page.page_id)),
        },
        Permission {
            resource_type: Resource::Page,
            resource_category: Some(Reference::Id(page.page_category_id)),
            action: Action::View,
        },
    )
    .await
    .or_raise(|| {
        Error::new(
            "failed to check forum post page permissions",
            ErrorType::Permission,
        )
    })
}

async fn resolve_optional_user_filter(
    ctx: &ServiceContext<'_>,
    created_by: Option<&str>,
) -> Result<Option<Option<i64>>> {
    let Some(created_by) = created_by else {
        return Ok(Some(None));
    };

    if let Ok(user_id) = created_by.parse::<i64>() {
        return Ok(Some(Some(user_id)));
    }

    let user = UserService::get_optional(ctx, Reference::from(created_by))
        .await
        .or_raise(|| Error::new("failed to resolve forum post user", ErrorType::User))?;

    Ok(user.map(|user| Some(user.user_id)))
}

fn parse_parent_post_filter(reply_to: Option<&str>) -> Result<ParentPostFilter> {
    match reply_to {
        None => Ok(ParentPostFilter::All),
        Some("-") => Ok(ParentPostFilter::TopLevel),
        Some(reply_to) => reply_to
            .parse::<i64>()
            .map(ParentPostFilter::Direct)
            .map_err(|_| {
                Error::new(
                    format!("invalid forum post reply_to value '{reply_to}'"),
                    ErrorType::BadRequest,
                )
                .into()
            }),
    }
}

fn parse_post_ids(posts: &[ForumPostIdInput]) -> Result<Vec<i64>> {
    posts
        .iter()
        .map(|post| match post {
            ForumPostIdInput::Number(post_id) => Ok(*post_id),
            ForumPostIdInput::String(post) => post.parse::<i64>().map_err(|_| {
                Error::new(
                    format!("invalid forum post ID '{post}'"),
                    ErrorType::BadRequest,
                )
                .into()
            }),
        })
        .collect()
}

async fn user_slug(ctx: &ServiceContext<'_>, user_id: i64) -> Result<String> {
    let user = UserService::get_optional(ctx, Reference::Id(user_id))
        .await
        .or_raise(|| Error::new("failed to resolve forum post user", ErrorType::User))?;

    Ok(user
        .map(|user| user.slug)
        .unwrap_or_else(|| user_id.to_string()))
}

fn empty_page_summary() -> ForumPostPageSummary {
    ForumPostPageSummary {
        comments: 0,
        commented_at: None,
        commented_by: None,
    }
}
