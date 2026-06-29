/*
 * endpoints/text.rs
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
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::services::text_block::TextBlockIndex;
use crate::types::{Action, Permission, Reference, Resource, TextBlockType};

#[derive(Deserialize, Debug, Clone)]
struct GetIndexInput {
    site_id: i64,
    page_id: i64,
    block_type: TextBlockType,
    index: Option<i16>,
    name: Option<String>,
    session_token: Option<String>,
}

pub async fn text_block_get_index(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<Option<TextBlockIndex>> {
    let GetIndexInput {
        site_id,
        page_id,
        block_type,
        index,
        name,
        session_token,
    } = parse!(params);

    ensure_parent_page_view_permission(ctx, site_id, page_id, session_token.as_deref())
        .await?;

    match (index, name) {
        (Some(index), None) if index > 0 => {
            TextBlockService::get_block_by_index(ctx, page_id, block_type, index)
                .await
                .or_raise(|| {
                    Error::new(
                        format!(
                            "failed to get text block {:?} index {} for page ID {}",
                            block_type, index, page_id,
                        ),
                        ErrorType::Request,
                    )
                })
        }
        (None, Some(name)) => {
            TextBlockService::get_block_index(ctx, page_id, block_type, &name)
                .await
                .or_raise(|| {
                    Error::new(
                        format!(
                            "failed to get text block {:?} '{}' for page ID {}",
                            block_type, name, page_id,
                        ),
                        ErrorType::Request,
                    )
                })
        }
        _ => Err(Error::new(
            "text block lookup must provide exactly one positive index or name",
            ErrorType::Request,
        )
        .into()),
    }
}

async fn ensure_parent_page_view_permission(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_id: i64,
    session_token: Option<&str>,
) -> Result<()> {
    let make_error = || {
        Error::new(
            "failed to check parent page view permission for text block",
            ErrorType::Permission,
        )
    };

    let user_id = match session_token {
        Some("") | None => None,
        Some(token) => SessionService::get_optional(ctx, token)
            .await
            .or_raise(make_error)?
            .map(|session| session.user_id),
    };

    let page = PageService::get(ctx, site_id, Reference::Id(page_id))
        .await
        .or_raise(make_error)?;

    let can_view = PermissionService::check_user_can(
        ctx,
        &CheckPermissionContext {
            user_id,
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
    .or_raise(make_error)?;

    if can_view {
        Ok(())
    } else {
        Err(Error::new(
            "user does not have permission to view this text block's parent page",
            ErrorType::PermissionDenied,
        )
        .into())
    }
}
