/*
 * endpoints/info.rs
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

//! Endpoints associated with getting DEEPWELL daemon information.

use super::prelude::*;
use crate::info;
use crate::utils::now;
use std::path::PathBuf;
use time::OffsetDateTime;

#[derive(Serialize, Debug, Clone)]
pub struct Info {
    package: PackageInfo,
    compile_info: CompileInfo,

    #[serde(with = "time::serde::rfc3339")]
    current_time: OffsetDateTime,
    hostname: &'static str,
    config_path: PathBuf,
}

#[derive(Serialize, Debug, Clone)]
pub struct PackageInfo {
    name: &'static str,
    description: &'static str,
    license: &'static str,
    repository: &'static str,
    version: &'static str,
}

#[derive(Serialize, Debug, Clone)]
pub struct CompileInfo {
    #[serde(with = "time::serde::rfc3339")]
    built_at: OffsetDateTime,
    rustc_version: &'static str,
    endian: &'static str,
    target: &'static str,
    threads: u32,
    git_commit: Option<&'static str>,
}

pub async fn server_info(
    ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<Info> {
    let config = ctx.config();

    info!("Building server information response");
    Ok(Info {
        package: PackageInfo {
            name: info::PKG_NAME,
            version: &info::VERSION_INFO,
            description: info::PKG_DESCRIPTION,
            license: info::PKG_LICENSE,
            repository: info::PKG_REPOSITORY,
        },
        compile_info: CompileInfo {
            built_at: *info::BUILT_TIME_UTC,
            rustc_version: info::RUSTC_VERSION,
            endian: info::CFG_ENDIAN,
            target: info::TARGET,
            threads: info::NUM_JOBS,
            git_commit: info::GIT_COMMIT_HASH,
        },
        config_path: config.raw_toml_path.clone(),
        hostname: &info::HOSTNAME,
        current_time: now(),
    })
}
