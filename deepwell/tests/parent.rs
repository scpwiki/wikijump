/*
 * tests/parent.rs
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
use deepwell::constants::ADMIN_USER_ID;
use deepwell::services::RequestContext;
use deepwell::services::job::JOB_QUEUE_NAME;
use deepwell::types::Reference;
use rsmq_async::RsmqConnection;
use serde_json::json;
use std::borrow::Cow;

async fn queued_job_count(runner: &TestRunner) -> u64 {
    runner
        .context()
        .rsmq()
        .get_queue_attributes(JOB_QUEUE_NAME)
        .await
        .expect("job queue attributes should be readable")
        .totalsent
}

async fn create_page(
    runner: &mut TestRunner,
    site_id: i64,
    slug: &'static str,
    wikitext: &str,
) -> i64 {
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(slug))),
    });
    let created = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": wikitext,
            "title": slug,
            "alt_title": null,
            "slug": slug,
            "layout": "wikidot",
            "revision_comments": "create parent outdate fixture",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    created.revision_id
}

#[tokio::test]
async fn changed_parent_relationships_queue_parent_rerenders() {
    const PARENT_SLUG: &str = "fixture-parent-outdate-listpages";
    const CHILD_SLUG: &str = "fixture-parent-outdate-child";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    create_page(
        &mut runner,
        site_id,
        PARENT_SLUG,
        "[[module ListPages parent=\".\"]]\n%%fullname%%\n[[/module]]",
    )
    .await;
    let child_revision =
        create_page(&mut runner, site_id, CHILD_SLUG, "Child body").await;

    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(Cow::Borrowed(CHILD_SLUG))),
    });

    let before_create = queued_job_count(&runner).await;
    let created = run_endpoint!(
        runner,
        parent_set,
        json!({
            "site_id": site_id,
            "parent": PARENT_SLUG,
            "child": CHILD_SLUG,
        }),
    );
    assert!(created.is_some());
    assert_eq!(queued_job_count(&runner).await, before_create + 1);

    let duplicate = run_endpoint!(
        runner,
        parent_set,
        json!({
            "site_id": site_id,
            "parent": PARENT_SLUG,
            "child": CHILD_SLUG,
        }),
    );
    assert!(duplicate.is_none());
    assert_eq!(queued_job_count(&runner).await, before_create + 1);

    let removed = run_endpoint!(
        runner,
        parent_remove,
        json!({
            "site_id": site_id,
            "parent": PARENT_SLUG,
            "child": CHILD_SLUG,
        }),
    );
    assert!(removed.was_deleted);
    assert_eq!(queued_job_count(&runner).await, before_create + 2);

    let absent = run_endpoint!(
        runner,
        parent_remove,
        json!({
            "site_id": site_id,
            "parent": PARENT_SLUG,
            "child": CHILD_SLUG,
        }),
    );
    assert!(!absent.was_deleted);
    assert_eq!(queued_job_count(&runner).await, before_create + 2);

    let recreated = run_endpoint!(
        runner,
        parent_set,
        json!({
            "site_id": site_id,
            "parent": PARENT_SLUG,
            "child": CHILD_SLUG,
        }),
    );
    assert!(recreated.is_some());
    assert_eq!(queued_job_count(&runner).await, before_create + 3);

    run_endpoint!(
        runner,
        page_delete,
        json!({
            "site_id": site_id,
            "page": CHILD_SLUG,
            "last_revision_id": child_revision,
            "revision_comments": "delete parent outdate fixture child",
            "user_id": ADMIN_USER_ID,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    assert_eq!(queued_job_count(&runner).await, before_create + 4);
}
