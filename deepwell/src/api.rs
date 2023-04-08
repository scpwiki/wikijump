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

use crate::config::Config;
use crate::database;
use crate::locales::Localizations;
use crate::methods::{
    auth::*, category::*, file::*, file_revision::*, link::*, locale::*, misc::*,
    page::*, page_revision::*, parent::*, site::*, text::*, user::*, user_bot::*,
    vote::*,
};
use crate::services::blob::spawn_magic_thread;
use crate::services::job::JobRunner;
use crate::utils::error_response;
use anyhow::Result;
use s3::bucket::Bucket;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tide::StatusCode;

pub type ApiServerState = Arc<ServerState>;
pub type ApiServer = tide::Server<ApiServerState>;
pub type ApiRequest = tide::Request<ApiServerState>;
pub type ApiResponse = tide::Result;

#[derive(Debug)]
pub struct ServerState {
    pub config: Config,
    pub database: DatabaseConnection,
    pub localizations: Localizations,
    pub s3_bucket: Bucket,
}

pub async fn build_server_state(config: Config) -> Result<ApiServerState> {
    // Connect to database
    tide::log::info!("Connecting to PostgreSQL database");
    let database = database::connect(&config.database_url).await?;

    // Load localization data
    tide::log::info!("Loading localization data");
    let localizations = Localizations::open(&config.localization_path).await?;

    // Create S3 bucket
    tide::log::info!("Opening S3 bucket");

    let s3_bucket = Bucket::new(
        &config.s3_bucket,
        config.s3_region.clone(),
        config.s3_credentials.clone(),
    )?;

    // Return server state
    Ok(Arc::new(ServerState {
        config,
        database,
        localizations,
        s3_bucket,
    }))
}

pub fn build_server(state: ApiServerState) -> ApiServer {
    macro_rules! new {
        () => {
            tide::Server::with_state(Arc::clone(&state))
        };
    }

    // Start main job executor task
    // (and ancillary repeated tasks)
    JobRunner::spawn(&state);

    // Start MIME evaluator thread
    spawn_magic_thread();

    // Create server and add routes
    //
    // Prefix is present to avoid ambiguity about what this
    // API is meant to be and the fact that it's not to be publicly-facing.
    let mut app = new!();
    app.at("/api/trusted").nest(build_routes(new!()));
    app
}

fn build_routes(mut app: ApiServer) -> ApiServer {
    // Miscellaneous
    app.at("/ping").all(ping);
    app.at("/version").get(version);
    app.at("/version/full").get(full_version);
    app.at("/normalize/:input").all(normalize_method);
    app.at("/teapot")
        .get(|_| async { error_response(StatusCode::ImATeapot, "🫖") });

    // Localization
    app.at("/locale/:locale").get(locale_get);
    app.at("/message/:locale/:message_key").get(message_get);

    // Authentication
    app.at("/auth/login").post(auth_login);
    app.at("/auth/logout").delete(auth_logout);
    app.at("/auth/mfa").post(auth_mfa_verify); // Is part of the login process,
                                               // which is why it's up here.
    app.at("/auth/session").get(auth_session_get);
    app.at("/auth/session/validate").put(auth_session_validate);
    app.at("/auth/session/renew").put(auth_session_renew);
    app.at("/auth/session/others")
        .delete(auth_session_invalidate_others);
    app.at("/auth/mfa/setup").post(auth_mfa_setup);
    app.at("/auth/mfa/disable").post(auth_mfa_disable);
    app.at("/auth/mfa/resetRecovery")
        .post(auth_mfa_reset_recovery);

    // Site
    app.at("/site").get(site_get).put(site_put);
    app.at("/site/create").post(site_create);

    // Category
    app.at("/category").get(category_get);
    app.at("/category/site").get(category_all_get);

    // Page
    app.at("/page")
        .get(page_get)
        .post(page_edit)
        .delete(page_delete);
    app.at("/page/create").post(page_create);
    app.at("/page/direct").get(page_get_direct);

    app.at("/page/move").post(page_move);
    app.at("/page/rerender").put(page_rerender);
    app.at("/page/restore").post(page_restore);

    // Page revisions
    app.at("/page/revision")
        .get(page_revision_get)
        .put(page_revision_put);
    app.at("/page/revision/latest").get(page_revision_info);

    app.at("/page/revision/rollback").post(page_rollback);
    app.at("/page/revision/range").get(page_revision_range_get);

    // Page links
    app.at("/page/:site_id/:type/:id_or_slug/links/from")
        .get(page_links_from_get);

    app.at("/page/:site_id/:type/:id_or_slug/links/to")
        .get(page_links_to_get);

    app.at("/page/:site_id/slug/:page_slug/links/to/missing")
        .get(page_links_to_missing_get);

    app.at("/page/:site_id/:type/:id_or_slug/urls")
        .get(page_links_external_from);

    app.at("/page/:site_id/urls/:url")
        .get(page_links_external_to);

    // Page parents
    app.at(
        "/page/:site_id/:parent_type/:parent_id_or_slug/:child_type/:child_id_or_slug",
    )
    .get(parent_get)
    .put(parent_put)
    .delete(parent_delete);

    app.at("/page/:site_id/:relationship_type/:type/:id_or_slug")
        .get(parent_relationships_get);

    // Page (invalid routes)
    app.at("/page").all(page_invalid);
    app.at("/page/:type/:id_or_slug").all(page_invalid);
    app.at("/page/:site_id/id/:page_slug/links/to/missing")
        .all(page_invalid);

    // Files
    app.at("/file/:site_id/:type/:id_or_slug").post(file_create);
    app.at("/file/:site_id/:page_type/:id_or_slug/:file_type/:id_or_name")
        .get(file_get)
        .post(file_edit)
        .delete(file_delete);

    app.at("/file/:site_id/:page_type/:id_or_slug/move")
        .post(file_move);

    app.at("/file/:site_id/:page_type/:id_or_slug/restore")
        .post(file_restore);

    // File revisions
    app.at("/file/:site_id/:page_type/:id_or_slug/:file_type/:id_or_name/revision")
        .get(file_revision_info);

    app.at("/file/:site_id/:page_type/:id_or_slug/:file_type/:id_or_name/revision/:revision_number")
        .get(file_revision_get)
        .put(file_revision_put);

    app.at("/file/:site_id/:page_type/:id_or_slug/:file_type/:id_or_name/revision/:revision_number/:direction")
        .get(file_revision_range_get);

    // Text
    app.at("/text").put(text_put);
    app.at("/text/:hash").get(text_get);

    // User
    app.at("/user").post(user_create);
    app.at("/user/:type/:id_or_slug")
        .get(user_get)
        .put(user_put)
        .delete(user_delete);

    app.at("/user/:type/:id_or_slug/addNameChange")
        .post(user_add_name_change);

    // User bot information
    app.at("/user/bot").post(user_bot_create);
    app.at("/user/bot/:bot_type/:bot_id_or_slug")
        .get(user_bot_get);
    app.at("/user/bot/:bot_type/:bot_id_or_slug/owner/:human_type/:human_id_or_slug")
        .put(user_bot_owner_put)
        .delete(user_bot_owner_delete);

    // Votes
    app.at("/vote")
        .get(vote_get)
        .put(vote_put)
        .delete(vote_delete);

    app.at("/vote/action").put(vote_action);
    app.at("/vote/list").get(vote_list_get);
    app.at("/vote/count").get(vote_count_get);

    app
}
