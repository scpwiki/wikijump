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
use crate::utils::error_response;
use anyhow::Result;
use jsonrpsee::server::{RpcModule, Server, ServerHandle};
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

pub async fn build_server_state(config: Config, secrets: Secrets) -> Result<ServerState> {
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

pub async fn build_server(app_state: ServerState) -> Result<ServerHandle> {
    let socket_address = app_state.config.address;
    let server = Server::builder().build(socket_address).await?;
    let module = build_module(app_state).await?;
    let handle = server.start(module);
    Ok(handle)
}

async fn build_module(app_state: ServerState) -> Result<RpcModule<ServerState>> {
    let mut module = RpcModule::new(app_state);

    // Miscellaneous
    module.register_async_method("ping", |_params, _state| async { todo!() })?;
    module.register_async_method("version", |_params, _state| async { todo!() })?;
    module.register_async_method("version_full", |_params, _state| async { todo!() })?;
    module.register_async_method("hostname", |_params, _state| async { todo!() })?;
    module.register_async_method("config", |_params, _state| async { todo!() })?;
    module.register_async_method("config_path", |_params, _state| async { todo!() })?;
    module.register_async_method("normalize", |_params, _state| async { todo!() })?;

    // Localization
    module.register_async_method("locale", |_params, _state| async { todo!() })?;
    module.register_async_method("translate", |_params, _state| async { todo!() })?;

    // Web server
    module.register_async_method("page_view", |_params, _state| async { todo!() })?;

    // Authentication
    module.register_async_method("login", |_params, _state| async { todo!() })?;
    module.register_async_method("logout", |_params, _state| async { todo!() })?;
    module.register_async_method("session_get", |_params, _state| async { todo!() })?;
    module.register_async_method("session_get_others", |_params, _state| async {
        todo!()
    })?;
    module.register_async_method("session_renew", |_params, _state| async { todo!() })?;
    module.register_async_method("mfa_verify", |_params, _state| async { todo!() })?;
    module.register_async_method("mfa_setup", |_params, _state| async { todo!() })?;
    module.register_async_method("mfa_disable", |_params, _state| async { todo!() })?;
    module.register_async_method("mfa_reset_recovery", |_params, _state| async {
        todo!()
    })?;

    // Site
    module.register_async_method("site_create", |_params, _state| async { todo!() })?;
    module.register_async_method("site_get", |_params, _state| async { todo!() })?;
    module.register_async_method("site_update", |_params, _state| async { todo!() })?;
    module
        .register_async_method("site_from_domain", |_params, _state| async { todo!() })?;

    // Site custom domain
    module.register_async_method("custom_domain_create", |_params, _state| async {
        todo!()
    })?;
    module.register_async_method("custom_domain_get", |_params, _state| async {
        todo!()
    })?;
    module.register_async_method("custom_domain_delete", |_params, _state| async {
        todo!()
    })?;

    // Site membership
    module.register_async_method("member_create", |_params, _state| async { todo!() })?;
    module.register_async_method("member_get", |_params, _state| async { todo!() })?;
    module.register_async_method("member_delete", |_params, _state| async { todo!() })?;

    // Category
    module.register_async_method("category_get", |_params, _state| async { todo!() })?;
    module
        .register_async_method("category_get_all", |_params, _state| async { todo!() })?;

    // Page
    module.register_async_method("page_create", |_params, _state| async { todo!() })?;
    module.register_async_method("page_get", |_params, _state| async { todo!() })?;
    module
        .register_async_method("page_get_direct", |_params, _state| async { todo!() })?;
    module.register_async_method("page_edit", |_params, _state| async { todo!() })?;
    module.register_async_method("page_delete", |_params, _state| async { todo!() })?;
    module.register_async_method("page_move", |_params, _state| async { todo!() })?;
    module.register_async_method("page_rerender", |_params, _state| async { todo!() })?;
    module.register_async_method("page_restore", |_params, _state| async { todo!() })?;

    // Page revisions
    module.register_async_method("page_revision_create", |_params, _state| async {
        todo!()
    })?;
    module.register_async_method("page_revision_get", |_params, _state| async {
        todo!()
    })?;
    module.register_async_method("page_revision_count", |_params, _state| async {
        todo!()
    })?;
    module.register_async_method("page_revision_rollback", |_params, _state| async {
        todo!()
    })?;
    module.register_async_method("page_revision_range", |_params, _state| async {
        todo!()
    })?;

    // Page links
    module.register_async_method("page_get_links_from", |_params, _state| async {
        todo!()
    })?;
    module.register_async_method("page_get_links_to", |_params, _state| async {
        todo!()
    })?;
    module
        .register_async_method("page_get_links_to_missing", |_params, _state| async {
            todo!()
        })?;
    module.register_async_method("page_get_urls_from", |_params, _state| async {
        todo!()
    })?;
    module
        .register_async_method("page_get_urls_to", |_params, _state| async { todo!() })?;

    // Page parents
    module.register_async_method("parent_create", |_params, _state| async { todo!() })?;
    module.register_async_method("parent_get", |_params, _state| async { todo!() })?;
    module.register_async_method("parent_delete", |_params, _state| async { todo!() })?;
    module.register_async_method("parent_relationship", |_params, _state| async {
        todo!()
    })?;

    // Files
    module.register_async_method("file_create", |_params, _state| async { todo!() })?;
    module.register_async_method("file_get", |_params, _state| async { todo!() })?;
    module.register_async_method("file_edit", |_params, _state| async { todo!() })?;
    module.register_async_method("file_delete", |_params, _state| async { todo!() })?;
    module.register_async_method("file_move", |_params, _state| async { todo!() })?;
    module.register_async_method("file_restore", |_params, _state| async { todo!() })?;

    // File revisions
    module.register_async_method("file_revision_create", |_params, _state| async {
        todo!()
    })?;
    module.register_async_method("file_revision_get", |_params, _state| async {
        todo!()
    })?;
    module.register_async_method("file_revision_count", |_params, _state| async {
        todo!()
    })?;
    module.register_async_method("file_revision_range", |_params, _state| async {
        todo!()
    })?;

    // Text
    module.register_async_method("text_create", |_params, _state| async { todo!() })?;
    module.register_async_method("text_get", |_params, _state| async { todo!() })?;

    // User
    module.register_async_method("user_create", |_params, _state| async { todo!() })?;
    module.register_async_method("user_get", |_params, _state| async { todo!() })?;
    module.register_async_method("user_edit", |_params, _state| async { todo!() })?;
    module.register_async_method("user_delete", |_params, _state| async { todo!() })?;
    module.register_async_method("user_import", |_params, _state| async { todo!() })?;
    module.register_async_method("user_add_name_change", |_params, _state| async {
        todo!()
    })?;
    module
        .register_async_method("user_avatar_set", |_params, _state| async { todo!() })?;

    // Bot user
    module
        .register_async_method("bot_user_create", |_params, _state| async { todo!() })?;
    module.register_async_method("bot_user_get", |_params, _state| async { todo!() })?;
    module.register_async_method("bot_user_owner_set", |_params, _state| async {
        todo!()
    })?;
    module.register_async_method("bot_user_owner_delete", |_params, _state| async {
        todo!()
    })?;

    // Direct messages
    module.register_async_method("message_draft_create", |_params, _state| async {
        todo!()
    })?;
    module.register_async_method("message_draft_edit", |_params, _state| async {
        todo!()
    })?;
    module.register_async_method("message_draft_delete", |_params, _state| async {
        todo!()
    })?;
    module.register_async_method("message_draft_send", |_params, _state| async {
        todo!()
    })?;

    // Email
    module
        .register_async_method("email_validate", |_params, _state| async { todo!() })?;

    // Votes
    module.register_async_method("vote_set", |_params, _state| async { todo!() })?;
    module.register_async_method("vote_get", |_params, _state| async { todo!() })?;
    module.register_async_method("vote_delete", |_params, _state| async { todo!() })?;
    module.register_async_method("vote_action", |_params, _state| async { todo!() })?;
    module.register_async_method("vote_list", |_params, _state| async { todo!() })?;
    module.register_async_method("vote_count", |_params, _state| async { todo!() })?;

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
    app.at("/ping").all(ping);
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
