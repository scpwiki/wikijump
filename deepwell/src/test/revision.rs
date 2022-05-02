/*
 * test/revision.rs
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
use crate::services::page::{EditPageOutput, GetPageOutput};
use std::borrow::Cow;

#[async_test]
async fn edits() -> Result<()> {
    let runner = Runner::setup().await?;
    let GeneratedPage { page_id, slug, .. } = runner.page().await?;

    // Edit wikitext via page ID
    let (output, status) = runner
        .post(format!("/page/{WWW_SITE_ID}/id/{page_id}"))?
        .body_json(json!({
            "wikitext": "Apple banana",
            "revisionComments": "Edit wikitext (page ID)",
            "userId": REGULAR_USER_ID,
        }))?
        .recv_json::<Option<EditPageOutput>>()
        .await?;

    let output = output.expect("No new revision created");
    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_number, 1);
    assert_eq!(output.parser_warnings, Some(vec![]));

    // Check page (full)
    let (output, status) = runner
        .get(format!("/page/{WWW_SITE_ID}/slug/{slug}?wikitext=true"))?
        .recv_json::<GetPageOutput>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.page_id, page_id);
    assert_eq!(output.slug, slug);
    assert!(output.page_updated_at.is_some());
    assert!(output.page_deleted_at.is_none());
    assert_eq!(output.wikitext.expect("No wikitext"), "Apple banana");
    assert_eq!(output.revision_type, RevisionType::Regular);
    assert_eq!(output.revision_number, 1);
    assert_eq!(output.page_revision_count, 2);
    assert_eq!(output.title, "Test page!");
    assert!(output.alt_title.is_none());

    // Edit wikitext via page slug
    let (output, status) = runner
        .post(format!("/page/{WWW_SITE_ID}/slug/{slug}"))?
        .body_json(json!({
            "wikitext": "Apple banana cherry",
            "revisionComments": "Edit wikitext (slug)",
            "userId": REGULAR_USER_ID,
        }))?
        .recv_json::<Option<EditPageOutput>>()
        .await?;

    let output = output.expect("No new revision created");
    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_number, 1);
    assert_eq!(output.parser_warnings, Some(vec![]));

    // Check page (wikitext)
    let (output, status) = runner
        .get(format!("/page/{WWW_SITE_ID}/slug/{slug}?wikitext=true"))?
        .recv_json::<GetPageOutput>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_type, RevisionType::Regular);
    assert_eq!(output.wikitext.expect("No wikitext"), "Apple banana cherry");
    assert_eq!(output.title, "Test page!");
    assert!(output.alt_title.is_none());

    // Edit title
    let (output, status) = runner
        .post(format!("/page/{WWW_SITE_ID}/id/{page_id}"))?
        .body_json(json!({
            "title": "A great page",
            "altTitle": "Fantastic",
            "revisionComments": "Edit title",
            "userId": REGULAR_USER_ID,
        }))?
        .recv_json::<Option<EditPageOutput>>()
        .await?;

    let output = output.expect("No new revision created");
    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_number, 2);
    assert_eq!(output.parser_warnings, Some(vec![]));

    // Check page (wikitext and title)
    let (output, status) = runner
        .get(format!("/page/{WWW_SITE_ID}/slug/{slug}?wikitext=true"))?
        .recv_json::<GetPageOutput>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_type, RevisionType::Regular);
    assert_eq!(output.revision_number, 2);
    assert_eq!(output.wikitext.expect("No wikitext"), "Apple banana cherry");
    assert_eq!(output.title, "A great page");
    assert_eq!(output.alt_title, Some(cow!("Fantastic")));
    assert!(output.alt_title.is_none());

    // Edit, remove alt title
    let (output, status) = runner
        .post(format!("/page/{WWW_SITE_ID}/id/{page_id}"))?
        .body_json(json!({
            "altTitle": null,
            "revisionComments": "Remove alt title",
            "userId": REGULAR_USER_ID,
        }))?
        .recv_json::<Option<EditPageOutput>>()
        .await?;

    let output = output.expect("No new revision created");
    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_number, 3);
    assert_eq!(output.parser_warnings, Some(vec![]));

    // Check page (wikitext and title)
    let (output, status) = runner
        .get(format!("/page/{WWW_SITE_ID}/slug/{slug}?wikitext=true"))?
        .recv_json::<GetPageOutput>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_type, RevisionType::Regular);
    assert_eq!(output.revision_number, 3);
    assert_eq!(output.wikitext.expect("No wikitext"), "Apple banana cherry");
    assert_eq!(output.title, "A great page");
    assert_eq!(output.alt_title, None);
    assert!(output.alt_title.is_none());

    // Edit (nothing, no revision)
    let (output, status) = runner
        .post(format!("/page/{WWW_SITE_ID}/id/{page_id}"))?
        .body_json(json!({
            "title": "A great page",
            "altTitle": null,
            "revisionComments": "Null edit",
            "userId": REGULAR_USER_ID,
        }))?
        .recv_json::<Option<EditPageOutput>>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert!(output.is_none());

    // Check page (revision number should be the same)
    let (output, status) = runner
        .get(format!("/page/{WWW_SITE_ID}/slug/{slug}"))?
        .recv_json::<GetPageOutput>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_type, RevisionType::Regular);
    assert_eq!(output.revision_number, 3);

    // Edit (tags)
    let (output, status) = runner
        .post(format!("/page/{WWW_SITE_ID}/id/{page_id}"))?
        .body_json(json!({
            "tags": ["apple"],
            "revisionComments": "Null edit",
            "userId": REGULAR_USER_ID,
        }))?
        .recv_json::<Option<EditPageOutput>>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert!(output.is_none());

    // Check page (tags)
    let (output, status) = runner
        .get(format!("/page/{WWW_SITE_ID}/slug/{slug}"))?
        .recv_json::<GetPageOutput>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_type, RevisionType::Regular);
    assert_eq!(output.revision_number, 4);
    assert_eq!(output.tags, json!(["apple"]));

    // Edit (nothing, no revision)
    let (output, status) = runner
        .post(format!("/page/{WWW_SITE_ID}/id/{page_id}"))?
        .body_json(json!({
            "tags": ["apple"],
            "revisionComments": "Null edit",
            "userId": REGULAR_USER_ID,
        }))?
        .recv_json::<Option<EditPageOutput>>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert!(output.is_none());

    // Check page (revision number should be the same)
    let (output, status) = runner
        .get(format!("/page/{WWW_SITE_ID}/slug/{slug}"))?
        .recv_json::<GetPageOutput>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_type, RevisionType::Regular);
    assert_eq!(output.revision_number, 4);
    assert_eq!(output.tags, json!(["apple"]));

    Ok(())
}
