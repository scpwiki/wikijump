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
mod object;

use self::include::route_include;
use self::object::*;
use self::prelude::CONTENT_LENGTH_LIMIT;
use crate::info;
use ftml::ParseOutcome;
use warp::{Filter, Rejection, Reply};

// TODO: add include to other routes

// Routes

pub fn route_preproc(
    log: slog::Logger,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::post()
        .and(warp::path("preprocess"))
        .and(warp::body::content_length_limit(CONTENT_LENGTH_LIMIT))
        .and(warp::body::json())
        .map(move |input| {
            let TextInput { mut text } = input;

            ftml::preprocess(&log, &mut text);

            text
        })
}

pub fn route_tokenize(
    log: &slog::Logger,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let factory = |preprocess| {
        let log = log.clone();

        move |input| {
            let TextInput { mut text } = input;

            if preprocess {
                ftml::preprocess(&log, &mut text);
            }

            let result = ftml::tokenize(&log, &text);
            let tokens = result.tokens();
            warp::reply::json(&tokens)
        }
    };

    let regular = warp::path("tokenize")
        .and(warp::path::end())
        .and(warp::body::content_length_limit(CONTENT_LENGTH_LIMIT))
        .and(warp::body::json())
        .map(factory(true));

    let only = warp::path!("tokenize" / "only")
        .and(warp::body::content_length_limit(CONTENT_LENGTH_LIMIT))
        .and(warp::body::json())
        .map(factory(false));

    regular.or(only)
}

pub fn route_parse(
    log: &slog::Logger,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let factory = |preprocess| {
        let log = log.clone();

        move |input| {
            let TextInput { mut text } = input;

            if preprocess {
                ftml::preprocess(&log, &mut text);
            }

            let tokens = ftml::tokenize(&log, &text);
            let tree = ftml::parse(&log, &tokens);
            warp::reply::json(&tree)
        }
    };

    let regular = warp::path("parse")
        .and(warp::path::end())
        .and(warp::body::content_length_limit(CONTENT_LENGTH_LIMIT))
        .and(warp::body::json())
        .map(factory(true));

    let only = warp::path!("parse" / "only")
        .and(warp::body::content_length_limit(CONTENT_LENGTH_LIMIT))
        .and(warp::body::json())
        .map(factory(false));

    regular.or(only)
}

pub fn route_render_html(
    log: &slog::Logger,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    use ftml::Render;

    let factory = |preprocess| {
        let log = log.clone();

        move |input| {
            let TextInput { mut text } = input;

            if preprocess {
                ftml::preprocess(&log, &mut text);
            }

            let tokens = ftml::tokenize(&log, &text);
            let parsed = ftml::parse(&log, &tokens);
            let (tree, errors) = parsed.into();
            let output = ftml::HtmlRender.render(&tree);
            let result = ParseOutcome::new(output, errors);

            warp::reply::json(&result)
        }
    };

    let regular = warp::path!("render" / "html")
        .and(warp::path::end())
        .and(warp::body::content_length_limit(CONTENT_LENGTH_LIMIT))
        .and(warp::body::json())
        .map(factory(true));

    let only = warp::path!("render" / "html" / "only")
        .and(warp::body::content_length_limit(CONTENT_LENGTH_LIMIT))
        .and(warp::body::json())
        .map(factory(false));

    regular.or(only)
}

pub fn route_misc() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let ping = warp::path("ping").map(|| "Pong!");
    let version = warp::path("version").map(|| &**info::VERSION);
    let wikidot = warp::path("wikidot").map(|| ";-)");

    ping.or(version).or(wikidot)
}

// Collect the routes

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
