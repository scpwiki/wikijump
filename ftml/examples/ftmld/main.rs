/*
 * ftmld/main.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith for Project Foundation
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

// TEMP
#![allow(dead_code)]

#![deny(missing_debug_implementations)]

#[macro_use]
extern crate cfg_if;
extern crate clap;
extern crate ftml;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde;
extern crate serde_json as json;

cfg_if! {
    if #[cfg(unix)] {
        mod request;
        mod response;
        mod server;

        use self::request::Request;
        use self::response::Response;

        fn main() {}
    } else {
        use std::process;

        fn main() {
            eprintln!("This application uses Unix Domain Sockets and thus is not compatible with this platform");
            process::exit(1);
        }
    }
}
