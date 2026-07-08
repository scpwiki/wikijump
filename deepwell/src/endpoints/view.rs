/*
 * endpoints/view.rs
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
use crate::services::view::{
    GetAdminView, GetAdminViewOutput, GetArticleViewOutput, GetPageView,
    GetPageViewOutput, GetPreloadView, GetPreloadViewOutput, GetUserView,
    GetUserViewOutput, ViewType,
};

/// Returns relevant context for rendering a view from a processed web request.
pub async fn preload_view(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<GetPreloadViewOutput> {
    let input: GetPreloadView = parse!(params => ErrorType::GetView(ViewType::Preload));

    ViewService::preload(ctx, input).await.or_raise(|| {
        Error::new(
            "failed to get preload view",
            ErrorType::GetView(ViewType::Preload),
        )
    })
}

/// Returns common preload data plus page data for an article request.
pub async fn article_view(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<GetArticleViewOutput> {
    let input: GetPageView = parse!(params => ErrorType::GetView(ViewType::Page));

    ViewService::article(ctx, input).await.or_raise(|| {
        Error::new(
            "failed to get article view",
            ErrorType::GetView(ViewType::Page),
        )
    })
}

/// Returns relevant context for rendering a page from a processed web request.
pub async fn page_view(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<GetPageViewOutput> {
    let input: GetPageView = parse!(params => ErrorType::GetView(ViewType::Page));

    ViewService::page(ctx, input).await.or_raise(|| {
        Error::new(
            "failed to get page view",
            ErrorType::GetView(ViewType::Page),
        )
    })
}

/// Returns relevant context for rendering a user profile from a processed web request.
pub async fn user_view(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<GetUserViewOutput> {
    let input: GetUserView = parse!(params => ErrorType::GetView(ViewType::User));

    ViewService::user(ctx, input).await.or_raise(|| {
        Error::new(
            "failed to get user view",
            ErrorType::GetView(ViewType::User),
        )
    })
}

/// Returns relevant context for rendering admin panel from a processed web request.
pub async fn admin_view(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<GetAdminViewOutput> {
    let input: GetAdminView = parse!(params => ErrorType::GetView(ViewType::Admin));

    ViewService::admin(ctx, input).await.or_raise(|| {
        Error::new(
            "failed to get admin view",
            ErrorType::GetView(ViewType::Admin),
        )
    })
}
