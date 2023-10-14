/*
 * services/context.rs
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

use crate::api::{ApiRequest, ServerState};
use crate::config::Config;
use crate::locales::Localizations;
use crate::services::blob::MimeAnalyzer;
use crate::services::job::JobQueue;
use s3::bucket::Bucket;
use sea_orm::DatabaseTransaction;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ServiceContext<'txn> {
    state: ServerState,
    transaction: &'txn DatabaseTransaction,
}

impl<'txn> ServiceContext<'txn> {
    pub fn from_req(req: &ApiRequest, transaction: &'txn DatabaseTransaction) -> Self {
        Self::new(req.state(), transaction)
    }

    // NOTE: It is the responsibility of the caller to manage commit / rollback
    //       for transactions.
    //
    //       For our endpoints, this is managed in the wrapper macro in api.rs
    pub fn new(state: &ServerState, transaction: &'txn DatabaseTransaction) -> Self {
        ServiceContext {
            state: Arc::clone(state),
            transaction,
        }
    }

    // Getters
    #[inline]
    pub fn config(&self) -> &Config {
        &self.state.config
    }

    #[inline]
    pub fn localization(&self) -> &Localizations {
        &self.state.localizations
    }

    #[inline]
    pub fn mime(&self) -> &MimeAnalyzer {
        &self.state.mime_analyzer
    }

    #[inline]
    pub fn job_queue(&self) -> &JobQueue {
        &self.state.job_queue
    }

    #[inline]
    pub fn s3_bucket(&self) -> &Bucket {
        &self.state.s3_bucket
    }

    #[inline]
    pub fn transaction(&self) -> &'txn DatabaseTransaction {
        self.transaction
    }
}
