/*
 * services/import/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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

use super::prelude::*;
use crate::models::sea_orm_active_enums::UserType;
use crate::models::site::{self, Entity as Site};
use crate::models::user::{self, Entity as User};
use crate::services::BlobService;

#[derive(Debug)]
pub struct ImportService;

impl ImportService {
    pub async fn add_user(
        ctx: &ServiceContext<'_>,
        ImportUser {
            user_id,
            created_at,
            name,
            slug,
            email,
            locale,
            avatar,
            real_name,
            gender,
            birthday,
            location,
            biography,
            user_page,
        }: ImportUser,
    ) -> Result<()> {
        tide::log::info!("Importing user (name '{}', slug '{}')", name, slug);

        let txn = ctx.transaction();

        // Upload avatar to S3
        let avatar_s3_hash = match avatar {
            None => None,
            Some(bytes) => {
                let output = BlobService::create(ctx, &bytes).await?;
                Some(output.hash.to_vec())
            }
        };

        // Insert user row into table
        let user = user::ActiveModel {
            user_id: Set(user_id),
            user_type: Set(UserType::Regular),
            created_at: Set(created_at),
            from_wikidot: Set(true),
            name: Set(name),
            slug: Set(slug),
            email: Set(email),
            locale: Set(locale),
            avatar_s3_hash: Set(avatar_s3_hash),
            real_name: Set(real_name),
            gender: Set(gender),
            birthday: Set(birthday),
            location: Set(location),
            ..Default::default()
        };

        User::insert(user).exec(txn).await?;
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
        tide::log::info!(
            "Importing site (name '{}', slug '{}', locale '{}')",
            name,
            slug,
            locale,
        );

        let txn = ctx.transaction();
        let site = site::ActiveModel {
            site_id: Set(site_id),
            created_at: Set(created_at),
            name: Set(name),
            slug: Set(slug),
            locale: Set(locale),
            ..Default::default()
        };

        Site::insert(site).exec(txn).await?;
        Ok(())
    }
}
