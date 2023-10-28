/*
 * services/job/worker.rs
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

//! Module for the worker which consumes `Job`s and performs the relevant task.

use super::prelude::*;
use crate::api::ServerState;
use crate::services::{PageRevisionService, SessionService, TextService};
use rsmq_async::{MultiplexedRsmq, RsmqConnection, RsmqMessage};
use sea_orm::TransactionTrait;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::time;

/// Tells the main loop of the worker whether the queue had an item or not.
#[derive(Debug)]
enum JobProcessStatus {
    ReceivedJob,
    NoJob,
}

/// Used to queue a follow-up job, if needed.
#[derive(Debug)]
enum NextJob {
    Next { job: Job, delay: Option<u64> },
    Done,
}

#[derive(Debug, Clone)]
pub struct JobWorker {
    state: ServerState,
    rsmq: MultiplexedRsmq,
    id: u16,
}

impl JobWorker {
    /// Spawns a number of local job workers.
    /// The number of workers is specified in the configuration.
    pub fn spawn_all(state: &ServerState) {
        let worker_count = state.config.job_workers.into();

        info!("Spawning {worker_count} local job workers");
        for id in 0..worker_count {
            Self::spawn_one(state, id);
        }
    }

    /// Spawns one local job worker with the given ID.
    ///
    /// Each worker within a process should have a unique ID,
    /// but this will not cause breakages if this is violated.
    fn spawn_one(state: &ServerState, id: u16) {
        info!("Spawning job worker ID {id}");
        let state = Arc::clone(state);
        let rsmq = MultiplexedRsmq::clone(&state.rsmq);
        let worker = JobWorker { state, rsmq, id };
        tokio::spawn(worker.main_loop());
    }

    /// The main execution loop for a job worker.
    ///
    /// This will listen to the queue, picking up new items as they arrive,
    /// and sleeping for a random duration when there are no jobs available.
    /// (This way we do not hammer the queue when all the workers wake up
    /// at once.)
    ///
    /// # Clearing job items
    /// RSMQ (and other queue systems) attempt to maintain job durability,
    /// such that if a worker takes a job and then crashes, the job is later
    /// retried. This is behavior we wish to have as well, but it means that
    /// our job queue flow is a bit different than what may seem obvious.
    ///
    /// The `rsmq_async` crate provides two sets of methods for retrieving jobs:
    /// 1. `pop_message()`
    ///
    /// This removes a job from the queue and yields it to a worker. If the worker
    /// then dies, the job is no longer on the queue and thus is effectively lost.
    ///
    /// 2. `receive_message()` followed by `delete_message()`
    ///
    /// This receives a job from the queue, during which time the job can no longer
    /// be picked up by any other consumers. This prevents work being done twice.
    ///
    /// After the job execution period, if no further updates are received, then the
    /// queue assumes the worker died or failed, and then the job is later available
    /// on the queue for workers to retry.
    ///
    /// This means that, after a `receive_message()`, if and only if the job was
    /// successfully run (aside from any cases where we specifically decide we do
    /// not want this job to re-run), we will then run `delete_message()` so that
    /// it is no longer enqueued.
    async fn main_loop(mut self) -> Infallible {
        trace!("Beginning main execution of worker ID {}", self.id);

        loop {
            let result = self.process_job().await;
            let duration = match result {
                Ok(JobProcessStatus::NoJob) => {
                    trace!("No job for us to process, sleeping a while");
                    todo!("long delay"); // XXX
                }
                Ok(JobProcessStatus::ReceivedJob) => {
                    trace!("Job processing finished, sleeping a bit to avoid overloading the database");
                    todo!("job delay"); // XXX
                }
                Err(error) => {
                    error!("Error while processing job: {error}");
                    todo!("job delay"); // XXX
                }
            };

            time::sleep(duration).await;
        }
    }

    async fn process_job(&mut self) -> Result<JobProcessStatus> {
        let data: RsmqMessage<Vec<u8>> =
            match self.rsmq.receive_message(JOB_QUEUE_NAME, None).await? {
                None => return Ok(JobProcessStatus::NoJob),
                Some(data) => data,
            };

        debug!("Received raw data from queue");
        debug!("* Message ID:          {}", data.id);
        debug!("* Previously received: {}", data.rc);
        debug!("* Created:             {}", data.sent);
        debug!("* Received:            {}", data.fr);
        let job = serde_json::from_slice(&data.message)?;

        debug!("Received job from queue: {job:?}");
        trace!("Setting up ServiceContext for job processing");
        let txn = self.state.database.begin().await?;
        let ctx = &ServiceContext::new(&self.state, &txn);

        trace!("Beginning job processing");
        let next = match job {
            Job::RerenderPage { site_id, page_id } => {
                debug!("Rerendering page ID {page_id} in site ID {site_id}");
                PageRevisionService::rerender(ctx, site_id, page_id).await?;
                NextJob::Done
            }
            Job::PruneSessions => {
                debug!("Pruning all expired sesions from database");
                SessionService::prune(ctx).await?;
                NextJob::Next {
                    job: Job::PruneSessions,
                    delay: Some(self.state.config.job_prune_session_secs),
                }
            }
            Job::PruneText => {
                debug!("Pruning all unused text items from database");
                TextService::prune(ctx).await?;
                NextJob::Next {
                    job: Job::PruneText,
                    delay: Some(self.state.config.job_prune_text_secs),
                }
            }
            Job::NameChangeRefill => {
                debug!("Checking users for those who can get a name change token refill");
                // TODO implement name change refill
                //
                //      check users whose time since refill_name_change_days
                //      refill_name_change_days being zero means disable
                //      add user credits to each where they are above that time
                NextJob::Next {
                    job: Job::NameChangeRefill,
                    delay: Some(self.state.config.job_name_change_refill_secs),
                }
            }
        };

        // XXX process 'next', queue job

        trace!("Job execution finished, cleaning up");
        self.rsmq.delete_message(JOB_QUEUE_NAME, &data.id).await?;
        txn.commit().await?;
        Ok(JobProcessStatus::ReceivedJob)
    }
}
