/*
 * endpoints/forum_post.rs
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
use crate::services::forum_post::{ForumPostPageSummary, WikidotForumPost};

pub async fn forum_post_select(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<i64>> {
    crate::services::forum_post::forum_post_select(ctx, parse!(params, ForumPost)).await
}

pub async fn forum_post_get(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Vec<WikidotForumPost>> {
    crate::services::forum_post::forum_post_get(ctx, parse!(params, ForumPost)).await
}

pub async fn forum_post_page_summary(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<ForumPostPageSummary> {
    crate::services::forum_post::forum_post_page_summary(ctx, parse!(params, ForumPost))
        .await
}
