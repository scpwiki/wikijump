/*
 * database/seeder/mod.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2022 Wikijump Team
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

mod data;

use self::data::SeedData;
use crate::api::ApiServerState;
use crate::services::ServiceContext;
use crate::services::page::PageService;
use crate::services::site::SiteService;
use crate::services::user::{UserService, CreateUser, CreateUserOutput};
use anyhow::Result;
use sea_orm::TransactionTrait;
use std::path::PathBuf;
use wikidot_normalize::normalize;

pub async fn seed(state: &ApiServerState) -> Result<()> {
    // Set up context
    let txn = state.database.begin().await?;
    let ctx = ServiceContext::from_raw(state, &txn);

    // Load seed data
    let SeedData { users, site_pages } = SeedData::load(&state.config.seeder_path)?;

    // Seed user data
    for user in users {
        // Hash password
        // TODO
        let password = user.password;

        // TODO Create with slug

        // TODO modify during user refactor
        let CreateUserOutput { user_id, slug } = UserService::create(&ctx, CreateUser {
            username: user.name,
            email: user.email,
            password,
            language: Some(user.locale),
        }).await?;

        assert_eq!(user_id, user.id, "Specified user ID doesn't match created");
    }

    // Seed site data
    for site_page in site_pages {
        // TODO create site

        for page in site_page.pages {
            // TODO create page
        }
    }

    txn.commit().await?;
    Ok(())
}
