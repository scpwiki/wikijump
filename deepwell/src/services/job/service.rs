/*
 * services/job/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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
use rsmq_async::RsmqConnection;
use std::time::Duration;

pub const JOB_QUEUE_NAME: &str = "job";

/// How long messages, after being delivered, cannot be delivered to another consumer.
///
/// This feature is a part of job queues to prevent a job from being run twice by
/// two different consumers. Because a job which fails won't report back to the queue,
/// so you cannot tell the difference between a job which is in progress vs a job that failed.
///
/// The way we resolve this is setting a time limit for "process time", which is the designated
/// period a job is allowed to run. If a job takes longer than that, then we assume it failed or
/// died. This risks a false positive of still-running jobs, but as long as this time is well
/// above what a job should take to run this risk is minimal.
pub const JOB_QUEUE_PROCESS_TIME: Option<Duration> = Some(Duration::from_secs(30));

/// How long to wait before messages are delivered to consumers.
pub const JOB_QUEUE_DELAY: Option<Duration> = None;

/// The maximum size, in bytes, that a job payload is allowed to be
///
/// Presently, our jobs are mostly unit types, and the biggest variant
/// is composed of three integers, so this is more than large enough.
/// If larger jobs become a thing in the future, this may need to be updated.
///
/// (But as a general code principle there shouldn't be huge jobs, they should
/// just use IDs and references to items in the database.)
pub const JOB_QUEUE_MAXIMUM_SIZE: Option<i32> = Some(1024);

#[derive(Debug)]
pub struct JobService;

impl JobService {
    pub async fn queue_job(
        ctx: &ServiceContext,
        job: &Job,
        delay: Option<Duration>,
    ) -> Result<()> {
        info!("Queuing job {job:?} (delay {delay:?})");
        let payload = serde_json::to_vec(job)?;
        ctx.rsmq()
            .send_message(JOB_QUEUE_NAME, payload, delay)
            .await?;

        Ok(())
    }

    /// Queues a page for being rerendered soon.
    ///
    /// # Arguments
    /// * `site_id` &mdash; The ID of the site the page is on.
    /// * `page_id` &mdash; The ID of the page.
    /// * `depth` &mdash; If rerendering a page causes more pages to be rerendered due to
    ///                   outdating, then this value should be incremented with each layer
    ///                   of job depth. This way we can avoid infinite loop conditions where
    ///                   jobs endlessly pile onto the queue, rerendering each other.
    pub async fn queue_rerender_page(
        ctx: &ServiceContext,
        site_id: i64,
        page_id: i64,
        depth: u32,
    ) -> Result<()> {
        debug!("Queuing page rerender for page ID {page_id} and site ID {site_id}");
        Self::queue_job(
            ctx,
            &Job::RerenderPage {
                site_id,
                page_id,
                depth,
            },
            None,
        )
        .await
    }
}
