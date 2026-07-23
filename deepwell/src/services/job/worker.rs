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

trait JobQueue {
    async fn receive_job(&mut self) -> Result<Option<RsmqMessage<Vec<u8>>>>;
    async fn delete_job(&mut self, message_id: &str) -> Result<()>;
    async fn queue_follow_up(&mut self, job: &Job, delay: Option<Duration>)
    -> Result<()>;
}

impl JobQueue for Rsmq {
    async fn receive_job(&mut self) -> Result<Option<RsmqMessage<Vec<u8>>>> {
        self.receive_message(JOB_QUEUE_NAME, None)
            .await
            .or_raise(|| Error::new("failed to receive queued job", ErrorType::Job))
    }

    async fn delete_job(&mut self, message_id: &str) -> Result<()> {
        self.delete_message(JOB_QUEUE_NAME, message_id)
            .await
            .or_raise(|| Error::new("failed to delete queued job", ErrorType::Job))?;
        Ok(())
    }

    async fn queue_follow_up(
        &mut self,
        job: &Job,
        delay: Option<Duration>,
    ) -> Result<()> {
        JobService::queue_job_inner(self, job, delay).await
    }
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

        let next_job = self.rsmq.receive_job().await.or_raise(make_error)?;

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
        let job = decode_job_message(&mut self.rsmq, &data, no_more_retries)
            .await
            .or_raise(make_error)?;

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

        settle_job_execution(&mut self.rsmq, &data.id, no_more_retries, execution)
            .await
            .or_raise(make_error)?;

        Ok(JobProcessStatus::ReceivedJob)
    }
}

async fn decode_job_message<Q: JobQueue>(
    queue: &mut Q,
    data: &RsmqMessage<Vec<u8>>,
    no_more_retries: bool,
) -> Result<Job> {
    match serde_json::from_slice(&data.message) {
        Ok(job) => Ok(job),
        Err(error) => {
            if no_more_retries {
                queue.delete_job(&data.id).await?;
            }
            Err(error).or_raise(|| {
                Error::new(
                    format!("queued job {} contains malformed JSON", data.id),
                    ErrorType::Job,
                )
            })
        }
    }
}

