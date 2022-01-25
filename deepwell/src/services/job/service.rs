/*
 * services/job/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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

//! The job service, for enqueuing changes to be performed asynchronously.
//!
//! This is current structure is temporary, until we have some kind of persistent
//! queue we can reference instead of this.

use super::prelude::*;
use crate::api::ApiServerState;
use crate::models::job::{self, Entity as Job, Model as JobModel};
use crate::services::{RevisionService, TextService};
use async_std::task;
use sea_orm::DatabaseTransaction;
use serde::Serialize;
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;
use void::Void;

#[derive(Debug)]
pub struct JobService;

impl JobService {
    // Job worker
    pub fn launch_worker(state: &ApiServerState) {
        let state = Arc::clone(state);
        task::spawn(async move { JobWorker::main_loop(state).await });
    }

    // Job processing
    async fn get_job(txn: &DatabaseTransaction) -> Result<Option<JobModel>> {
        let job = Job::find()
            .filter(job::Column::IsClaimed.eq(false))
            .order_by_asc(job::Column::JobId)
            .one(txn)
            .await?;

        if let Some(ref job) = job {
            // Mark as claimed
            Self::mark_claimed(txn, job.job_id, true).await?;
        }

        Ok(job)
    }

    async fn mark_claimed(
        txn: &DatabaseTransaction,
        job_id: i32,
        value: bool,
    ) -> Result<()> {
        let job = job::ActiveModel {
            job_id: Set(job_id),
            is_claimed: Set(value),
            ..Default::default()
        };
        job.update(txn).await?;
        Ok(())
    }

    async fn mark_complete(ctx: &ServiceContext<'_>, job: JobModel) -> Result<()> {
        let txn = ctx.transaction();
        job.delete(txn).await?;
        Ok(())
    }

    // Enqueueing
    async fn enqueue<T: Serialize>(
        ctx: &ServiceContext<'_>,
        job_type: &'static str,
        data: T,
    ) -> Result<()> {
        let txn = ctx.transaction();
        let json_data = serde_json::to_value(data)?;
        let job = job::ActiveModel {
            job_type: Set(str!(job_type)),
            job_data: Set(json_data),
            ..Default::default()
        };

        job.insert(txn).await?;
        Ok(())
    }

    pub async fn enqueue_rerender_pages(
        ctx: &ServiceContext<'_>,
        ids: Vec<RerenderPage>,
    ) -> Result<()> {
        let data = RerenderPagesJobData { ids };
        Self::enqueue(ctx, JOB_TYPE_RERENDER_PAGES, data).await
    }
}

#[derive(Debug)]
struct JobWorker(ApiServerState);

impl JobWorker {
    // Main worker functions
    pub async fn main_loop(state: ApiServerState) -> Void {
        const JOB_DELAY: Duration = Duration::from_secs(10);
        tide::log::info!("Launching job worker");

        loop {
            let worker = JobWorker(Arc::clone(&state));
            task::spawn(async move {
                worker.process().await;
            });
            task::sleep(JOB_DELAY).await;
        }
    }

    async fn process(&self) {
        tide::log::info!("Processing job batch");

        match self.process_inner().await {
            Ok(_) => tide::log::debug!("Finished batch, sleeping for a bit"),
            Err(error) => tide::log::error!("Error processing batch: {}", error),
        }
    }

    async fn process_inner(&self) -> Result<()> {
        let txn = self.0.database.begin().await?;

        // Get next job to do
        if let Some(job) = JobService::get_job(&txn).await? {
            // Process based on job type
            match job.job_type.as_str() {
                JOB_TYPE_RERENDER_PAGES => {
                    let data = serde_json::from_value(job.job_data)?;
                    self.process_rerender_pages(&txn, data).await?;
                }
                _ => panic!("Invalid job type: {}", job.job_type),
            }
        }

        txn.commit().await?;
        Ok(())
    }

    // Job implementations
    async fn process_rerender_pages(
        &self,
        txn: &DatabaseTransaction,
        job: RerenderPagesJobData,
    ) -> Result<()> {
        let ctx = self.make_context(txn);

        for RerenderPage { site_id, page_id } in job.ids {
            tide::log::debug!(
                "Rerendering page: (site ID {}, page ID {})",
                site_id,
                page_id
            );
            RevisionService::rerender(&ctx, site_id, page_id).await?;
        }

        Ok(())
    }

    // Helpers
    #[inline]
    fn make_context<'txn>(
        &self,
        transaction: &'txn DatabaseTransaction,
    ) -> ServiceContext<'txn> {
        ServiceContext::from_raw(&self.0, transaction)
    }
}
