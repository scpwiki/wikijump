/*
 * redis.rs
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

use anyhow::Result;
use redis::aio::ConnectionManager;
use rsmq_async::Rsmq;

pub async fn connect(redis_uri: &str) -> Result<(ConnectionManager, Rsmq)> {
    // Create regular redis client
    let client = redis::Client::open(redis_uri)?;
    let rsmq_connection = client.get_async_connection().await?;
    let redis = ConnectionManager::new(client).await?;

    // Create RSMQ client
    let rsmq = Rsmq::new_with_connection(rsmq_connection, true, None);

    Ok((redis, rsmq))
}
