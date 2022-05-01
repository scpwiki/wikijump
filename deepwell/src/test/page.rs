/*
 * test/page.rs
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
use crate::models::sea_orm_active_enums::RevisionType;
use crate::services::page::{
    CreatePageOutput, EditPageOutput, GetPageOutput, RestorePageOutput,
};
use crate::services::revision::CreateRevisionOutput;

#[async_test]
async fn exists() -> Result<()> {
    let env = TestEnvironment::setup().await?;

    macro_rules! check {
        ($slug:expr, $exists:expr $(,)?) => {
            let path = format!("/page/{WWW_SITE_ID}/slug/{}", $slug);
            let actual_status = env.head(path)?.recv().await?;
            let expected_status = if $exists {
                StatusCode::NoContent
            } else {
                StatusCode::NotFound
            };

            assert_eq!(
                actual_status, expected_status,
                "Actual HTTP status doesn't match expect",
            );
        };
    }

    check!("start", true);
    check!("xyz", false);
    check!("system:members", true);
    check!("system:xyz", false);

    Ok(())
}

#[async_std::test]
async fn basic_create() -> Result<()> {
    let env = TestEnvironment::setup().await?;
    let slug = env.random_slug();

    // Create page
    let (output, status) = env
        .post(format!("/page/{WWW_SITE_ID}"))?
        .body_json(json!({
            "wikitext": "Page contents",
            "title": "Test page!",
            "altTitle": null,
            "slug": slug,
            "revisionComments": "Create page test",
            "userId": ADMIN_USER_ID,
        }))?
        .recv_json_serde::<CreatePageOutput>()
        .await?;

    let page_id = output.page_id;
    let revision_id = output.revision_id;
    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.slug, slug);
    assert!(output.parser_warnings.is_empty());

    // Check presence
    let status = env
        .head(format!("/page/{WWW_SITE_ID}/slug/{}", slug))?
        .recv()
        .await?;

    assert_eq!(status, StatusCode::NoContent);

    // Get page
    let (output, status) = env
        .get(format!("/page/{WWW_SITE_ID}/slug/{}", slug))?
        .recv_json_serde::<GetPageOutput>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.page_id, page_id);
    assert!(output.page_updated_at.is_none());
    assert!(output.page_deleted_at.is_none());
    assert_eq!(output.page_revision_count, 1);
    assert_eq!(output.site_id, WWW_SITE_ID);
    assert_eq!(output.page_category_slug, "_default");
    assert!(output.discussion_thread_id.is_none());
    assert_eq!(output.revision_id, revision_id);
    assert_eq!(output.revision_type, RevisionType::Create);
    assert_eq!(output.revision_number, 0);
    assert_eq!(output.revision_user_id, ADMIN_USER_ID);
    assert_eq!(output.revision_comments, "Create page test");
    assert_eq!(output.title, "Test page!");
    assert!(output.alt_title.is_none());
    assert_eq!(output.slug, slug);

    Ok(())
}

// TODO tests for edits

#[async_test]
async fn deletion_lifecycle() -> Result<()> {
    let env = TestEnvironment::setup().await?;
    let slug = env.random_slug();

    // Create
    let (output, status) = env
        .post(format!("/page/{WWW_SITE_ID}"))?
        .body_json(json!({
            "wikitext": "Apple banana",
            "title": "Test page!",
            "altTitle": null,
            "slug": slug,
            "revisionComments": "Create page",
            "userId": ADMIN_USER_ID,
        }))?
        .recv_json_serde::<CreatePageOutput>()
        .await?;

    let page_id = output.page_id;
    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.slug, slug);
    assert!(output.parser_warnings.is_empty());

    // Edit
    let (output, status) = env
        .post(format!("/page/{WWW_SITE_ID}/id/{page_id}"))?
        .body_json(json!({
            "wikitext": "Apple banana cherry",
            "revisionComments": "Edit page",
            "userId": REGULAR_USER_ID,
        }))?
        .recv_json_serde::<EditPageOutput>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_number, 1);
    assert!(output.parser_warnings.is_none());

    // Delete
    let (output, status) = env
        .delete(format!("/page/{WWW_SITE_ID}/id/{page_id}"))?
        .body_json(json!({
            "revisionComments": "Delete page",
            "userId": ADMIN_USER_ID,
        }))?
        .recv_json_serde::<CreateRevisionOutput>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_number, 2);
    assert!(output.parser_warnings.is_none());

    // Edit (fails)
    let status = env
        .post(format!("/page/{WWW_SITE_ID}/id/{page_id}"))?
        .body_json(json!({
            "wikitext": "Apple banana durian",
            "revisionComments": "Edit deleted page",
            "userId": REGULAR_USER_ID,
        }))?
        .recv()
        .await?;

    assert_eq!(status, StatusCode::NotFound);

    // Restore
    let (output, status) = env
        .post(format!("/page/{WWW_SITE_ID}/id/{page_id}/restore"))?
        .body_json(json!({
            "slug": null,
            "revisionComments": "Restore page",
            "userId": ADMIN_USER_ID,
        }))?
        .recv_json_serde::<RestorePageOutput>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_number, 3);
    assert!(output.parser_warnings.is_empty());

    // Edit again
    let (output, status) = env
        .post(format!("/page/{WWW_SITE_ID}/id/{page_id}"))?
        .body_json(json!({
            "wikitext": "Apple banana cherry",
            "revisionComments": "Edit page",
            "userId": REGULAR_USER_ID,
        }))?
        .recv_json_serde::<EditPageOutput>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_number, 1);
    assert!(output.parser_warnings.is_none());

    Ok(())
}

#[async_test]
async fn multiple_deleted() -> Result<()> {
    let env = TestEnvironment::setup().await?;
    let slug = env.random_slug();

    // Create, then delete multiple pages
    for i in 0..5 {
        // Create
        let (output, status) = env
            .post(format!("/page/{WWW_SITE_ID}"))?
            .body_json(json!({
                "wikitext": "Page contents",
                "title": "Test page!",
                "altTitle": null,
                "slug": slug,
                "revisionComments": format!("Create page {i}"),
                "userId": REGULAR_USER_ID,
            }))?
            .recv_json_serde::<CreatePageOutput>()
            .await?;

        assert_eq!(status, StatusCode::Ok);
        assert_eq!(output.slug, slug);
        assert!(output.parser_warnings.is_empty());

        // Delete
        let (output, status) = env
            .delete(format!("/page/{WWW_SITE_ID}"))?
            .body_json(json!({
                "revisionComments": format!("Delete page {i}"),
                "userId": ADMIN_USER_ID,
            }))?
            .recv_json_serde::<CreateRevisionOutput>()
            .await?;

        assert_eq!(status, StatusCode::Ok);
        assert_eq!(output.revision_number, 1);
        assert!(output.parser_warnings.is_none());
    }

    Ok(())
}
