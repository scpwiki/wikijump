/*
 * tests/rated_pages.rs
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
use deepwell::constants::{ADMIN_USER_ID, SAMPLE_USER_ID, SYSTEM_USER_ID};
use deepwell::services::forum::{CreateForumCategory, CreateForumGroup};
use deepwell::services::forum_post::CreateForumPost;
use deepwell::services::forum_thread::CreateForumThread;
use deepwell::services::{
    ForumPostService, ForumService, ForumThreadService, RequestContext,
};
use deepwell::types::Reference;
use sea_orm::{ConnectionTrait, Statement, Value};
use serde_json::json;

async fn set_page_created_at(runner: &TestRunner, page_id: i64, created_at: &str) {
    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_string(
            transaction.get_database_backend(),
            format!(
                "UPDATE \"page\" SET created_at = TIMESTAMPTZ '{created_at}' \
                 WHERE page_id = {page_id}",
            ),
        ))
        .await
        .expect("failed to set deterministic page creation timestamp");
}

async fn insert_vote(runner: &TestRunner, page_id: i64, user_id: i64, value: i16) {
    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            "INSERT INTO page_vote (from_wikidot, page_id, user_id, value) \
             VALUES (false, $1, $2, $3)",
            [
                Value::from(page_id),
                Value::from(user_id),
                Value::from(value),
            ],
        ))
        .await
        .expect("rating fixture should receive its stored point vote");
}

async fn insert_discussion_comments(
    runner: &TestRunner,
    site_id: i64,
    page_id: i64,
    count: usize,
) {
    let group = ForumService::create_group(
        runner.context(),
        CreateForumGroup {
            site_id,
            user_id: ADMIN_USER_ID,
            name: "RatedPages comments fixture group".to_owned(),
            description: "RatedPages comments fixture group".to_owned(),
            visible: true,
            sort_index: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("RatedPages comments forum group should be created");
    let category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "RatedPages comments fixture category".to_owned(),
            description: "RatedPages comments fixture category".to_owned(),
            sort_index: None,
            max_nest_level: Some(3),
            per_page_discussion: Some(true),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("RatedPages comments forum category should be created");
    let thread = ForumThreadService::create(
        runner.context(),
        CreateForumThread {
            forum_category_id: category.forum_category_id,
            user_id: ADMIN_USER_ID,
            associated_page_id: Some(page_id),
            title: "RatedPages comments fixture thread".to_owned(),
            description: String::new(),
            sticky: false,
            from_wikidot: false,
        },
    )
    .await
    .expect("RatedPages comments thread should be created");

    for index in 0..count {
        let post = ForumPostService::create(
            runner.context(),
            CreateForumPost {
                forum_thread_id: thread.forum_thread_id,
                parent_post_id: None,
                user_id: ADMIN_USER_ID,
                title: format!("RatedPages comment {}", index + 1),
                wikitext: format!("RatedPages controlled comment {}", index + 1),
                comments: "create RatedPages comment fixture".to_owned(),
                from_wikidot: false,
            },
        )
        .await
        .expect("RatedPages comment should be created");
        assert!(post.parser_errors.is_empty());
    }
}

fn set_page_context(runner: &mut TestRunner, site_id: i64, slug: &'static str) {
    runner.set_request_context(RequestContext {
        session: None,
        user_id: Some(ADMIN_USER_ID),
        site_id: Some(site_id),
        page_reference: Some(Reference::Slug(slug.into())),
    });
}

async fn create_page(
    runner: &mut TestRunner,
    site_id: i64,
    slug: &'static str,
    title: &str,
    created_at: &str,
) -> i64 {
    set_page_context(runner, site_id, slug);
    let page = run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": "RatedPages target",
            "title": title,
            "alt_title": null,
            "slug": slug,
            "layout": "wikidot",
            "revision_comments": "RatedPages module fixture",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );
    set_page_created_at(runner, page.page_id, created_at).await;
    page.page_id
}

#[tokio::test]
async fn ratedpages_renders_live_top_rated_box_and_rating_filters() {
    const CATEGORY: &str = "ratedpages-basic";
    const HIGH: &str = "ratedpages-basic:high";
    const ZERO: &str = "ratedpages-basic:zero";
    const LOW: &str = "ratedpages-basic:low";
    const HOLDER: &str = "ratedpages-holder";

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    let high_id = create_page(
        &mut runner,
        site_id,
        HIGH,
        "Run RatedPages High",
        "2026-07-28T00:00:01Z",
    )
    .await;
    create_page(
        &mut runner,
        site_id,
        ZERO,
        "Run RatedPages Zero",
        "2026-07-28T00:00:02Z",
    )
    .await;
    let low_id = create_page(
        &mut runner,
        site_id,
        LOW,
        "Run RatedPages Low",
        "2026-07-28T00:00:03Z",
    )
    .await;

    insert_vote(&runner, high_id, ADMIN_USER_ID, 1).await;
    insert_vote(&runner, high_id, SAMPLE_USER_ID, 1).await;
    insert_vote(&runner, low_id, SYSTEM_USER_ID, -1).await;
    insert_discussion_comments(&runner, site_id, high_id, 2).await;

    let source = format!(
        r#"[[div class="ratedpages-case default"]]
DEFAULT
[[module RatedPages category="{CATEGORY}" limit="3"]]
[[/div]]
[[div class="ratedpages-case rating-asc"]]
RATING ASC
[[module RatedPages category="{CATEGORY}" order="rating-asc" limit="3"]]
[[/div]]
[[div class="ratedpages-case rate-asc"]]
RATE ASC
[[module RatedPages category="{CATEGORY}" order="rate-asc" limit="3"]]
[[/div]]
[[div class="ratedpages-case created-desc"]]
CREATED DESC
[[module RatedPages category="{CATEGORY}" order="date-created-desc" limit="3"]]
[[/div]]
[[div class="ratedpages-case created-asc"]]
CREATED ASC
[[module RatedPages category="{CATEGORY}" order="date-created-asc" limit="3"]]
[[/div]]
[[div class="ratedpages-case min-rating"]]
MIN RATING
[[module RatedPages category="{CATEGORY}" minRating="1" limit="5"]]
[[/div]]
[[div class="ratedpages-case max-rating"]]
MAX RATING
[[module RatedPages category="{CATEGORY}" maxRating="-1" limit="5"]]
[[/div]]
[[div class="ratedpages-case comments-true"]]
COMMENTS TRUE
[[module RatedPages category="{CATEGORY}" comments="true" minRating="2" limit="1"]]
[[/div]]
[[div class="ratedpages-case comments-false"]]
COMMENTS FALSE
[[module RatedPages category="{CATEGORY}" comments="false" minRating="2" limit="1"]]
[[/div]]
[[div class="ratedpages-case comments-uppercase-value"]]
COMMENTS UPPERCASE VALUE
[[module RatedPages category="{CATEGORY}" comments="TRUE" minRating="2" limit="1"]]
[[/div]]
[[div class="ratedpages-case comments-empty"]]
COMMENTS EMPTY
[[module RatedPages category="{CATEGORY}" comments="" minRating="2" limit="1"]]
[[/div]]
[[div class="ratedpages-case comments-bare"]]
COMMENTS BARE
[[module RatedPages category="{CATEGORY}" comments minRating="2" limit="1"]]
[[/div]]
[[div class="ratedpages-case comments-uppercase-key"]]
COMMENTS UPPERCASE KEY
[[module RatedPages category="{CATEGORY}" Comments="true" minRating="2" limit="1"]]
[[/div]]
[[div class="ratedpages-case comments-last-empty"]]
COMMENTS LAST EMPTY
[[module RatedPages category="{CATEGORY}" comments="true" comments="" minRating="2" limit="1"]]
[[/div]]"#,
    );
    set_page_context(&mut runner, site_id, HOLDER);
    run_endpoint!(
        runner,
        page_create,
        json!({
            "site_id": site_id,
            "wikitext": source,
            "title": "Run RatedPages Holder",
            "alt_title": null,
            "slug": HOLDER,
            "layout": "wikidot",
            "revision_comments": "RatedPages module holder",
            "user_id": ADMIN_USER_ID,
            "bypass_filter": true,
            "ip_address": common::IP_ADDRESS,
        }),
    );

    let page = deepwell::endpoints::all::page_get(
        runner.context(),
        common::make_params(json!({
            "site_id": site_id,
            "page": HOLDER,
            "details": {
                "compiled": true
            },
        })),
    )
    .await
    .expect("RatedPages holder page_get should succeed")
    .expect("RatedPages holder should exist");
    let html = page
        .compiled_body_html
        .expect("RatedPages holder should have compiled HTML");

    assert!(
        html.contains(r#"<div class="top-rated-pages-box">"#)
            && html.contains(r#"<div class="top-rated-pages-list">"#)
            && html.contains(r#"<div class="list-item">"#),
        "RatedPages should render Wikidot's top-rated list DOM:\n{html}",
    );
    assert!(
        html.contains(r#"<a href="/ratedpages-basic:high">Run RatedPages High</a>"#)
            && html.contains(r#"<span style="color: #777">(Rating: 2)</span>"#),
        "RatedPages should render linked titles with the live rating label:\n{html}",
    );
    let section = |start: &str, end: &str| {
        let start_index = html.find(start).expect("section start should exist");
        let end_index = html[start_index..]
            .find(end)
            .map(|offset| start_index + offset)
            .expect("section end should exist");
        &html[start_index..end_index]
    };
    let final_section = |start: &str| {
        let start_index = html.find(start).expect("section start should exist");
        &html[start_index..]
    };
    let default_section = section("DEFAULT", "RATING ASC");
    assert!(
        default_section.contains("Run RatedPages High")
            && default_section.contains("Run RatedPages Zero")
            && default_section.contains("Run RatedPages Low"),
        "default RatedPages output should include all controlled pages:\n{html}",
    );
    let default_high = default_section.find("Run RatedPages High").unwrap();
    let default_zero = default_section.find("Run RatedPages Zero").unwrap();
    let default_low = default_section.find("Run RatedPages Low").unwrap();
    assert!(
        default_high < default_zero && default_zero < default_low,
        "default RatedPages order should be rating-desc:\n{html}",
    );
    let rating_asc_section = section("RATING ASC", "RATE ASC");
    assert!(
        rating_asc_section.find("Run RatedPages Low").unwrap()
            < rating_asc_section.find("Run RatedPages High").unwrap(),
        "rating-asc should put the negative page before the positive page:\n{html}",
    );
    assert!(
        html.contains("RATE ASC") && html.contains("Run RatedPages Low"),
        "the documented example spelling rate-asc should be accepted like rating-asc:\n{html}",
    );
    let created_desc_section = section("CREATED DESC", "CREATED ASC");
    assert!(
        created_desc_section.find("Run RatedPages Low").unwrap()
            < created_desc_section.find("Run RatedPages Zero").unwrap()
            && created_desc_section.find("Run RatedPages Zero").unwrap()
                < created_desc_section.find("Run RatedPages High").unwrap(),
        "date-created-desc should sort by newest first:\n{html}",
    );
    let created_asc_section = section("CREATED ASC", "MIN RATING");
    assert!(
        created_asc_section.find("Run RatedPages High").unwrap()
            < created_asc_section.find("Run RatedPages Zero").unwrap()
            && created_asc_section.find("Run RatedPages Zero").unwrap()
                < created_asc_section.find("Run RatedPages Low").unwrap(),
        "date-created-asc should sort by oldest first:\n{html}",
    );
    let min_section = section("MIN RATING", "MAX RATING");
    assert!(
        min_section.contains("Run RatedPages High")
            && !min_section.contains("Run RatedPages Zero")
            && !min_section.contains("Run RatedPages Low"),
        "minRating=1 should include only positive matching rows:\n{html}",
    );
    let max_section = section("MAX RATING", "COMMENTS TRUE");
    assert!(
        max_section.contains("Run RatedPages Low")
            && !max_section.contains("Run RatedPages Zero")
            && !max_section.contains("Run RatedPages High"),
        "maxRating=-1 should include only negative matching rows:\n{html}",
    );
    let comments_true_section = section("COMMENTS TRUE", "COMMENTS FALSE");
    assert!(
        comments_true_section.contains("(Rating: 2, Comments: 2)"),
        "comments=true should append the live comments label:\n{html}",
    );
    let comments_false_section = section("COMMENTS FALSE", "COMMENTS UPPERCASE VALUE");
    assert!(
        comments_false_section.contains("(Rating: 2, Comments: 2)"),
        "live Wikidot treats any final non-empty exact comments value as enabled:\n{html}",
    );
    let comments_uppercase_value_section =
        section("COMMENTS UPPERCASE VALUE", "COMMENTS EMPTY");
    assert!(
        comments_uppercase_value_section.contains("(Rating: 2, Comments: 2)"),
        "comments values are non-empty string switches, not case-sensitive booleans:\n{html}",
    );
    let comments_empty_section = section("COMMENTS EMPTY", "COMMENTS BARE");
    assert!(
        comments_empty_section.contains("(Rating: 2)")
            && !comments_empty_section.contains("Comments:"),
        "comments=\"\" should disable the comments label:\n{html}",
    );
    let comments_bare_section = section("COMMENTS BARE", "COMMENTS UPPERCASE KEY");
    assert!(
        comments_bare_section.contains("(Rating: 2)")
            && !comments_bare_section.contains("Comments:"),
        "a bare comments token should not enable the comments label:\n{html}",
    );
    let comments_uppercase_key_section =
        section("COMMENTS UPPERCASE KEY", "COMMENTS LAST EMPTY");
    assert!(
        comments_uppercase_key_section.contains("(Rating: 2)")
            && !comments_uppercase_key_section.contains("Comments:"),
        "live Wikidot ignores uppercase Comments for RatedPages:\n{html}",
    );
    let comments_last_empty_section = final_section("COMMENTS LAST EMPTY");
    assert!(
        comments_last_empty_section.contains("(Rating: 2)")
            && !comments_last_empty_section.contains("Comments:"),
        "the final exact comments value controls comment-label presence:\n{html}",
    );
    assert!(
        !html.contains("[[module RatedPages"),
        "compiled RatedPages output should not leak raw module markup:\n{html}",
    );
}
