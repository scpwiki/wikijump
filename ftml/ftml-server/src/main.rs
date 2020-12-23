/*
 * main.rs
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

#![deny(missing_debug_implementations)]
#![forbid(unsafe_code)]

//! REST web server to allow clients to preprocess, parse, and render Wikidot text.
//!
//! This is a wrapper around the `ftml` crate to provide its Rust library API over
//! a REST interface, usable by any programming language, or via network.

extern crate clap;
extern crate ftml;
extern crate hostname;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate slog;
extern crate slog_bunyan;
extern crate sloggers;

#[macro_use]
extern crate str_macro;
extern crate tokio;
extern crate users;
extern crate warp;

mod config;
mod info;
mod logger;

use self::config::Config;
use warp::Filter;

#[tokio::main]
async fn main() {
    let config = Config::parse_args();
    let log = logger::build(&config.log_file, config.log_level);

    info::print(&log, config.address);

    let routes = {
        let misc = {
            let ping = warp::path("ping").map(|| "Pong!");
            let version = warp::path("version").map(|| &**info::VERSION);
            let wikidot = warp::path("wikidot").map(|| ";-)");

            ping.or(version).or(wikidot)
        };

        let log_middleware = warp::log::custom(move |info| {
            debug!(
                log,
                "Received web request {}",
                info.path();
                "path" => info.path(),
                "address" => info.remote_addr(),
                "method" => info.method().as_str(),
                "referer" => info.referer(),
                "user-agent" => info.user_agent(),
                "host" => info.host(),
            );
        });

        warp::any().and(misc).with(log_middleware)
    };

    warp::serve(routes).run(config.address).await;
}
