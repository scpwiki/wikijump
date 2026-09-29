/*
 * role/resolvers.rs
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

use crate::error::Result;
use crate::services::permission::PermissionTarget;
use crate::services::relation::GetPageAttributions;
use crate::services::role::SystemRole;
use crate::services::{RelationService, ServiceContext};
use crate::types::{Reference, Resource};

pub(super) async fn resolve_virtual_roles_for_user_and_resource(
    ctx: &ServiceContext<'_>,
    user_id: Option<i64>,
    site_id: i64,
    target: &PermissionTarget,
) -> Result<Vec<SystemRole>> {
    match target {
        PermissionTarget::Site => {
            resolve_virtual_roles_for_user_and_site(ctx, user_id, site_id).await
        }
        PermissionTarget::Page { page_id, .. } => {
            resolve_virtual_roles_for_user_and_page(
                ctx,
                user_id,
                site_id,
                &Reference::Id(*page_id),
            )
            .await
        }
        PermissionTarget::Lock => Ok(vec![]),
    }
}

async fn resolve_virtual_roles_for_user_and_site(
    ctx: &ServiceContext<'_>,
    user_id: Option<i64>,
    site_id: i64,
) -> Result<Vec<SystemRole>> {
    let is_logged_in = user_id.is_some();
    let is_member = if let Some(user_id) = user_id {
        let membership = RelationService::get_optional_site_member(
            ctx,
            crate::services::relation::GetSiteMember { site_id, user_id },
        )
        .await?;

        membership.is_some()
    } else {
        false
    };
    let is_banned = if let Some(user_id) = user_id {
        RelationService::site_ban_exists(
            ctx,
            crate::services::relation::GetSiteBan { site_id, user_id },
        )
        .await?
    } else {
        false
    };

    let mut roles = Vec::with_capacity(5);
    if is_logged_in {
        roles.push(SystemRole::Registered);
        if is_banned {
            roles.push(SystemRole::Banned);
        } else if is_member {
            roles.push(SystemRole::Member);
        } else {
            roles.push(SystemRole::Guest);
        }
    } else {
        roles.push(SystemRole::Anonymous);
        roles.push(SystemRole::Guest);
    }
    roles.push(SystemRole::Everyone);

    Ok(roles)
}

async fn resolve_virtual_roles_for_user_and_page(
    ctx: &ServiceContext<'_>,
    user_id: Option<i64>,
    site_id: i64,
    reference: &Reference<'_>,
) -> Result<Vec<SystemRole>> {
    if let Some(user) = user_id {
        let attributions = RelationService::get_page_attributions(
            ctx,
            GetPageAttributions {
                site_id,
                page: reference.clone(),
            },
        )
        .await?;
        if attributions.iter().any(|attr| attr.user_id == user) {
            return Ok(vec![SystemRole::PageAuthor]);
        }
    }
    Ok(vec![])
}
