/*
 * cache.rs
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

use crate::services::Result;
use rsmq_async::Rsmq;

/// Creates primary `redis::Client` instance.
///
/// Used to spawn off other connections to Redis as needed.
pub async fn connect_redis(redis_uri: &str) -> Result<redis::Client> {
    let client = redis::Client::open(redis_uri)?;
    Ok(client)
}

/// Creates an RSMQ instance for temporary use.
///
/// A connection is fetched from the `Client`, which is then used to create
/// an owned `Rsmq` available for use by the caller.
pub async fn connect_rsmq(client: &redis::Client) -> Result<Rsmq> {
    let connection = client.get_tokio_connection().await?;
    Ok(Rsmq::new_with_connection(connection, true, None))
}
