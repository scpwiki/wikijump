/*
 * services/job/worker.rs
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

//! Module for the worker which consumes `Job`s and performs the relevant task.

use super::prelude::*;
use crate::runtime::ServerState;
use crate::services::page_revision::RerenderType;
use crate::services::{
    BlobService, PageRevisionService, SessionService, TextService, UserService,
};
use crate::types::PageId;
use crate::utils::debug_pointer;
use rsmq_async::{Rsmq, RsmqConnection, RsmqMessage};
use sea_orm::TransactionTrait;
use std::convert::Infallible;
use std::fmt::{self, Debug};
use std::sync::Arc;
use std::time::Duration;
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
    Next { job: Job, delay: Option<Duration> },
    Done,
}

#[derive(Clone)]
pub struct JobWorker {
    state: ServerState,
    rsmq: Rsmq,
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
        let rsmq = Rsmq::clone(&state.rsmq);
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

        macro_rules! config {
            ($field:ident $(,)?) => {
                self.state.config.$field
            };
        }

        let mut empty_queue_delay = config!(job_min_poll_delay);
        loop {
            let result = self.process_job().await;
            let duration = match result {
                Ok(JobProcessStatus::NoJob) => {
                    trace!("No job for us to process, sleeping a while");

                    // Exponential backoff, double wait up to the cap
                    if empty_queue_delay < config!(job_max_poll_delay) {
                        empty_queue_delay *= 2;
                    }

                    empty_queue_delay
                }
                Ok(JobProcessStatus::ReceivedJob) => {
                    trace!(
                        "Job processing finished, sleeping a bit to avoid overloading the database"
                    );
                    empty_queue_delay = config!(job_min_poll_delay);
                    config!(job_work_delay)
                }
                Err(error) => {
                    error!("Error while processing job: {error}");
                    empty_queue_delay = config!(job_min_poll_delay);
                    config!(job_work_delay)
                }
            };

            time::sleep(duration).await;
        }
    }

    async fn process_job(&mut self) -> Result<JobProcessStatus> {
        let make_error =
            || Error::new("failed to process job from queue", ErrorType::Job);

        let next_job = self
            .rsmq
            .receive_message(JOB_QUEUE_NAME, None)
            .await
            .or_raise(make_error)?;

        let data: RsmqMessage<Vec<u8>> = match next_job {
            None => return Ok(JobProcessStatus::NoJob),
            Some(data) => data,
        };

        debug!("Received raw data from queue (worker {})", self.id);
        debug!("* Message ID:          {}", data.id);
        debug!("* Previously received: {}", data.rc);
        debug!("* Created:             {}", data.sent);
        debug!("* Received:            {}", data.fr);
        let no_more_retries =
            is_final_attempt(data.rc, self.state.config.job_max_attempts);
        let job = match serde_json::from_slice(&data.message) {
            Ok(job) => job,
            Err(error) => {
                if no_more_retries {
                    self.rsmq
                        .delete_message(JOB_QUEUE_NAME, &data.id)
                        .await
                        .or_raise(make_error)?;
                }
                return Err(error).or_raise(make_error);
            }
        };

        let make_error = || {
            Error::new(
                format!("failed to process job ID {}: {:#?}", data.id, job),
                ErrorType::Job,
            )
        };

        let execution: Result<NextJob> = async {
        debug!("Received job from queue: {job:?}");
        trace!("Setting up ServiceContext for job processing");
        let txn = self.state.database.begin().await.or_raise(make_error)?;
        let ctx = &ServiceContext::new(&self.state, &txn);

        trace!("Beginning job processing");
        let next = match job {
            Job::RerenderPage {
                id:
                    PageId {
                        site_id,
                        category_id,
                        page_id,
                    },
                depth,
                r#type: rerender_type,
            } => {
                let extra = match rerender_type {
                    RerenderType::Full => "normal",
                    RerenderType::NavigationOnly => "nav only",
                };

                debug!(
                    "Rerendering page ID {} in site ID {} (category ID {}, depth {}) ({})",
                    page_id, site_id, category_id, depth, extra,
                );

                PageRevisionService::rerender(
                    ctx,
                    PageId {
                        site_id,
                        category_id,
                        page_id,
                    },
                    depth,
                    rerender_type,
                )
                .await
                .or_raise(make_error)?;

                NextJob::Done
            }
            Job::PruneSessions => {
                debug!("Pruning all expired sesions from database");
                SessionService::prune(ctx).await.or_raise(make_error)?;
                NextJob::Next {
                    job: Job::PruneSessions,
                    delay: Some(self.state.config.job_prune_session),
                }
            }
            Job::PrunePendingUploads => {
                debug!("Pruning all expired pending uploads from database and S3");
                BlobService::prune(ctx).await.or_raise(make_error)?;
                NextJob::Next {
                    job: Job::PrunePendingUploads,
                    delay: Some(self.state.config.job_prune_uploads),
                }
            }
            Job::PruneText => {
                debug!("Pruning all unused text items from database");
                TextService::prune(ctx).await.or_raise(make_error)?;
                NextJob::Next {
                    job: Job::PruneText,
                    delay: Some(self.state.config.job_prune_text),
                }
            }
            Job::NameChangeRefill => {
                debug!("Checking users for those who can get a name change token refill");

                UserService::refresh_name_change_tokens(ctx)
                    .await
                    .or_raise(make_error)?;

                NextJob::Next {
                    job: Job::NameChangeRefill,
                    delay: Some(self.state.config.job_name_change_refill),
                }
            }
            Job::LiftExpiredPunishments => {
                debug!("Checking if any outstanding punishments have expired");
                // TODO implement tempban removal
                //
                //      We aren't going to be able to create jobs that have a wait time of say,
                //      2 years, so instead we will just have this job run daily and check
                //      to see if any bans have expired
                //
                //      currently only bans are the temporary, but others can be added here
                NextJob::Next {
                    job: Job::LiftExpiredPunishments,
                    delay: Some(self.state.config.job_lift_expired_punishments),
                }
            }
        };

        let post_commit_actions = ctx.drain_post_commit_actions().or_raise(make_error)?;

        trace!("Committing transaction, returning success");
        txn.commit().await.or_raise(make_error)?;
        if let Err(error) = ServiceContext::run_post_commit_actions_for_state(
            &self.state,
            post_commit_actions,
        )
        .await
        {
            error!("job committed but post-commit actions failed: {}", error);
        }
        Ok(next)
        }
        .await;

        let next = match execution {
            Ok(next) => next,
            Err(error) if no_more_retries => {
                debug!("Final job attempt failed; deleting the exhausted message");
                self.rsmq
                    .delete_message(JOB_QUEUE_NAME, &data.id)
                    .await
                    .or_raise(make_error)?;
                return Err(error);
            }
            Err(error) => return Err(error),
        };

        match next {
            NextJob::Done => debug!("Job execution finished, no follow-up job to add"),
            NextJob::Next { job, delay } => {
                debug!("Job execution finished, follow-up job has been produced");
                JobService::queue_job_inner(&mut self.rsmq, &job, delay)
                    .await
                    .or_raise(make_error)?;
            }
        }

        trace!("Job execution finished, deleting message");
        self.rsmq
            .delete_message(JOB_QUEUE_NAME, &data.id)
            .await
            .or_raise(make_error)?;

        Ok(JobProcessStatus::ReceivedJob)
    }
}

fn is_final_attempt(receive_count: u64, max_attempts: u16) -> bool {
    receive_count >= u64::from(max_attempts)
}

impl Debug for JobWorker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("JobWorker")
            .field("state", &self.state)
            .field("rsmq", &debug_pointer(&self.rsmq))
            .field("id", &self.id)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_attempt_policy_keeps_retryable_failures() {
        assert!(!is_final_attempt(1, 3));
        assert!(!is_final_attempt(2, 3));
        assert!(is_final_attempt(3, 3));
        assert!(is_final_attempt(4, 3));
    }

    #[test]
    fn malformed_jobs_are_detected_before_execution() {
        assert!(serde_json::from_slice::<Job>(b"not-json").is_err());
        assert!(serde_json::from_slice::<Job>(br#"{"unknown-job":true}"#).is_err());
    }
}
