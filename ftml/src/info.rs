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

#[allow(unused)]
mod build {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub use self::build::{
    BUILT_TIME_UTC, CFG_ENV, CFG_OS, CFG_TARGET_ARCH, CI_PLATFORM, DEBUG,
    GIT_COMMIT_HASH, PKG_LICENSE, PKG_NAME, PKG_REPOSITORY, PKG_VERSION, RUSTC_VERSION,
};

lazy_static! {
    pub static ref VERSION: String = {
        let mut version = format!("{} v{}", PKG_NAME, PKG_VERSION);

        if let Some(commit_hash) = *GIT_COMMIT_HASH_SHORT {
            str_write!(&mut version, " [{}]", commit_hash);
        }

        version
    };
    pub static ref TARGET_TRIPLET: String = {
        format!(
            "{}-{}-{}", //
            CFG_TARGET_ARCH,
            CFG_ENV,
            CFG_OS,
        )
    };
    pub static ref GIT_COMMIT_HASH_SHORT: Option<&'static str> =
        GIT_COMMIT_HASH.map(|s| &s[..8]);
}

#[test]
fn info() {
    assert!(VERSION.starts_with(PKG_NAME));
    assert!(TARGET_TRIPLET.starts_with(CFG_TARGET_ARCH));

    if let Some(hash) = *GIT_COMMIT_HASH_SHORT {
        assert_eq!(hash.len(), 8);
    }
}
