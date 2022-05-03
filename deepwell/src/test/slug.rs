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
use crate::services::page::GetPageOutput;

async fn create_page(
    runner: &Runner,
    create_slug: &str,
    expected_slug: &str,
) -> Result<()> {
    let GeneratedPage { page_id, .. } = runner
        .page2(
            Some(WWW_SITE_ID),
            Some(ANONYMOUS_USER_ID),
            Some(str!(create_slug)),
        )
        .await?;

    // Get page data
    let (output, status) = runner
        .get(format!("/page/{WWW_SITE_ID}/id/{page_id}"))?
        .recv_json::<GetPageOutput>()
        .await?;

    // Ensure request worked
    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.page_id, page_id);

    // Actual assertion for this test
    assert_eq!(
        output.slug, expected_slug,
        "Actual created page slug doesn't match expected",
    );

    // Delete afterwards so the test is idempotent on the same database.
    // Since tests still work on the main site and don't create a dummy one.

    let status = runner
        .delete(format!("/page/{WWW_SITE_ID}/id/{page_id}"))?
        .body_json(json!({
            "revisionComments": "Delete slug test page",
            "userId": AUTOMATIC_USER_ID,
        }))?
        .recv()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    Ok(())
}

#[async_test]
async fn categories() -> Result<()> {
    let runner = Runner::setup().await?;

    create_page(&runner, "apple", "apple").await?;
    create_page(&runner, "_default:apple", "apple").await?;
    create_page(&runner, "category:apple", "category:apple").await?;
    create_page(
        &runner,
        "category-with-dashes:apple",
        "category-with-dashes:apple",
    )
    .await?;
    create_page(&runner, "_template", "_template").await?;
    create_page(&runner, "_default:_template", "_template").await?;

    Ok(())
}

#[async_test]
async fn normalization() -> Result<()> {
    let runner = Runner::setup().await?;

    create_page(&runner, "Apple", "apple").await?;
    create_page(&runner, "AppleBanana", "applebanana").await?;
    create_page(&runner, "Apple Banana", "apple-banana").await?;
    create_page(&runner, " apple ", "apple").await?;
    create_page(&runner, " APPLE ", "apple").await?;
    create_page(&runner, "-apple-", "apple").await?;
    create_page(&runner, "-apple", "apple").await?;
    create_page(&runner, "apple-", "apple").await?;
    create_page(&runner, "apple-banana", "apple-banana").await?;
    create_page(&runner, "apple--banana", "apple-banana").await?;
    create_page(&runner, "apple---banana", "apple-banana").await?;
    create_page(&runner, "__template", "_template").await?;
    create_page(&runner, "_template_", "_template").await?;
    create_page(&runner, "template_", "template").await?;
    create_page(&runner, "Tufto's Proposal", "tufto-s-proposal").await?;
    create_page(&runner, "SCP-001", "scp-001").await?;

    create_page(&runner, "_default:Apple", "apple").await?;
    create_page(&runner, "_default:APPLE", "apple").await?;
    create_page(&runner, "_default::Apple", "apple").await?;
    create_page(&runner, "_default::APPLE", "apple").await?;
    create_page(&runner, "category:Apple", "category:apple").await?;
    create_page(&runner, "category:APPLE", "category:apple").await?;
    create_page(&runner, "category::apple", "category:apple").await?;
    create_page(&runner, "category:::apple", "category:apple").await?;
    create_page(&runner, "category-:-apple", "category:apple").await?;
    create_page(&runner, "category:-apple", "category:apple").await?;
    create_page(&runner, "category-:apple", "category:apple").await?;

    create_page(&runner, "category:xyz:apple", "category:xyz:apple").await?;
    create_page(&runner, "category::xyz::apple", "category:xyz:apple").await?;
    create_page(&runner, "category::xyz:apple", "category:xyz:apple").await?;
    create_page(&runner, "category:xyz::apple", "category:xyz:apple").await?;

    create_page(&runner, ":apple:", "apple").await?;
    create_page(&runner, "apple:", "apple").await?;
    create_page(&runner, ":apple", "apple").await?;
    create_page(&runner, "_default::apple:", "apple").await?;
    create_page(&runner, "_default:apple:", "apple").await?;
    create_page(&runner, "_default::apple", "apple").await?;
    create_page(&runner, "category::apple:", "category:apple").await?;
    create_page(&runner, "category:apple:", "category:apple").await?;
    create_page(&runner, "category::apple", "category:apple").await?;

    Ok(())
}
