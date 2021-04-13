/*
 * info.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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

pub use ftml::info::*;

lazy_static! {
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
    let host = address.ip();
    let port = address.port();

    // Print intro
    println!("{}", *VERSION);
    println!();
    println!("License: {}", PKG_LICENSE);
    println!("Repository: {}", PKG_REPOSITORY);
    println!();
    println!("Compiled:");
    println!("  - {}", compile_env);
    println!("  - on {}", BUILT_TIME_UTC);
    println!("  - by {}", RUSTC_VERSION);
    println!("  - for {}", *TARGET_TRIPLET);
    println!();
    println!("Running as {}:{} ({}:{})", username, groupname, uid, gid);
    println!("Serving on port {}", port);
    println!();

    // Log all of this
    info!(
        log,
        "Starting ftml-http...";
        "version" => &*VERSION,
        "compiled-debug" => DEBUG,
        "compiled-on" => BUILT_TIME_UTC,
        "compiled-by" => RUSTC_VERSION,
        "compiled-for" => &*TARGET_TRIPLET,
        "running-as-username" => username,
        "running-as-groupname" => groupname,
        "running-as-uid" => uid,
        "running-as-gid" => gid,
        "host" => str!(host),
        "port" => port,
    );
}
