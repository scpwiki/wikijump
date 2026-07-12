/*
 * database/mod.rs
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

mod seeder;

pub use self::seeder::seed;

use crate::error::prelude::*;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;

pub async fn connect<S: Into<String>>(
    database_uri: S,
    sqlx_logging: bool,
) -> Result<DatabaseConnection> {
    let make_error =
        || Error::new("failed to connect to database", ErrorType::DatabaseSetup);

    let database_uri = database_uri.into();
    let mut options = ConnectOptions::new(database_uri);
    options
        .min_connections(4)
        .max_connections(100)
        .connect_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(10))
        .sqlx_logging(sqlx_logging);

    let sea_orm_db = Database::connect(options).await.or_raise(make_error)?;
    Ok(sea_orm_db)
}
