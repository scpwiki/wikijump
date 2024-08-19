/*
 * services/context/mod.rs
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

mod transaction;

pub use self::transaction::ServiceTransaction;

use crate::api::ServerState;
use crate::config::Config;
use crate::database::SqlxTransaction;
use crate::locales::Localizations;
use crate::services::blob::MimeAnalyzer;
use crate::services::error::Result;
use redis::aio::MultiplexedConnection as RedisMultiplexedConnection;
use rsmq_async::PooledRsmq;
use s3::bucket::Bucket;
use sqlx::{Database, Postgres};
use std::cell::{RefCell, RefMut};
use std::mem;
use std::sync::Arc;

#[derive(Debug)]
pub struct ServiceContext {
    state: ServerState,
    transaction: Arc<ServiceTransaction>,
}

impl ServiceContext {
    // NOTE: It is the responsibility of the caller to run commit / rollback
    //       for transactions.
    //
    //       For our endpoints, this is managed in the wrapper macro in api.rs
    #[inline]
    pub fn new(state: &ServerState) -> Self {
        ServiceContext {
            state: Arc::clone(state),
            transaction: Arc::new(ServiceTransaction::new()),
        }
    }

    // Getters
    #[inline]
    pub fn config(&self) -> &Config {
        &self.state.config
    }

    #[inline]
    pub fn redis_client(&self) -> &redis::Client {
        &self.state.redis
    }

    pub async fn redis_connect(&self) -> Result<RedisMultiplexedConnection> {
        let conn = self
            .redis_client()
            .get_multiplexed_tokio_connection()
            .await?;

        Ok(conn)
    }

    #[inline]
    pub fn rsmq(&self) -> PooledRsmq {
        PooledRsmq::clone(&self.state.rsmq)
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
    pub fn s3_bucket(&self) -> &Bucket {
        &self.state.s3_bucket
    }

    #[inline]
    // #[deprecated] XXX
    pub fn seaorm_transaction(&self) -> &sea_orm::DatabaseTransaction {
        // Need to remove
        todo!()
    }

    #[inline]
    pub fn sqlx_transaction(&self) -> Arc<ServiceTransaction> {
        Arc::clone(&self.transaction)
    }

    #[inline]
    pub async fn commit(&self) -> Result<()> {
        self.transaction.commit().await
    }

    #[inline]
    pub async fn rollback(&self) -> Result<()> {
        self.transaction.rollback().await
    }
}
