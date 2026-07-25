//! Shared application state used by transport, services, and background work.

use crate::config::Config;
use crate::locales::Localizations;
use crate::services::blob::MimeAnalyzer;
use crate::utils::debug_pointer;
use redis::aio::MultiplexedConnection as RedisMultiplexedConnection;
use reqwest::Client as ReqwestClient;
use rsmq_async::Rsmq;
use s3::bucket::Bucket;
use sea_orm::DatabaseConnection;
use std::fmt::{self, Debug};
use std::sync::Arc;

pub type ServerState = Arc<ServerStateInner>;

pub struct ServerStateInner {
    pub config: Config,
    pub database: DatabaseConnection,
    pub redis: RedisMultiplexedConnection,
    pub rsmq: Rsmq,
    pub localizations: Localizations,
    pub mime_analyzer: MimeAnalyzer,
    pub s3_files_bucket: Box<Bucket>,
    pub s3_tblocks_bucket: Box<Bucket>,
    pub mailcheck_api_client: ReqwestClient,
}

impl Debug for ServerStateInner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ServerStateInner")
            .field("config", &self.config)
            .field("database", &self.database)
            .field("redis", &self.redis)
            .field("rsmq", &debug_pointer(&self.rsmq))
            .field("localizations", &self.localizations)
            .field("mime_analyzer", &self.mime_analyzer)
            .field("s3_files_bucket", &self.s3_files_bucket)
            .field("s3_tblocks_bucket", &self.s3_tblocks_bucket)
            .field("mailcheck_api_client", &self.mailcheck_api_client)
            .finish()
    }
}
