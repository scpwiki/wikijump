/*
 * info.rs
 *
 * ftml - Library to parse Wikidot text
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

//! This module has build and meta information about the library.

#[allow(unused)]
mod build {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub use self::build::{
    BUILT_TIME_UTC, CFG_ENV, CFG_OS, CFG_TARGET_ARCH, CI_PLATFORM, DEBUG,
    GIT_COMMIT_HASH, NUM_JOBS, PKG_AUTHORS, PKG_DESCRIPTION, PKG_LICENSE, PKG_NAME,
    PKG_REPOSITORY, PKG_VERSION, RUSTC_VERSION, TARGET,
};

use std::sync::LazyLock;

static VERSION_INFO: LazyLock<String> = LazyLock::new(|| {
    let mut version = format!("v{PKG_VERSION}");

    if let Some(commit_hash) = *GIT_COMMIT_HASH_SHORT {
        str_write!(&mut version, " [{commit_hash}]");
    }

    version
});

/// The package name and version info.
pub static VERSION: LazyLock<String> =
    LazyLock::new(|| format!("{PKG_NAME} {}", *VERSION_INFO));

/// The full version info, including build information.
pub static FULL_VERSION: LazyLock<String> = LazyLock::new(|| {
    let mut version = format!("{}\n\nCompiled:\n", *VERSION_INFO);

    str_writeln!(&mut version, "* across {NUM_JOBS} threads");
    str_writeln!(&mut version, "* by {RUSTC_VERSION}");
    str_writeln!(&mut version, "* for {TARGET}");
    str_writeln!(&mut version, "* on {BUILT_TIME_UTC}");

    version
});
/// The package name and full version info, including build information.
pub static FULL_VERSION_WITH_NAME: LazyLock<String> =
    LazyLock::new(|| format!("{PKG_NAME} {}", *FULL_VERSION));

// The last 8 characters of the commit hash for this version.
pub static GIT_COMMIT_HASH_SHORT: LazyLock<Option<&'static str>> =
    LazyLock::new(|| GIT_COMMIT_HASH.map(|s| &s[..8]));

#[test]
fn info() {
    assert!(VERSION.starts_with(PKG_NAME));
    assert!(VERSION.ends_with(&*VERSION_INFO));

    if let Some(hash) = *GIT_COMMIT_HASH_SHORT {
        assert_eq!(hash.len(), 8);
    }
}
