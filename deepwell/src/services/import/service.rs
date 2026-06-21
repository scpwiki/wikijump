/*
 * services/import/service.rs
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

//! Importer service, for ingesting data from Wikidot.
//!
//! This does not perform checks such as name / slug correspodnence,
//! uniqueness (this will get blocked by the database probably),
//! inconsistency, or perform filter validation.
//!
//! It is for limited use during initial setup only.

// TODO implement and use this service
#![allow(dead_code)]

use super::prelude::*;
use crate::constants::SYSTEM_USER_ID;
use crate::models::page::{self, Entity as Page};
use crate::models::page_category::Model as PageCategoryModel;
use crate::models::site::{self, Entity as Site};
use crate::models::wikidot_user::{self, Entity as WikidotUser};
use crate::services::page_lock::{CreatePageLockInput, PageLockService};
use crate::services::{BlobService, CategoryService};
use crate::types::PageLockType;
use crate::utils::get_category_name;

#[derive(Debug)]
pub struct ImportService;

impl ImportService {
    pub async fn add_user(
        ctx: &ServiceContext<'_>,
        ImportUser {
            user_id,
            created_at,
            fetched_at,
            wikidot_user_type,
            real_name,
            gender,
            birthday,
            location,
            biography,
            website,
            karma,
            is_pro,
        }: ImportUser,
    ) -> Result<()> {
        info!(
            "Importing Wikidot user (user ID {}, created {}, karma {})",
            user_id,
            created_at,
            karma.value(),
        );

        let make_error = || {
            Error::new(
                format!("failed to import wikidot user (user ID {user_id})"),
                ErrorType::DatabaseImport,
            )
        };

        let (is_deleted, name, slug) = match wikidot_user_type {
            ImportedUserType::Extant { name, slug } => (false, Some(name), Some(slug)),
            ImportedUserType::Deleted => (true, None, None),
        };

        let txn = ctx.transaction();
        let model = wikidot_user::ActiveModel {
            user_id: Set(user_id),
            created_at: Set(created_at),
            fetched_at: Set(fetched_at),
            is_deleted: Set(is_deleted),
            name: Set(name),
            slug: Set(slug),
            real_name: Set(real_name),
            gender: Set(gender),
            birthday: Set(birthday),
            location: Set(location),
            biography: Set(biography),
            website: Set(website),
            karma: Set(i16::from(karma.value())),
            is_pro: Set(is_pro),
        };

        WikidotUser::insert(model)
            .exec(txn)
            .await
            .or_raise(make_error)?;

        Ok(())
    }

    pub async fn add_site(
        ctx: &ServiceContext<'_>,
        ImportSite {
            site_id,
            created_at,
            name,
            slug,
            locale,
        }: ImportSite,
    ) -> Result<()> {
        info!("Importing site (name '{name}', slug '{slug}', locale '{locale}')");

        let make_error = || {
            Error::new(
                format!(
                    "failed to import site (name '{}', slug '{}', ID {})",
                    name, slug, site_id,
                ),
                ErrorType::DatabaseImport,
            )
        };

        let txn = ctx.transaction();
        let site = site::ActiveModel {
            site_id: Set(site_id),
            created_at: Set(created_at),
            from_wikidot: Set(true),
            name: Set(name.clone()),
            slug: Set(slug.clone()),
            locale: Set(locale),
            ..Default::default()
        };

        Site::insert(site).exec(txn).await.or_raise(make_error)?;
        Ok(())
    }

    pub async fn add_page(
        ctx: &ServiceContext<'_>,
        ImportPage {
            page_id,
            site_id,
            created_at,
            slug,
            locked,
            discussion_thread_id,
            ip_address,
        }: ImportPage,
    ) -> Result<()> {
        info!("Creating page '{slug}' in site ID {site_id}");

        let make_error = || {
            Error::new(
                format!(
                    "failed to import page (slug '{}', ID {} in site ID {})",
                    slug, page_id, site_id,
                ),
                ErrorType::DatabaseImport,
            )
        };

        let txn = ctx.transaction();

        // Create category if not already present
        let PageCategoryModel { category_id, .. } =
            CategoryService::get_or_create(ctx, site_id, get_category_name(&slug))
                .await
                .or_raise(make_error)?;

        // Insert page row into table
        let page = page::ActiveModel {
            page_id: Set(page_id),
            site_id: Set(site_id),
            created_at: Set(created_at),
            from_wikidot: Set(true),
            slug: Set(slug.clone()),
            page_category_id: Set(category_id),
            discussion_thread_id: Set(discussion_thread_id),
            ..Default::default()
        };
        Page::insert(page).exec(txn).await.or_raise(make_error)?;

        // If locked, add that too
        if locked {
            PageLockService::create(
                ctx,
                site_id,
                SYSTEM_USER_ID,
                Reference::Id(page_id),
                CreatePageLockInput {
                    from_wikidot: true,
                    lock_type: PageLockType::Wikidot,
                    reason: None,
                    expires_at: None,
                    override_existing: false,
                    ip_address,
                },
            )
            .await
            .or_raise(make_error)?;
        }

        Ok(())
    }

    // TODO page_revision
    // TODO page_vote

    // TODO file
    // TODO forum
}
