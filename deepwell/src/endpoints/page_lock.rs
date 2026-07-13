/*
 * endpoints/page_lock.rs
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
use crate::models::page_lock::Model as PageLockModel;
use crate::services::page_lock::{CreatePageLockInput, RemovePageLockInput};
use crate::services::{MutationAuthorization, PageLockService};
use crate::types::{Action, Permission, Resource};

async fn require_page_lock_permission(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_ref: crate::types::Reference<'_>,
    action: &str,
) -> Result<()> {
    MutationAuthorization::require_permission(
        ctx,
        site_id,
        Some(page_ref),
        Permission {
            resource_type: Resource::Page,
            resource_category: None,
            action: Action::BypassLock,
        },
        action,
    )
    .await?;
    Ok(())
}

pub async fn page_lock_create(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let input: CreatePageLockInput = parse!(params, PageLock);

    let request = ctx.request();
    let site_id = request
        .site_id()
        .or_raise(|| Error::new("no site ID found", ErrorType::PageLock))?;
    let user_id = request
        .user_id()
        .or_raise(|| Error::new("no user ID found", ErrorType::PageLock))?;
    let page_ref = request
        .page_reference()
        .or_raise(|| Error::new("no page reference found", ErrorType::PageLock))?;
    require_page_lock_permission(ctx, site_id, page_ref.borrow(), "create a page lock")
        .await?;

    info!(
        "Creating page lock of type {:?} for page {:?} in site {}",
        input.lock_type, page_ref, site_id,
    );

    PageLockService::create(ctx, site_id, user_id, page_ref.borrow(), input)
        .await
        .or_raise(|| Error::new("failed to create page lock", ErrorType::PageLock))?;

    Ok(())
}

pub async fn page_lock_remove(
    ctx: &ServiceContext<'_>,
    params: Params<'static>,
) -> Result<()> {
    let input: RemovePageLockInput = parse!(params, PageLock);

    let request = ctx.request();
    let site_id = request
        .site_id()
        .or_raise(|| Error::new("no site ID found", ErrorType::PageLock))?;
    let user_id = request
        .user_id()
        .or_raise(|| Error::new("no user ID found", ErrorType::PageLock))?;
    let page_ref = request
        .page_reference()
        .or_raise(|| Error::new("no page reference found", ErrorType::PageLock))?;
    require_page_lock_permission(ctx, site_id, page_ref.borrow(), "remove a page lock")
        .await?;

    info!(
        "Removing active page lock for page {:?} in site {}",
        page_ref, site_id,
    );

    PageLockService::remove(ctx, site_id, user_id, page_ref.borrow(), input.ip_address)
        .await
        .or_raise(|| Error::new("failed to remove page lock", ErrorType::PageLock))?;

    Ok(())
}

pub async fn page_lock_get_history(
    ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<Vec<PageLockModel>> {
    let request = ctx.request();
    let site_id = request
        .site_id()
        .or_raise(|| Error::new("no site ID found", ErrorType::PageLock))?;
    let page_ref = request
        .page_reference()
        .or_raise(|| Error::new("no page reference found", ErrorType::PageLock))?;

    info!(
        "Fetching lock history for page {:?} in site {}",
        page_ref, site_id,
    );

    PageLockService::get_locks_for_page(ctx, site_id, page_ref.borrow())
        .await
        .or_raise(|| Error::new("failed to fetch page lock history", ErrorType::PageLock))
}
