/*
 * services/context.rs
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

use crate::api::ServerState;
use crate::config::Config;
use crate::locales::Localizations;
use crate::services::blob::MimeAnalyzer;
use crate::services::error::Result;
use redis::aio::MultiplexedConnection as RedisMultiplexedConnection;
use rsmq_async::PooledRsmq;
use s3::bucket::Bucket;
use sea_orm::DatabaseTransaction;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ServiceContext<'txn> {
    state: ServerState,
    transaction: &'txn DatabaseTransaction,
}

impl<'txn> ServiceContext<'txn> {
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
    pub fn transaction(&self) -> &'txn DatabaseTransaction {
        self.transaction
    }

    #[inline]
    pub async fn make_sqlx_transaction(
        &self,
    ) -> Result<sqlx::Transaction<sqlx::Postgres>> {
        let txn = self.state.database_sqlx.begin().await?;
        Ok(txn)
    }
}
