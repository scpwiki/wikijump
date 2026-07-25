/*
 * redis.rs
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

use crate::error::prelude::*;
use crate::services::job::{
    JOB_QUEUE_DELAY, JOB_QUEUE_MAXIMUM_SIZE, JOB_QUEUE_NAME, JOB_QUEUE_PROCESS_TIME, Job,
    JobService,
};
use redis::aio::MultiplexedConnection;
use rsmq_async::{Rsmq, RsmqConnection};

const RSMQ_NAMESPACE: &str = "rsmq";
const RSMQ_REALTIME: bool = false;

pub async fn connect(redis_uri: &str) -> Result<(MultiplexedConnection, Rsmq)> {
    let make_error = || Error::new("failed to connect to redis", ErrorType::RedisSetup);

    let client = redis::Client::open(redis_uri).or_raise(make_error)?;
    let connection = client
        .get_multiplexed_async_connection()
        .await
        .or_raise(make_error)?;

    let mut rsmq = {
        let connection2 = MultiplexedConnection::clone(&connection);
        Rsmq::new_with_connection(connection2, RSMQ_REALTIME, Some(RSMQ_NAMESPACE))
            .await
            .or_raise(make_error)?
    };

    // Set up queue if it doesn't already exist
    let queue_exists = job_queue_exists(&mut rsmq).await.or_raise(make_error)?;
    if !queue_exists {
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
        .await
        .or_raise(make_error)?;

        // Then add initial repeating jobs
        macro_rules! queue_job {
            ($job_case:ident) => {
                JobService::queue_job_inner(&mut rsmq, &Job::$job_case, None)
                    .await
                    .or_raise(make_error)?
            };
        }

        queue_job!(PruneSessions);
        queue_job!(PrunePendingUploads);
        queue_job!(PruneText);
        queue_job!(LiftExpiredPunishments);
    }

    Ok((connection, rsmq))
}

async fn job_queue_exists(rsmq: &mut Rsmq) -> Result<bool> {
    let make_error = || {
        Error::new(
            format!(
                "failed to determine if the job queue '{JOB_QUEUE_NAME}' exists in RSMQ",
            ),
            ErrorType::RedisSetup,
        )
    };

    // NOTE: Effectively the same as rsmq.list_queues().await?.contains(JOB_QUEUE_NAME),
    //       except we don't have to deal with the "&String" type issue.
    let queues = rsmq.list_queues().await.or_raise(make_error)?;
    let exists = queues.iter().any(|name| JOB_QUEUE_NAME == name);
    Ok(exists)
}
