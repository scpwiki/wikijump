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
    // TODO

    // Seed site data
    // TODO

        // Seed page data
        // TODO

    txn.commit().await?;
    Ok(())
}
