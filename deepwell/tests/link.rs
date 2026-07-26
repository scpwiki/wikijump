/*
 * tests/link.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
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

#[macro_use]
mod common;

use self::common::TestRunner;
use deepwell::constants::{ADMIN_USER_ID, SYSTEM_USER_ID};
use deepwell::models::alias;
use deepwell::models::page::{self, Entity as PageTable};
use deepwell::services::alias::{AliasService, CreateAlias};
use deepwell::services::{LinkService, RequestContext};
use deepwell::types::{AliasType, ConnectionType, Reference};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_n() -> u64 {
    FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn set_page_actor(runner: &mut TestRunner, site_id: i64, page: Reference<'static>) {
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(page),
    });
}

async fn test_site_id(runner: &TestRunner) -> i64 {
    run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist")
        .site
        .site_id
}

async fn create_page(
    runner: &mut TestRunner,
    site_id: i64,
    slug: &str,
    wikitext: &str,
) -> page::Model {
    set_page_actor(
        runner,
        site_id,
        Reference::Slug(Cow::Owned(slug.to_owned())),
    );
    let output = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": wikitext,
            "title": slug,
            "alt_title": null,
            "slug": slug,
            "layout": "wikidot",
            "revision_comments": "create link resolver fixture",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert!(
        output.parser_errors.is_empty(),
        "unexpected parser errors: {:?}",
        output.parser_errors,
    );

    PageTable::find_by_id(output.page_id)
        .one(runner.context().transaction())
        .await
        .expect("page lookup should succeed")
        .expect("created page should exist")
}

#[tokio::test]
async fn repeated_explicit_site_includes_keep_count_and_timestamps() {
    let mut runner = TestRunner::setup().await;
    let n = next_n();
    let site_id = test_site_id(&runner).await;
    let target_slug = format!("fixture-link-batch-{n}-target");
    let consumer_slug = format!("fixture-link-batch-{n}-consumer");
    let target = create_page(&mut runner, site_id, &target_slug, "Batch target").await;
    let source = format!("[[include :test:{target_slug}]]\n").repeat(176);
    let consumer = create_page(&mut runner, site_id, &consumer_slug, &source).await;

    let before = LinkService::get_from(runner.context(), consumer.page_id)
        .await
        .expect("outgoing links should load");
    assert_eq!(before.present.len(), 1);
    let connection = &before.present[0];
    assert_eq!(connection.to_page_id, target.page_id);
    assert_eq!(connection.connection_type, ConnectionType::IncludeMessy);
    assert_eq!(connection.count, 176);

    run_endpoint!(
        runner,
        page_rerender,
        json!({
            "site_id": site_id,
            "category_id": consumer.page_category_id,
            "page_id": consumer.page_id,
        }),
    );
    let after = LinkService::get_from(runner.context(), consumer.page_id)
        .await
        .expect("outgoing links should load after rerender");
    assert_eq!(after.present[0].count, 176);
    assert_eq!(after.present[0].created_at, connection.created_at);
    assert_eq!(after.present[0].updated_at, connection.updated_at);
}

#[tokio::test]
async fn explicit_site_resolution_preserves_alias_and_missing_semantics() {
    let mut runner = TestRunner::setup().await;
    let n = next_n();
    let site_id = test_site_id(&runner).await;
    let target_slug = format!("fixture-link-semantics-{n}-target");
    let consumer_slug = format!("fixture-link-semantics-{n}-consumer");
    let alias_slug = format!("fixture-link-alias-{n}");
    let dangling_slug = format!("fixture-link-dangling-{n}");
    let missing_site_slug = format!("fixture-link-missing-site-{n}");
    let missing_page_slug = format!("fixture-link-missing-page-{n}");
    let target = create_page(&mut runner, site_id, &target_slug, "Semantic target").await;

    AliasService::create(
        runner.context(),
        CreateAlias {
            slug: alias_slug.clone(),
            alias_type: AliasType::Site,
            target_id: site_id,
            created_by: SYSTEM_USER_ID,
            bypass_filter: true,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("site alias should be created");
    alias::ActiveModel {
        alias_type: Set(AliasType::Site),
        created_by: Set(SYSTEM_USER_ID),
        target_id: Set(i64::MAX - i64::try_from(n).unwrap()),
        slug: Set(dangling_slug.clone()),
        ..Default::default()
    }
    .insert(runner.context().transaction())
    .await
    .expect("dangling alias fixture should be inserted");

    let source = format!(
        concat!(
            "[[include {target_slug}]]\n",
            "[[include :test:{target_slug}]]\n",
            "[[include :{alias_slug}:{target_slug}]]\n",
            "[[include :{alias_slug}:{target_slug}]]\n",
            "[[include :test:{missing_page_slug}]]\n",
            "[[include :{missing_site_slug}:{target_slug}]]\n",
            "[[include :{dangling_slug}:{target_slug}]]\n",
            "[[[{target_slug}|current link]]]\n",
            "[[[:test:{target_slug}|explicit link]]]\n"
        ),
        target_slug = target_slug,
        alias_slug = alias_slug,
        missing_page_slug = missing_page_slug,
        missing_site_slug = missing_site_slug,
        dangling_slug = dangling_slug,
    );
    let consumer = create_page(&mut runner, site_id, &consumer_slug, &source).await;
    let links = LinkService::get_from(runner.context(), consumer.page_id)
        .await
        .expect("outgoing links should load");

    let present = links
        .present
        .iter()
        .map(|connection| {
            (
                (connection.to_page_id, connection.connection_type),
                connection.count,
            )
        })
        .collect::<HashMap<_, _>>();
    assert_eq!(
        present,
        HashMap::from([
            ((target.page_id, ConnectionType::IncludeMessy), 4),
            ((target.page_id, ConnectionType::Link), 1),
        ]),
    );

    let missing = links
        .absent
        .iter()
        .map(|connection| {
            (
                (
                    connection.to_site_id,
                    connection.to_page_slug.clone(),
                    connection.connection_type,
                ),
                connection.count,
            )
        })
        .collect::<HashMap<_, _>>();
    assert_eq!(
        missing,
        HashMap::from([
            (
                (site_id, missing_page_slug, ConnectionType::IncludeMessy),
                1,
            ),
            (
                (site_id, format!("test:{target_slug}"), ConnectionType::Link,),
                1,
            ),
            (
                (
                    site_id,
                    format!(
                        "\u{1f}wikijump-cross-site\u{1f}{missing_site_slug}\u{1f}{target_slug}"
                    ),
                    ConnectionType::IncludeMessy,
                ),
                1,
            ),
            (
                (
                    site_id,
                    format!(
                        "\u{1f}wikijump-cross-site\u{1f}{dangling_slug}\u{1f}{target_slug}"
                    ),
                    ConnectionType::IncludeMessy,
                ),
                1,
            ),
        ]),
    );
}
