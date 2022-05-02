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

#[async_test]
async fn ping() -> Result<()> {
    let runner = Runner::setup().await?;

    // GET
    let (output, status) = runner.get("/ping")?.recv_string().await?;
    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output, "Pong!");

    // POST
    let (output, status) = runner.post("/ping")?.recv_string().await?;
    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output, "Pong!");

    Ok(())
}

#[async_test]
async fn version() -> Result<()> {
    let runner = Runner::setup().await?;

    // Regular
    let (output, status) = runner.get("/version")?.recv_string().await?;
    assert_eq!(status, StatusCode::Ok);
    assert_eq!(&output, &*info::VERSION);

    // Full
    let (output, status) = runner.get("/version/full")?.recv_string().await?;
    assert_eq!(status, StatusCode::Ok);
    assert_eq!(&output, &*info::FULL_VERSION_WITH_NAME);

    Ok(())
}

#[async_test]
async fn teapot() -> Result<()> {
    let runner = Runner::setup().await?;

    // GET
    let status = runner.get("/teapot")?.recv().await?;
    assert_eq!(status, StatusCode::ImATeapot);

    Ok(())
}
