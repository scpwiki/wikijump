/*
 * test/mod.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2022 Wikijump Team
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

mod prelude {
    pub use super::setup;
    pub use anyhow::Result;
    pub use tide::Body;
    pub use tide_testing::TideTestingExt;

    use serde::Serialize;

    pub const WWW_SITE: i64 = 1;
    pub const EN_TEMPLATE_SITE: i64 = 2;

    pub const ADMIN_USER: i64 = 1;
    pub const AUTOMATIC_USER: i64 = 2;
    pub const ANONYMOUS_USER: i64 = 3;
    pub const REGULAR_USER: i64 = 4;

    #[inline]
    pub fn create_body<T: Serialize>(data: T) -> Body {
        Body::from_json(&data).expect("Unable to create JSON body")
    }
}

mod misc;
mod page;

use crate::api::{self, ApiServer};
use crate::config::Config;
use anyhow::Result;

pub async fn setup() -> Result<ApiServer> {
    // The Default impl is different in the test environment
    let config = Config::default();

    // Build API server
    crate::setup(&config).await?;
    let app = api::build_server(config).await?;
    Ok(app)
}
