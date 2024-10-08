/*
 * info.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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

use once_cell::sync::Lazy;
use time::format_description::well_known::Rfc2822;
use time::OffsetDateTime;

#[allow(unused_imports)]
pub use self::build::{
    BUILT_TIME_UTC as BUILT_TIME_UTC_STR, CFG_ENDIAN, GIT_COMMIT_HASH, NUM_JOBS,
    PKG_AUTHORS, PKG_DESCRIPTION, PKG_LICENSE, PKG_NAME, PKG_REPOSITORY, PKG_VERSION,
    RUSTC_VERSION, TARGET,
};

pub static BUILT_TIME_UTC: Lazy<OffsetDateTime> = Lazy::new(|| {
    OffsetDateTime::parse(BUILT_TIME_UTC_STR, &Rfc2822)
        .expect("Unable to parse built time string")
});

pub static VERSION_INFO: Lazy<String> = Lazy::new(|| {
    let mut version = format!("v{PKG_VERSION}");

    if let Some(commit_hash) = *GIT_COMMIT_HASH_SHORT {
        str_write!(&mut version, " [{commit_hash}]");
    }

    version
});

pub static COMPILE_INFO: Lazy<String> = Lazy::new(|| {
    let mut info = str!("Compile info:\n");
    str_writeln!(&mut info, "* on {BUILT_TIME_UTC_STR}");
    str_writeln!(&mut info, "* by {RUSTC_VERSION}");
    str_writeln!(&mut info, "* for {TARGET}");
    str_writeln!(&mut info, "* across {NUM_JOBS} threads");
    info
});

pub static VERSION: Lazy<String> = Lazy::new(|| format!("{PKG_NAME} {}", *VERSION_INFO));

pub static FULL_VERSION: Lazy<String> =
    Lazy::new(|| format!("{}\n\n{}", *VERSION, *COMPILE_INFO));

pub static GIT_COMMIT_HASH_SHORT: Lazy<Option<&'static str>> =
    Lazy::new(|| build::GIT_COMMIT_HASH.map(|s| &s[..8]));

pub static HOSTNAME: Lazy<String> = Lazy::new(|| {
    // According to the gethostname(3p) man page,
    // there don't seem to be any errors possible.
    //
    // However it is possible that converting from
    // OsStr can fail.
    hostname::get()
        .expect("Unable to get hostname")
        .into_string()
        .expect("Unable to convert to UTF-8 string")
});

#[test]
fn info() {
    assert!(VERSION.starts_with(PKG_NAME));
    assert!(VERSION.ends_with(&*VERSION_INFO));

    if let Some(hash) = *GIT_COMMIT_HASH_SHORT {
        assert_eq!(hash.len(), 8);
    }
}
