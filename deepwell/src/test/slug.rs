/*
 * test/slug.rs
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

macro_rules! create_page {
    ($runner:expr, $create_slug:expr, $expected_slug:expr $(,)?) => {{
        let page = $runner.page2(
            Some(WWW_SITE_ID),
            Some(ANONYMOUS_USER_ID),
            Some(str!($create_slug)),
        ).await?;

        assert_eq!(
            page.slug,
            $expected_slug,
            "Actual created page slug doesn't match expected",
        );

        // Delete afterwards so the test is idempotent on the same database.
        // Since tests still work on the main site and don't create a dummy one.

        let status = $runner
            .delete(format!("/page/{WWW_SITE_ID}/id/{}", page.page_id))?
            .body_json(json!({
                "revisionComments": "Delete slug test page",
                "userId": AUTOMATIC_USER_ID,
            }))?
            .recv()
            .await?;

        assert_eq!(status, StatusCode::Ok);
    }};
}

#[async_test]
async fn categories() -> Result<()> {
    let runner = Runner::setup().await?;

    create_page!(runner, "apple", "apple");
    create_page!(runner, "_default:apple", "apple");
    create_page!(runner, "category:apple", "category:apple");
    create_page!(runner, "category-with-dashes:apple", "category-with-dashes:apple");
    create_page!(runner, "_template", "_template");
    create_page!(runner, "_default:_template", "_template");

    Ok(())
}

#[async_test]
async fn invalid() -> Result<()> {
    let runner = Runner::setup().await?;

    // TODO

    Ok(())
}

#[async_test]
async fn normalization() -> Result<()> {
    let runner = Runner::setup().await?;

    // TODO

    Ok(())
}
