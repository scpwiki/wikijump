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
    assert_eq!(output.title, "Page title");
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
    assert_eq!(output.revision_number, 2);
    assert_eq!(output.parser_warnings, Some(vec![]));

    // Check page (wikitext)
    let (output, status) = runner
        .get(format!("/page/{WWW_SITE_ID}/slug/{slug}?wikitext=true"))?
        .recv_json::<GetPageOutput>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_type, RevisionType::Regular);
    assert_eq!(output.revision_number, 2);
    assert_eq!(output.wikitext.expect("No wikitext"), "Apple banana cherry");
    assert_eq!(output.title, "Page title");
    assert!(output.alt_title.is_none());

    // Edit title
    let (output, status) = runner
        .post(format!("/page/{WWW_SITE_ID}/id/{page_id}"))?
        .body_json(json!({
            "title": "Test page!",
            "altTitle": "A place to call home",
            "revisionComments": "Edit title",
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
    assert_eq!(output.title, "Test page!");
    assert_eq!(output.alt_title, Some(cow!("A place to call home")));

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
    assert_eq!(output.revision_number, 4);
    assert_eq!(output.parser_warnings, Some(vec![]));

    // Check page (wikitext and title)
    let (output, status) = runner
        .get(format!("/page/{WWW_SITE_ID}/slug/{slug}?wikitext=true"))?
        .recv_json::<GetPageOutput>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_type, RevisionType::Regular);
    assert_eq!(output.revision_number, 4);
    assert_eq!(output.wikitext.expect("No wikitext"), "Apple banana cherry");
    assert_eq!(output.title, "Test page!");
    assert_eq!(output.alt_title, None);
    assert!(output.alt_title.is_none());

    // Edit (nothing, no revision)
    let (output, status) = runner
        .post(format!("/page/{WWW_SITE_ID}/id/{page_id}"))?
        .body_json(json!({
            "title": "Test page!",
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
    assert_eq!(output.revision_number, 4);

    // Edit (tags)
    let (output, status) = runner
        .post(format!("/page/{WWW_SITE_ID}/id/{page_id}"))?
        .body_json(json!({
            "tags": ["apple"],
            "revisionComments": "Add tags",
            "userId": ADMIN_USER_ID,
        }))?
        .recv_json::<Option<EditPageOutput>>()
        .await?;

    let output = output.expect("No new revision created");
    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_number, 5);

    // Check page (tags)
    let (output, status) = runner
        .get(format!("/page/{WWW_SITE_ID}/slug/{slug}"))?
        .recv_json::<GetPageOutput>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_type, RevisionType::Regular);
    assert_eq!(output.revision_number, 5);
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
    assert_eq!(output.revision_number, 5);
    assert_eq!(output.tags, json!(["apple"]));

    // Edit (tags again)
    let (output, status) = runner
        .post(format!("/page/{WWW_SITE_ID}/id/{page_id}"))?
        .body_json(json!({
            "tags": ["apple", "banana"],
            "revisionComments": "More tags",
            "userId": ADMIN_USER_ID,
        }))?
        .recv_json::<Option<EditPageOutput>>()
        .await?;

    let output = output.expect("No new revision created");
    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_number, 6);

    // Check page (tags again)
    let (output, status) = runner
        .get(format!("/page/{WWW_SITE_ID}/slug/{slug}"))?
        .recv_json::<GetPageOutput>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_type, RevisionType::Regular);
    assert_eq!(output.revision_number, 6);
    assert_eq!(output.tags, json!(["apple", "banana"]));

    Ok(())
}

#[async_test]
async fn big_page() -> Result<()> {
    const INSERT_ITERATIONS: i32 = 5;
    const EXPANSION_ITERATIONS: i32 = 128;

    let runner = Runner::setup().await?;
    let GeneratedPage { page_id, .. } = runner.page().await?;

    // Build large wikitext
    let mut body = str!("++ Very large page\n\n");

    for _ in 0..EXPANSION_ITERATIONS {
        body.push_str("alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega\n");
        body.push_str("Α α, Β β, Γ γ, Δ δ, Ε ε, Ζ ζ, Η η, Θ θ, Ι ι, Κ κ, Λ λ, Μ μ, Ν ν, Ξ ξ, Ο ο, Π π, Ρ ρ, Σ σ/ς, Τ τ, Υ υ, Φ φ, Χ χ, Ψ ψ, Ω ω.\n\n");
    }

    // Insert multiple times, increasing the size
    for revision_number in 0..INSERT_ITERATIONS {
        let (output, status) = runner
            .post(format!("/page/{WWW_SITE_ID}/id/{page_id}"))?
            .body_json(json!({
                "wikitext": body,
                "tags": ["big"],
                "revisionComments": "Append more text",
                "userId": REGULAR_USER_ID,
            }))?
            .recv_json::<Option<EditPageOutput>>()
            .await?;

        let output = output.expect("No new revision created");
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(output.revision_number, revision_number);

        for _ in 0..EXPANSION_ITERATIONS {
            body.push_str("[[div]]................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................[[/div]]\n");
        }
    }

    // Check wikitext matches
    let (output, status) = runner
        .get(format!("/page/{WWW_SITE_ID}/id/{page_id}?wikitext=true"))?
        .recv_json::<GetPageOutput>()
        .await?;

    assert_eq!(status, StatusCode::Ok);
    assert_eq!(output.revision_type, RevisionType::Regular);
    assert_eq!(output.revision_number, INSERT_ITERATIONS);
    assert_eq!(output.wikitext.expect("No wikitext"), body);

    Ok(())
}
