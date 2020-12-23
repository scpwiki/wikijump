/*
 * info.rs
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

use std::ffi::OsString;
use std::net::SocketAddr;

#[allow(unused)]
mod build {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub use self::build::{
    BUILT_TIME_UTC, CFG_ENV, CFG_OS, CFG_TARGET_ARCH, CI_PLATFORM,
    DEBUG, GIT_COMMIT_HASH, PKG_LICENSE, PKG_NAME, PKG_REPOSITORY, PKG_VERSION,
    RUSTC_VERSION,
};

lazy_static! {
    pub static ref VERSION: String = {
        format!(
            "{} v{} [{}]",
            PKG_NAME,
            PKG_VERSION,
            GIT_COMMIT_HASH.unwrap_or("nohash"),
        )
    };

    pub static ref TARGET_TRIPLET: String = {
        format!(
            "{}-{}-{}",
            CFG_TARGET_ARCH,
            CFG_ENV,
            CFG_OS,
        )
    };

    pub static ref HOSTNAME: String = {
        // There don't seem to be any errors possible
        // based on the gethostname(3p) man page.
        //
        // However converting from OsStr to &str
        // can fail.
        to_str(hostname::get().expect("Unable to get hostname"))
    };
}

fn to_str(value: OsString) -> String {
    value
        .into_string()
        .expect("Unable to convert to UTF-8 string")
}

pub fn print(log: &slog::Logger, address: SocketAddr) {
    macro_rules! to_str {
        ($os_str:expr) => {
            to_str($os_str.expect("Unable to get user or group name"))
        };
    }

    let compile_env = if DEBUG {
        "with debug symbols"
    } else {
        "for production"
    };

    let username = to_str!(users::get_effective_username());
    let groupname = to_str!(users::get_effective_groupname());
    let uid = users::get_effective_uid();
    let gid = users::get_effective_gid();
    let host = address.ip().to_string();
    let port = address.port();

    // Print intro
    println!("{}", *VERSION);
    println!();
    println!("Compiled:");
    println!("  - {}", compile_env);
    println!("  - on {}", BUILT_TIME_UTC);
    println!("  - by {}", RUSTC_VERSION);
    println!("  - for {}", *TARGET_TRIPLET);
    println!();
    println!("License: {}", PKG_LICENSE);
    println!("Repository: {}", PKG_REPOSITORY);
    println!();
    println!("Running as {}:{} ({}:{})", username, groupname, uid, gid);
    println!("Hosted at {} on port {}", host, port);

    // Log all of this
    info!(
        log,
        "Starting ftml-server...";
        "version" => &*VERSION,
        "compiled-debug" => DEBUG,
        "compiled-on" => BUILT_TIME_UTC,
        "compiled-by" => RUSTC_VERSION,
        "compiled-for" => &*TARGET_TRIPLET,
        "running-as-username" => username,
        "running-as-groupname" => groupname,
        "running-as-uid" => uid,
        "running-as-gid" => gid,
        "host" => host,
        "port" => port,
    );
}
