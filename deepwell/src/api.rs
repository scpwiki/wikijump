/*
 * api.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
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
    auth::*, basic_error::*, blob::*, category::*, domain::*, email::*, file::*,
    file_revision::*, health::*, info::*, link::*, locale::*, message::*, misc::*,
    page::*, page_attribution::*, page_revision::*, parent::*, routing::*, site::*,
    site_member::*, text::*, text_block::*, user::*, user_bot::*, view::*, vote::*,
};
use crate::error::prelude::*;
use crate::locales::Localizations;
use crate::services::ServiceContext;
use crate::services::blob::MimeAnalyzer;
use crate::services::job::JobWorker;
use crate::utils::debug_pointer;
use crate::{database, info, redis as redis_db};
use jsonrpsee::server::{RpcModule, Server, ServerHandle};
use redis::aio::MultiplexedConnection as RedisMultiplexedConnection;
use reqwest::Client as ReqwestClient;
use rsmq_async::Rsmq;
use s3::bucket::Bucket;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::fmt::{self, Debug};
use std::sync::Arc;
use std::time::Duration;

const BUCKET_REQUEST_TIMEOUT: Duration = Duration::from_millis(500);

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

pub async fn build_server_state(
    config: Config,
    Secrets {
        database_url,
        redis_url,
        s3_files_bucket,
        s3_tblocks_bucket,
        s3_region,
        s3_path_style,
        s3_credentials,
        mailcheck_api_key,
    }: Secrets,
) -> Result<ServerState> {
    let make_error =
        || Error::new("failed to build server state", ErrorType::ServerSetup);

    // Connect to databases
    info!("Connecting to PostgreSQL database");
    let database = database::connect(&database_url)
        .await
        .or_raise(make_error)?;

    info!("Connecting to Redis");
    let (redis, rsmq) = redis_db::connect(&redis_url).await.or_raise(make_error)?;

    // Load localization data
    info!("Loading localization data");
    let localizations = Localizations::open(&config.localization_path)
        .await
        .or_raise(make_error)?;

    // Load magic data and start MIME thread
    let mime_analyzer = MimeAnalyzer::spawn();

    // Create S3 bucket
    info!("Opening S3 bucket");

    let (s3_files_bucket, s3_tblocks_bucket) = {
        let mut files_bucket =
            Bucket::new(&s3_files_bucket, s3_region.clone(), s3_credentials.clone())
                .or_raise(make_error)?;

        let mut tblocks_bucket = Bucket::new(
            &s3_tblocks_bucket,
            s3_region.clone(),
            s3_credentials.clone(),
        )
        .or_raise(make_error)?;

        if s3_path_style {
            files_bucket = files_bucket.with_path_style();
            tblocks_bucket = tblocks_bucket.with_path_style();
        }

        files_bucket.request_timeout = Some(BUCKET_REQUEST_TIMEOUT);
        tblocks_bucket.request_timeout = Some(BUCKET_REQUEST_TIMEOUT);
        (files_bucket, tblocks_bucket)
    };

    // Set up reqwest client for the MailCheck API
    let mailcheck_api_client = {
        use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
        use reqwest::redirect::Policy as RedirectPolicy;

        let mut builder = ReqwestClient::builder();

        if let Some(mut api_token) = mailcheck_api_key {
            // Authorization: Bearer {token}
            api_token.insert_str(0, "Bearer ");
            let value = HeaderValue::from_str(&api_token).or_raise(make_error)?;

            let mut headers = HeaderMap::default();
            headers.insert(AUTHORIZATION, value);
            builder = builder.default_headers(headers);
        }

        builder
            .user_agent(&*info::VERSION)
            .redirect(RedirectPolicy::none())
            .timeout(Duration::from_secs(2))
            .build()
            .or_raise(make_error)?
    };

    // Build server state
    let state = Arc::new(ServerStateInner {
        config,
        database,
        redis,
        rsmq,
        localizations,
        mime_analyzer,
        s3_files_bucket,
        s3_tblocks_bucket,
        mailcheck_api_client,
    });

    // Start workers listening to the job queue (requires ServerState)
    JobWorker::spawn_all(&state);

    // Return server state
    Ok(state)
}

pub async fn build_server(app_state: ServerState) -> Result<ServerHandle> {
    let make_error = || Error::new("failed to build server", ErrorType::ServerSetup);
    let socket_address = app_state.config.address;
    let server = Server::builder()
        .build(socket_address)
        .await
        .or_raise(make_error)?;

    let module = build_module(app_state).await.or_raise(make_error)?;
    let handle = server.start(module);
    Ok(handle)
}

async fn build_module(app_state: ServerState) -> Result<RpcModule<ServerState>> {
    use crate::error::{exn_error_to_rpc_error, unwrap_transaction_error};

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
                            // the crate's error type to an RPC error.
                            let ctx = ServiceContext::new(&state, &txn);
                            $method(&ctx, params)
                                .await
                                .or_raise(|| Error::new(
                                    format!("method '{}' failed", $name),
                                    ErrorType::Request,
                                ))
                        })
                    })
                    .await
                    .map_err(unwrap_transaction_error)
                    .inspect_err(|error| error!("JSONRPC method {} failed: {}", $name, error))
                    .map_err(exn_error_to_rpc_error)
            })
            .or_raise(|| Error::new(
                format!("failed to register JSONRPC method '{}'", $name),
                ErrorType::ServerSetup,
            ))?;
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

    // Web routing
    register!("domains", platform_domains);
    register!("caddyfile", generate_caddyfile);

    // Web server
    register!("page_view", page_view);
    register!("user_view", user_view);
    register!("admin_view", admin_view);

    // Basic errors
    register!(
        "basic_error_missing_site_slug",
        basic_error_missing_site_slug,
    );
    register!(
        "basic_error_missing_custom_domain",
        basic_error_missing_custom_domain,
    );
    register!(
        "basic_error_missing_page_slug",
        basic_error_missing_page_slug,
    );
    register!("basic_error_page_fetch", basic_error_page_fetch);
    register!(
        "basic_error_missing_file_name",
        basic_error_missing_file_name,
    );
    register!("basic_error_file_fetch", basic_error_file_fetch);
    register!("basic_error_text_block", basic_error_text_block);
    register!("basic_error_file_root", basic_error_file_root);

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
    register!("authorization_token_issue", auth_token_issue);

    // Site
    register!("site_create", site_create);
    register!("site_get", site_get);
    register!("site_update", site_update);
    register!("site_domain", site_get_domain);

    // Site custom domain
    register!("custom_domain_create", site_custom_domain_create);
    register!("custom_domain_remove", site_custom_domain_remove);
    register!("custom_domain_list", site_custom_domain_list);

    // Site membership
    register!("member_set", membership_set);
    register!("member_get", membership_get);
    register!("member_remove", membership_remove);

    // Category
    register!("category_get", category_get);
    register!("category_get_all", category_get_all);
    register!("category_get_all_active", category_get_all_active);

    // Page
    register!("page_create", page_create);
    register!("page_get", page_get);
    register!("page_get_direct", page_get_direct);
    register!("page_get_deleted", page_get_deleted);
    register!("page_get_score", page_get_score);
    register!("page_get_files", page_get_files);
    register!("page_edit", page_edit);
    register!("page_delete", page_delete);
    register!("page_move", page_move);
    register!("page_rollback", page_rollback);
    register!("page_rerender", page_rerender);
    register!("page_restore", page_restore);
    register!("page_set_layout", page_set_layout);

    // Page attributions
    register!("page_attribution_get_page", page_attribution_get_page);
    register!("page_attribution_update", page_attribution_update);
    register!("page_attribution_delete", page_attribution_delete);

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

    // Hosted text blocks
    register!("text_block_get_index", text_block_get_index);

    // Blob data
    register!("blob_get", blob_get);
    register!("blob_upload", blob_upload);
    register!("blob_cancel", blob_cancel);

    // Blob hard deletion
    register!("blob_hard_delete_preview", blob_hard_delete_preview);
    register!("blob_hard_delete_confirm", blob_hard_delete_confirm);

    // Blob blacklist
    register!("blob_blacklist_add", blob_blacklist_add);
    register!("blob_blacklist_remove", blob_blacklist_remove);
    register!("blob_blacklist_check", blob_blacklist_check);

    // Files
    register!("file_create", file_create);
    register!("file_edit", file_edit);
    register!("file_get", file_get);
    register!("file_delete", file_delete);
    register!("file_move", file_move);
    register!("file_restore", file_restore);
    register!("file_rollback", file_rollback);

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
    register!("bot_user_get_owners", bot_user_get_owners); // gets all owners of a bot
    register!("bot_user_get_bots", bot_user_get_bots); // gets all bots owned by a user
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
