/*
 * services/page_lock/service.rs
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

use std::net::IpAddr;

use super::structs::{CheckLockBypassOutput, CreatePageLockInput};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::page_lock::{self, Entity as PageLock, Model as PageLockModel};
use crate::services::ServiceContext;
use crate::services::audit::{AuditEvent, AuditService};
use crate::services::relation::GetPageAttributions;
use crate::services::{PageService, RelationService};
use crate::types::{Action, PageLockType, Permission, Reference, Resource};
use crate::utils::now;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, Set,
};

#[derive(Debug, Clone)]
pub struct PageLockService;

impl PageLockService {
    pub async fn create(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        user_id: i64,
        page_ref: Reference<'_>,
        input: CreatePageLockInput,
    ) -> Result<PageLockModel> {
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!("failed to create page lock for page {:?}", page_ref),
                ErrorType::PageLock,
            )
        };

        // Fetch the page to be locked
        let page_id = match page_ref {
            Reference::Id(page_id) => page_id,
            _ => {
                PageService::get(ctx, site_id, page_ref.borrow())
                    .await
                    .or_raise(make_error)?
                    .page_id
            }
        };

        // Check if any active lock exists for the page
        let existing_lock = Self::get_active_lock_for_page(ctx, page_id)
            .await
            .or_raise(make_error)?;

        if let Some(old_lock) = existing_lock {
            if !input.override_existing {
                bail!(Error::new(
                    format!(
                        "an active lock already exists for page {:?}, please remove it first.",
                        page_ref
                    ),
                    ErrorType::PageLockExists
                ));
            } else {
                // Soft delete the old lock
                page_lock::ActiveModel {
                    page_lock_id: Set(old_lock.page_lock_id),
                    deleted_at: Set(Some(now())),
                    updated_at: Set(Some(now())),
                    ..Default::default()
                }
                .update(txn)
                .await
                .or_raise(make_error)?;
            }
        }

        // Create the page lock
        let new_lock = page_lock::ActiveModel {
            page_id: Set(page_id),
            user_id: Set(user_id),
            lock_type: Set(input.lock_type),
            reason: Set(input.reason.unwrap_or_default()),
            expires_at: Set(input.expires_at),
            from_wikidot: Set(input.from_wikidot),
            created_at: Set(now()),
            deleted_at: Set(None),
            updated_at: Set(None),
            ..Default::default()
        }
        .insert(txn)
        .await
        .or_raise(make_error)?;

        AuditService::log(
            ctx,
            input.ip_address,
            AuditEvent::PageLockCreate {
                user_id,
                site_id,
                page_id,
                page_lock_id: new_lock.page_lock_id,
                lock_type: input.lock_type,
            },
        )
        .await
        .or_raise(make_error)?;

        Ok(new_lock)
    }

    pub async fn remove(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        user_id: i64,
        page_ref: Reference<'_>,
        ip_address: IpAddr,
    ) -> Result<Option<PageLockModel>> {
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!("failed to remove page lock for page {:?}", page_ref),
                ErrorType::PageLock,
            )
        };

        // Resolve page reference to ID
        let page_id = PageService::get_id(ctx, site_id, page_ref.borrow())
            .await
            .or_raise(make_error)?;

        // Fetch the active lock to be removed
        let maybe_lock = Self::get_active_lock_for_page(ctx, page_id)
            .await
            .or_raise(make_error)?;

        // If no lock, return
        let page_lock = match maybe_lock {
            Some(lock) => lock,
            None => return Ok(None),
        };

        // Mark the page lock as deleted
        let removed_lock = page_lock::ActiveModel {
            page_lock_id: Set(page_lock.page_lock_id),
            deleted_at: Set(Some(now())),
            updated_at: Set(Some(now())),
            ..Default::default()
        }
        .update(txn)
        .await
        .or_raise(make_error)?;

        AuditService::log(
            ctx,
            ip_address,
            AuditEvent::PageLockRemove {
                user_id,
                page_id: page_lock.page_id,
                page_lock_id: page_lock.page_lock_id,
                lock_type: page_lock.lock_type,
            },
        )
        .await
        .or_raise(make_error)?;

        Ok(Some(removed_lock))
    }

    pub async fn get_locks_for_page(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_ref: Reference<'_>,
    ) -> Result<Vec<PageLockModel>> {
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!("failed to fetch active lock for page {:?}", page_ref),
                ErrorType::PageLock,
            )
        };

        // Fetch the page to get its ID
        let page_id = PageService::get_id(ctx, site_id, page_ref.borrow())
            .await
            .or_raise(make_error)?;

        // Fetch all historical locks for the page, including expired and deleted ones
        let locks = PageLock::find()
            .filter(page_lock::Column::PageId.eq(page_id))
            .order_by_desc(page_lock::Column::CreatedAt)
            .all(txn)
            .await
            .or_raise(make_error)?;

        Ok(locks)
    }

    async fn get_active_lock_for_page(
        ctx: &ServiceContext<'_>,
        page_id: i64,
    ) -> Result<Option<PageLockModel>> {
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!("failed to fetch active lock for page ID {}", page_id),
                ErrorType::PageLock,
            )
        };

        // Fetch the active lock for the page
        let active_lock = PageLock::find()
            .filter(
                Condition::all()
                    .add(page_lock::Column::PageId.eq(page_id))
                    .add(page_lock::Column::DeletedAt.is_null())
                    .add(
                        Condition::any()
                            .add(page_lock::Column::ExpiresAt.gt(now()))
                            .add(page_lock::Column::ExpiresAt.is_null()),
                    ),
            )
            .one(txn)
            .await
            .or_raise(make_error)?;

        Ok(active_lock)
    }

    pub async fn can_user_bypass_lock(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        page_category_id: Option<i64>,
        user_id: i64,
    ) -> Result<CheckLockBypassOutput> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to check lock bypass for page ID {} and user ID {}",
                    page_id, user_id
                ),
                ErrorType::PageLock,
            )
        };

        // Check if any active lock exists for the page
        let active_lock = Self::get_active_lock_for_page(ctx, page_id)
            .await
            .or_raise(make_error)?;

        if let Some(lock) = active_lock {
            let can_bypass = match lock.lock_type {
                // Mod is not a native Wikijump role; treat it as a permission check instead
                PageLockType::PermissionOnly | PageLockType::Wikidot => ctx
                    .user_has_permission(Permission {
                        resource_type: Resource::Page,
                        resource_category: page_category_id.map(Reference::Id),
                        action: Action::BypassLock,
                    })
                    .await
                    .or_raise(make_error)?,
                PageLockType::AuthorOrPermissionOnly => {
                    // Check if the user is the author of the page
                    let attributions = RelationService::get_page_attributions(
                        ctx,
                        GetPageAttributions {
                            site_id,
                            page: page_id.into(),
                        },
                    )
                    .await
                    .or_raise(make_error)?;

                    // User can bypass if they are an author of this page or have bypass permission
                    let is_author =
                        attributions.iter().any(|attr| attr.user_id == user_id);
                    is_author
                        || ctx
                            .user_has_permission(Permission {
                                resource_type: Resource::Page,
                                resource_category: page_category_id.map(Reference::Id),
                                action: Action::BypassLock,
                            })
                            .await
                            .or_raise(make_error)?
                }
            };
            Ok(CheckLockBypassOutput {
                lock_present: true,
                can_edit: can_bypass,
            })
        } else {
            // No active lock, so no need to bypass
            Ok(CheckLockBypassOutput {
                lock_present: false,
                can_edit: true,
            })
        }
    }
}
