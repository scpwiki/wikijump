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
use sqlx::{Database, Postgres};
use std::cell::{RefCell, RefMut};
use std::sync::Arc;

#[derive(Debug)]
pub struct ServiceContext<'conn> {
    state: ServerState,
    seaorm_transaction: sea_orm::DatabaseTransaction,
    sqlx_transaction: RefCell<sqlx::Transaction<'conn, Postgres>>,
}

impl<'conn> ServiceContext<'conn> {
    // NOTE: It is the responsibility of the caller to call commit or rollback
    //       on this object.
    pub fn new(
        state: &ServerState,
        seaorm_transaction: sea_orm::DatabaseTransaction,
        sqlx_transaction: sqlx::Transaction<'conn, Postgres>,
    ) -> Self {
        ServiceContext {
            state: Arc::clone(state),
            seaorm_transaction,
            sqlx_transaction: RefCell::new(sqlx_transaction),
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
    pub fn seaorm_transaction(&self) -> &sea_orm::DatabaseTransaction {
        &self.seaorm_transaction
    }

    #[inline]
    pub fn sqlx_transaction(&'conn self) -> RefMut<sqlx::Transaction<sqlx::Postgres>> {
        self.sqlx_transaction.borrow_mut()
    }

    pub async fn commit(self) -> Result<()> {
        self.seaorm_transaction.commit().await?;
        self.sqlx_transaction.into_inner().commit().await?;
        Ok(())
    }

    pub async fn rollback(self) -> Result<()> {
        self.seaorm_transaction.rollback().await?;
        self.sqlx_transaction.into_inner().rollback().await?;
        Ok(())
    }
}
