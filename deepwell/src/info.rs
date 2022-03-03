/*
 * info.rs
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

mod build {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub use self::build::{
    BUILT_TIME_UTC, GIT_COMMIT_HASH, NUM_JOBS, PKG_AUTHORS, PKG_DESCRIPTION, PKG_LICENSE,
    PKG_NAME, PKG_REPOSITORY, PKG_VERSION, RUSTC_VERSION, TARGET,
};

lazy_static! {
    static ref VERSION_INFO: String = {
        let mut version = format!("v{PKG_VERSION}");

        if let Some(commit_hash) = *GIT_COMMIT_HASH_SHORT {
            str_write!(&mut version, " [{commit_hash}]");
        }

        version
    };

    pub static ref VERSION: String = format!("{PKG_NAME} {}", *VERSION_INFO);

    pub static ref FULL_VERSION: String = {
        let mut version = format!("{}\n\nCompiled:\n", *VERSION_INFO);

        str_writeln!(&mut version, "* across {NUM_JOBS} threads");
        str_writeln!(&mut version, "* by {RUSTC_VERSION}");
        str_writeln!(&mut version, "* for {TARGET}");
        str_writeln!(&mut version, "* on {BUILT_TIME_UTC}");

        version
    };

    pub static ref VERSION_WITH_NAME: String = {
        format!("{} {}", PKG_NAME, *VERSION)
    };

    pub static ref FULL_VERSION_WITH_NAME: String = {
        format!("{} {}", PKG_NAME, *FULL_VERSION)
    };

    pub static ref GIT_COMMIT_HASH_SHORT: Option<&'static str> = {
        build::GIT_COMMIT_HASH.map(|s| &s[..8])
    };

    pub static ref HOSTNAME: String = {
        // According to the gethostname(3p) man page,
        // there don't seem to be any errors possible.
        //
        // However it is possible that converting from
        // OsStr can fail.
        hostname::get()
            .expect("Unable to get hostname")
            .into_string()
            .expect("Unable to convert to UTF-8 string")
    };
}

#[test]
fn info() {
    assert!(VERSION.starts_with(PKG_NAME));
    assert!(VERSION.ends_with(*VERSION_INFO));

    if let Some(hash) = *GIT_COMMIT_HASH_SHORT {
        assert_eq!(hash.len(), 8);
    }
}