async fn settle_job_execution<Q: JobQueue>(
    queue: &mut Q,
    message_id: &str,
    no_more_retries: bool,
    execution: Result<NextJob>,
) -> Result<()> {
    let next = match execution {
        Ok(next) => next,
        Err(error) if no_more_retries => {
            debug!("Final job attempt failed; deleting the exhausted message");
            queue.delete_job(message_id).await?;
            return Err(error);
        }
        Err(error) => return Err(error),
    };

    match next {
        NextJob::Done => debug!("Job execution finished, no follow-up job to add"),
        NextJob::Next { job, delay } => {
            debug!("Job execution finished, follow-up job has been produced");
            queue.queue_follow_up(&job, delay).await?;
        }
    }

    trace!("Job execution finished, deleting message");
    queue.delete_job(message_id).await
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
    use uuid::Uuid;

    #[derive(Debug, Default)]
    struct MockJobQueue {
        received: Option<RsmqMessage<Vec<u8>>>,
        deleted: Vec<String>,
        follow_ups: Vec<(String, Option<Duration>)>,
    }

    impl JobQueue for MockJobQueue {
        async fn receive_job(&mut self) -> Result<Option<RsmqMessage<Vec<u8>>>> {
            Ok(self.received.take())
        }

        async fn delete_job(&mut self, message_id: &str) -> Result<()> {
            self.deleted.push(message_id.to_owned());
            Ok(())
        }

        async fn queue_follow_up(
            &mut self,
            job: &Job,
            delay: Option<Duration>,
        ) -> Result<()> {
            self.follow_ups.push((format!("{job:?}"), delay));
            Ok(())
        }
    }

    fn message(payload: &[u8], receive_count: u64) -> RsmqMessage<Vec<u8>> {
        RsmqMessage {
            id: "message-id".to_owned(),
            message: payload.to_vec(),
            rc: receive_count,
            fr: 0,
            sent: 0,
        }
    }

    fn execution_error() -> Error {
        Error::new("job execution failed", ErrorType::Job)
    }

    #[test]
    fn final_attempt_policy_keeps_retryable_failures() {
        assert!(!is_final_attempt(1, 3));
        assert!(!is_final_attempt(2, 3));
        assert!(is_final_attempt(3, 3));
        assert!(is_final_attempt(4, 3));
    }

    #[tokio::test]
    async fn empty_queue_has_no_message_to_execute() {
        let mut queue = MockJobQueue::default();
        assert!(queue.receive_job().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn malformed_retryable_job_is_retained() {
        let mut queue = MockJobQueue::default();
        let error = decode_job_message(&mut queue, &message(b"not-json", 1), false)
            .await
            .unwrap_err();
        assert_eq!(error.error_type, ErrorType::Job);
        assert!(queue.deleted.is_empty());
    }

    #[tokio::test]
    async fn malformed_final_job_is_deleted() {
        let mut queue = MockJobQueue::default();
        decode_job_message(&mut queue, &message(br#"{"unknown-job":true}"#, 3), true)
            .await
            .unwrap_err();
        assert_eq!(queue.deleted, ["message-id"]);
    }

    #[tokio::test]
    async fn successful_job_is_deleted() {
        let mut queue = MockJobQueue::default();
        settle_job_execution(&mut queue, "message-id", false, Ok(NextJob::Done))
            .await
            .unwrap();
        assert_eq!(queue.deleted, ["message-id"]);
        assert!(queue.follow_ups.is_empty());
    }

    #[tokio::test]
    async fn retryable_failure_is_retained() {
        let mut queue = MockJobQueue::default();
        settle_job_execution(
            &mut queue,
            "message-id",
            false,
            Err(execution_error().into()),
        )
        .await
        .unwrap_err();
        assert!(queue.deleted.is_empty());
    }

    #[tokio::test]
    async fn final_failure_is_deleted() {
        let mut queue = MockJobQueue::default();
        settle_job_execution(
            &mut queue,
            "message-id",
            true,
            Err(execution_error().into()),
        )
        .await
        .unwrap_err();
        assert_eq!(queue.deleted, ["message-id"]);
    }

    #[tokio::test]
    async fn delayed_follow_up_is_queued_before_acknowledgement() {
        let mut queue = MockJobQueue::default();
        let delay = Duration::from_secs(30);
        settle_job_execution(
            &mut queue,
            "message-id",
            false,
            Ok(NextJob::Next {
                job: Job::PruneSessions,
                delay: Some(delay),
            }),
        )
        .await
        .unwrap();
        assert_eq!(
            queue.follow_ups,
            [("PruneSessions".to_owned(), Some(delay))]
        );
        assert_eq!(queue.deleted, ["message-id"]);
    }

    #[tokio::test]
    #[ignore = "requires an isolated Redis instance in REDIS_URL"]
    async fn real_redis_adapter_round_trip() {
        let redis_url =
            std::env::var("REDIS_URL").expect("REDIS_URL is required for this test");
        let client = redis::Client::open(redis_url).expect("Redis URL should be valid");
        let connection = client
            .get_multiplexed_async_connection()
            .await
            .expect("Redis should accept a connection");
        let namespace = format!("deepwell-job-adapter-test-{}", Uuid::new_v4());
        let mut queue = Rsmq::new_with_connection(connection, false, Some(&namespace))
            .await
            .expect("RSMQ adapter should initialize");
        queue
            .create_queue(
                JOB_QUEUE_NAME,
                JOB_QUEUE_PROCESS_TIME,
                JOB_QUEUE_DELAY,
                JOB_QUEUE_MAXIMUM_SIZE,
            )
            .await
            .expect("test queue should be created");

        queue
            .queue_follow_up(&Job::PruneSessions, Some(Duration::ZERO))
            .await
            .expect("adapter should enqueue a job");
        let received = queue
            .receive_job()
            .await
            .expect("adapter should receive a job")
            .expect("queued job should be present");
        let decoded: Job =
            serde_json::from_slice(&received.message).expect("job payload should decode");
        queue
            .delete_job(&received.id)
            .await
            .expect("adapter should acknowledge a job");
        let empty_after_delete = queue
            .receive_job()
            .await
            .expect("adapter should query the emptied queue")
            .is_none();
        queue
            .delete_queue(JOB_QUEUE_NAME)
            .await
            .expect("test queue should be deleted");

        assert!(matches!(decoded, Job::PruneSessions));
        assert!(empty_after_delete);
    }
}
