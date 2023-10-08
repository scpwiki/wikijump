/*
 * services/job/service.rs
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

use super::prelude::*;
use crate::api::ApiServerState;
use crate::services::{PageRevisionService, SessionService, TextService};
use sea_orm::TransactionTrait;
use tokio::sync::{mpsc, oneshot};
use tokio::{task, time};

type RequestSender = mpsc::UnboundedSender<Job>;
type RequestReceiver = mpsc::UnboundedReceiver<Job>;

type StateSender = oneshot::Sender<ApiServerState>;
type StateReceiver = oneshot::Receiver<ApiServerState>;

#[derive(Debug)]
pub struct JobService;

impl JobService {
    pub fn queue_rerender_page(queue: &JobQueue, site_id: i64, page_id: i64) {
        tide::log::debug!(
            "Queueing page ID {page_id} in site ID {site_id} for rerendering",
        );
        queue
            .sink
            .send(Job::RerenderPageId { site_id, page_id })
            .expect("Job channel is closed");
    }

    pub fn queue_prune_sessions(queue: &JobQueue) {
        tide::log::debug!("Queueing sessions list for pruning");
        queue
            .sink
            .send(Job::PruneSessions)
            .expect("Job channel is closed");
    }

    pub fn queue_prune_text(queue: &JobQueue) {
        tide::log::debug!("Queueing unused text for pruning");
        queue
            .sink
            .send(Job::PruneText)
            .expect("Job channel is closed");
    }
}

#[derive(Debug, Clone)]
pub struct JobQueue {
    sink: RequestSender,
}

impl JobQueue {
    pub fn spawn(config: &Config) -> (Self, StateSender) {
        // Create channels
        let (sink, source) = mpsc::unbounded_channel();
        let (state_sender, state_getter) = oneshot::channel();
        let job_queue = JobQueue { sink };

        // Copy fields for ancillary tasks
        let session_prune_delay = config.job_prune_session_period;
        let text_prune_delay = config.job_prune_text_period;
        let job_queue_1 = job_queue.clone();
        let job_queue_2 = job_queue.clone();

        // Main runner
        task::spawn(Self::main_loop(state_getter, source));

        // Ancillary tasks
        task::spawn(async move {
            loop {
                tide::log::trace!("Running repeat job: prune expired sessions");
                JobService::queue_prune_sessions(&job_queue_1);
                time::sleep(session_prune_delay).await;
            }
        });

        task::spawn(async move {
            loop {
                tide::log::trace!("Running repeat job: prune unused text rows");
                JobService::queue_prune_text(&job_queue_2);
                time::sleep(text_prune_delay).await;
            }
        });

        // TODO job that checks hourly for users who can get a name change token refill
        //      see config.refill_name_change

        (job_queue, state_sender)
    }

    async fn main_loop(state_getter: StateReceiver, mut source: RequestReceiver) {
        tide::log::info!("Waiting for server state (to start job runner)");
        let state = state_getter.await.expect("Unable to get server state");
        let delay = state.config.job_delay;

        tide::log::info!("Starting job runner");
        while let Some(job) = source.recv().await {
            tide::log::debug!("Received job from queue: {job:?}");
            match Self::process_job(&state, job).await {
                Ok(()) => tide::log::debug!("Finished processing job"),
                Err(error) => tide::log::warn!("Error processing job: {error}"),
            }

            tide::log::trace!("Sleeping a bit to avoid overloading the database");
            time::sleep(delay).await;
        }
    }

    async fn process_job(state: &ApiServerState, job: Job) -> Result<()> {
        let txn = state.database.begin().await?;
        let ctx = &ServiceContext::from_raw(state, &txn);

        match job {
            Job::RerenderPageId { site_id, page_id } => {
                PageRevisionService::rerender(ctx, site_id, page_id).await?;
            }
            Job::PruneSessions => {
                SessionService::prune(ctx).await?;
            }
            Job::PruneText => {
                TextService::prune(ctx).await?;
            }
        }

        txn.commit().await?;
        Ok(())
    }
}
