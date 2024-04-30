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

use crate::services::job::{
    JOB_QUEUE_DELAY, JOB_QUEUE_MAXIMUM_SIZE, JOB_QUEUE_NAME, JOB_QUEUE_PROCESS_TIME,
};
use anyhow::Result;
use redis::aio::ConnectionManager;
use rsmq_async::{PooledRsmq, RsmqConnection};

pub async fn connect(redis_uri: &str) -> Result<(ConnectionManager, PooledRsmq)> {
    // Create regular redis client
    let client = redis::Client::open(redis_uri)?;
    let rsmq_connection = client.get_multiplexed_tokio_connection().await?;
    let redis = ConnectionManager::new(client).await?;

    // Create RSMQ client
    let mut rsmq = PooledRsmq::new_with_connection(rsmq_connection, true, None);

    // Set up queue if it doesn't already exist
    if !job_queue_exists(&mut rsmq).await? {
        info!("Creating Redis job queue '{JOB_QUEUE_NAME}'");
        info!("* Process time: {JOB_QUEUE_PROCESS_TIME:?}");
        info!("* Delay time:   {JOB_QUEUE_DELAY:?}");
        info!("* Maximum body: {JOB_QUEUE_MAXIMUM_SIZE:?} bytes");

        rsmq.create_queue(
            JOB_QUEUE_NAME,
            JOB_QUEUE_PROCESS_TIME,
            JOB_QUEUE_DELAY,
            JOB_QUEUE_MAXIMUM_SIZE,
        )
        .await?;
    }

    Ok((redis, rsmq))
}

async fn job_queue_exists(rsmq: &mut PooledRsmq) -> Result<bool> {
    // NOTE: Effectively the same as rsmq.list_queues().await?.contains(JOB_QUEUE_NAME),
    //       except we don't have to deal with the "&String" type issue.
    let queues = rsmq.list_queues().await?;
    let exists = queues.iter().any(|name| JOB_QUEUE_NAME == name);
    Ok(exists)
}
