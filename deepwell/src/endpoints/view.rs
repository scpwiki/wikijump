/*
 * endpoints/view.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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
    GetPageView, GetPageViewOutput, GetUserView, GetUserViewOutput,
};

/// Returns relevant context for rendering a page from a processed web request.
pub async fn page_view(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<GetPageViewOutput> {
    let input: GetPageView = params.parse()?;
    ViewService::page(ctx, input).await
}

/// Returns relevant context for rendering a user profile from a processed web request.
pub async fn user_view(
    ctx: &ServiceContext,
    params: Params<'static>,
) -> Result<GetUserViewOutput> {
    let input: GetUserView = params.parse()?;
    ViewService::user(ctx, input).await
}
