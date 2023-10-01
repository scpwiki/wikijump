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
    misc::*, page::*, page_revision::*, parent::*, site::*, site_member::*, text::*,
    user::*, user_bot::*, view::*, vote::*,
};
use crate::locales::Localizations;
use crate::services::blob::spawn_magic_thread;
use crate::services::job::JobRunner;
use crate::utils::error_response;
use anyhow::Result;
use s3::bucket::Bucket;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::time::Duration;
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

pub async fn build_server_state(
    config: Config,
    secrets: Secrets,
) -> Result<ApiServerState> {
    // Connect to database
    tide::log::info!("Connecting to PostgreSQL database");
    let database = database::connect(&secrets.database_url).await?;

    // Load localization data
    tide::log::info!("Loading localization data");
    let localizations = Localizations::open(&config.localization_path).await?;

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
    app.at("/hostname").get(hostname);
    app.at("/config").get(config_dump);
    app.at("/config/path").get(config_path);
    app.at("/normalize/:input").all(normalize_method);
    app.at("/teapot")
        .all(|_| async { error_response(StatusCode::ImATeapot, "🫖") });

    // Localization
    app.at("/locale/:locale").get(locale_get);
    app.at("/message/:locale/translate").put(translate_put);

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
    app.at("/site/member/get")
        .put(membership_retrieve);
    app.at("/site/member/list/get")
        .put(membership_site_retrieve);
    app.at("/user/sites/get").put(membership_user_retrieve); // More appropriate to put here,
                                                             // as part of membership endpoints.

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
