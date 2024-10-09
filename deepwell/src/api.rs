/*
 * api.rs
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

//! All routes for the API.
//!
//! This API is to be used internally only, and is subject to change in coordination with
//! Framerail (the API consumer). No guarantees are made as to backwards compatibility.
//!
//! This module should only contain definitions for the web server and its routes, and
//! not any of the implementations themselves. Those should be in the `methods` module.

use crate::config::{Config, Secrets};
use crate::endpoints::{
    auth::*, blob::*, category::*, domain::*, email::*, file::*, file_revision::*,
    info::*, link::*, locale::*, message::*, misc::*, page::*, page_revision::*,
    parent::*, site::*, site_member::*, text::*, user::*, user_bot::*, view::*, vote::*,
};
use crate::locales::Localizations;
use crate::services::blob::MimeAnalyzer;
use crate::services::job::JobWorker;
use crate::services::{into_rpc_error, ServiceContext};
use crate::utils::debug_pointer;
use crate::{database, redis as redis_db};
use jsonrpsee::server::{RpcModule, Server, ServerHandle};
use jsonrpsee::types::error::ErrorObjectOwned;
use rsmq_async::PooledRsmq;
use s3::bucket::Bucket;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::fmt::{self, Debug};
use std::sync::Arc;
use std::time::Duration;

pub type ServerState = Arc<ServerStateInner>;

pub struct ServerStateInner {
    pub config: Config,
    pub database: DatabaseConnection,
    pub redis: redis::Client,
    pub rsmq: PooledRsmq,
    pub localizations: Localizations,
    pub mime_analyzer: MimeAnalyzer,
    pub s3_bucket: Bucket,
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
            .field("s3_bucket", &self.s3_bucket)
            .finish()
    }
}

pub async fn build_server_state(
    config: Config,
    secrets: Secrets,
) -> anyhow::Result<ServerState> {
    // Connect to databases
    info!("Connecting to PostgreSQL database");
    let database = database::connect(&secrets.database_url).await?;

    info!("Connecting to Redis");
    let (redis, rsmq) = redis_db::connect(&secrets.redis_url).await?;

    // Load localization data
    info!("Loading localization data");
    let localizations = Localizations::open(&config.localization_path).await?;

    // Load magic data and start MIME thread
    let mime_analyzer = MimeAnalyzer::spawn();

    // Create S3 bucket
    info!("Opening S3 bucket");

    let s3_bucket = {
        let mut bucket = Bucket::new(
            &secrets.s3_bucket,
            secrets.s3_region.clone(),
            secrets.s3_credentials.clone(),
        )?;

        if secrets.s3_path_style {
            bucket = bucket.with_path_style();
        }

        bucket.request_timeout = Some(Duration::from_millis(500));
        bucket
    };

    // Build server state
    let state = Arc::new(ServerStateInner {
        config,
        database,
        redis,
        rsmq,
        localizations,
        mime_analyzer,
        s3_bucket,
    });

    // Start workers listening to the job queue (requires ServerState)
    JobWorker::spawn_all(&state);

    // Return server state
    Ok(state)
}

pub async fn build_server(app_state: ServerState) -> anyhow::Result<ServerHandle> {
    let socket_address = app_state.config.address;
    let server = Server::builder().build(socket_address).await?;
    let module = build_module(app_state).await?;
    let handle = server.start(module);
    Ok(handle)
}

async fn build_module(app_state: ServerState) -> anyhow::Result<RpcModule<ServerState>> {
    let mut module = RpcModule::new(app_state);

    macro_rules! register {
        ($name:expr, $method:ident $(,)?) => {{
            // Register async method.
            //
            // Contains a wrapper around each to set up state, convert error types,
            // and produce a transaction used in ServiceContext, passed in.
            module.register_async_method($name, |params, state, _extensions| async move {
                // NOTE: We have our own Arc because we need to share it in some places
                //       before setting up, but RpcModule insists on adding its own.
                //       So we need to "unwrap it" before each method invocation.
                //       Oh well.
                let state = Arc::clone(&*state);

                // Wrap each call in a transaction, which commits or rolls back
                // automatically based on whether the Result is Ok or Err.
                //
                // At this level, we take the database-or-RPC error and make it just an RPC error.
                let db_state = Arc::clone(&state);
                db_state
                    .database
                    .transaction(move |txn| {
                        Box::pin(async move {
                            // Run the endpoint's implementation, and convert from
                            // ServiceError to an RPC error.
                            let ctx = ServiceContext::new(&state, &txn);
                            $method(&ctx, params).await.map_err(ErrorObjectOwned::from)
                        })
                    })
                    .await
                    .map_err(into_rpc_error)
            })?;
        }};
    }

    // Miscellaneous
    register!("ping", ping);
    register!("echo", echo);
    register!("error", yield_error);
    register!("config", config_dump);
    register!("normalize", normalize_method);

    // Server Information
    register!("info", server_info);

    // Localization
    register!("locale", locale_info);
    register!("translate", translate_strings);

    // Web server
    register!("page_view", page_view);
    register!("user_view", user_view);
    register!("admin_view", admin_view);

    // Authentication
    register!("login", auth_login);
    register!("logout", auth_logout);
    register!("session_get", auth_session_get);
    register!("session_get_others", auth_session_get_others);
    register!("session_invalidate_others", auth_session_invalidate_others);
    register!("session_renew", auth_session_renew);
    register!("mfa_verify", auth_mfa_verify);
    register!("mfa_setup", auth_mfa_setup);
    register!("mfa_disable", auth_mfa_disable);
    register!("mfa_reset_recovery", auth_mfa_reset_recovery);

    // Site
    register!("site_create", site_create);
    register!("site_get", site_get);
    register!("site_update", site_update);
    register!("site_from_domain", site_get_from_domain);

    // Site custom domain
    register!("custom_domain_create", site_custom_domain_create);
    register!("custom_domain_get", site_custom_domain_get);
    register!("custom_domain_delete", site_custom_domain_delete);

    // Site membership
    register!("member_set", membership_set);
    register!("member_get", membership_get);
    register!("member_delete", membership_delete);

    // Category
    register!("category_get", category_get);
    register!("category_get_all", category_get_all);

    // Page
    register!("page_create", page_create);
    register!("page_get", page_get);
    register!("page_get_direct", page_get_direct);
    register!("page_get_deleted", page_get_deleted);
    register!("page_get_score", page_get_score);
    register!("page_edit", page_edit);
    register!("page_delete", page_delete);
    register!("page_move", page_move);
    register!("page_rollback", page_rollback);
    register!("page_rerender", page_rerender);
    register!("page_restore", page_restore);
    register!("page_set_layout", page_set_layout);

    // Page revisions
    register!("page_revision_create", page_revision_edit);
    register!("page_revision_get", page_revision_get);
    register!("page_revision_count", page_revision_count);
    register!("page_revision_range", page_revision_range);

    // Page links
    register!("page_get_links_from", page_links_from_get);
    register!("page_get_links_to", page_links_to_get);
    register!("page_get_links_to_missing", page_links_to_missing_get);
    register!("page_get_urls_from", page_links_external_from);
    register!("page_get_urls_to", page_links_external_to);

    // Page parents
    register!("parent_set", parent_set);
    register!("parent_get", parent_get);
    register!("parent_remove", parent_remove);
    register!("parent_relationships_get", parent_relationships_get);
    register!("parent_get_all", parent_get_all);
    register!("parent_update", parent_update);

    // Blob data
    register!("blob_get", blob_get);
    register!("blob_upload", blob_upload);
    register!("blob_cancel", blob_cancel);

    // Files
    register!("file_create", file_create);
    register!("file_edit", file_edit);
    register!("file_get", file_get);
    register!("file_delete", file_delete);
    register!("file_move", file_move);
    register!("file_restore", file_restore);
    register!("file_rollback", file_rollback);
    register!("file_hard_delete", file_hard_delete);

    // File revisions
    register!("file_revision_get", file_revision_get);
    register!("file_revision_edit", file_revision_edit);
    register!("file_revision_count", file_revision_count);
    register!("file_revision_range", file_revision_range);

    // Text
    register!("text_create", text_create);
    register!("text_get", text_get);

    // User
    register!("user_create", user_create);
    register!("user_import", user_import);
    register!("user_get", user_get);
    register!("user_edit", user_edit);
    register!("user_delete", user_delete);
    register!("user_add_name_change", user_add_name_change);

    // Bot user
    register!("bot_user_create", bot_user_create);
    register!("bot_user_get", bot_user_get);
    register!("bot_user_owner_set", bot_user_owner_set);
    register!("bot_user_owner_remove", bot_user_owner_remove);

    // Direct messages
    register!("message_draft_create", message_draft_create);
    register!("message_draft_edit", message_draft_edit);
    register!("message_draft_delete", message_draft_delete);
    register!("message_draft_send", message_draft_send);

    // Email
    register!("email_validate", validate_email);

    // Votes
    register!("vote_set", vote_set);
    register!("vote_get", vote_get);
    register!("vote_remove", vote_remove);
    register!("vote_action", vote_action);
    register!("vote_list", vote_list_get);
    register!("vote_list_count", vote_list_count);

    // Return
    Ok(module)
}
