/*
 * api/v0.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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

//! Routes for version 0 of the API.
//!
//! This version has no commitments to stability and will change as development progresses.

use crate::api::ApiServer;
use crate::methods::locales::*;
use crate::methods::misc::*;
use crate::methods::page::*;
use crate::methods::user::*;
use crate::web::utils::error_response;
use tide::StatusCode;

pub fn build(mut app: ApiServer) -> ApiServer {
    // Miscellaneous
    app.at("/ping").all(ping);
    app.at("/version").get(version);
    app.at("/version/full").get(full_version);
    app.at("/ratelimit-exempt").all(ratelimit_exempt);
    app.at("/teapot")
        .get(|_| async { error_response(StatusCode::ImATeapot, "🫖") });

    // Localization
    app.at("/locale/:locale").head(locale_head).get(locale_get);

    app.at("/message/:locale/:message_key")
        .head(message_head)
        .get(message_post)
        .put(message_post)
        .post(message_post);

    // Page
    app.at("/page/:site_id").post(page_create);
    app.at("/page/:site_id/:type/:id_or_slug")
        .head(page_head)
        .get(page_get)
        .delete(page_delete);

    app.at("/page/:site_id/:type/:id_or_slug/links")
        .put(page_links_put); // TEMP

    app.at("/page/:site_id/:slug/links/missing")
        .put(page_links_missing_put); // TEMP

    app.at("/page/:site_id/:type/:id_or_slug/links/from")
        .get(page_links_from_get);

    app.at("/page/:site_id/:type/:id_or_slug/links/to")
        .get(page_links_to_get);

    app.at("/page/:site_id/slug/:page_slug/links/to/missing")
        .get(page_links_to_missing_get);

    // Page -- invalid routes
    app.at("/page").all(page_invalid);
    app.at("/page/:type/:id_or_slug").all(page_invalid);
    app.at("/page/:site_id/id/:page_slug/links/to/missing")
        .all(page_invalid);

    // User
    app.at("/user").post(user_create);
    app.at("/user/:type/:id_or_slug")
        .head(user_head)
        .get(user_get)
        .put(user_put)
        .delete(user_delete);

    app
}
