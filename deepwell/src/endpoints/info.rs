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
    name: &'static str,
    description: &'static str,
    license: &'static str,
    repository: &'static str,
    version: &'static str,
    compile_info: &'static str,
    git_commit: Option<&'static str>,
    hostname: &'static str,

    #[serde(with = "time::serde::rfc3339")]
    current_time: OffsetDateTime,
    config_path: PathBuf,
}

pub async fn server_info(
    ctx: &ServiceContext<'_>,
    _params: Params<'static>,
) -> Result<Info> {
    let config = ctx.config();

    info!("Building server information for response");
    Ok(Info {
        name: info::PKG_NAME,
        description: info::PKG_DESCRIPTION,
        license: info::PKG_LICENSE,
        repository: info::PKG_REPOSITORY,
        version: &info::VERSION_INFO,
        compile_info: &info::COMPILE_INFO,
        git_commit: info::GIT_COMMIT_HASH,
        hostname: &info::HOSTNAME,
        config_path: config.raw_toml_path.clone(),
        current_time: now(),
    })
}
