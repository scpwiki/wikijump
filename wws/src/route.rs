/*
 * route.rs
 *
 * Wilson's Web Server - Serves a zoo of user-generated content
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

use crate::handler::*;
use crate::state::ServerState;
use axum::Router;
use axum::http::header::{ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue};
use axum::routing::{any, get};
use tower_http::compression::CompressionLayer;
use tower_http::normalize_path::NormalizePathLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;

pub fn build_router(state: ServerState) -> Router {
    // NOTE: For all GET routes, axum automatically handles HEAD requests.
    //       The same logic is run, but the body is removed, which is very
    //       convenient for us.
    //
    //       If we can avoid an expensive operation in a HEAD, then add
    //       a "method: http::Method" parameter in the request then check
    //       that before doing the relevant operation.

    Router::new()
        // Wikidot redirects
        .route(
            "/local--files/{page_slug}/{filename}",
            any(handle_file_redirect),
        )
        .route(
            "/local--code/{page_slug}/{index}",
            any(handle_code_redirect),
        )
        .route("/local--html/{page_slug}/{id}", any(handle_html_redirect))
        // Wikijump redirects
        .route("/-/files/{page_slug}/{filename}", any(handle_file_redirect))
        .route("/{page_slug}/code/{filename}", any(handle_code_redirect))
        .route("/{page_slug}/html/{filename}", any(handle_html_redirect))
        .route("/{page_slug}/file/{filename}", any(handle_file_redirect))
        .route(
            "/{page_slug}/download/{filename}",
            any(handle_download_redirect),
        )
        // Files
        .route("/-/file/{page_slug}/{filename}", get(handle_file_fetch))
        .route("/-/file/{page_slug}/{filename}", any(handle_invalid_method))
        .route(
            "/-/download/{page_slug}/{filename}",
            get(handle_file_download),
        )
        .route(
            "/-/download/{page_slug}/{filename}",
            any(handle_invalid_method),
        )
        .route("/-/avatar/{user_id}", get(handle_user_avatar))
        .route("/-/avatar/{user_id}", any(handle_invalid_method))
        // Code and HTML
        .route("/-/code/{page_slug}/{index}", get(handle_code_block))
        .route("/-/code/{page_slug}/{index}", any(handle_invalid_method))
        .route("/-/html/{page_slug}/{id}", get(handle_html_block))
        .route("/-/html/{page_slug}/{id}", any(handle_invalid_method))
        // System routes
        .route("/-/health-check", any(handle_health_check))
        .route("/-/basic-error/{error_code}", get(handle_basic_error))
        .route("/-/basic-error/{error_code}", any(handle_invalid_method))
        // General routes
        .route("/robots.txt", get(handle_robots_txt)) // TODO
        .route("/.well-known", any(handle_well_known)) // TODO
        .fallback(redirect_to_main)
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(NormalizePathLayer::trim_trailing_slash())
        .layer(SetResponseHeaderLayer::if_not_present(
            ACCESS_CONTROL_ALLOW_ORIGIN,
            Some(HeaderValue::from_static("*")),
        ))
        .layer(
            CompressionLayer::new()
                .gzip(true)
                .deflate(true)
                .br(true)
                .zstd(true),
        )
        .with_state(state)
}
