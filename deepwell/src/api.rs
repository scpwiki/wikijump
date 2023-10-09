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
    auth::*, category::*, email::*, file::*, file_revision::*, link::*, locale::*,
    message::*, misc::*, page::*, page_revision::*, parent::*, site::*, site_member::*,
    text::*, user::*, user_bot::*, view::*, vote::*,
};
use crate::locales::Localizations;
use crate::services::blob::MimeAnalyzer;
use crate::services::job::JobQueue;
use crate::services::Result as ServiceResult;
use crate::utils::error_response;
use jsonrpsee::server::{RpcModule, Server, ServerHandle};
use jsonrpsee::types::{error::ErrorObjectOwned, params::Params};
use s3::bucket::Bucket;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::time::Duration;
use tide::StatusCode;

#[deprecated]
pub type ApiRequest = tide::Request<ServerState>;
#[deprecated]
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
            module.register_async_method($name, |params, state| async move {
                // NOTE: We have our own Arc because we need to share it in some places
                //       before setting up, but RpcModule insists on adding its own.
                //       So we need to "unwrap it" before each method invocation.
                //       Oh well.
                let state = Arc::clone(&*state);
                $method(state, params).await.map_err(ErrorObjectOwned::from)
            })?;
        }};
    }

    async fn not_implemented(
        state: ServerState,
        params: Params<'static>,
    ) -> ServiceResult<()> {
        tide::log::error!("Method not implemented yet!");
        todo!()
    }

    // Miscellaneous
    register!("ping", ping);
    register!("version", not_implemented);
    register!("version_full", not_implemented);
    register!("hostname", not_implemented);
    register!("config", not_implemented);
    register!("config_path", not_implemented);
    register!("normalize", not_implemented);

    // Localization
    register!("locale", not_implemented);
    register!("translate", not_implemented);

    // Web server
    register!("page_view", not_implemented);

    // Authentication
    register!("login", not_implemented);
    register!("logout", not_implemented);
    register!("session_get", not_implemented);
    register!("session_get_others", not_implemented);
    register!("session_renew", not_implemented);
    register!("mfa_verify", not_implemented);
    register!("mfa_setup", not_implemented);
    register!("mfa_disable", not_implemented);
    register!("mfa_reset_recovery", not_implemented);

    // Site
    register!("site_create", not_implemented);
    register!("site_get", not_implemented);
    register!("site_update", not_implemented);
    register!("site_from_domain", not_implemented);

    // Site custom domain
    register!("custom_domain_create", not_implemented);
    register!("custom_domain_get", not_implemented);
    register!("custom_domain_delete", not_implemented);

    // Site membership
    register!("member_create", not_implemented);
    register!("member_get", not_implemented);
    register!("member_delete", not_implemented);

    // Category
    register!("category_get", not_implemented);
    register!("category_get_all", not_implemented);

    // Page
    register!("page_create", not_implemented);
    register!("page_get", not_implemented);
    register!("page_get_direct", not_implemented);
    register!("page_edit", not_implemented);
    register!("page_delete", not_implemented);
    register!("page_move", not_implemented);
    register!("page_rerender", not_implemented);
    register!("page_restore", not_implemented);

    // Page revisions
    register!("page_revision_create", not_implemented);
    register!("page_revision_get", not_implemented);
    register!("page_revision_count", not_implemented);
    register!("page_revision_rollback", not_implemented);
    register!("page_revision_range", not_implemented);

    // Page links
    register!("page_get_links_from", not_implemented);
    register!("page_get_links_to", not_implemented);
    register!("page_get_links_to_missing", not_implemented);
    register!("page_get_urls_from", not_implemented);
    register!("page_get_urls_to", not_implemented);

    // Page parents
    register!("parent_create", not_implemented);
    register!("parent_get", not_implemented);
    register!("parent_delete", not_implemented);
    register!("parent_relationship", not_implemented);

    // Files
    register!("file_create", not_implemented);
    register!("file_get", not_implemented);
    register!("file_edit", not_implemented);
    register!("file_delete", not_implemented);
    register!("file_move", not_implemented);
    register!("file_restore", not_implemented);

    // File revisions
    register!("file_revision_create", not_implemented);
    register!("file_revision_get", not_implemented);
    register!("file_revision_count", not_implemented);
    register!("file_revision_range", not_implemented);

    // Text
    register!("text_create", not_implemented);
    register!("text_get", not_implemented);

    // User
    register!("user_create", not_implemented);
    register!("user_get", not_implemented);
    register!("user_edit", not_implemented);
    register!("user_delete", not_implemented);
    register!("user_import", not_implemented);
    register!("user_add_name_change", not_implemented);
    register!("user_avatar_set", not_implemented);

    // Bot user
    register!("bot_user_create", not_implemented);
    register!("bot_user_get", not_implemented);
    register!("bot_user_owner_set", not_implemented);
    register!("bot_user_owner_delete", not_implemented);

    // Direct messages
    register!("message_draft_create", not_implemented);
    register!("message_draft_edit", not_implemented);
    register!("message_draft_delete", not_implemented);
    register!("message_draft_send", not_implemented);

    // Email
    register!("email_validate", not_implemented);

    // Votes
    register!("vote_set", not_implemented);
    register!("vote_get", not_implemented);
    register!("vote_delete", not_implemented);
    register!("vote_action", not_implemented);
    register!("vote_list", not_implemented);
    register!("vote_count", not_implemented);

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
    // Miscellaneous
    app.at("/version").get(version);
    app.at("/version/full").get(full_version);
    app.at("/hostname").get(hostname);
    app.at("/config").get(config_dump);
    app.at("/config/path").get(config_path);
    app.at("/normalize/:input").all(normalize_method);
    app.at("/teapot")
        .all(|_| async { error_response(StatusCode::ImATeapot, "🫖") });

    // Localization
    app.at("/locale/:locale").get(locale_get);
    app.at("/translate/:locale").put(translate_put);

    // Routes for web server
    app.at("/view/page").put(view_page);

    // Authentication
    app.at("/auth/login").post(auth_login);
    app.at("/auth/logout").delete(auth_logout);
    app.at("/auth/mfa").post(auth_mfa_verify); // Is part of the login process,
                                               // which is why it's up here.

    app.at("/auth/session/get").put(auth_session_retrieve);
    app.at("/auth/session/renew").post(auth_session_renew);
    app.at("/auth/session/others")
        .delete(auth_session_invalidate_others);
    app.at("/auth/session/others/get")
        .put(auth_session_retrieve_others);
    app.at("/auth/mfa/install")
        .post(auth_mfa_setup)
        .delete(auth_mfa_disable);
    app.at("/auth/mfa/resetRecovery")
        .post(auth_mfa_reset_recovery);

    // Site
    app.at("/site").put(site_put);
    app.at("/site/get").put(site_retrieve);
    app.at("/site/create").post(site_create);
    app.at("/site/domain/custom")
        .post(site_custom_domain_post)
        .delete(site_custom_domain_delete);
    app.at("/site/domain/custom/get")
        .put(site_custom_domain_retrieve);
    app.at("/site/fromDomain/:domain").get(site_get_from_domain);

    // Site Membership
    app.at("/site/member")
        .put(membership_put)
        .delete(membership_delete);
    app.at("/site/member/get").put(membership_retrieve);

    // Category
    app.at("/category").get(category_get);
    app.at("/category/site").get(category_all_get);

    // Page
    app.at("/page").post(page_edit).delete(page_delete);
    app.at("/page/get").put(page_retrieve);
    app.at("/page/create").post(page_create);
    app.at("/page/direct/:page_id").get(page_get_direct);
    app.at("/page/move").post(page_move);
    app.at("/page/rerender").put(page_rerender);
    app.at("/page/restore").post(page_restore);

    // Page revisions
    app.at("/page/revision").put(page_revision_put);
    app.at("/page/revision/get").put(page_revision_retrieve);
    app.at("/page/revision/count").get(page_revision_count);
    app.at("/page/revision/rollback").post(page_rollback);
    app.at("/page/revision/range")
        .put(page_revision_range_retrieve);

    // Page links
    app.at("/page/links/from").put(page_links_from_retrieve);
    app.at("/page/links/to").put(page_links_to_retrieve);
    app.at("/page/links/to/missing")
        .put(page_links_to_missing_retrieve);
    app.at("/page/urls/from").put(page_links_external_from);
    app.at("/page/urls/to").put(page_links_external_to);

    // Page parents
    app.at("/page/parent").put(parent_put).delete(parent_delete);
    app.at("/page/parent/get").put(parent_retrieve);
    app.at("/page/parent/:relationship_type")
        .put(parent_relationships_retrieve);

    // Files
    app.at("/file").post(file_edit).delete(file_delete);
    app.at("/file/get").put(file_retrieve);
    app.at("/file/upload").post(file_create);
    app.at("/file/move").post(file_move);
    app.at("/file/restore").post(file_restore);

    // File revisions
    app.at("/file/revision").put(file_revision_put);
    app.at("/file/revision/get").put(file_revision_retrieve);
    app.at("/file/revision/count").put(file_revision_count);
    app.at("/file/revision/range/:direction")
        .put(file_revision_range_retrieve);

    // Text
    app.at("/text").put(text_put);
    app.at("/text/:hash").get(text_get);

    // User
    app.at("/user").put(user_put).delete(user_delete);
    app.at("/user/get").put(user_retrieve);
    app.at("/user/avatar").put(user_avatar_put);
    app.at("/user/create").post(user_create);
    app.at("/user/import").post(user_import);
    app.at("/user/addNameChange").post(user_add_name_change);

    // User bot information
    app.at("/user/bot/get").put(user_bot_retrieve);
    app.at("/user/bot/create").post(user_bot_create);
    app.at("/user/bot/owner")
        .put(user_bot_owner_put)
        .delete(user_bot_owner_delete);

    // Message
    app.at("/message/draft")
        .post(message_draft_create)
        .put(message_draft_update)
        .delete(message_draft_delete);
    app.at("/message").post(message_draft_send);

    // Email
    app.at("/email/validate").put(validate_email);

    // Votes
    app.at("/vote").put(vote_put).delete(vote_delete);
    app.at("/vote/get").put(vote_retrieve);
    app.at("/vote/action").put(vote_action);
    app.at("/vote/list").put(vote_list_retrieve);
    app.at("/vote/count").put(vote_count_retrieve);

    app
}
