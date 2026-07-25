/*
 * services/forum_post_revision/service.rs
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
    CreateFirstForumPostRevision, CreateFirstForumPostRevisionOutput,
    CreateForumPostRevision, CreateForumPostRevisionBody, CreateForumPostRevisionOutput,
    GetForumPostRevision, GetForumPostRevisionRange, UpdateForumPostRevision,
};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::forum_post::Model as ForumPostModel;
use crate::models::forum_post_revision::{
    self, Entity as ForumPostRevision, Model as ForumPostRevisionModel,
};
use crate::services::ServiceContext;
use crate::services::render::RenderOutput;
use crate::services::score::ScoreValue;
use crate::services::{RenderService, SiteService, TextService};
use crate::types::FetchDirection;
use crate::types::{Maybe, Reference};
use crate::utils::locale_for_ftml;
use crate::utils::now;
use ftml::data::PageInfo;
use ftml::settings::{WikitextMode, WikitextSettings};
use paste::paste;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use sea_query::Order;
use std::borrow::Cow;

#[derive(Debug)]
pub struct ForumPostRevisionService;

impl ForumPostRevisionService {
    pub async fn create_first(
        ctx: &ServiceContext<'_>,
        post: &ForumPostModel,
        CreateFirstForumPostRevision {
            user_id,
            comments,
            title,
            wikitext,
        }: CreateFirstForumPostRevision,
    ) -> Result<CreateFirstForumPostRevisionOutput> {
        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!(
                    "failed to create first revision for forum post ID {} in thread ID {} by user ID {}",
                    post.forum_post_id, post.forum_thread_id, user_id,
                ),
                ErrorType::ForumPostRevision,
            )
        };

        let wikitext_hash = TextService::create(ctx, wikitext.clone())
            .await
            .or_raise(make_error)?;

        let RenderOutput {
            html_output: _,
            errors,
            compiled_hash: compiled_html_hash,
            compiled_at,
            compiled_generator,
        } = Self::render(ctx, post.site_id, &title, wikitext)
            .await
            .or_raise(make_error)?;

        let model = forum_post_revision::ActiveModel {
            forum_post_id: Set(post.forum_post_id),
            forum_thread_id: Set(post.forum_thread_id),
            forum_category_id: Set(post.forum_category_id),
            forum_group_id: Set(post.forum_group_id),
            site_id: Set(post.site_id),
            user_id: Set(user_id),
            revision_number: Set(0),
            title: Set(title),
            wikitext_hash: Set(wikitext_hash.to_vec()),
            compiled_html_hash: Set(compiled_html_hash.to_vec()),
            compiled_at: Set(compiled_at),
            compiled_generator: Set(compiled_generator),
            comments: Set(comments),
            ..Default::default()
        };

        let ForumPostRevisionModel {
            forum_post_revision_id,
            revision_number,
            ..
        } = model.insert(txn).await.or_raise(make_error)?;

        Ok(CreateFirstForumPostRevisionOutput {
            forum_post_revision_id,
            revision_number,
            parser_errors: errors,
        })
    }

    pub async fn create(
        ctx: &ServiceContext<'_>,
        post: &ForumPostModel,
        CreateForumPostRevision {
            user_id,
            comments,
            body: CreateForumPostRevisionBody { title, wikitext },
        }: CreateForumPostRevision,
        previous: ForumPostRevisionModel,
    ) -> Result<Option<CreateForumPostRevisionOutput>> {
        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!(
                    "failed to create new revision for forum post ID {} in thread ID {} by user ID {}",
                    post.forum_post_id, post.forum_thread_id, user_id,
                ),
                ErrorType::ForumPostRevision,
            )
        };

        let revision_number = next_revision_number(&previous, post);
        let mut changed = false;
        let mut should_render = false;
        let mut parser_errors = None;
        let mut changed_wikitext = None;

        let mut revision_title = previous.title;
        let mut wikitext_hash = previous.wikitext_hash;
        let mut compiled_html_hash = previous.compiled_html_hash;
        let mut compiled_at = previous.compiled_at;
        let mut compiled_generator = previous.compiled_generator;

        if let Maybe::Set(title) = title
            && revision_title != title
        {
            revision_title = title;
            changed = true;
            should_render = true;
        }

        if let Maybe::Set(wikitext) = wikitext {
            let new_hash = TextService::create(ctx, wikitext.clone())
                .await
                .or_raise(make_error)?;

            if wikitext_hash != new_hash {
                wikitext_hash = new_hash.to_vec();
                changed_wikitext = Some(wikitext);
                changed = true;
                should_render = true;
            }
        }

        if !changed {
            debug!(
                "No effective forum post changes for post ID {}, skipping revision",
                post.forum_post_id,
            );
            return Ok(None);
        }

        if should_render {
            let wikitext = match changed_wikitext {
                Some(wikitext) => wikitext,
                None => TextService::get(ctx, &wikitext_hash)
                    .await
                    .or_raise(make_error)?,
            };

            let RenderOutput {
                html_output: _,
                errors,
                compiled_hash,
                compiled_at: new_compiled_at,
                compiled_generator: new_compiled_generator,
            } = Self::render(ctx, post.site_id, &revision_title, wikitext)
                .await
                .or_raise(make_error)?;

            parser_errors = Some(errors);
            compiled_html_hash = compiled_hash.to_vec();
            compiled_at = new_compiled_at;
            compiled_generator = new_compiled_generator;
        }

        let model = forum_post_revision::ActiveModel {
            forum_post_id: Set(post.forum_post_id),
            forum_thread_id: Set(post.forum_thread_id),
            forum_category_id: Set(post.forum_category_id),
            forum_group_id: Set(post.forum_group_id),
            site_id: Set(post.site_id),
            user_id: Set(user_id),
            revision_number: Set(revision_number),
            title: Set(revision_title),
            wikitext_hash: Set(wikitext_hash),
            compiled_html_hash: Set(compiled_html_hash),
            compiled_at: Set(compiled_at),
            compiled_generator: Set(compiled_generator),
            comments: Set(comments),
            ..Default::default()
        };

        let ForumPostRevisionModel {
            forum_post_revision_id,
            revision_number,
            ..
        } = model.insert(txn).await.or_raise(make_error)?;

        Ok(Some(CreateForumPostRevisionOutput {
            forum_post_revision_id,
            revision_number,
            parser_errors,
        }))
    }

    pub async fn update(
        ctx: &ServiceContext<'_>,
        UpdateForumPostRevision {
            forum_post_revision_id,
            user_id,
            comments,
        }: UpdateForumPostRevision,
    ) -> Result<ForumPostRevisionModel> {
        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!(
                    "failed to update forum post revision ID {} by user ID {}",
                    forum_post_revision_id, user_id,
                ),
                ErrorType::ForumPostRevision,
            )
        };

        Self::assert_exists_direct(ctx, forum_post_revision_id)
            .await
            .or_raise(make_error)?;

        let mut model = forum_post_revision::ActiveModel {
            forum_post_revision_id: Set(forum_post_revision_id),
            updated_at: Set(Some(now())),
            ..Default::default()
        };

        if let Maybe::Set(comments) = comments {
            model.comments = Set(comments);
        }

        let revision = model.update(txn).await.or_raise(make_error)?;
        Ok(revision)
    }

    pub async fn get_latest(
        ctx: &ServiceContext<'_>,
        forum_post_id: i64,
    ) -> Result<ForumPostRevisionModel> {
        let txn = ctx.transaction();
        let revision = ForumPostRevision::find()
            .filter(forum_post_revision::Column::ForumPostId.eq(forum_post_id))
            .order_by_desc(forum_post_revision::Column::RevisionNumber)
            .one(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to get latest revision for forum post ID {}",
                        forum_post_id,
                    ),
                    ErrorType::ForumPostRevision,
                )
            })?;

        match revision {
            Some(revision) => Ok(revision),
            None => bail!(Error::new(
                format!(
                    "no latest revision exists for forum post ID {}",
                    forum_post_id,
                ),
                ErrorType::ForumPostRevisionNotFound,
            )),
        }
    }

    pub async fn get_optional(
        ctx: &ServiceContext<'_>,
        GetForumPostRevision {
            forum_post_id,
            revision_number,
        }: GetForumPostRevision,
    ) -> Result<Option<ForumPostRevisionModel>> {
        let txn = ctx.transaction();
        let revision = ForumPostRevision::find()
            .filter(
                Condition::all()
                    .add(forum_post_revision::Column::ForumPostId.eq(forum_post_id))
                    .add(forum_post_revision::Column::RevisionNumber.eq(revision_number)),
            )
            .one(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to get forum post revision number {} for post ID {}",
                        revision_number, forum_post_id,
                    ),
                    ErrorType::ForumPostRevision,
                )
            })?;

        Ok(revision)
    }

    pub async fn get(
        ctx: &ServiceContext<'_>,
        key: GetForumPostRevision,
    ) -> Result<ForumPostRevisionModel> {
        find_or_error!(
            Self::get_optional(ctx, key),
            "forum post revision",
            ForumPostRevision,
        )
    }

    pub async fn get_direct(
        ctx: &ServiceContext<'_>,
        forum_post_revision_id: i64,
    ) -> Result<ForumPostRevisionModel> {
        let txn = ctx.transaction();
        let revision = ForumPostRevision::find_by_id(forum_post_revision_id)
            .one(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to get forum post revision ID {} directly",
                        forum_post_revision_id,
                    ),
                    ErrorType::ForumPostRevision,
                )
            })?;

        Ok(revision.ok_or_else(|| {
            Error::new(
                format!(
                    "forum post revision ID {} does not exist",
                    forum_post_revision_id,
                ),
                ErrorType::ForumPostRevisionNotFound,
            )
        })?)
    }

    #[inline]
    pub async fn assert_exists_direct(
        ctx: &ServiceContext<'_>,
        forum_post_revision_id: i64,
    ) -> Result<()> {
        let _ = Self::get_direct(ctx, forum_post_revision_id).await?;
        Ok(())
    }

    pub async fn count(ctx: &ServiceContext<'_>, forum_post_id: i64) -> Result<u64> {
        let txn = ctx.transaction();
        let count = ForumPostRevision::find()
            .filter(forum_post_revision::Column::ForumPostId.eq(forum_post_id))
            .count(txn)
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to count revisions for forum post ID {}",
                        forum_post_id,
                    ),
                    ErrorType::ForumPostRevision,
                )
            })?;

        Ok(count)
    }

    pub async fn get_range(
        ctx: &ServiceContext<'_>,
        GetForumPostRevisionRange {
            forum_post_id,
            revision_number,
            revision_direction,
            limit,
        }: GetForumPostRevisionRange,
    ) -> Result<Vec<ForumPostRevisionModel>> {
        let txn = ctx.transaction();
        let make_error = || {
            Error::new(
                format!(
                    "failed to get forum post revisions from revision {} ({}) for post ID {} (limit {})",
                    revision_number,
                    revision_direction.name(),
                    forum_post_id,
                    limit,
                ),
                ErrorType::ForumPostRevision,
            )
        };

        let revision_condition = {
            use forum_post_revision::Column::RevisionNumber;

            let revision_number = if revision_number >= 0 {
                revision_number
            } else {
                Self::get_latest(ctx, forum_post_id)
                    .await
                    .or_raise(make_error)?
                    .revision_number
            };

            match revision_direction {
                FetchDirection::Before => RevisionNumber.lte(revision_number),
                FetchDirection::After => RevisionNumber.gte(revision_number),
            }
        };

        let mut query = ForumPostRevision::find().filter(
            Condition::all()
                .add(forum_post_revision::Column::ForumPostId.eq(forum_post_id))
                .add(revision_condition),
        );

        query =
            match revision_direction {
                FetchDirection::Before => query
                    .order_by(forum_post_revision::Column::RevisionNumber, Order::Desc),
                FetchDirection::After => query
                    .order_by(forum_post_revision::Column::RevisionNumber, Order::Asc),
            };

        let revisions = query.limit(limit).all(txn).await.or_raise(make_error)?;
        Ok(revisions)
    }

    async fn render(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        title: &str,
        wikitext: String,
    ) -> Result<RenderOutput> {
        let site = SiteService::get(ctx, Reference::Id(site_id))
            .await
            .or_raise(|| {
                Error::new(
                    format!(
                        "failed to get site ID {} to render forum post revision",
                        site_id,
                    ),
                    ErrorType::ForumPostRevision,
                )
            })?;

        let settings = WikitextSettings::from_mode(
            WikitextMode::ForumPost,
            ctx.config().message_layout,
        );
        let page_info = PageInfo {
            page: Cow::Borrowed(""),
            category: None,
            site: Cow::Owned(site.slug),
            title: Cow::Owned(title.to_owned()),
            alt_title: None,
            score: ScoreValue::Integer(0),
            tags: vec![],
            language: Cow::Owned(locale_for_ftml(&site.locale).to_owned()),
        };

        let render = RenderService::render(ctx, wikitext, &page_info, &settings)
            .await
            .or_raise(|| {
                Error::new("failed to render forum post revision", ErrorType::Forum)
            })?;

        Ok(render)
    }
}

fn next_revision_number(previous: &ForumPostRevisionModel, post: &ForumPostModel) -> i32 {
    assert_eq!(
        previous.forum_post_id, post.forum_post_id,
        "Previous forum post revision has an inconsistent forum post ID",
    );
    assert_eq!(
        previous.forum_thread_id, post.forum_thread_id,
        "Previous forum post revision has an inconsistent forum thread ID",
    );
    assert_eq!(
        previous.forum_category_id, post.forum_category_id,
        "Previous forum post revision has an inconsistent forum category ID",
    );
    assert_eq!(
        previous.forum_group_id, post.forum_group_id,
        "Previous forum post revision has an inconsistent forum group ID",
    );
    assert_eq!(
        previous.site_id, post.site_id,
        "Previous forum post revision has an inconsistent site ID",
    );

    previous.revision_number + 1
}
