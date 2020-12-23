/*
 * routes/mod.rs
 *
 * ftml - Library to parse Wikidot code
 * Copyright (C) 2019-2020 Ammon Smith
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

use crate::info;
use warp::{Filter, Rejection, Reply};

const CONTENT_LENGTH_LIMIT: u64 = 12 * 1024 * 1024 * 1024; /* 12 MiB */

// Helper struct
#[derive(Deserialize, Debug)]
struct TextInput {
    text: String,
}

// Routes

fn preproc(
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

fn tokenize(
    log: &slog::Logger,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let factory = |preprocess| {
        let log = log.clone();

        move |mut text| {
            if preprocess {
                ftml::preprocess(&log, &mut text);
            }

            let result = ftml::tokenize(&log, &text);
            let tokens = result.tokens();
            warp::reply::json(&tokens)
        }
    };

    let regular = warp::path("tokenize")
        .and(warp::path::param::<String>())
        .and(warp::body::content_length_limit(CONTENT_LENGTH_LIMIT))
        .map(factory(true));

    let no_tokens = warp::path!("tokenize" / "only")
        .and(warp::path::param::<String>())
        .and(warp::body::content_length_limit(CONTENT_LENGTH_LIMIT))
        .map(factory(false));

    regular.or(no_tokens)
}

fn misc() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
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

    let preproc = preproc(log.clone());
    let tokenize = tokenize(&log);
    let misc = misc();

    let routes = preproc.or(tokenize).or(misc);

    warp::any()
        .and(routes)
        .with(log_middleware)
}
