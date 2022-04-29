/*
 * test/misc.rs
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

use super::prelude::*;
use crate::info;

#[async_std::test]
async fn ping() -> Result<()> {
    run_test! {{
        let env = TestEnvironment::setup().await?;

        // GET
        let (output, status) = env.get("/ping")?.recv_string().await?;
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(output, "Pong!");

        // POST
        let (output, status) = env.post("/ping")?.recv_string().await?;
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(output, "Pong!");

        Ok(())
    }}
}

#[async_std::test]
async fn version() -> Result<()> {
    run_test! {{
        let env = TestEnvironment::setup().await?;

        // Regular
        let (output, status) = env.get("/version")?.recv_string().await?;
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(&output, &*info::VERSION);

        // Full
        let (output, status) = env.get("/version/full")?.recv_string().await?;
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(&output, &*info::FULL_VERSION_WITH_NAME);

        Ok(())
    }}
}

#[async_std::test]
async fn teapot() -> Result<()> {
    run_test! {{
        let env = TestEnvironment::setup().await?;

        // GET
        let (_, status) = env.get("/teapot")?.recv_string().await?;
        assert_eq!(status, StatusCode::ImATeapot);

        Ok(())
    }}
}
