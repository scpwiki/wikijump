/*
 * routes/mod.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Ammon Smith
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

mod prelude {
    pub use super::object::*;
    pub use crate::error::Error;
    pub use crate::includer::HttpIncluder;
    pub use ftml::PageRef;
    pub use warp::{Filter, Rejection, Reply};

    pub const CONTENT_LENGTH_LIMIT: u64 = 4 * 1024 * 1024 * 1024; /* 2 MiB */
}

mod include;
mod misc;
mod object;
mod parse;
mod preproc;
mod render;
mod tokenize;

use self::include::route_include;
use self::misc::route_misc;
use self::parse::route_parse;
use self::preproc::route_preproc;
use self::render::route_render_html;
use self::tokenize::route_tokenize;
use warp::{Filter, Rejection, Reply};

// TODO: add include to other routes

// Collected routes into a server
pub fn build(
    log: slog::Logger,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let log_middleware = {
        let log = log.clone();
        warp::log::custom(move |info| {
            debug!(
                &log,
                "Received web request {}",
                info.path();
                "path" => info.path(),
                "address" => info.remote_addr(),
                "method" => info.method().as_str(),
                "referer" => info.referer(),
                "user-agent" => info.user_agent(),
                "host" => info.host(),
            );
        })
    };

    let include = route_include(log.clone());
    let preproc = route_preproc(log.clone());
    let tokenize = route_tokenize(&log);
    let parse = route_parse(&log);
    let render_html = route_render_html(&log);
    let misc = route_misc();

    warp::any()
        .and(
            include
                .or(preproc)
                .or(tokenize)
                .or(parse)
                .or(render_html)
                .or(misc),
        )
        .with(log_middleware)
        .with(warp::filters::compression::gzip())
}
