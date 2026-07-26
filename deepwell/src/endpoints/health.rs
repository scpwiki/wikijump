/*
 * endpoints/health.rs
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
use sea_orm::{ConnectionTrait, raw_sql};

async fn postgres_check(ctx: &ServiceContext<'_>) -> Result<()> {
    ctx.transaction()
        .query_one_raw(raw_sql!(Postgres, "SELECT 1"))
        .await
        .or_raise(|| Error::new("failed to ping PostgreSQL", ErrorType::DatabaseQuery))?;

    debug!("Successfully pinged Postgres");
    Ok(())
}

async fn redis_check(ctx: &ServiceContext<'_>) -> Result<()> {
    let mut redis = ctx.redis();

    redis
        .send_packed_command(redis::Cmd::new().arg("PING"))
        .await
        .or_raise(|| Error::new("failed to ping Redis", ErrorType::RedisQuery))?;

    debug!("Successfully pinged Redis");
    Ok(())
}

pub async fn ping(
    ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<&'static str> {
    // Ensure the database and cache are connected, and only then return.
    info!("Ping request");

    try_join!(postgres_check(ctx), redis_check(ctx))
        .or_raise(|| Error::new("ping failed", ErrorType::HealthCheck))?;

    Ok("Pong!")
}
