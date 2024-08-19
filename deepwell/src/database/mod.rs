/*
 * database/mod.rs
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

mod seeder;

pub use self::seeder::seed;

pub type SqlxTransaction<'txn> = sqlx::Transaction<'txn, Postgres>;

use anyhow::Result;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sqlx::{Pool, Postgres};
use std::time::Duration;

pub async fn connect<S: Into<String>>(
    database_uri: S,
) -> Result<(Pool<Postgres>, DatabaseConnection)> {
    let database_uri = database_uri.into();
    let sqlx_db = Pool::<Postgres>::connect(&database_uri).await?;

    let mut options = ConnectOptions::new(database_uri);
    options
        .min_connections(4)
        .max_connections(100)
        .connect_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(10))
        .sqlx_logging(true);

    let sea_orm_db = Database::connect(options).await?;
    Ok((sqlx_db, sea_orm_db))
}

pub async fn migrate(database_uri: &str) -> Result<()> {
    let pool = Pool::<Postgres>::connect(database_uri).await?;

    info!("Running migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(())
}
