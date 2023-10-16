/*
 * api.rs
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

//! All routes for the API.
//!
//! This API is to be used internally only, and is subject to change in coordination with
//! Framerail (the API consumer). No guarantees are made as to backwards compatibility.
//!
//! This module should only contain definitions for the web server and its routes, and
//! not any of the implementations themselves. Those should be in the `methods` module.

use crate::config::{Config, Secrets};
use crate::database;
use crate::endpoints::{
    auth::*, category::*, domain::*, email::*, file::*, file_revision::*, link::*,
    locale::*, message::*, misc::*, page::*, page_revision::*, parent::*, site::*,
    site_member::*, text::*, user::*, user_bot::*, view::*, vote::*,
};
use crate::locales::Localizations;
use crate::services::blob::MimeAnalyzer;
use crate::services::job::JobQueue;
use crate::services::{into_rpc_error, Result as ServiceResult, ServiceContext};
use jsonrpsee::server::{RpcModule, Server, ServerHandle};
use jsonrpsee::types::{error::ErrorObjectOwned, params::Params};
use s3::bucket::Bucket;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;
use std::time::Duration;

// #[deprecated] XXX
pub type ApiRequest = tide::Request<ServerState>;
// #[deprecated] XXX
pub type ApiResponse = tide::Result;

pub type ServerState = Arc<ServerStateInner>;

#[derive(Debug)]
pub struct ServerStateInner {
    pub config: Config,
    pub database: DatabaseConnection,
    pub localizations: Localizations,
    pub mime_analyzer: MimeAnalyzer,
    pub job_queue: JobQueue,
    pub s3_bucket: Bucket,
}

pub async fn build_server_state(
    config: Config,
    secrets: Secrets,
) -> anyhow::Result<ServerState> {
    // Connect to database
    tide::log::info!("Connecting to PostgreSQL database");
    let database = database::connect(&secrets.database_url).await?;

    // Load localization data
    tide::log::info!("Loading localization data");
    let localizations = Localizations::open(&config.localization_path).await?;

    // Set up job queue
    let (job_queue, job_state_sender) = JobQueue::spawn(&config);

    // Load magic data and start MIME thread
    let mime_analyzer = MimeAnalyzer::spawn();

    // Create S3 bucket
    tide::log::info!("Opening S3 bucket");

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
        localizations,
        mime_analyzer,
        job_queue,
        s3_bucket,
    });

    // Start the job queue (requires ServerState)
    job_state_sender
        .send(Arc::clone(&state))
        .expect("Unable to send ServerState");

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
            module.register_async_method($name, |params, state| async move {
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

    async fn not_implemented(
        _ctx: &ServiceContext<'_>,
        _params: Params<'static>,
    ) -> ServiceResult<()> {
        tide::log::error!("Method not implemented yet!");
        todo!()
    }

    // Miscellaneous
    register!("ping", ping);
    register!("version", version);
    register!("version_full", full_version);
    register!("hostname", hostname);
    register!("config", config_dump);
    register!("config_path", config_path);
    register!("normalize", normalize_method);

    // Localization
    register!("locale", locale_info);
    register!("translate", translate_strings);

    // Web server
    register!("page_view", page_view);

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
    register!("page_edit", page_edit);
    register!("page_delete", page_delete);
    register!("page_move", page_move);
    register!("page_rerender", page_rerender);
    register!("page_restore", page_restore);

    // Page revisions
    register!("page_revision_create", not_implemented);
    register!("page_revision_get", page_revision_get);
    register!("page_revision_count", not_implemented);
    register!("page_revision_rollback", not_implemented);
    register!("page_revision_range", not_implemented);

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

    // Files
    register!("file_upload", file_upload);
    register!("file_get", file_get);
    register!("file_edit", file_edit);
    register!("file_delete", file_delete);
    register!("file_move", file_move);
    register!("file_restore", file_restore);
    register!("file_hard_delete", file_hard_delete);

    // File revisions
    register!("file_revision_create", not_implemented);
    register!("file_revision_get", not_implemented);
    register!("file_revision_count", not_implemented);
    register!("file_revision_range", not_implemented);

    // Text
    register!("text_create", text_create);
    register!("text_get", text_get);

    // User
    register!("user_create", not_implemented);
    register!("user_get", not_implemented);
    register!("user_edit", not_implemented);
    register!("user_delete", not_implemented);
    register!("user_import", not_implemented);
    register!("user_add_name_change", not_implemented);
    register!("user_avatar_set", not_implemented);

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

/* *** */

pub fn tide_build_server(state: ServerState) -> tide::Server<ServerState> {
    macro_rules! new {
        () => {
            tide::Server::with_state(Arc::clone(&state))
        };
    }

    // Create server and add routes
    //
    // Prefix is present to avoid ambiguity about what this
    // API is meant to be and the fact that it's not to be publicly-facing.
    let mut app = new!();
    app.at("/api/trusted").nest(tide_build_routes(new!()));
    app
}

fn tide_build_routes(mut app: tide::Server<ServerState>) -> tide::Server<ServerState> {
    // File revisions
    app.at("/file/revision").put(file_revision_put);
    app.at("/file/revision/get").put(file_revision_retrieve);
    app.at("/file/revision/count").put(file_revision_count);
    app.at("/file/revision/range/:direction")
        .put(file_revision_range_retrieve);

    // User
    app.at("/user").put(user_put).delete(user_delete);
    app.at("/user/get").put(user_retrieve);
    app.at("/user/avatar").put(user_avatar_put);
    app.at("/user/create").post(user_create);
    app.at("/user/import").post(user_import);
    app.at("/user/addNameChange").post(user_add_name_change);

    app
}
