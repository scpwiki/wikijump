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
use crate::models::job::{self, Entity as Job, Model as JobModel};
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug)]
pub struct JobService;

impl JobService {
    // Job processing
    pub async fn get_batch(ctx: &ServiceContext<'_>) -> Result<Vec<JobModel>> {
        const BATCH_SIZE: u64 = 8;
        let txn = ctx.transaction();

        // Fetch next batch of jobs
        let jobs = Job::find()
            .order_by_asc(job::Column::JobId)
            .limit(8)
            .all(txn)
            .await?;

        // Mark as claimed
        for job in &jobs {
            Self::mark_claimed(ctx, job.job_id, true).await?;
        }

        Ok(jobs)
    }

    async fn mark_claimed(
        ctx: &ServiceContext<'_>,
        job_id: i32,
        value: bool,
    ) -> Result<()> {
        let txn = ctx.transaction();
        let job = job::ActiveModel {
            job_id: Set(job_id),
            is_claimed: Set(value),
            ..Default::default()
        };
        job.update(txn).await?;
        Ok(())
    }

    #[inline]
    pub async fn mark_retry(ctx: &ServiceContext<'_>, job_id: i32) -> Result<()> {
        Self::mark_claimed(ctx, job_id, true).await
    }

    pub async fn mark_complete(ctx: &ServiceContext<'_>, job: JobModel) -> Result<()> {
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
        page_ids: &[i64],
    ) -> Result<()> {
        let data = RerenderPagesJobData {
            page_ids: Cow::Borrowed(page_ids),
        };

        Self::enqueue(ctx, JOB_TYPE_RERENDER_PAGES, data).await
    }
}
