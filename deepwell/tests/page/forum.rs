/*
 * tests/page/forum.rs
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

//! Forum runtime module integration tests.
//!
//! Extracted from `tests/page.rs` to keep the integration-test crate root
//! focused. These cases exercise the forum-oriented runtime modules (ForumMini,
//! Forum, RecentThreads, Comments, FrontForum, ForumStart, and RecentPosts)
//! through the public JSONRPC API. Shared crate-root imports and helpers are
//! reused via `super::*`.

use super::*;

#[tokio::test]
async fn forum_mini_modules_match_live_order_limits_routes_and_owner_boundaries() {
    async fn load_forum_mini_page_view(
        runner: &mut TestRunner,
        site_id: i64,
        slug: &str,
        user_id: Option<i64>,
        session_token: Option<&str>,
    ) -> String {
        runner.set_request_context(RequestContext {
            user_id,
            site_id: Some(site_id),
            page_reference: Some(Reference::Slug(Cow::Owned(slug.to_owned()))),
            ..Default::default()
        });
        match run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": session_token,
                "route": {"slug": slug, "extra": ""},
                "locales": ["en-US", "en"],
            }),
        ) {
            GetPageViewOutput::Found {
                compiled_body_html, ..
            } => compiled_body_html,
            other => {
                panic!("expected found forum mini page view for {slug}, got {other:?}")
            }
        }
    }

    async fn create_thread_with_replies(
        runner: &TestRunner,
        category_id: i64,
        title: &str,
        root_title: &str,
        reply_titles: &[&str],
    ) -> (i64, Vec<i64>) {
        let thread = ForumThreadService::create(
            runner.context(),
            CreateForumThread {
                forum_category_id: category_id,
                user_id: SAMPLE_USER_ID,
                associated_page_id: None,
                title: title.to_owned(),
                description: String::new(),
                sticky: false,
                from_wikidot: false,
            },
        )
        .await
        .expect("forum mini fixture thread should be created");
        let root = ForumPostService::create(
            runner.context(),
            CreateForumPost {
                forum_thread_id: thread.forum_thread_id,
                parent_post_id: None,
                user_id: SAMPLE_USER_ID,
                title: root_title.to_owned(),
                wikitext: format!("{root_title} body"),
                comments: "create forum mini root fixture".to_owned(),
                from_wikidot: false,
            },
        )
        .await
        .expect("forum mini fixture root should be created");
        let mut reply_ids = Vec::with_capacity(reply_titles.len());
        for reply_title in reply_titles {
            let reply = ForumPostService::create(
                runner.context(),
                CreateForumPost {
                    forum_thread_id: thread.forum_thread_id,
                    parent_post_id: Some(root.forum_post_id),
                    user_id: SAMPLE_USER_ID,
                    title: (*reply_title).to_owned(),
                    wikitext: format!("{reply_title} body <unsafe>"),
                    comments: "create forum mini reply fixture".to_owned(),
                    from_wikidot: false,
                },
            )
            .await
            .expect("forum mini fixture reply should be created");
            reply_ids.push(reply.forum_post_id);
        }
        (thread.forum_thread_id, reply_ids)
    }

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist");
    let site_id = site.site.site_id;
    let visible_group = ForumService::create_group(
        runner.context(),
        CreateForumGroup {
            site_id,
            user_id: ADMIN_USER_ID,
            name: "Forum mini visible group".to_owned(),
            description: "Visible forum mini fixture".to_owned(),
            visible: true,
            sort_index: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("visible forum mini group should be created");
    let visible_category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: visible_group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "Forum mini visible category".to_owned(),
            description: "Visible forum mini category fixture".to_owned(),
            sort_index: None,
            max_nest_level: Some(3),
            per_page_discussion: Some(false),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("visible forum mini category should be created");

    for (case_id, source) in [
        ("mini-recent-threads-empty", "[[module MiniRecentThreads]]"),
        ("mini-active-threads-empty", "[[module MiniActiveThreads]]"),
        ("mini-recent-posts-empty", "[[module MiniRecentPosts]]"),
    ] {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": source,
            }),
        );
        assert!(
            preview.body.contains(r#"<div class="forum-mini-stat""#)
                && preview.body.matches(r#"<div class="item""#).count() == 0,
            "{case_id}: {}",
            preview.body,
        );
    }

    let (high_thread_id, high_reply_ids) = create_thread_with_replies(
        &runner,
        visible_category.forum_category_id,
        "High Activity Thread",
        "High Root",
        &["High Reply One", "High Reply Two", "High Reply Three"],
    )
    .await;

    for (slug, title, source) in [
        (
            "fixture-mini-recent-threads",
            "Fixture Mini Recent Threads",
            "[[module MiniRecentThreads limit=\"2\"]]",
        ),
        (
            "fixture-mini-active-threads",
            "Fixture Mini Active Threads",
            "[[module MiniActiveThreads limit=\"2\"]]",
        ),
        (
            "fixture-mini-recent-posts",
            "Fixture Mini Recent Posts",
            "[[module MiniRecentPosts limit=\"3\"]]",
        ),
    ] {
        create_listpages_test_page(&mut runner, site_id, slug, title, source).await;
    }

    let (low_thread_id, low_reply_ids) = create_thread_with_replies(
        &runner,
        visible_category.forum_category_id,
        "Low Recent Thread",
        "Low Root",
        &["Low Newest Reply"],
    )
    .await;

    const PRIVATE_PAGE_SLUG: &str = "fixture-forum-mini-private-page";
    const PRIVATE_PAGE_CATEGORY: &str = "fixture-forum-mini-private-category";
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_PAGE_CATEGORY)
        .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        PRIVATE_PAGE_SLUG,
        "Forum Mini Private Page",
        "private forum mini page fixture",
    )
    .await;
    set_listpages_test_category_slug(
        &runner,
        site_id,
        PRIVATE_PAGE_SLUG,
        PRIVATE_PAGE_CATEGORY,
    )
    .await;
    let private_page_id =
        listpages_test_page_id(&runner, site_id, PRIVATE_PAGE_SLUG).await;
    let private_page_thread = ForumThreadService::create(
        runner.context(),
        CreateForumThread {
            forum_category_id: visible_category.forum_category_id,
            user_id: ADMIN_USER_ID,
            associated_page_id: Some(private_page_id),
            title: "Private Page Activity Marker".to_owned(),
            description: String::new(),
            sticky: false,
            from_wikidot: false,
        },
    )
    .await
    .expect("private-page forum mini thread should be created");
    let private_page_post = ForumPostService::create(
        runner.context(),
        CreateForumPost {
            forum_thread_id: private_page_thread.forum_thread_id,
            parent_post_id: None,
            user_id: ADMIN_USER_ID,
            title: "Private Page Post Marker".to_owned(),
            wikitext: "private page post marker body".to_owned(),
            comments: "create private-page forum mini fixture".to_owned(),
            from_wikidot: false,
        },
    )
    .await
    .expect("private-page forum mini post should be created");
    ForumPostService::create(
        runner.context(),
        CreateForumPost {
            forum_thread_id: private_page_thread.forum_thread_id,
            parent_post_id: Some(private_page_post.forum_post_id),
            user_id: ADMIN_USER_ID,
            title: "Private Page Reply Marker".to_owned(),
            wikitext: "private page reply marker body".to_owned(),
            comments: "create private-page forum mini reply fixture".to_owned(),
            from_wikidot: false,
        },
    )
    .await
    .expect("private-page forum mini reply should be created");

    let hidden_group = ForumService::create_group(
        runner.context(),
        CreateForumGroup {
            site_id,
            user_id: ADMIN_USER_ID,
            name: "Forum mini hidden group".to_owned(),
            description: "Hidden forum mini fixture".to_owned(),
            visible: false,
            sort_index: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("hidden forum mini group should be created");
    let hidden_category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: hidden_group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "Forum mini hidden category".to_owned(),
            description: "Hidden forum mini category fixture".to_owned(),
            sort_index: None,
            max_nest_level: Some(3),
            per_page_discussion: Some(false),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("hidden forum mini category should be created");
    create_thread_with_replies(
        &runner,
        hidden_category.forum_category_id,
        "Hidden Newest Thread",
        "Hidden Root",
        &[
            "Hidden Reply One",
            "Hidden Reply Two",
            "Hidden Reply Three",
            "Hidden Reply Four",
        ],
    )
    .await;

    runner.set_request_context(RequestContext::default());

    let recent_threads = load_forum_mini_page_view(
        &mut runner,
        site_id,
        "fixture-mini-recent-threads",
        None,
        None,
    )
    .await;
    let low_recent_position = recent_threads
        .find("Low Recent Thread")
        .expect("newer visible thread should render");
    let high_recent_position = recent_threads
        .find("High Activity Thread")
        .expect("older visible thread should render");
    assert!(
        low_recent_position < high_recent_position,
        "{recent_threads}"
    );
    assert!(
        recent_threads.contains(&format!(
            r#"href="/forum/t-{low_thread_id}/low-recent-thread""#
        )) && recent_threads.contains("Posts: 1"),
        "{recent_threads}",
    );
    assert!(
        !recent_threads.contains("Hidden Newest Thread")
            && !recent_threads.contains("Private Page Activity Marker")
            && recent_threads.contains("class=\"odate time_")
            && recent_threads.contains("format_%25O%20ago"),
        "{recent_threads}",
    );

    let active_threads = load_forum_mini_page_view(
        &mut runner,
        site_id,
        "fixture-mini-active-threads",
        None,
        None,
    )
    .await;
    let high_active_position = active_threads
        .find("High Activity Thread")
        .expect("more active visible thread should render");
    let low_active_position = active_threads
        .find("Low Recent Thread")
        .expect("less active visible thread should render");
    assert!(
        high_active_position < low_active_position,
        "{active_threads}"
    );
    assert!(
        active_threads.contains(&format!(
            r#"href="/forum/t-{high_thread_id}/high-activity-thread""#
        )) && active_threads.contains("Posts: 3")
            && !active_threads.contains("Hidden Newest Thread")
            && !active_threads.contains("Private Page Activity Marker"),
        "{active_threads}",
    );
    assert!(active_threads.contains("</span> ,"), "{active_threads}",);

    let recent_posts = load_forum_mini_page_view(
        &mut runner,
        site_id,
        "fixture-mini-recent-posts",
        None,
        None,
    )
    .await;
    let low_reply_id = low_reply_ids[0];
    assert!(
        recent_posts.contains(&format!(
            r#"href="/forum/t-{low_thread_id}/low-recent-thread#post-{low_reply_id}""#
        )) && recent_posts.contains("Low Newest Reply")
            && recent_posts.contains("Low Newest Reply body &lt;unsafe&gt;")
            && recent_posts.contains("class=\"printuser\""),
        "{recent_posts}",
    );
    assert!(
        !recent_posts.contains("Low Root")
            && !recent_posts.contains("High Root")
            && !recent_posts.contains("Hidden Reply")
            && !recent_posts.contains("Private Page Reply Marker")
            && recent_posts.contains(&format!("#post-{}", high_reply_ids[2])),
        "{recent_posts}",
    );

    // Seal the actor and deletion boundary independently of the small live-observation
    // limits above. The matrix holder deliberately requests a large local limit so
    // permission filtering must happen before row limiting and cannot be hidden by a
    // coincidentally full public prefix.
    const MATRIX_HOLDER: &str = "fixture-forum-mini-actor-deletion-matrix";
    create_listpages_test_page(
        &mut runner,
        site_id,
        MATRIX_HOLDER,
        "Forum Mini Actor Deletion Matrix",
        concat!(
            "[[module MiniRecentThreads limit=\"100\"]]\n",
            "[[module MiniActiveThreads limit=\"100\"]]\n",
            "[[module MiniRecentPosts limit=\"100\"]]",
        ),
    )
    .await;

    let (deleted_thread_id, _) = create_thread_with_replies(
        &runner,
        visible_category.forum_category_id,
        "Deleted Thread Marker",
        "Deleted Thread Root",
        &["Deleted Thread Reply Marker"],
    )
    .await;
    let deleted_thread = ForumThreadTable::find_by_id(deleted_thread_id)
        .one(runner.context().transaction())
        .await
        .expect("forum-mini deleted-thread fixture lookup should succeed")
        .expect("forum-mini deleted-thread fixture should exist");
    let mut deleted_thread = deleted_thread.into_active_model();
    deleted_thread.deleted_by = Set(Some(ADMIN_USER_ID));
    deleted_thread.deleted_at = Set(Some(OffsetDateTime::now_utc()));
    deleted_thread.updated_by = Set(Some(ADMIN_USER_ID));
    deleted_thread.updated_at = Set(Some(OffsetDateTime::now_utc()));
    deleted_thread
        .update(runner.context().transaction())
        .await
        .expect("forum-mini deleted-thread fixture should be soft-deleted");

    let (post_delete_thread_id, post_delete_reply_ids) = create_thread_with_replies(
        &runner,
        visible_category.forum_category_id,
        "Deleted Post Control Thread",
        "Deleted Post Control Root",
        &["Deleted Post Marker"],
    )
    .await;
    let deleted_post_id = post_delete_reply_ids[0];
    ForumPostService::delete(
        runner.context(),
        DeleteForumPost {
            forum_post_id: deleted_post_id,
            user_id: ADMIN_USER_ID,
        },
    )
    .await
    .expect("forum-mini deleted-post fixture should be soft-deleted");
    assert!(
        ForumThreadTable::find_by_id(post_delete_thread_id)
            .one(runner.context().transaction())
            .await
            .expect("deleted-post control thread lookup should succeed")
            .is_some(),
        "deleting one post must not delete its control thread",
    );

    RelationService::create_site_member(
        runner.context(),
        CreateSiteMember {
            site_id,
            user_id: SAMPLE_USER_ID,
            metadata: SiteMemberData {
                accepted: SiteMemberAccepted::Accepted(SYSTEM_USER_ID),
            },
            created_by: SYSTEM_USER_ID,
        },
        common::IP_ADDRESS,
    )
    .await
    .expect("forum-mini member actor should be created");
    let member_session = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: SAMPLE_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "forum-mini actor matrix member".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("forum-mini member session should be created");
    let admin_session = SessionService::create(
        runner.context(),
        CreateSession {
            user_id: ADMIN_USER_ID,
            ip_address: common::IP_ADDRESS,
            user_agent: "forum-mini actor matrix administrator".to_owned(),
            restricted: false,
        },
    )
    .await
    .expect("forum-mini administrator session should be created");

    let anonymous_matrix =
        load_forum_mini_page_view(&mut runner, site_id, MATRIX_HOLDER, None, None).await;
    let member_matrix = load_forum_mini_page_view(
        &mut runner,
        site_id,
        MATRIX_HOLDER,
        Some(SAMPLE_USER_ID),
        Some(member_session.as_str()),
    )
    .await;
    for (actor, body) in [("anonymous", &anonymous_matrix), ("member", &member_matrix)] {
        assert!(
            !body.contains("Private Page Activity Marker")
                && !body.contains("Private Page Reply Marker"),
            "{actor} must not receive private page forum activity:\n{body}",
        );
    }

    let moderator_role = RoleService::get(
        runner.context(),
        site_id,
        Reference::Slug(Cow::Borrowed("moderator")),
    )
    .await
    .expect("seeded moderator role should exist");
    RoleService::grant_role_to_user(
        runner.context(),
        GrantUserRoleInput {
            user_id: SAMPLE_USER_ID,
            role_id: moderator_role.role_id,
            site_id,
            assigning_user_id: SYSTEM_USER_ID,
            expires_at: None,
            ip_address: common::IP_ADDRESS,
        },
    )
    .await
    .expect("forum-mini member should become a moderator");
    let private_category_id =
        CategoryService::get_or_create(runner.context(), site_id, PRIVATE_PAGE_CATEGORY)
            .await
            .expect("forum-mini private category should still exist")
            .category_id;
    role_permission::ActiveModel {
        role_id: Set(moderator_role.role_id),
        site_id: Set(site_id),
        resource_type: Set(Resource::Page),
        resource_category_id: Set(Some(private_category_id)),
        action: Set(Action::View),
        ..Default::default()
    }
    .insert(runner.context().transaction())
    .await
    .expect("forum-mini moderator private-view permission should be inserted");
    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("forum-mini actor permission cache should invalidate");

    let moderator_matrix = load_forum_mini_page_view(
        &mut runner,
        site_id,
        MATRIX_HOLDER,
        Some(SAMPLE_USER_ID),
        Some(member_session.as_str()),
    )
    .await;
    let administrator_matrix = load_forum_mini_page_view(
        &mut runner,
        site_id,
        MATRIX_HOLDER,
        Some(ADMIN_USER_ID),
        Some(admin_session.as_str()),
    )
    .await;
    for (actor, body) in [
        ("moderator", &moderator_matrix),
        ("administrator", &administrator_matrix),
    ] {
        assert!(
            body.contains("Private Page Activity Marker")
                && body.contains("Private Page Reply Marker"),
            "{actor} with explicit page visibility must receive private page forum activity:\n{body}",
        );
    }
    for (actor, body) in [
        ("anonymous", &anonymous_matrix),
        ("member", &member_matrix),
        ("moderator", &moderator_matrix),
        ("administrator", &administrator_matrix),
    ] {
        assert!(
            !body.contains("Hidden Newest Thread")
                && !body.contains("Hidden Reply")
                && !body.contains("Deleted Thread Marker")
                && !body.contains("Deleted Thread Reply Marker")
                && !body.contains("Deleted Post Marker")
                && !body.contains(&format!("#post-{deleted_post_id}")),
            "{actor} must not receive hidden or deleted forum activity:\n{body}",
        );
    }
    let post_delete_control = ForumThreadTable::find_by_id(post_delete_thread_id)
        .one(runner.context().transaction())
        .await
        .expect("deleted-post control thread lookup should succeed after matrix")
        .expect("deleted-post control thread should still exist after matrix");
    let mut post_delete_control = post_delete_control.into_active_model();
    post_delete_control.deleted_by = Set(Some(ADMIN_USER_ID));
    post_delete_control.deleted_at = Set(Some(OffsetDateTime::now_utc()));
    post_delete_control.updated_by = Set(Some(ADMIN_USER_ID));
    post_delete_control.updated_at = Set(Some(OffsetDateTime::now_utc()));
    post_delete_control
        .update(runner.context().transaction())
        .await
        .expect("deleted-post control thread should be retired after matrix");
    runner.set_request_context(RequestContext::default());

    for (case_id, source, expected_items) in [
        (
            "mini-recent-threads-bare",
            "[[module MiniRecentThreads]]",
            2usize,
        ),
        (
            "mini-recent-threads-limit-one",
            "[[module MiniRecentThreads limit=\"1\"]]",
            1usize,
        ),
        (
            "mini-recent-threads-limit-zero",
            "[[module MiniRecentThreads limit=\"0\"]]",
            2usize,
        ),
        (
            "mini-recent-threads-limit-negative",
            "[[module MiniRecentThreads limit=\"-1\"]]",
            2usize,
        ),
        (
            "mini-recent-threads-limit-text",
            "[[module MiniRecentThreads limit=\"abc\"]]",
            2usize,
        ),
        (
            "mini-recent-threads-unknown-argument",
            "[[module MiniRecentThreads unknown=\"x\"]]",
            2usize,
        ),
        (
            "mini-active-threads-bare",
            "[[module MiniActiveThreads]]",
            2usize,
        ),
        (
            "mini-active-threads-limit-one",
            "[[module MiniActiveThreads limit=\"1\"]]",
            1usize,
        ),
        (
            "mini-active-threads-limit-zero",
            "[[module MiniActiveThreads limit=\"0\"]]",
            2usize,
        ),
        (
            "mini-active-threads-limit-negative",
            "[[module MiniActiveThreads limit=\"-1\"]]",
            2usize,
        ),
        (
            "mini-active-threads-limit-text",
            "[[module MiniActiveThreads limit=\"abc\"]]",
            2usize,
        ),
        (
            "mini-active-threads-unknown-argument",
            "[[module MiniActiveThreads unknown=\"x\"]]",
            2usize,
        ),
        (
            "mini-recent-posts-bare",
            "[[module MiniRecentPosts]]",
            4usize,
        ),
        (
            "mini-recent-posts-limit-one",
            "[[module MiniRecentPosts limit=\"1\"]]",
            1usize,
        ),
        (
            "mini-recent-posts-limit-zero",
            "[[module MiniRecentPosts limit=\"0\"]]",
            4usize,
        ),
        (
            "mini-recent-posts-limit-negative",
            "[[module MiniRecentPosts limit=\"-1\"]]",
            4usize,
        ),
        (
            "mini-recent-posts-limit-text",
            "[[module MiniRecentPosts limit=\"abc\"]]",
            4usize,
        ),
        (
            "mini-recent-posts-unknown-argument",
            "[[module MiniRecentPosts unknown=\"x\"]]",
            4usize,
        ),
    ] {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": source,
            }),
        );
        assert_eq!(
            preview.body.matches(r#"<div class="item""#).count(),
            expected_items,
            "{case_id}: {}",
            preview.body,
        );
    }

    for (case_id, source) in [
        (
            "mini-recent-threads-inline",
            "before [[module MiniRecentThreads limit=\"1\"]] after",
        ),
        (
            "mini-recent-threads-literal",
            "@@[[module MiniRecentThreads limit=\"1\"]]@@",
        ),
        (
            "mini-active-threads-inline",
            "before [[module MiniActiveThreads limit=\"1\"]] after",
        ),
        (
            "mini-active-threads-literal",
            "@@[[module MiniActiveThreads limit=\"1\"]]@@",
        ),
        (
            "mini-recent-posts-inline",
            "before [[module MiniRecentPosts limit=\"1\"]] after",
        ),
        (
            "mini-recent-posts-literal",
            "@@[[module MiniRecentPosts limit=\"1\"]]@@",
        ),
    ] {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": source,
            }),
        );
        assert!(
            preview.body.contains("[[module")
                && !preview.body.contains("forum-mini-stat")
                && !preview.body.contains("No such module"),
            "{case_id}: {}",
            preview.body,
        );
    }
}

#[tokio::test]
async fn forum_modules_match_live_missing_context_and_owner_boundaries() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (case_id, source, expected) in [
        (
            "forum-comments-no-context",
            "[[module Comments]]",
            concat!(
                r#"<div class="comments-box"><div class="options" id="comments-options-hidden" >"#,
                r#"<a href="javascript:;" onclick="WIKIDOT.modules.ForumCommentsModule.listeners.showComments(event)">Show Comments</a>"#,
                r#"</div><div id="thread-container" class="thread-container" style="margin-top: 1em"></div></div>"#,
            ),
        ),
        (
            "forum-front-no-category",
            "[[module FrontForum]]",
            concat!(
                r#"<div class="error-block">No forum category has been specified. "#,
                r#"Please use attribute category="id" where id is the index number of the category.</div>"#,
            ),
        ),
        (
            "forum-category-no-context",
            "[[module ForumCategory]]",
            r#"<div class="error-block">No forum category has been specified.</div>"#,
        ),
        (
            "forum-new-thread-no-context",
            "[[module ForumNewThread]]",
            r#"<div class="error-block">No forum category has been specified.</div>"#,
        ),
        (
            "forum-thread-no-context",
            "[[module ForumThread]]",
            concat!(
                r#"<div class="error-block">No thread to show - click Back once or twice "#,
                r#"and try again</div>"#,
            ),
        ),
    ] {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": source,
            }),
        );
        assert!(
            preview.body.contains(expected) && !preview.body.contains("[[module"),
            "{case_id}: {}",
            preview.body,
        );
    }

    for (case_id, source, expected_heading, reverse) in [
        (
            "comments-title",
            r#"[[module Comments title="Alpha Heading"]]"#,
            Some("<h1>Alpha Heading</h1>"),
            false,
        ),
        (
            "comments-escaped-title",
            r#"[[module Comments title="<Tag & Text>"]]"#,
            Some("<h1>&lt;Tag &amp; Text&gt;</h1>"),
            false,
        ),
        (
            "comments-mixed-module-case",
            r#"[[MoDuLe cOmMeNtS title="Mixed Module"]]"#,
            Some("<h1>Mixed Module</h1>"),
            false,
        ),
        (
            "comments-reverse",
            r#"[[module Comments order="reverse"]]"#,
            None,
            true,
        ),
        (
            "comments-reverse-with-title",
            r#"[[module Comments title="Reverse Two" order="reverse"]]"#,
            Some("<h1>Reverse Two</h1>"),
            true,
        ),
        (
            "comments-forwards",
            r#"[[module Comments order="forwards"]]"#,
            None,
            false,
        ),
    ] {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": source,
            }),
        );
        assert!(
            preview.body.contains(r#"<div class="comments-box">"#)
                && preview.body.contains(r#"id="comments-options-hidden""#)
                && preview.body.contains(r#"id="thread-container""#)
                && !preview.body.contains("comments-options-shown")
                && !preview.body.contains("thread-container-posts")
                && !preview.body.contains("[[module"),
            "{case_id}: {}",
            preview.body,
        );
        assert_eq!(
            preview.body.contains(r#"class="thread-container reverse""#),
            reverse,
            "{case_id}: {}",
            preview.body,
        );
        match expected_heading {
            Some(heading) => assert!(
                preview.body.contains(heading),
                "{case_id}: {}",
                preview.body,
            ),
            None => assert!(
                !preview.body.contains("<h1>"),
                "{case_id}: {}",
                preview.body,
            ),
        }
    }

    for (case_id, source) in [
        ("comments-title-empty", r#"[[module Comments title=""]]"#),
        (
            "comments-title-single-quoted",
            "[[module Comments title='Alpha Heading']]",
        ),
        ("comments-title-bare", "[[module Comments title=Alpha]]"),
        (
            "comments-title-key-case",
            r#"[[module Comments Title="Alpha Heading"]]"#,
        ),
        (
            "comments-order-forward-singular",
            r#"[[module Comments order="forward"]]"#,
        ),
        (
            "comments-order-uppercase",
            r#"[[module Comments order="REVERSE"]]"#,
        ),
        ("comments-order-empty", r#"[[module Comments order=""]]"#),
        (
            "comments-order-single-quoted",
            "[[module Comments order='reverse']]",
        ),
        ("comments-order-bare", "[[module Comments order=reverse]]"),
        (
            "comments-order-key-case",
            r#"[[module Comments Order="reverse"]]"#,
        ),
        (
            "comments-hide-single-quoted",
            "[[module Comments hide='true']]",
        ),
        ("comments-hide-bare", "[[module Comments hide=true]]"),
        ("comments-hide-empty", r#"[[module Comments hide=""]]"#),
        (
            "comments-hide-key-case",
            r#"[[module Comments Hide="true"]]"#,
        ),
        (
            "comments-unknown-attribute",
            r#"[[module Comments unknown="x"]]"#,
        ),
    ] {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": source,
            }),
        );
        assert!(
            preview.body.contains(r#"<div class="comments-box">"#)
                && preview.body.contains(r#"class="thread-container""#)
                && !preview.body.contains(r#"class="thread-container reverse""#)
                && !preview.body.contains("<h1>")
                && !preview.body.contains("comments-options-shown")
                && !preview.body.contains("thread-container-posts")
                && !preview.body.contains("[[module"),
            "{case_id}: {}",
            preview.body,
        );
    }

    for (case_id, source) in [
        (
            "forum-start-inline-owner",
            "before [[module ForumStart]] after",
        ),
        ("recent-posts-raw-owner", "@@[[module RecentPosts]]@@"),
    ] {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": source,
            }),
        );
        assert!(
            preview.body.contains("[[module")
                && !preview.body.contains("forum-start-box")
                && !preview.body.contains("forum-recent-posts-box")
                && !preview.body.contains("error-block"),
            "{case_id}: {}",
            preview.body,
        );
    }

    let unsupported = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "front-forum-unobserved-arguments",
            "wikitext": r#"[[module FrontForum category="9223372036854775807" feed="news"]]"#,
        }),
    );
    assert_eq!(
        unsupported.body,
        r#"<div class="error-block">Requested forum category does not exist.</div>"#,
        "an unobserved FrontForum query with a missing category must fail closed",
    );

    let unsupported_malformed_category = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "front-forum-unobserved-arguments-with-malformed-category",
            "wikitext": r#"[[module FrontForum category="bad" feed="news"]]"#,
        }),
    );
    assert!(
        unsupported_malformed_category.body.contains(
            r#"<div class="error-block">Problem parsing attribute "category".</div>"#
        ),
        "a malformed FrontForum category must retain the documented parser error: {}",
        unsupported_malformed_category.body,
    );

    let custom_body = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "front-forum-unobserved-custom-format",
            "wikitext": "[[module FrontForum category=\"1\"]]\n%%linked_title%%\n[[/module]]",
        }),
    );
    assert_eq!(
        custom_body.body,
        r#"<div class="error-block">Requested forum category does not exist.</div>"#,
        "a missing FrontForum category must fail closed before evaluating its custom body",
    );
}

#[tokio::test]
async fn recent_threads_matches_live_placeholder_and_owner_boundaries() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;

    for (case_id, source) in [
        ("recentthreads-bare", "[[module RecentThreads]]"),
        (
            "recentthreads-limit",
            r#"[[module RecentThreads limit="5"]]"#,
        ),
        (
            "recentthreads-unknown-argument",
            r#"[[module RecentThreads unknown="x"]]"#,
        ),
        ("recentthreads-mixed-case", "[[MoDuLe rEcEnTtHrEaDs]]"),
        (
            "recentthreads-body",
            "[[module RecentThreads]]\nRECENT_THREADS_BODY_MUST_BE_CONSUMED\n[[/module]]",
        ),
    ] {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": source,
            }),
        );
        assert_eq!(
            preview.body.matches("later.").count(),
            1,
            "{case_id}: {}",
            preview.body,
        );
        assert!(
            !preview.body.contains("[[module")
                && !preview.body.contains("No such module")
                && !preview
                    .body
                    .contains("RECENT_THREADS_BODY_MUST_BE_CONSUMED"),
            "{case_id}: {}",
            preview.body,
        );
    }

    for (case_id, source) in [
        (
            "recentthreads-inline",
            "before [[module RecentThreads]] after",
        ),
        ("recentthreads-raw-owner", "@@[[module RecentThreads]]@@"),
    ] {
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": source,
            }),
        );
        assert!(
            preview.body.contains("[[module") && !preview.body.contains("later."),
            "{case_id}: {}",
            preview.body,
        );
    }

    let lookalike = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "recentthreads-lookalike",
            "wikitext": "[[module RecentThreadsX]]",
        }),
    );
    assert!(
        lookalike.body.contains("No such module") && !lookalike.body.contains("later."),
        "{}",
        lookalike.body,
    );

    let unrelated_closer = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "recentthreads-unrelated-module-closer",
            "wikitext": concat!(
                "[[module RecentThreads]]\n",
                "RECENT_THREADS_BOUNDARY_BEFORE\n",
                "[[module FrontForum]]\n",
                "UNRELATED_FRONT_FORUM_BODY\n",
                "[[/module]]\n",
                "RECENT_THREADS_BOUNDARY_AFTER",
            ),
        }),
    );
    assert!(
        unrelated_closer.body.contains("later.")
            && unrelated_closer
                .body
                .contains("RECENT_THREADS_BOUNDARY_BEFORE")
            && unrelated_closer
                .body
                .contains("RECENT_THREADS_BOUNDARY_AFTER")
            && unrelated_closer.body.contains("No such module"),
        "RecentThreads must not consume a later module's body closer: {}",
        unrelated_closer.body,
    );
}

#[test]
fn forum_comments_list_resolves_only_visible_page_discussions() {
    // The imported-depth fixture exceeds the test harness's 2 MiB async poll
    // stack. Keep it on the bounded runtime used by the adjacent forum fixture.
    std::thread::Builder::new()
        .name("forum-comments-list-test".to_owned())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("forum comments list test runtime should build")
                .block_on(
                    forum_comments_list_resolves_only_visible_page_discussions_impl(),
                );
        })
        .expect("forum comments list test thread should spawn")
        .join()
        .expect("forum comments list test thread should complete");
}

async fn forum_comments_list_resolves_only_visible_page_discussions_impl() {
    async fn create_comment(
        runner: &TestRunner,
        forum_thread_id: i64,
        parent_post_id: Option<i64>,
        number: usize,
    ) -> i64 {
        ForumPostService::create(
            runner.context(),
            CreateForumPost {
                forum_thread_id,
                parent_post_id,
                user_id: SAMPLE_USER_ID,
                title: format!("Page Comment {number:02}"),
                wikitext: "Shared page comment body <observable>".to_owned(),
                comments: "create page comments read-model fixture".to_owned(),
                from_wikidot: false,
            },
        )
        .await
        .expect("page comment fixture should be created")
        .forum_post_id
    }

    async fn point_page_at_discussion(
        runner: &TestRunner,
        page_id: i64,
        forum_thread_id: i64,
    ) {
        let page = PageTable::find_by_id(page_id)
            .one(runner.context().transaction())
            .await
            .expect("page discussion fixture lookup should succeed")
            .expect("page discussion fixture should exist");
        let mut page = page.into_active_model();
        page.discussion_thread_id = Set(Some(forum_thread_id));
        page.update(runner.context().transaction())
            .await
            .expect("page discussion fixture should be linked");
    }

    async fn create_discussion_page(
        runner: &mut TestRunner,
        site_id: i64,
        forum_category_id: i64,
        slug: &str,
        title: &str,
        source: &str,
    ) -> (i64, i64) {
        create_listpages_test_page(runner, site_id, slug, title, source).await;
        let page_id = listpages_test_page_id(runner, site_id, slug).await;
        let thread = ForumThreadService::create(
            runner.context(),
            CreateForumThread {
                forum_category_id,
                user_id: SAMPLE_USER_ID,
                associated_page_id: Some(page_id),
                title: format!("{title} Thread"),
                description: String::new(),
                sticky: false,
                from_wikidot: false,
            },
        )
        .await
        .expect("page discussion fixture thread should be created");
        point_page_at_discussion(runner, page_id, thread.forum_thread_id).await;
        (page_id, thread.forum_thread_id)
    }

    async fn saved_comments_body(
        runner: &TestRunner,
        site_id: i64,
        slug: &str,
    ) -> String {
        let saved_page = run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": null,
                "route": {"slug": slug, "extra": ""},
                "locales": ["en-US", "en"],
            }),
        );
        match saved_page {
            GetPageViewOutput::Found {
                compiled_body_html, ..
            } => compiled_body_html,
            other => {
                panic!("expected a found Comments attribute page view, got {other:?}")
            }
        }
    }

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let transaction = runner.context().transaction();
    transaction
        .execute_raw(Statement::from_sql_and_values(
            transaction.get_database_backend(),
            concat!(
                "INSERT INTO wikidot_user (",
                "user_id, created_at, fetched_at, is_deleted, name, slug, karma, is_pro",
                ") VALUES ($1, NOW() - INTERVAL '1 second', NOW(), FALSE, ",
                "'Comments Imported User', 'comments-imported-user', 3, FALSE) ",
                "ON CONFLICT (user_id) DO UPDATE SET is_deleted = FALSE, ",
                "name = EXCLUDED.name, slug = EXCLUDED.slug, karma = EXCLUDED.karma",
            ),
            [Value::from(SAMPLE_USER_ID)],
        ))
        .await
        .expect("page comments Wikidot user fixture should be inserted");
    let group = ForumService::create_group(
        runner.context(),
        CreateForumGroup {
            site_id,
            user_id: ADMIN_USER_ID,
            name: "Page Comments Read Model Group".to_owned(),
            description: String::new(),
            visible: true,
            sort_index: Some(20_000),
            from_wikidot: false,
        },
    )
    .await
    .expect("page comments forum group should be created");
    let category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "Page Comments Read Model".to_owned(),
            description: String::new(),
            sort_index: Some(10),
            max_nest_level: Some(2),
            per_page_discussion: Some(true),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("page comments forum category should be created");

    const PUBLIC_PAGE: &str = "fixture-forum-comments-public";
    create_listpages_test_page(
        &mut runner,
        site_id,
        PUBLIC_PAGE,
        "Public Page Comments Fixture",
        "[[module Comments]]",
    )
    .await;
    let public_page_id = listpages_test_page_id(&runner, site_id, PUBLIC_PAGE).await;
    let public_thread = ForumThreadService::create(
        runner.context(),
        CreateForumThread {
            forum_category_id: category.forum_category_id,
            user_id: SAMPLE_USER_ID,
            associated_page_id: Some(public_page_id),
            title: "Public Page Comments Thread".to_owned(),
            description: String::new(),
            sticky: false,
            from_wikidot: false,
        },
    )
    .await
    .expect("public page discussion should be created");
    point_page_at_discussion(&runner, public_page_id, public_thread.forum_thread_id)
        .await;
    let mut root_comment_ids = Vec::new();
    for number in 0..12 {
        root_comment_ids.push(
            create_comment(&runner, public_thread.forum_thread_id, None, number).await,
        );
    }
    let early_reply = create_comment(
        &runner,
        public_thread.forum_thread_id,
        Some(root_comment_ids[0]),
        100,
    )
    .await;
    create_comment(
        &runner,
        public_thread.forum_thread_id,
        Some(early_reply),
        101,
    )
    .await;
    create_comment(
        &runner,
        public_thread.forum_thread_id,
        Some(root_comment_ids[11]),
        200,
    )
    .await;

    let (_, forwards_thread_id) = create_discussion_page(
        &mut runner,
        site_id,
        category.forum_category_id,
        "fixture-forum-comments-forwards-attributes",
        "Page Comments Forwards Attributes",
        r#"[[module Comments title="<Attribute & Forward>" hide="false" order="forwards"]]"#,
    )
    .await;
    create_comment(&runner, forwards_thread_id, None, 400).await;

    let (_, reverse_thread_id) = create_discussion_page(
        &mut runner,
        site_id,
        category.forum_category_id,
        "fixture-forum-comments-reverse-attributes",
        "Page Comments Reverse Attributes",
        r#"[[module Comments title="Reverse Saved" order="reverse"]]"#,
    )
    .await;
    create_comment(&runner, reverse_thread_id, None, 410).await;
    create_comment(&runner, reverse_thread_id, None, 411).await;

    let (_, hidden_thread_id) = create_discussion_page(
        &mut runner,
        site_id,
        category.forum_category_id,
        "fixture-forum-comments-hidden",
        "Page Comments Hidden",
        r#"[[module Comments hide="true"]]"#,
    )
    .await;
    create_comment(&runner, hidden_thread_id, None, 420).await;

    let (_, _hide_form_denied_thread_id) = create_discussion_page(
        &mut runner,
        site_id,
        category.forum_category_id,
        "fixture-forum-comments-hideform-denied",
        "Page Comments Hide Form Denied",
        r#"[[module Comments hideForm="true"]]"#,
    )
    .await;

    let (_, invalid_thread_id) = create_discussion_page(
        &mut runner,
        site_id,
        category.forum_category_id,
        "fixture-forum-comments-invalid-attributes",
        "Page Comments Invalid Attributes",
        "[[module Comments title='Invalid Heading' hide='true' order=reverse]]",
    )
    .await;
    create_comment(&runner, invalid_thread_id, None, 430).await;

    let (deep_page_id, deep_thread_id) = create_discussion_page(
        &mut runner,
        site_id,
        category.forum_category_id,
        "fixture-forum-comments-depth-overflow",
        "Page Comments Depth Overflow",
        "[[module Comments]]",
    )
    .await;
    let mut parent_post_id = create_comment(&runner, deep_thread_id, None, 300).await;
    let mut deep_post_ids = vec![parent_post_id];
    for number in 301..=430 {
        parent_post_id =
            create_comment(&runner, deep_thread_id, Some(parent_post_id), number).await;
        deep_post_ids.push(parent_post_id);
    }
    // ForumPostService intentionally caps ordinary replies at the category's
    // nesting setting. Rewire this defensive read fixture to model an imported
    // relation graph that exceeds the renderer's independent safety boundary.
    for pair in deep_post_ids.windows(2) {
        runner
            .context()
            .transaction()
            .execute_raw(Statement::from_sql_and_values(
                runner.context().transaction().get_database_backend(),
                "UPDATE forum_post SET parent_post_id = $1 WHERE forum_post_id = $2",
                [Value::from(pair[0]), Value::from(pair[1])],
            ))
            .await
            .expect("deep imported comment fixture should be rewired");
    }

    let deleted_category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "Deleted Page Comments Category".to_owned(),
            description: String::new(),
            sort_index: Some(11),
            max_nest_level: Some(2),
            per_page_discussion: Some(true),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("deleted page comments category should be created");
    let (deleted_category_page_id, _) = create_discussion_page(
        &mut runner,
        site_id,
        deleted_category.forum_category_id,
        "fixture-forum-comments-deleted-category",
        "Deleted Category Page Comments",
        "[[module Comments]]",
    )
    .await;
    ForumService::delete_category(
        runner.context(),
        DeleteForumCategory {
            forum_category_id: deleted_category.forum_category_id,
            user_id: ADMIN_USER_ID,
        },
    )
    .await
    .expect("page comments category fixture should be deleted");

    let deleted_group = ForumService::create_group(
        runner.context(),
        CreateForumGroup {
            site_id,
            user_id: ADMIN_USER_ID,
            name: "Deleted Page Comments Group".to_owned(),
            description: String::new(),
            visible: true,
            sort_index: Some(20_001),
            from_wikidot: false,
        },
    )
    .await
    .expect("deleted page comments group should be created");
    let deleted_group_category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: deleted_group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "Deleted Group Page Comments Category".to_owned(),
            description: String::new(),
            sort_index: Some(10),
            max_nest_level: Some(2),
            per_page_discussion: Some(true),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("deleted-group page comments category should be created");
    let (deleted_group_page_id, _) = create_discussion_page(
        &mut runner,
        site_id,
        deleted_group_category.forum_category_id,
        "fixture-forum-comments-deleted-group",
        "Deleted Group Page Comments",
        "[[module Comments]]",
    )
    .await;
    ForumService::delete_group(
        runner.context(),
        DeleteForumGroup {
            forum_group_id: deleted_group.forum_group_id,
            user_id: ADMIN_USER_ID,
        },
    )
    .await
    .expect("page comments group fixture should be deleted");

    const PRIVATE_CATEGORY: &str = "forum-comments-private";
    const PRIVATE_PAGE: &str = "fixture-forum-comments-private";
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_CATEGORY).await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        PRIVATE_PAGE,
        "Private Page Comments Fixture",
        "private page comments fixture",
    )
    .await;
    set_listpages_test_category_slug(&runner, site_id, PRIVATE_PAGE, PRIVATE_CATEGORY)
        .await;
    let private_page_id = listpages_test_page_id(&runner, site_id, PRIVATE_PAGE).await;
    let private_thread = ForumThreadService::create(
        runner.context(),
        CreateForumThread {
            forum_category_id: category.forum_category_id,
            user_id: ADMIN_USER_ID,
            associated_page_id: Some(private_page_id),
            title: "Private Page Comments Thread".to_owned(),
            description: String::new(),
            sticky: false,
            from_wikidot: false,
        },
    )
    .await
    .expect("private page discussion should be created");
    point_page_at_discussion(&runner, private_page_id, private_thread.forum_thread_id)
        .await;
    create_comment(&runner, private_thread.forum_thread_id, None, 99).await;

    let foreign_site = run_endpoint!(runner, site_get, json!({"site": "scp-jp"}))
        .expect("seeded SCP-JP site should exist");
    let foreign_page_id =
        listpages_test_page_id(&runner, foreign_site.site.site_id, "boundary-check")
            .await;
    runner.set_request_context(RequestContext::default());

    let public_page_before = PageTable::find_by_id(public_page_id)
        .one(runner.context().transaction())
        .await
        .expect("public comments page snapshot should load")
        .expect("public comments page should exist");
    let public_thread_before =
        ForumThreadTable::find_by_id(public_thread.forum_thread_id)
            .one(runner.context().transaction())
            .await
            .expect("public comments thread snapshot should load")
            .expect("public comments thread should exist");
    let public_posts_before = ForumPostTable::find()
        .filter(forum_post::Column::ForumThreadId.eq(public_thread.forum_thread_id))
        .order_by_asc(forum_post::Column::ForumPostId)
        .all(runner.context().transaction())
        .await
        .expect("public comments post snapshot should load");

    let saved_page = run_endpoint!(
        runner,
        page_view,
        json!({
            "site_id": site_id,
            "session_token": null,
            "route": {"slug": PUBLIC_PAGE, "extra": ""},
            "locales": ["en-US", "en"],
        }),
    );
    let saved_body = match saved_page {
        GetPageViewOutput::Found {
            compiled_body_html, ..
        } => compiled_body_html,
        other => panic!("expected a found Comments page view, got {other:?}"),
    };
    assert!(
        saved_body.contains(r#"<div class="comments-box">"#)
            && saved_body.contains(r#"id="comments-options-hidden""#)
            && saved_body.contains(r#"id="thread-container""#)
            && saved_body.contains(r#"id="comments-options-shown""#)
            && saved_body.contains("Page Comment 00")
            && saved_body.contains("Page Comment 09")
            && !saved_body.contains(r#">Page Comment 10</div>"#)
            && !saved_body.contains("[[module Comments]]"),
        "saved page_view should embed the permission-filtered first Comments page:\n{saved_body}",
    );

    let forwards_body = saved_comments_body(
        &runner,
        site_id,
        "fixture-forum-comments-forwards-attributes",
    )
    .await;
    assert!(
        forwards_body.contains("<h1>&lt;Attribute &amp; Forward&gt;</h1>")
            && forwards_body
                .contains(r#"id="comments-options-hidden" style="display: none""#)
            && forwards_body.contains(r#"class="thread-container""#)
            && !forwards_body.contains(r#"class="thread-container reverse""#)
            && forwards_body.contains("Page Comment 400"),
        "exact hide=false and order=forwards should embed the forward page:\n{forwards_body}",
    );

    let reverse_body = saved_comments_body(
        &runner,
        site_id,
        "fixture-forum-comments-reverse-attributes",
    )
    .await;
    let comment_411 = reverse_body.find("Page Comment 411");
    let comment_410 = reverse_body.find("Page Comment 410");
    assert!(
        reverse_body.contains("<h1>Reverse Saved</h1>")
            && reverse_body.contains(r#"class="thread-container reverse""#)
            && comment_411
                .is_some_and(|position| comment_410.is_some_and(|other| position < other))
            && reverse_body.find("new-post-button")
                < reverse_body.find("comments-options-shown"),
        "exact order=reverse should embed the reverse page:\n{reverse_body}",
    );

    let hidden_body =
        saved_comments_body(&runner, site_id, "fixture-forum-comments-hidden").await;
    assert!(
        hidden_body.contains(r#"id="comments-options-hidden""#)
            && hidden_body.contains(r#"id="thread-container""#)
            && !hidden_body.contains("comments-options-shown")
            && !hidden_body.contains("thread-container-posts")
            && !hidden_body.contains("Page Comment 420"),
        "exact hide=true should keep the saved Comments shell inert:\n{hidden_body}",
    );

    let hide_form_denied_body =
        saved_comments_body(&runner, site_id, "fixture-forum-comments-hideform-denied")
            .await;
    assert!(
        hide_form_denied_body.contains(r#"id="comments-options-shown""#)
            && hide_form_denied_body.contains(r#"id="new-post-button""#)
            && !hide_form_denied_body.contains(r#"id="new-post-form""#),
        "a denied actor must retain the Comments link without an input form:\n{hide_form_denied_body}",
    );

    let invalid_body = saved_comments_body(
        &runner,
        site_id,
        "fixture-forum-comments-invalid-attributes",
    )
    .await;
    assert!(
        invalid_body.contains(r#"<div class="comments-box">"#)
            && invalid_body.contains(r#"class="thread-container""#)
            && !invalid_body.contains(r#"class="thread-container reverse""#)
            && !invalid_body.contains("<h1>")
            && !invalid_body.contains("comments-options-shown")
            && !invalid_body.contains("thread-container-posts")
            && !invalid_body.contains("Page Comment 430"),
        "unobserved saved attribute forms must fail closed without querying:\n{invalid_body}",
    );

    let forward = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumCommentsListModule",
            "parameters": {"pageId": public_page_id.to_string()},
        }),
    );
    assert_eq!(forward.status, "ok");
    assert_eq!(forward.thread_id, Some(public_thread.forum_thread_id));
    assert!(
        forward
            .body
            .contains(r#"<div class="options" id="comments-options-shown">"#)
            && forward
                .body
                .contains(r#"<div id="thread-container-posts" style="display: none">"#)
            && forward
                .body
                .contains(r#"<span class="pager-no">page 1 of 2</span>"#)
            && forward
                .body
                .matches(r#"<div class="post-container" id="fpc-"#)
                .count()
                == 12
            && forward.body.contains("Page Comment 00")
            && forward.body.contains("Page Comment 09")
            && forward.body.contains("Page Comment 100")
            && forward.body.contains("Page Comment 101")
            && forward
                .body
                .matches("Shared page comment body &lt;observable&gt;")
                .count()
                == 12
            && !forward.body.contains(r#">Page Comment 10</div>"#)
            && !forward.body.contains(r#">Page Comment 11</div>"#)
            && !forward.body.contains("Page Comment 200"),
        "{}",
        forward.body,
    );
    for expected in [
        format!(
            "https://www.wikidot.com/avatar.php?userid={SAMPLE_USER_ID}&amp;amp;size=small&amp;amp;timestamp="
        ),
        format!(
            "background-image:url(https://www.wikidot.com/userkarma.php?u={SAMPLE_USER_ID})"
        ),
    ] {
        assert!(
            forward.body.contains(&expected),
            "Comments must use the sealed HTTPS Wikidot user resources ({expected}):\n{}",
            forward.body,
        );
    }
    assert!(
        !forward.body.contains(&format!(
            "http://www.wikidot.com/avatar.php?userid={SAMPLE_USER_ID}"
        )) && !forward.body.contains(&format!(
            "background-image:url(http://www.wikidot.com/userkarma.php?u={SAMPLE_USER_ID})"
        )),
        "Comments must not downgrade imported Wikidot user resources:\n{}",
        forward.body,
    );
    assert!(
        forward.body.find("Page Comment 00") < forward.body.find("Page Comment 100")
            && forward.body.find("Page Comment 100")
                < forward.body.find("Page Comment 101")
            && forward.body.find("Page Comment 101")
                < forward.body.find("Page Comment 01"),
        "{}",
        forward.body,
    );
    assert!(
        forward.body.find("comments-options-shown")
            < forward.body.find("new-post-button"),
        "{}",
        forward.body,
    );
    assert_eq!(forward.js_include.len(), 3);
    assert!(
        forward.js_include[0].ends_with("/ForumViewThreadModule.js")
            && forward.js_include[1].ends_with("/ForumViewThreadPostsModule.js")
            && forward.js_include[2].ends_with("/ForumNewPostFormModule.js"),
        "{:?}",
        forward.js_include,
    );

    let reverse = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumCommentsListModule",
            "parameters": {
                "pageId": public_page_id.to_string(),
                "order": "reverse",
            },
        }),
    );
    assert_eq!(reverse.status, "ok");
    assert_eq!(reverse.thread_id, Some(public_thread.forum_thread_id));
    assert!(
        reverse.body.contains(r#">Page Comment 11</div>"#)
            && reverse.body.contains(r#">Page Comment 02</div>"#)
            && reverse.body.contains("Page Comment 200")
            && !reverse.body.contains(r#">Page Comment 00</div>"#)
            && !reverse.body.contains(r#">Page Comment 01</div>"#)
            && !reverse.body.contains("Page Comment 100")
            && !reverse.body.contains("Page Comment 101")
            && reverse
                .body
                .matches(r#"<div class="post-container" id="fpc-"#)
                .count()
                == 11
            && reverse.body.find("Page Comment 11")
                < reverse.body.find("Page Comment 200")
            && reverse.body.find("Page Comment 200")
                < reverse.body.find("Page Comment 10")
            && reverse.body.find("Page Comment 10")
                < reverse.body.find("Page Comment 09")
            && reverse.body.find("new-post-button")
                < reverse.body.find("comments-options-shown"),
        "{}",
        reverse.body,
    );
    assert_eq!(reverse.js_include.len(), 3);
    assert!(
        reverse.js_include[0].ends_with("/ForumViewThreadModule.js")
            && reverse.js_include[1].ends_with("/ForumNewPostFormModule.js")
            && reverse.js_include[2].ends_with("/ForumViewThreadPostsModule.js"),
        "{:?}",
        reverse.js_include,
    );

    let ordinary_thread = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumViewThreadModule",
            "parameters": {"t": public_thread.forum_thread_id.to_string()},
        }),
    );
    assert_eq!(ordinary_thread.status, "ok");
    assert!(
        ordinary_thread.body.contains(&format!(
            "http://www.wikidot.com/avatar.php?userid={SAMPLE_USER_ID}&amp;amp;size=small&amp;amp;timestamp="
        )) && ordinary_thread.body.contains(&format!(
            "background-image:url(http://www.wikidot.com/userkarma.php?u={SAMPLE_USER_ID})"
        )) && !ordinary_thread.body.contains(&format!(
            "https://www.wikidot.com/avatar.php?userid={SAMPLE_USER_ID}"
        )),
        "non-Comments forum surfaces must retain their sealed HTTP resource scheme:\n{}",
        ordinary_thread.body,
    );

    let depth_overflow = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumCommentsListModule",
            "parameters": {"pageId": deep_page_id.to_string()},
        }),
    );
    assert_eq!(depth_overflow.status, "not_ok");
    assert!(depth_overflow.body.is_empty());
    assert_eq!(depth_overflow.thread_id, None);
    assert!(depth_overflow.js_include.is_empty());

    let mut hidden_results = Vec::new();
    for page_id in [
        i64::MAX,
        private_page_id,
        foreign_page_id,
        deleted_category_page_id,
        deleted_group_page_id,
    ] {
        let output = run_endpoint!(
            runner,
            wikidot_forum_module,
            json!({
                "site_id": site_id,
                "module_name": "forum/ForumCommentsListModule",
                "parameters": {"pageId": page_id.to_string()},
            }),
        );
        hidden_results.push((
            output.status,
            output.body,
            output.thread_id,
            output.js_include,
        ));
    }
    assert!(
        hidden_results
            .iter()
            .all(|result| result == &hidden_results[0])
    );
    assert_eq!(hidden_results[0].0, "no_page");
    assert!(hidden_results[0].1.is_empty());
    assert_eq!(hidden_results[0].2, None);
    assert!(hidden_results[0].3.is_empty());

    let explicit_forwards = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumCommentsListModule",
            "parameters": {
                "pageId": public_page_id.to_string(),
                "order": "forwards",
            },
        }),
    );
    assert_eq!(explicit_forwards.status, "ok");
    assert_eq!(
        explicit_forwards.thread_id,
        Some(public_thread.forum_thread_id)
    );
    assert!(
        explicit_forwards.body.contains("Page Comment 00")
            && explicit_forwards.body.contains("Page Comment 09")
            && !explicit_forwards.body.contains(r#">Page Comment 10</div>"#)
            && explicit_forwards.js_include[0].ends_with("/ForumViewThreadModule.js")
            && explicit_forwards.js_include[1]
                .ends_with("/ForumViewThreadPostsModule.js")
            && explicit_forwards.js_include[2].ends_with("/ForumNewPostFormModule.js"),
        "exact order=forwards should select the forward first page:\n{}",
        explicit_forwards.body,
    );

    for page_id in [format!("+{public_page_id}"), format!("0{public_page_id}")] {
        let noncanonical = run_endpoint!(
            runner,
            wikidot_forum_module,
            json!({
                "site_id": site_id,
                "module_name": "forum/ForumCommentsListModule",
                "parameters": {"pageId": page_id},
            }),
        );
        assert_eq!(noncanonical.status, "no_page");
        assert!(noncanonical.body.is_empty());
        assert_eq!(noncanonical.thread_id, None);
        assert!(noncanonical.js_include.is_empty());
    }

    for order in [
        "forward",
        "REVERSE",
        "FORWARDS",
        "",
        "yes",
        "true",
        " reverse ",
    ] {
        let unsupported = run_endpoint!(
            runner,
            wikidot_forum_module,
            json!({
                "site_id": site_id,
                "module_name": "forum/ForumCommentsListModule",
                "parameters": {
                    "pageId": public_page_id.to_string(),
                    "order": order,
                },
            }),
        );
        assert_eq!(unsupported.status, "not_ok", "order={order:?}");
    }

    runner.set_request_context(RequestContext {
        site_id: Some(foreign_site.site.site_id),
        ..Default::default()
    });
    let mismatched_site = run_endpoint_err!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumCommentsListModule",
            "parameters": {"pageId": public_page_id.to_string()},
        }),
    );
    assert_contains_error!(mismatched_site, ErrorType::PermissionDenied);

    runner.set_request_context(RequestContext::default());
    let public_page_after = PageTable::find_by_id(public_page_id)
        .one(runner.context().transaction())
        .await
        .expect("public comments page snapshot should reload")
        .expect("public comments page should still exist");
    let public_thread_after = ForumThreadTable::find_by_id(public_thread.forum_thread_id)
        .one(runner.context().transaction())
        .await
        .expect("public comments thread snapshot should reload")
        .expect("public comments thread should still exist");
    let public_posts_after = ForumPostTable::find()
        .filter(forum_post::Column::ForumThreadId.eq(public_thread.forum_thread_id))
        .order_by_asc(forum_post::Column::ForumPostId)
        .all(runner.context().transaction())
        .await
        .expect("public comments post snapshot should reload");
    assert_eq!(public_page_after, public_page_before);
    assert_eq!(public_thread_after, public_thread_before);
    assert_eq!(public_posts_after, public_posts_before);
}

#[tokio::test]
async fn frontforum_custom_body_matches_observed_public_preview_boundaries() {
    fn custom_div_body<'a>(html: &'a str, class: &str) -> &'a str {
        let marker = format!(r#"<div class="{class}">"#);
        let (_, after_marker) = html
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {marker}: {html}"));
        after_marker
            .split_once("</div>")
            .unwrap_or_else(|| panic!("unclosed {marker}: {html}"))
            .0
    }

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let group = ForumService::create_group(
        runner.context(),
        CreateForumGroup {
            site_id,
            user_id: ADMIN_USER_ID,
            name: "Custom Body Group".to_owned(),
            description: "Custom body group description".to_owned(),
            visible: true,
            sort_index: Some(20_000),
            from_wikidot: false,
        },
    )
    .await
    .expect("custom-body forum group should be created");
    let category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "Custom Body Category".to_owned(),
            description: "Custom body category description".to_owned(),
            sort_index: Some(10),
            max_nest_level: Some(3),
            per_page_discussion: Some(false),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("custom-body forum category should be created");
    let thread = ForumThreadService::create(
        runner.context(),
        CreateForumThread {
            forum_category_id: category.forum_category_id,
            user_id: SAMPLE_USER_ID,
            associated_page_id: None,
            title: "Custom Body Thread".to_owned(),
            description: "Independent custom body description".to_owned(),
            sticky: false,
            from_wikidot: false,
        },
    )
    .await
    .expect("custom-body forum thread should be created");
    ForumPostService::create(
        runner.context(),
        CreateForumPost {
            forum_thread_id: thread.forum_thread_id,
            parent_post_id: None,
            user_id: SAMPLE_USER_ID,
            title: "Custom Body Post".to_owned(),
            wikitext: "Independent custom body content <observable>".to_owned(),
            comments: "custom-body public preview fixture".to_owned(),
            from_wikidot: false,
        },
    )
    .await
    .expect("custom-body first post should be created");
    runner.set_request_context(RequestContext::default());

    let output = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "front-forum custom-body fixture",
            "wikitext": format!(
                concat!(
                    "[[module FrontForum category=\"{}\" limit=\"1\"]]\n",
                    "[[div class=\"custom-title\"]]\n%%title%%\n[[/div]]\n",
                    "[[div class=\"custom-linked\"]]\n%%linked_title%%\n[[/div]]\n",
                    "[[div class=\"custom-link\"]]\n%%link%%\n[[/div]]\n",
                    "[[div class=\"custom-author\"]]\n%%author%%\n[[/div]]\n",
                    "[[div class=\"custom-date\"]]\n%%date|%Y-%m-%d%%\n[[/div]]\n",
                    "[[div class=\"custom-comments\"]]\n%%comments%%\n[[/div]]\n",
                    "[[div class=\"custom-category\"]]\n%%category%%\n[[/div]]\n",
                    "[[div class=\"custom-description\"]]\n%%short%%\n[[/div]]\n",
                    "[[div class=\"custom-content\"]]\n%%body%%\n[[/div]]\n",
                    "[[div class=\"custom-unknown\"]]\n%%unknown%%\n[[/div]]\n",
                    "[[/module]]",
                ),
                category.forum_category_id,
            ),
        }),
    )
    .body;
    let thread_path = format!("/forum/t-{}/custom-body-thread", thread.forum_thread_id,);
    assert!(
        custom_div_body(&output, "custom-title").contains("Custom Body Thread"),
        "{output}",
    );
    assert!(
        custom_div_body(&output, "custom-linked").contains(&format!(
            r#"<a href="{thread_path}">Custom Body Thread</a>"#,
        )),
        "{output}",
    );
    assert!(
        custom_div_body(&output, "custom-link")
            .contains(&thread_path.replace('/', "&#x2F;")),
        "{output}",
    );
    assert!(
        custom_div_body(&output, "custom-author")
            .contains(r#"<span class="printuser">User</span>"#),
        "{output}",
    );
    assert!(
        custom_div_body(&output, "custom-date").contains("format_%25Y-%25m-%25d"),
        "{output}",
    );
    assert!(
        custom_div_body(&output, "custom-comments")
            .contains(&format!(r#"<a href="{thread_path}">Comments: 0</a>"#)),
        "{output}",
    );
    assert!(
        custom_div_body(&output, "custom-category")
            .contains("Custom Body Group / Custom Body Category"),
        "{output}",
    );
    assert!(
        custom_div_body(&output, "custom-description")
            .contains("Independent custom body description"),
        "{output}",
    );
    assert!(
        custom_div_body(&output, "custom-content")
            .contains("Independent custom body content &lt;observable&gt;"),
        "{output}",
    );
    assert!(
        custom_div_body(&output, "custom-unknown").contains("%%unknown%%")
            && !output.contains("[[/module]]"),
        "{output}",
    );

    let malformed = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "front-forum malformed custom-body fixture",
            "wikitext": format!(
                "[[module FrontForum category=\"{};bad\" limit=\"1\"]]\nCUSTOM-BODY-SHOULD-NOT-RENDER %%title%%\n[[/module]]",
                category.forum_category_id,
            ),
        }),
    )
    .body;
    assert!(
        malformed.contains(r#"Problem parsing attribute "category"."#)
            && !malformed.contains("CUSTOM-BODY-SHOULD-NOT-RENDER")
            && !malformed.contains("Custom Body Thread"),
        "{malformed}",
    );
}

#[tokio::test]
async fn frontforum_feed_arguments_preserve_the_ordinary_public_render() {
    async fn saved_body(runner: &TestRunner, site_id: i64, slug: &str) -> String {
        match run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": null,
                "route": {"slug": slug, "extra": ""},
                "locales": ["en-US", "en"],
            }),
        ) {
            GetPageViewOutput::Found {
                compiled_body_html, ..
            } => compiled_body_html,
            other => {
                panic!("expected a found FrontForum feed argument page, got {other:?}")
            }
        }
    }

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let group = ForumService::create_group(
        runner.context(),
        CreateForumGroup {
            site_id,
            user_id: ADMIN_USER_ID,
            name: "FrontForum Feed Argument Group".to_owned(),
            description: String::new(),
            visible: true,
            sort_index: Some(20_001),
            from_wikidot: false,
        },
    )
    .await
    .expect("FrontForum feed argument group should be created");
    let category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "FrontForum Feed Argument Category".to_owned(),
            description: String::new(),
            sort_index: Some(1),
            max_nest_level: Some(3),
            per_page_discussion: Some(false),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("FrontForum feed argument category should be created");
    let thread = ForumThreadService::create(
        runner.context(),
        CreateForumThread {
            forum_category_id: category.forum_category_id,
            user_id: SAMPLE_USER_ID,
            associated_page_id: None,
            title: "FrontForum Feed Argument Thread".to_owned(),
            description: String::new(),
            sticky: false,
            from_wikidot: false,
        },
    )
    .await
    .expect("FrontForum feed argument thread should be created");
    ForumPostService::create(
        runner.context(),
        CreateForumPost {
            forum_thread_id: thread.forum_thread_id,
            parent_post_id: None,
            user_id: SAMPLE_USER_ID,
            title: "FrontForum Feed Argument Post".to_owned(),
            wikitext: "FrontForum feed argument body".to_owned(),
            comments: "create FrontForum feed argument fixture".to_owned(),
            from_wikidot: false,
        },
    )
    .await
    .expect("FrontForum feed argument post should be created");

    let ordinary_source = format!(
        r#"[[module FrontForum category="{}" limit="1"]]"#,
        category.forum_category_id,
    );
    let ordinary_preview = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "FrontForum ordinary feed-argument control",
            "wikitext": ordinary_source,
        }),
    )
    .body;
    assert!(
        ordinary_preview.contains("FrontForum Feed Argument Thread")
            && ordinary_preview.contains(r#"<div class="front-forum-box">"#),
        "ordinary FrontForum fixture must be populated: {ordinary_preview}",
    );

    let accepted_suffixes = [
        r#"feed="readonlyfeed" feedTitle="Read only feed""#,
        r#"feed="readonlyfeed2""#,
        r#"feed="""#,
        r#"feed="../bad""#,
    ];
    let literal_suffixes = ["feed='readonlyfeed'", "feedTitle='Read only feed'"];
    let rejected_suffixes = [
        r#"Feed="readonlyfeed""#,
        r#"feedName="readonlyfeed""#,
        r#"feed="one" feed="two""#,
        r#"feedTitle="one" feedTitle="two""#,
    ];
    let mut failures = Vec::new();
    for (index, suffix) in accepted_suffixes.iter().enumerate() {
        let source = format!(
            r#"[[module FrontForum category="{}" limit="1" {suffix}]]"#,
            category.forum_category_id,
        );
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": format!("FrontForum feed argument preview {index}"),
                "wikitext": source,
            }),
        )
        .body;
        if preview != ordinary_preview {
            failures.push(format!(
                "preview accepted suffix {suffix:?} did not preserve the ordinary body: {preview}"
            ));
        }
    }
    for (index, suffix) in rejected_suffixes.iter().enumerate() {
        let source = format!(
            r#"[[module FrontForum category="{}" limit="1" {suffix}]]"#,
            category.forum_category_id,
        );
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": format!("FrontForum rejected feed argument preview {index}"),
                "wikitext": source,
            }),
        )
        .body;
        if !preview.contains("No such module") || preview.contains("front-forum-box") {
            failures.push(format!(
                "preview rejected suffix {suffix:?} did not fail closed: {preview}"
            ));
        }
    }
    for (index, suffix) in literal_suffixes.iter().enumerate() {
        let source = format!(
            r#"[[module FrontForum category="{}" limit="1" {suffix}]]"#,
            category.forum_category_id,
        );
        let expected = format!(
            "<p>{}</p>",
            source.replace('"', "&quot;").replace('\'', "&#39;")
        );
        let preview = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": format!("FrontForum literal feed argument preview {index}"),
                "wikitext": source,
            }),
        )
        .body;
        if preview != expected || preview.contains("front-forum-box") {
            failures.push(format!(
                "preview literal suffix {suffix:?} was not preserved exactly: {preview}"
            ));
        }
    }

    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-frontforum-feed-argument-control",
        "FrontForum Feed Argument Control",
        &ordinary_source,
    )
    .await;
    for (index, suffix) in accepted_suffixes.iter().enumerate() {
        let source = format!(
            r#"[[module FrontForum category="{}" limit="1" {suffix}]]"#,
            category.forum_category_id,
        );
        create_listpages_test_page(
            &mut runner,
            site_id,
            &format!("fixture-frontforum-feed-argument-{index}"),
            &format!("FrontForum Feed Argument {index}"),
            &source,
        )
        .await;
    }
    for (index, suffix) in rejected_suffixes.iter().enumerate() {
        let source = format!(
            r#"[[module FrontForum category="{}" limit="1" {suffix}]]"#,
            category.forum_category_id,
        );
        create_listpages_test_page(
            &mut runner,
            site_id,
            &format!("fixture-frontforum-rejected-feed-argument-{index}"),
            &format!("FrontForum Rejected Feed Argument {index}"),
            &source,
        )
        .await;
    }
    for (index, suffix) in literal_suffixes.iter().enumerate() {
        let source = format!(
            r#"[[module FrontForum category="{}" limit="1" {suffix}]]"#,
            category.forum_category_id,
        );
        create_listpages_test_page(
            &mut runner,
            site_id,
            &format!("fixture-frontforum-literal-feed-argument-{index}"),
            &format!("FrontForum Literal Feed Argument {index}"),
            &source,
        )
        .await;
    }
    runner.set_request_context(RequestContext::default());

    let ordinary_saved =
        saved_body(&runner, site_id, "fixture-frontforum-feed-argument-control").await;
    for (index, suffix) in accepted_suffixes.iter().enumerate() {
        let saved = saved_body(
            &runner,
            site_id,
            &format!("fixture-frontforum-feed-argument-{index}"),
        )
        .await;
        if saved != ordinary_saved {
            failures.push(format!(
                "page_view accepted suffix {suffix:?} did not preserve the ordinary body: {saved}"
            ));
        }
    }
    for (index, suffix) in rejected_suffixes.iter().enumerate() {
        let saved = saved_body(
            &runner,
            site_id,
            &format!("fixture-frontforum-rejected-feed-argument-{index}"),
        )
        .await;
        if !saved.contains("No such module") || saved.contains("front-forum-box") {
            failures.push(format!(
                "page_view rejected suffix {suffix:?} did not fail closed: {saved}"
            ));
        }
    }
    for (index, suffix) in literal_suffixes.iter().enumerate() {
        let source = format!(
            r#"[[module FrontForum category="{}" limit="1" {suffix}]]"#,
            category.forum_category_id,
        );
        let expected = format!(
            "<p>{}</p>",
            source.replace('"', "&quot;").replace('\'', "&#39;")
        );
        let saved = saved_body(
            &runner,
            site_id,
            &format!("fixture-frontforum-literal-feed-argument-{index}"),
        )
        .await;
        if saved != expected || saved.contains("front-forum-box") {
            failures.push(format!(
                "page_view literal suffix {suffix:?} was not preserved exactly: {saved}"
            ));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn forum_start_and_recent_posts_filter_before_counts_order_and_pagination() {
    // This comprehensive forum fixture exceeds the test harness's 2 MiB async
    // poll stack. Keep it on the same bounded named-thread runtime pattern used
    // by the large ListPages pagination regression below.
    std::thread::Builder::new()
        .name("forum-read-model-test".to_owned())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("forum read-model test runtime should build")
                .block_on(
                    forum_start_and_recent_posts_filter_before_counts_order_and_pagination_impl(),
                );
        })
        .expect("forum read-model test thread should spawn")
        .join()
        .expect("forum read-model test thread should complete");
}

async fn forum_start_and_recent_posts_filter_before_counts_order_and_pagination_impl() {
    async fn create_thread(
        runner: &TestRunner,
        category_id: i64,
        title: &str,
        associated_page_id: Option<i64>,
    ) -> i64 {
        ForumThreadService::create(
            runner.context(),
            CreateForumThread {
                forum_category_id: category_id,
                user_id: SAMPLE_USER_ID,
                associated_page_id,
                title: title.to_owned(),
                description: String::new(),
                sticky: false,
                from_wikidot: false,
            },
        )
        .await
        .expect("forum read-model fixture thread should be created")
        .forum_thread_id
    }

    async fn create_post(
        runner: &TestRunner,
        thread_id: i64,
        parent_post_id: Option<i64>,
        title: &str,
    ) -> i64 {
        ForumPostService::create(
            runner.context(),
            CreateForumPost {
                forum_thread_id: thread_id,
                parent_post_id,
                user_id: SAMPLE_USER_ID,
                title: title.to_owned(),
                wikitext: format!("{title} body <observable>"),
                comments: "create forum read-model fixture".to_owned(),
                from_wikidot: false,
            },
        )
        .await
        .expect("forum read-model fixture post should be created")
        .forum_post_id
    }

    async fn page_view_html(
        runner: &TestRunner,
        site_id: i64,
        slug: &str,
        extra: &str,
    ) -> String {
        match run_endpoint!(
            runner,
            page_view,
            json!({
                "site_id": site_id,
                "session_token": null,
                "route": {"slug": slug, "extra": extra},
                "locales": ["en-US", "en"],
            }),
        ) {
            GetPageViewOutput::Found {
                compiled_body_html, ..
            } => compiled_body_html,
            other => panic!("expected found forum read-model page view, got {other:?}"),
        }
    }

    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let visible_group = ForumService::create_group(
        runner.context(),
        CreateForumGroup {
            site_id,
            user_id: ADMIN_USER_ID,
            name: "Read Model Visible Group".to_owned(),
            description: "Visible <group> description".to_owned(),
            visible: true,
            sort_index: Some(10_000),
            from_wikidot: false,
        },
    )
    .await
    .expect("visible forum read-model group should be created");
    let primary_category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: visible_group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "Read Model: Primary".to_owned(),
            description: "Primary <category> description".to_owned(),
            sort_index: Some(10),
            max_nest_level: Some(3),
            per_page_discussion: Some(false),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("primary forum read-model category should be created");
    let empty_category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: visible_group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "Read Model Empty".to_owned(),
            description: "Empty category".to_owned(),
            sort_index: Some(20),
            max_nest_level: Some(3),
            per_page_discussion: Some(false),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("empty forum read-model category should be created");
    let pagination_category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: visible_group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "Read Model Pagination".to_owned(),
            description: "First-page category fixture".to_owned(),
            sort_index: Some(30),
            max_nest_level: Some(3),
            per_page_discussion: Some(false),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("pagination forum read-model category should be created");
    let hidden_group = ForumService::create_group(
        runner.context(),
        CreateForumGroup {
            site_id,
            user_id: ADMIN_USER_ID,
            name: "Read Model Hidden Group".to_owned(),
            description: "Hidden forum read-model fixture".to_owned(),
            visible: false,
            sort_index: Some(10_001),
            from_wikidot: false,
        },
    )
    .await
    .expect("hidden forum read-model group should be created");
    let hidden_category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: hidden_group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "Read Model Hidden Category".to_owned(),
            description: "Hidden category".to_owned(),
            sort_index: Some(10),
            max_nest_level: Some(3),
            per_page_discussion: Some(false),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("hidden forum read-model category should be created");

    let visible_structure = ForumService::get_structure(
        runner.context(),
        GetForumStructure {
            site_id,
            include_deleted: false,
            visible_groups_only: true,
        },
    )
    .await
    .expect("public forum structure should load");
    assert!(
        visible_structure
            .iter()
            .all(|group| group.group.forum_group_id != hidden_group.forum_group_id),
        "normal forum structure must exclude hidden groups before rendering",
    );
    let complete_structure = ForumService::get_structure(
        runner.context(),
        GetForumStructure {
            site_id,
            include_deleted: false,
            visible_groups_only: false,
        },
    )
    .await
    .expect("complete forum structure should load");
    assert!(
        complete_structure
            .iter()
            .any(|group| group.group.forum_group_id == hidden_group.forum_group_id),
        "direct hidden forum routes need the complete structure",
    );

    let empty_recent_posts = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "recent-posts empty fixture",
            "wikitext": "[[module RecentPosts]]",
        }),
    )
    .body;
    assert!(
        empty_recent_posts.contains(r#"<div class="forum-recent-posts-box" >"#)
            && empty_recent_posts.contains(r#"<div class="thread-container"></div>"#)
            && !empty_recent_posts.contains(r#"<div class="post" id="post-"#)
            && !empty_recent_posts.contains(r#"<div class="pager">"#),
        "{empty_recent_posts}",
    );

    UserService::update(
        runner.context(),
        Reference::Id(SAMPLE_USER_ID),
        common::IP_ADDRESS,
        UpdateUserBody {
            forum_signature: Maybe::Set(Some(
                "**Forum signature**\nSecond line".to_owned(),
            )),
            ..Default::default()
        },
    )
    .await
    .expect("forum signature fixture should be stored on the posting user");

    for number in 0..21 {
        let thread = create_thread(
            &runner,
            pagination_category.forum_category_id,
            &format!("Pagination Thread {number:02}"),
            None,
        )
        .await;
        create_post(
            &runner,
            thread,
            None,
            &format!("Pagination Post {number:02}"),
        )
        .await;
    }

    let visible_thread = create_thread(
        &runner,
        primary_category.forum_category_id,
        "Read Model Visible Thread",
        None,
    )
    .await;
    let root_id = create_post(&runner, visible_thread, None, "Visible Post 00").await;
    ForumPostService::update(
        runner.context(),
        UpdateForumPost {
            forum_post_id: root_id,
            user_id: SAMPLE_USER_ID,
            comments: "edit forum read-model fixture".to_owned(),
            body: UpdateForumPostBody {
                title: Maybe::Set("Visible Post 00 edited".to_owned()),
                wikitext: Maybe::Set(
                    "Visible Post 00 edited body <observable>".to_owned(),
                ),
            },
        },
    )
    .await
    .expect("forum read-model fixture post edit should succeed")
    .expect("forum read-model fixture post edit should create a revision");
    let mut visible_post_ids = vec![root_id];
    for number in 1..22 {
        visible_post_ids.push(
            create_post(
                &runner,
                visible_thread,
                Some(root_id),
                &format!("Visible Post {number:02}"),
            )
            .await,
        );
    }

    const PUBLIC_PAGE_SLUG: &str = "fixture-forum-read-model-public";
    create_listpages_test_page(
        &mut runner,
        site_id,
        PUBLIC_PAGE_SLUG,
        "Public Forum Read Model Page",
        "public forum read-model page",
    )
    .await;
    let public_page_id = listpages_test_page_id(&runner, site_id, PUBLIC_PAGE_SLUG).await;
    let public_page_thread = create_thread(
        &runner,
        primary_category.forum_category_id,
        "Public Page Discussion Marker",
        Some(public_page_id),
    )
    .await;
    let public_page_post_id =
        create_post(&runner, public_page_thread, None, "Public Page Comment").await;

    let hidden_thread = create_thread(
        &runner,
        hidden_category.forum_category_id,
        "Read Model Hidden Thread",
        None,
    )
    .await;
    create_post(&runner, hidden_thread, None, "Hidden Newest Post").await;

    const PRIVATE_PAGE_CATEGORY: &str = "forum-read-model-private";
    const PRIVATE_PAGE_SLUG: &str = "fixture-forum-read-model-private";
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_PAGE_CATEGORY)
        .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        PRIVATE_PAGE_SLUG,
        "Private Forum Read Model Page",
        "private forum read-model page",
    )
    .await;
    set_listpages_test_category_slug(
        &runner,
        site_id,
        PRIVATE_PAGE_SLUG,
        PRIVATE_PAGE_CATEGORY,
    )
    .await;
    let private_page_id =
        listpages_test_page_id(&runner, site_id, PRIVATE_PAGE_SLUG).await;
    let private_thread = create_thread(
        &runner,
        primary_category.forum_category_id,
        "Private Page Discussion Marker",
        Some(private_page_id),
    )
    .await;
    create_post(&runner, private_thread, None, "Private Newest Post").await;

    const RECENT_POSTS_PAGE: &str = "fixture-forum-recent-posts";
    create_listpages_test_page(
        &mut runner,
        site_id,
        RECENT_POSTS_PAGE,
        "Fixture Forum Recent Posts",
        "[[module RecentPosts]]",
    )
    .await;
    runner.set_request_context(RequestContext::default());

    let forum_start = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "forum-start permission-first fixture",
            "wikitext": "[[module ForumStart]]",
        }),
    )
    .body;
    assert!(
        forum_start.contains(r#"<div class="forum-start-box">"#)
            && forum_start.contains("Visible &lt;group&gt; description")
            && forum_start.contains("Primary &lt;category&gt; description")
            && forum_start.contains(&format!(
                r#"href="/forum/c-{}/{}""#,
                primary_category.forum_category_id,
                deepwell::utils::normalize_page_slug("Read Model: Primary"),
            ))
            && !forum_start.contains("Read Model Hidden Group")
            && !forum_start.contains("Private Page Discussion Marker"),
        "{forum_start}",
    );
    let primary_row_start = forum_start
        .find("Read Model: Primary")
        .expect("primary category should render");
    let primary_row = &forum_start[primary_row_start..];
    assert!(
        primary_row.contains(
            r#"</div></td><td class="threads">2</td><td class="posts">23</td>"#,
        ),
        "private page activity must be filtered before category counts: {primary_row}",
    );
    let empty_row_start = forum_start
        .find("Read Model Empty")
        .expect("empty category should render");
    let empty_row = &forum_start[empty_row_start..];
    assert!(
        empty_row.contains(
            r#"</div></td><td class="threads">0</td><td class="posts">0</td>"#,
        ),
        "{empty_row}",
    );
    assert!(primary_row_start < empty_row_start, "{forum_start}");
    assert!(
        forum_start.contains(&format!(
            r#"href="/forum/t-{public_page_thread}#post-{public_page_post_id}">Jump!</a>"#,
        )),
        "{forum_start}",
    );

    let front_forum = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "front-forum permission-first fixture",
            "wikitext": format!(
                r#"[[module FrontForum category="{}" limit="2"]]"#,
                primary_category.forum_category_id,
            ),
        }),
    )
    .body;
    assert!(
        front_forum.contains(r#"<div class="front-forum-box">"#)
            && front_forum
                .matches(r#"<h1><span><a href="/forum/t-"#)
                .count()
                == 2
            && front_forum.contains("Public Page Discussion Marker")
            && front_forum.contains("Public Page Comment body &lt;observable&gt;")
            && front_forum.contains("Comments: 0")
            && front_forum.contains("Read Model Visible Thread")
            && front_forum.contains("Comments: 21")
            && front_forum.contains("Read Model Visible Group / Read Model: Primary")
            && !front_forum.contains("Private Page Discussion Marker")
            && !front_forum.contains("Read Model Hidden Thread"),
        "{front_forum}",
    );
    let front_forum_missing = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "front-forum missing category fixture",
            "wikitext": r#"[[module FrontForum category="9223372036854775807" limit="1"]]"#,
        }),
    )
    .body;
    assert!(
        front_forum_missing.contains("Requested forum category does not exist.")
            && !front_forum_missing.contains(r#"<div class="front-forum-box">"#),
        "{front_forum_missing}",
    );

    for (case_id, categories) in [
        (
            "front-forum populated-empty categories",
            format!(
                "{};{}",
                primary_category.forum_category_id, empty_category.forum_category_id,
            ),
        ),
        (
            "front-forum empty-populated categories",
            format!(
                "{};{}",
                empty_category.forum_category_id, primary_category.forum_category_id,
            ),
        ),
    ] {
        let output = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": format!(
                    r#"[[module FrontForum category="{categories}" limit="2"]]"#,
                ),
            }),
        )
        .body;
        assert!(
            output.matches(r#"<h1><span><a href="/forum/t-"#).count() == 2
                && output.contains("Public Page Discussion Marker")
                && output.contains("Read Model Visible Thread")
                && !output.contains("Private Page Discussion Marker"),
            "{case_id}: {output}",
        );
        let newest = output
            .find("Public Page Discussion Marker")
            .expect("newest permitted category item should render");
        let next = output
            .find("Read Model Visible Thread")
            .expect("next permitted category item should render");
        assert!(newest < next, "{case_id}: {output}");
    }

    for (case_id, categories) in [
        (
            "front-forum populated-missing categories",
            format!("{};9223372036854775807", primary_category.forum_category_id,),
        ),
        (
            "front-forum missing-populated categories",
            format!("9223372036854775807;{}", primary_category.forum_category_id,),
        ),
    ] {
        let output = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": format!(
                    r#"[[module FrontForum category="{categories}" limit="2"]]"#,
                ),
            }),
        )
        .body;
        assert!(
            output.matches(r#"<h1><span><a href="/forum/t-"#).count() == 2
                && output.contains("Public Page Discussion Marker")
                && output.contains("Read Model Visible Thread")
                && !output.contains("Requested forum category does not exist."),
            "{case_id}: {output}",
        );
    }

    let front_forum_global = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "front-forum global category order",
            "wikitext": format!(
                r#"[[module FrontForum category="{};{}" limit="3"]]"#,
                primary_category.forum_category_id,
                pagination_category.forum_category_id,
            ),
        }),
    )
    .body;
    let public_position = front_forum_global
        .find("Public Page Discussion Marker")
        .expect("newest permitted thread should render");
    let visible_position = front_forum_global
        .find("Read Model Visible Thread")
        .expect("second newest permitted thread should render");
    let pagination_position = front_forum_global
        .find("Pagination Thread 20")
        .expect("newest thread from the second category should render");
    assert!(
        public_position < visible_position
            && visible_position < pagination_position
            && front_forum_global.contains(&format!(
                r#"href="/forum/c-{}/read-model:primary">Read Model Visible Group / Read Model: Primary</a>"#,
                primary_category.forum_category_id,
            ))
            && front_forum_global.contains(&format!(
                r#"href="/forum/c-{}/read-model-pagination">Read Model Visible Group / Read Model Pagination</a>"#,
                pagination_category.forum_category_id,
            )),
        "{front_forum_global}",
    );

    let front_forum_missing_list = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "front-forum all categories missing",
            "wikitext": r#"[[module FrontForum category="9223372036854775806;9223372036854775807" limit="2"]]"#,
        }),
    )
    .body;
    assert!(
        front_forum_missing_list.contains(
            r#"<div class="error-block">Requested forum category does not exist.</div>"#
        ) && !front_forum_missing_list.contains("front-forum-box"),
        "{front_forum_missing_list}",
    );

    let front_forum_malformed = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "front-forum malformed category list",
            "wikitext": format!(
                r#"[[module FrontForum category="{};bad" limit="2"]]"#,
                primary_category.forum_category_id,
            ),
        }),
    )
    .body;
    assert!(
        front_forum_malformed.contains(
            r#"<div class="error-block">Problem parsing attribute "category".</div>"#
        ) && !front_forum_malformed.contains("front-forum-box")
            && !front_forum_malformed.contains("[[module"),
        "{front_forum_malformed}",
    );

    for (case_id, offset) in [
        ("front-forum explicit zero offset", "0"),
        ("front-forum invalid offset defaults", "bad"),
    ] {
        let output = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": format!(
                    r#"[[module FrontForum category="{}" limit="2" offset="{offset}"]]"#,
                    primary_category.forum_category_id,
                ),
            }),
        )
        .body;
        assert!(
            output.matches(r#"<h1><span><a href="/forum/t-"#).count() == 2
                && output.contains("Public Page Discussion Marker")
                && output.contains("Read Model Visible Thread")
                && !output.contains("Private Page Discussion Marker"),
            "{case_id}: {output}",
        );
    }

    let front_forum_offset_one = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "front-forum offset one",
            "wikitext": format!(
                r#"[[module FrontForum category="{}" limit="1" offset="1"]]"#,
                primary_category.forum_category_id,
            ),
        }),
    )
    .body;
    assert!(
        front_forum_offset_one.contains("Read Model Visible Thread")
            && !front_forum_offset_one.contains("Public Page Discussion Marker")
            && !front_forum_offset_one.contains("Private Page Discussion Marker")
            && front_forum_offset_one
                .matches(r#"<h1><span><a href="/forum/t-"#)
                .count()
                == 1,
        "{front_forum_offset_one}",
    );

    let front_forum_large_offset = run_endpoint!(
        runner,
        wikidot_page_preview,
        json!({
            "site_id": site_id,
            "title": "front-forum large offset",
            "wikitext": format!(
                r#"[[module FrontForum category="{}" limit="2" offset="999"]]"#,
                primary_category.forum_category_id,
            ),
        }),
    )
    .body;
    assert!(
        front_forum_large_offset.contains(r#"<div class="front-forum-box"></div>"#)
            && !front_forum_large_offset.contains(r#"<h1><span><a href="/forum/t-"#),
        "{front_forum_large_offset}",
    );

    for (case_id, wikitext) in [
        (
            "front-forum duplicate limit",
            format!(
                r#"[[module FrontForum category="{}" limit="1" limit="2"]]"#,
                primary_category.forum_category_id,
            ),
        ),
        (
            "front-forum duplicate offset",
            format!(
                r#"[[module FrontForum category="{}" limit="1" offset="1" offset="0"]]"#,
                primary_category.forum_category_id,
            ),
        ),
    ] {
        let output = run_endpoint!(
            runner,
            wikidot_page_preview,
            json!({
                "site_id": site_id,
                "title": case_id,
                "wikitext": wikitext,
            }),
        )
        .body;
        assert!(
            output.contains("No such module")
                && !output.contains("front-forum-box")
                && !output.contains("Public Page Discussion Marker")
                && !output.contains("Read Model Visible Thread"),
            "{case_id}: {output}",
        );
    }

    let saved_multi_source = format!(
        r#"[[module FrontForum category="{};{}" limit="3"]]"#,
        primary_category.forum_category_id, pagination_category.forum_category_id,
    );
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-front-forum-multiple-categories",
        "Fixture FrontForum Multiple Categories",
        &saved_multi_source,
    )
    .await;
    let saved_offset_source = format!(
        r#"[[module FrontForum category="{}" limit="1" offset="1"]]"#,
        primary_category.forum_category_id,
    );
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-front-forum-offset",
        "Fixture FrontForum Offset",
        &saved_offset_source,
    )
    .await;
    let saved_malformed_source = format!(
        r#"[[module FrontForum category="{};bad" limit="2"]]"#,
        primary_category.forum_category_id,
    );
    create_listpages_test_page(
        &mut runner,
        site_id,
        "fixture-front-forum-malformed-category",
        "Fixture FrontForum Malformed Category",
        &saved_malformed_source,
    )
    .await;
    for (slug, title, source) in [
        (
            "fixture-front-forum-duplicate-limit",
            "Fixture FrontForum Duplicate Limit",
            format!(
                r#"[[module FrontForum category="{}" limit="1" limit="2"]]"#,
                primary_category.forum_category_id,
            ),
        ),
        (
            "fixture-front-forum-duplicate-offset",
            "Fixture FrontForum Duplicate Offset",
            format!(
                r#"[[module FrontForum category="{}" limit="1" offset="1" offset="0"]]"#,
                primary_category.forum_category_id,
            ),
        ),
    ] {
        create_listpages_test_page(&mut runner, site_id, slug, title, &source).await;
    }
    runner.set_request_context(RequestContext::default());

    let saved_multi = page_view_html(
        &runner,
        site_id,
        "fixture-front-forum-multiple-categories",
        "",
    )
    .await;
    assert!(
        saved_multi.matches(r#"<h1><span><a href="/forum/t-"#).count() == 3
            && saved_multi.contains("Public Page Discussion Marker")
            && saved_multi.contains("Read Model Visible Thread")
            && saved_multi.contains("Pagination Thread 20")
            && saved_multi.contains(&format!(
                r#"href="/forum/c-{}/read-model-pagination">Read Model Visible Group / Read Model Pagination</a>"#,
                pagination_category.forum_category_id,
            ))
            && !saved_multi.contains("Private Page Discussion Marker"),
        "{saved_multi}",
    );
    let saved_offset =
        page_view_html(&runner, site_id, "fixture-front-forum-offset", "").await;
    assert!(
        saved_offset.contains("Read Model Visible Thread")
            && !saved_offset.contains("Public Page Discussion Marker")
            && !saved_offset.contains("Private Page Discussion Marker"),
        "{saved_offset}",
    );
    let saved_malformed = page_view_html(
        &runner,
        site_id,
        "fixture-front-forum-malformed-category",
        "",
    )
    .await;
    assert!(
        saved_malformed.contains(
            r#"<div class="error-block">Problem parsing attribute "category".</div>"#
        ) && !saved_malformed.contains("front-forum-box")
            && !saved_malformed.contains("[[module"),
        "{saved_malformed}",
    );
    for slug in [
        "fixture-front-forum-duplicate-limit",
        "fixture-front-forum-duplicate-offset",
    ] {
        let output = page_view_html(&runner, site_id, slug, "").await;
        assert!(
            output.contains("No such module")
                && !output.contains("front-forum-box")
                && !output.contains("Public Page Discussion Marker")
                && !output.contains("Read Model Visible Thread"),
            "{slug}: {output}",
        );
    }

    let forum_start_ajax = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumStartModule",
            "parameters": {},
        }),
    );
    assert_eq!(forum_start_ajax.status, "ok");
    assert!(
        forum_start_ajax
            .body
            .contains(r#"<div class="forum-start-box">"#)
            && forum_start_ajax.body.contains("Read Model Visible Group")
            && !forum_start_ajax.body.contains("Read Model Hidden Group")
            && !forum_start_ajax.body.contains("Order by:"),
        "{}",
        forum_start_ajax.body,
    );

    let category_order = |category_id| {
        format!(
            r#"<div class="options">Order by: <div class="btn btn-primary disabled btn-small btn-sm"><strong>Last post date</strong></div> <a href="/forum/c-{category_id}/sort/start" class="btn btn-primary btn-small btn-sm">Thread starting date</a></div>"#,
        )
    };
    let primary_category_order = category_order(primary_category.forum_category_id);
    let category_ajax = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumViewCategoryModule",
            "parameters": {
                "c": primary_category.forum_category_id.to_string(),
                "p": "1",
            },
        }),
    );
    assert_eq!(category_ajax.status, "ok");
    assert!(
        category_ajax
            .body
            .contains(r#"<div class="forum-category-box">"#)
            && category_ajax.body.contains("Read Model Visible Thread")
            && category_ajax.body.contains("Public Page Discussion Marker")
            && !category_ajax
                .body
                .contains("Private Page Discussion Marker")
            && category_ajax.body.matches(&primary_category_order).count() == 1
            && !category_ajax.body.contains("Create a new thread"),
        "{}",
        category_ajax.body,
    );
    let description_position = category_ajax
        .body
        .find("Primary &lt;category&gt; description</div>")
        .expect("category description should render");
    let order_position = category_ajax
        .body
        .find(&primary_category_order)
        .expect("default category order projection should render");
    let table_position = category_ajax
        .body
        .find(r#"<table style="width: 98%" class="table">"#)
        .expect("category thread table should render");
    assert!(
        description_position < order_position && order_position < table_position,
        "{}",
        category_ajax.body,
    );
    let missing_category_ajax = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumViewCategoryModule",
            "parameters": {"c": "9223372036854775807", "p": "1"},
        }),
    );
    assert_eq!(missing_category_ajax.status, "no_category");
    assert!(
        missing_category_ajax.body.is_empty()
            && !missing_category_ajax.body.contains("Order by:")
    );
    let empty_category_ajax = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumViewCategoryModule",
            "parameters": {
                "c": empty_category.forum_category_id.to_string(),
                "p": "1",
            },
        }),
    );
    let empty_category_order = category_order(empty_category.forum_category_id);
    assert_eq!(empty_category_ajax.status, "ok");
    assert_eq!(
        empty_category_ajax
            .body
            .matches(&empty_category_order)
            .count(),
        1,
        "{}",
        empty_category_ajax.body,
    );
    let empty_second_page = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumViewCategoryModule",
            "parameters": {
                "c": empty_category.forum_category_id.to_string(),
                "p": "2",
            },
        }),
    );
    assert_eq!(empty_second_page.status, "not_ok");
    assert!(empty_second_page.body.is_empty());

    let hidden_category_ajax = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumViewCategoryModule",
            "parameters": {
                "c": hidden_category.forum_category_id.to_string(),
                "p": "1",
            },
        }),
    );
    assert_eq!(hidden_category_ajax.status, "ok");
    assert!(
        hidden_category_ajax
            .body
            .contains("Read Model Hidden Category"),
        "a hidden group remains directly addressable by its observed category route: {}",
        hidden_category_ajax.body,
    );

    let paginated_category_ajax = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumViewCategoryModule",
            "parameters": {
                "c": pagination_category.forum_category_id.to_string(),
                "p": "1",
            },
        }),
    );
    assert_eq!(paginated_category_ajax.status, "ok");
    assert!(
        paginated_category_ajax
            .body
            .contains("Number of threads: 21")
            && paginated_category_ajax.body.contains("Number of posts: 21")
            && paginated_category_ajax
                .body
                .matches(r#"<td class="name"><div class="title"><a href="/forum/t-"#)
                .count()
                == 20
            && paginated_category_ajax
                .body
                .contains("Pagination Thread 20")
            && !paginated_category_ajax
                .body
                .contains("Pagination Thread 00")
            && paginated_category_ajax
                .body
                .matches(r#"<div class="pager">"#)
                .count()
                == 2,
        "{}",
        paginated_category_ajax.body,
    );
    let second_category_page = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumViewCategoryModule",
            "parameters": {
                "c": pagination_category.forum_category_id.to_string(),
                "p": "2",
            },
        }),
    );
    assert_eq!(second_category_page.status, "ok");
    let pagination_category_order = category_order(pagination_category.forum_category_id);
    assert!(
        second_category_page.body.contains("Pagination Thread 00")
            && !second_category_page.body.contains("Pagination Thread 20")
            && second_category_page
                .body
                .matches(&pagination_category_order)
                .count()
                == 1
            && second_category_page
                .body
                .contains(r#"<span class="pager-no">page 2 of 2</span>"#)
            && second_category_page.body.contains(&format!(
                r#"href="/forum/c-{}/p/1">&laquo; previous</a>"#,
                pagination_category.forum_category_id,
            ))
            && second_category_page
                .body
                .matches(r#"<div class="pager">"#)
                .count()
                == 1,
        "{}",
        second_category_page.body,
    );
    assert!(
        second_category_page
            .body
            .find(&pagination_category_order)
            .expect("default category order projection should render")
            < second_category_page
                .body
                .find(r#"<div class="pager">"#)
                .expect("later category page pager should render"),
        "{}",
        second_category_page.body,
    );

    let thread_ajax = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumViewThreadModule",
            "parameters": {"t": visible_thread.to_string()},
        }),
    );
    assert_eq!(thread_ajax.status, "ok");
    assert!(
        thread_ajax
            .body
            .contains(r#"<div class="forum-thread-box ">"#)
            && thread_ajax.body.contains("Read Model Visible Thread")
            && thread_ajax.body.contains("Number of posts: 22")
            && thread_ajax
                .body
                .contains(r#"<div id="thread-container" class="thread-container">"#)
            && thread_ajax
                .body
                .contains(r#"<div id="thread-container-posts" style="display: none">"#,)
            && thread_ajax.body.contains(&format!(
                r#"<div class="post-container" id="fpc-{root_id}">"#,
            ))
            && thread_ajax.body.contains("Visible Post 00 edited")
            && thread_ajax
                .body
                .contains(r#"<div class="signature"><hr class="signature-separator"/>"#)
            && thread_ajax
                .body
                .contains("<strong>Forum signature</strong>")
            && thread_ajax.body.contains("Second line")
            && !thread_ajax.body.contains("**Forum signature**")
            && thread_ajax.body.contains(r#"<div class="changes">"#)
            && thread_ajax.body.contains("Last edited on")
            && thread_ajax
                .body
                .contains(r#"<div class="revisions" style="display: none"></div>"#)
            && thread_ajax.body.contains("Show more")
            && thread_ajax.body.contains("Unfold All")
            && thread_ajax.body.contains("Edit Title &amp; Description")
            && thread_ajax.body.contains("New Post")
            && !thread_ajax.body.contains("Order by:")
            && thread_ajax.body.contains(r#"id="post-options-template""#),
        "{}",
        thread_ajax.body,
    );
    assert_eq!(thread_ajax.js_include.len(), 2);
    assert!(
        thread_ajax.js_include[0].ends_with("/ForumViewThreadPostsModule.js")
            && thread_ajax.js_include[1].ends_with("/ForumViewThreadModule.js"),
        "{:?}",
        thread_ajax.js_include,
    );
    let thread_posts_ajax = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumViewThreadPostsModule",
            "parameters": {"t": visible_thread.to_string(), "pageNo": "1"},
        }),
    );
    assert_eq!(thread_posts_ajax.status, "ok");
    assert!(
        thread_posts_ajax.body.starts_with(&format!(
            r#"<div class="post-container" id="fpc-{root_id}">"#,
        )) && !thread_posts_ajax
            .body
            .contains(r#"id="thread-container-posts""#)
            && thread_posts_ajax.body.contains("Visible Post 00")
            && thread_posts_ajax.body.contains("Visible Post 19")
            && !thread_posts_ajax.body.contains("Visible Post 20")
            && !thread_posts_ajax.body.contains("Private Newest Post")
            && thread_posts_ajax.body.contains("Visible Post 00 edited")
            && thread_posts_ajax
                .body
                .contains(r#"<div class="signature"><hr class="signature-separator"/>"#)
            && thread_posts_ajax.body.contains("Last edited on")
            && !thread_posts_ajax.body.contains("Edit")
            && !thread_posts_ajax.body.contains("Delete"),
        "{}",
        thread_posts_ajax.body,
    );
    assert_eq!(thread_posts_ajax.js_include.len(), 1);
    assert!(
        thread_posts_ajax.js_include[0].ends_with("/ForumViewThreadPostsModule.js"),
        "{:?}",
        thread_posts_ajax.js_include,
    );
    let private_thread_ajax = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumViewThreadModule",
            "parameters": {"t": private_thread.to_string()},
        }),
    );
    assert_eq!(private_thread_ajax.status, "no_thread");

    for (module_name, parameters) in [
        (
            "forum/ForumViewCategoryModule",
            json!({"c": primary_category.forum_category_id.to_string()}),
        ),
        (
            "forum/ForumViewThreadPostsModule",
            json!({"t": visible_thread.to_string()}),
        ),
        ("forum/ForumRecentPostsListModule", json!({"page": "1"})),
    ] {
        let widened = run_endpoint!(
            runner,
            wikidot_forum_module,
            json!({
                "site_id": site_id,
                "module_name": module_name,
                "parameters": parameters,
            }),
        );
        assert_eq!(
            widened.status, "not_ok",
            "{module_name} must require its exact observed parameter set",
        );
    }

    let recent_posts_ajax = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumRecentPostsListModule",
            "parameters": {
                "page": "1",
                "categoryId": primary_category.forum_category_id.to_string(),
            },
        }),
    );
    assert_eq!(recent_posts_ajax.status, "ok");
    assert!(
        recent_posts_ajax
            .body
            .starts_with(r#"<div id="recent-posts-container">"#)
            && recent_posts_ajax.body.contains("Visible Post 21")
            && recent_posts_ajax.body.contains("Public Page Comment")
            && recent_posts_ajax
                .body
                .contains(r#"<div class="signature"><hr class="signature-separator"/>"#)
            && !recent_posts_ajax.body.contains("Private Newest Post")
            && !recent_posts_ajax.body.contains("Hidden Newest Post"),
        "{}",
        recent_posts_ajax.body,
    );

    let invalid_numeric_requests = [
        (
            "forum/ForumViewCategoryModule",
            json!({
                "c": format!("+{}", primary_category.forum_category_id),
                "p": "1",
            }),
            "no_category",
        ),
        (
            "forum/ForumViewCategoryModule",
            json!({
                "c": format!("0{}", primary_category.forum_category_id),
                "p": "1",
            }),
            "no_category",
        ),
        (
            "forum/ForumViewCategoryModule",
            json!({"c": primary_category.forum_category_id.to_string(), "p": "+1"}),
            "not_ok",
        ),
        (
            "forum/ForumViewCategoryModule",
            json!({"c": primary_category.forum_category_id.to_string(), "p": "01"}),
            "not_ok",
        ),
        (
            "forum/ForumViewThreadModule",
            json!({"t": format!("+{visible_thread}")}),
            "no_thread",
        ),
        (
            "forum/ForumViewThreadModule",
            json!({"t": format!("0{visible_thread}")}),
            "no_thread",
        ),
        (
            "forum/ForumViewThreadPostsModule",
            json!({"t": format!("+{visible_thread}"), "pageNo": "1"}),
            "no_thread",
        ),
        (
            "forum/ForumViewThreadPostsModule",
            json!({"t": format!("0{visible_thread}"), "pageNo": "1"}),
            "no_thread",
        ),
        (
            "forum/ForumViewThreadPostsModule",
            json!({"t": visible_thread.to_string(), "pageNo": "+1"}),
            "not_ok",
        ),
        (
            "forum/ForumViewThreadPostsModule",
            json!({"t": visible_thread.to_string(), "pageNo": "01"}),
            "not_ok",
        ),
        (
            "forum/ForumRecentPostsListModule",
            json!({"page": "+1", "categoryId": primary_category.forum_category_id.to_string()}),
            "not_ok",
        ),
        (
            "forum/ForumRecentPostsListModule",
            json!({"page": "01", "categoryId": primary_category.forum_category_id.to_string()}),
            "not_ok",
        ),
        (
            "forum/ForumRecentPostsListModule",
            json!({"page": "1", "categoryId": format!("+{}", primary_category.forum_category_id)}),
            "not_ok",
        ),
        (
            "forum/ForumRecentPostsListModule",
            json!({"page": "1", "categoryId": format!("0{}", primary_category.forum_category_id)}),
            "not_ok",
        ),
    ];
    for (module_name, parameters, expected_status) in invalid_numeric_requests {
        let noncanonical = run_endpoint!(
            runner,
            wikidot_forum_module,
            json!({
                "site_id": site_id,
                "module_name": module_name,
                "parameters": parameters,
            }),
        );
        assert_eq!(
            noncanonical.status, expected_status,
            "{module_name} must reject noncanonical numeric scalars",
        );
        assert!(noncanonical.body.is_empty(), "{module_name}");
        assert_eq!(noncanonical.thread_id, None, "{module_name}");
        assert!(noncanonical.js_include.is_empty(), "{module_name}");
    }

    let recent_posts_page_two_ajax = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumRecentPostsListModule",
            "parameters": {"page": "2", "categoryId": ""},
        }),
    );
    assert_eq!(recent_posts_page_two_ajax.status, "ok");
    assert_eq!(
        recent_posts_page_two_ajax
            .body
            .matches(r#"<div class="post" id="post-"#)
            .count(),
        20,
        "{}",
        recent_posts_page_two_ajax.body,
    );
    assert!(
        recent_posts_page_two_ajax.body.contains("Visible Post 02")
            && recent_posts_page_two_ajax.body.contains("Visible Post 00")
            && !recent_posts_page_two_ajax.body.contains("Visible Post 03")
            && !recent_posts_page_two_ajax
                .body
                .contains("Hidden Newest Post")
            && !recent_posts_page_two_ajax
                .body
                .contains("Private Newest Post")
            && recent_posts_page_two_ajax
                .body
                .contains(r#"<span class="pager-no">page 2</span>"#)
            && recent_posts_page_two_ajax.body.contains("updateList(1)"),
        "{}",
        recent_posts_page_two_ajax.body,
    );

    let first_page = page_view_html(&runner, site_id, RECENT_POSTS_PAGE, "").await;
    assert!(
        first_page.contains(r#"<div class="forum-recent-posts-box" >"#)
            && first_page.contains(r#"id="recent-posts-category""#)
            && first_page.contains("Read Model Visible Group: Read Model: Primary")
            && !first_page.contains("Read Model Hidden Group")
            && !first_page.contains("Hidden Newest Post")
            && !first_page.contains("Private Newest Post")
            && first_page.matches(r#"<div class="post" id="post-"#).count() == 20
            && first_page.contains("Public Page Comment")
            && first_page.contains(&format!(
                r#"href="/{PUBLIC_PAGE_SLUG}/comments/show#post-{public_page_post_id}""#,
            ))
            && first_page.contains(&format!(
                r#"href="/forum/t-{visible_thread}/read-model-visible-thread#post-{}""#,
                visible_post_ids[21],
            ))
            && first_page.contains("Visible Post 21")
            && !first_page.contains("Visible Post 00")
            && first_page.contains("updateList(2)")
            && first_page.contains("class=\"odate time_")
            && first_page
                .contains("format_%25e%20%25b%20%25Y%2C%20%25H%3A%25M%7Cagohover")
            && first_page.contains("Visible Post 21 body &lt;observable&gt;"),
        "{first_page}",
    );
    let post_21 = first_page
        .find("Visible Post 21")
        .expect("newest visible post should render");
    let post_20 = first_page
        .find("Visible Post 20")
        .expect("next visible post should render");
    assert!(post_21 < post_20, "{first_page}");

    let second_page = page_view_html(&runner, site_id, RECENT_POSTS_PAGE, "/p/2").await;
    let expected_second_page_titles = [
        "Visible Post 02".to_owned(),
        "Visible Post 01".to_owned(),
        "Visible Post 00".to_owned(),
    ]
    .into_iter()
    .chain(
        (4..=20)
            .rev()
            .map(|number| format!("Pagination Post {number:02}")),
    )
    .collect::<Vec<_>>();
    let second_page_positions = expected_second_page_titles
        .iter()
        .map(|title| {
            second_page
                .find(title)
                .unwrap_or_else(|| panic!("page 2 should contain {title}: {second_page}"))
        })
        .collect::<Vec<_>>();
    assert!(
        second_page
            .matches(r#"<div class="post" id="post-"#)
            .count()
            == 20
            && second_page_positions
                .windows(2)
                .all(|pair| pair[0] < pair[1])
            && !second_page.contains("Visible Post 03")
            && !second_page.contains("Pagination Post 03")
            && !second_page.contains("Hidden Newest Post")
            && !second_page.contains("Private Newest Post")
            && second_page.contains(r#"<span class="pager-no">page 2</span>"#)
            && second_page.contains("updateList(1)"),
        "{second_page}",
    );

    let guest_role = RoleService::get(
        runner.context(),
        site_id,
        Reference::Slug(Cow::Borrowed("guest")),
    )
    .await
    .expect("guest role should exist");
    let anonymous_role = RoleService::get(
        runner.context(),
        site_id,
        Reference::Slug(Cow::Borrowed("anonymous")),
    )
    .await
    .expect("anonymous role should exist");
    let everyone_role = RoleService::get(
        runner.context(),
        site_id,
        Reference::Slug(Cow::Borrowed("everyone")),
    )
    .await
    .expect("everyone role should exist");
    RolePermissionTable::delete_many()
        .filter(role_permission::Column::RoleId.is_in([
            guest_role.role_id,
            anonymous_role.role_id,
            everyone_role.role_id,
        ]))
        .filter(role_permission::Column::SiteId.eq(site_id))
        .filter(role_permission::Column::ResourceType.eq(Resource::Page))
        .filter(role_permission::Column::ResourceCategoryId.is_null())
        .filter(role_permission::Column::Action.eq(Action::View))
        .exec(runner.context().transaction())
        .await
        .expect("anonymous page view permissions should be revoked");
    PermissionCache::invalidate_site(runner.context(), site_id)
        .await
        .expect("permission cache invalidation should run");
    let denied = run_endpoint!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumViewThreadModule",
            "parameters": {"t": visible_thread.to_string()},
        }),
    );
    assert_eq!(denied.status, "not_ok");
    assert!(denied.body.is_empty());

    runner.set_request_context(RequestContext {
        site_id: Some(site_id + 1),
        ..Default::default()
    });
    let mismatched_site = run_endpoint_err!(
        runner,
        wikidot_forum_module,
        json!({
            "site_id": site_id,
            "module_name": "forum/ForumStartModule",
            "parameters": {},
        }),
    );
    assert_contains_error!(mismatched_site, ErrorType::PermissionDenied);
}

/// The AMC forum-module dispatcher must fail closed: only the exact evidenced
/// parameter shapes dispatch, and `hidden` must be the literal "true" to opt
/// into hidden groups.
#[tokio::test]
async fn forum_module_dispatch_is_fail_closed_on_unexpected_parameters() {
    let runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "scp-wiki"}))
        .expect("seeded SCP Wiki site should exist");
    let site_id = site.site.site_id;
    let group = ForumService::create_group(
        runner.context(),
        CreateForumGroup {
            site_id,
            user_id: ADMIN_USER_ID,
            name: "Dispatch Boundary Group".to_owned(),
            description: "Dispatch boundary group".to_owned(),
            visible: true,
            sort_index: Some(20_000),
            from_wikidot: false,
        },
    )
    .await
    .expect("dispatch boundary group should be created");
    let category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "Dispatch Boundary Category".to_owned(),
            description: "Dispatch boundary category".to_owned(),
            sort_index: Some(10),
            max_nest_level: Some(3),
            per_page_discussion: Some(false),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("dispatch boundary category should be created");
    let category_id = category.forum_category_id;
    let thread = ForumThreadService::create(
        runner.context(),
        CreateForumThread {
            forum_category_id: category_id,
            user_id: ADMIN_USER_ID,
            associated_page_id: None,
            title: "Dispatch Boundary Thread".to_owned(),
            description: String::new(),
            sticky: false,
            from_wikidot: false,
        },
    )
    .await
    .expect("dispatch boundary thread should be created");
    let _hidden_group = ForumService::create_group(
        runner.context(),
        CreateForumGroup {
            site_id,
            user_id: ADMIN_USER_ID,
            name: "Dispatch Hidden Group".to_owned(),
            description: "Dispatch hidden group".to_owned(),
            visible: false,
            sort_index: Some(20_001),
            from_wikidot: false,
        },
    )
    .await
    .expect("dispatch hidden group should be created");

    async fn module_status(
        runner: &TestRunner,
        site_id: i64,
        module_name: &str,
        parameters: serde_json::Value,
    ) -> String {
        run_endpoint!(
            runner,
            wikidot_forum_module,
            json!({
                "site_id": site_id,
                "module_name": module_name,
                "parameters": parameters,
            }),
        )
        .status
    }

    // The evidenced parameter shapes dispatch.
    assert_eq!(
        module_status(&runner, site_id, "forum/ForumStartModule", json!({})).await,
        "ok",
    );
    assert_eq!(
        module_status(
            &runner,
            site_id,
            "forum/ForumViewCategoryModule",
            json!({"c": category_id.to_string(), "p": "1"}),
        )
        .await,
        "ok",
    );

    // An unexpected extra parameter must fail closed rather than dispatch.
    assert_eq!(
        module_status(
            &runner,
            site_id,
            "forum/ForumStartModule",
            json!({"unexpected": "1"}),
        )
        .await,
        "not_ok",
    );
    assert_eq!(
        module_status(
            &runner,
            site_id,
            "forum/ForumViewCategoryModule",
            json!({"c": category_id.to_string(), "p": "1", "unexpected": "1"}),
        )
        .await,
        "not_ok",
    );
    assert_eq!(
        module_status(
            &runner,
            site_id,
            "forum/ForumCommentsListModule",
            json!({"pageId": "1", "unexpected": "1"}),
        )
        .await,
        "not_ok",
    );

    // `hidden` must be the literal "true" to opt in.
    assert_eq!(
        module_status(
            &runner,
            site_id,
            "forum/ForumStartModule",
            json!({"hidden": "true"}),
        )
        .await,
        "ok",
    );
    assert_eq!(
        module_status(
            &runner,
            site_id,
            "forum/ForumStartModule",
            json!({"hidden": "false"}),
        )
        .await,
        "not_ok",
    );

    // `hidden=true` must actually include hidden groups.
    assert!(
        run_endpoint!(
            runner,
            wikidot_forum_module,
            json!({
                "site_id": site_id,
                "module_name": "forum/ForumStartModule",
                "parameters": {"hidden": "true"},
            }),
        )
        .body
        .contains("Dispatch Hidden Group"),
        "hidden=true must render hidden groups",
    );

    // Thread module shapes fail closed on an extra parameter.
    let thread_id = thread.forum_thread_id.to_string();
    assert_eq!(
        module_status(
            &runner,
            site_id,
            "forum/ForumViewThreadModule",
            json!({"t": thread_id}),
        )
        .await,
        "ok",
    );
    assert_eq!(
        module_status(
            &runner,
            site_id,
            "forum/ForumViewThreadModule",
            json!({"t": thread_id, "unexpected": "1"}),
        )
        .await,
        "not_ok",
    );
    assert_eq!(
        module_status(
            &runner,
            site_id,
            "forum/ForumViewThreadPostsModule",
            json!({"t": thread_id, "pageNo": "1"}),
        )
        .await,
        "ok",
    );
    assert_eq!(
        module_status(
            &runner,
            site_id,
            "forum/ForumViewThreadPostsModule",
            json!({"t": thread_id, "pageNo": "1", "unexpected": "1"}),
        )
        .await,
        "not_ok",
    );

    // The category page cap is inclusive of the maximum page.
    assert_eq!(
        module_status(
            &runner,
            site_id,
            "forum/ForumViewCategoryModule",
            json!({"c": category_id.to_string(), "p": "50"}),
        )
        .await,
        "ok",
    );
    assert_eq!(
        module_status(
            &runner,
            site_id,
            "forum/ForumViewCategoryModule",
            json!({"c": category_id.to_string(), "p": "51"}),
        )
        .await,
        "not_ok",
    );
}

#[tokio::test]
async fn wikidot_forum_feeds_are_bounded_newest_first_and_hide_invisible_groups() {
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist");
    let site_id = site.site.site_id;

    let visible_group = ForumService::create_group(
        runner.context(),
        CreateForumGroup {
            site_id,
            user_id: ADMIN_USER_ID,
            name: "RSS visible group".to_owned(),
            description: String::new(),
            visible: true,
            sort_index: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("visible feed group should be created");
    let visible_category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: visible_group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "RSS visible category".to_owned(),
            description: String::new(),
            sort_index: None,
            max_nest_level: Some(3),
            per_page_discussion: Some(false),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("visible feed category should be created");
    let visible_thread = ForumThreadService::create(
        runner.context(),
        CreateForumThread {
            forum_category_id: visible_category.forum_category_id,
            user_id: ADMIN_USER_ID,
            associated_page_id: None,
            title: "Visible RSS thread".to_owned(),
            description: "Visible thread summary".to_owned(),
            sticky: false,
            from_wikidot: false,
        },
    )
    .await
    .expect("visible feed thread should be created");
    let visible_post = ForumPostService::create(
        runner.context(),
        CreateForumPost {
            forum_thread_id: visible_thread.forum_thread_id,
            parent_post_id: None,
            user_id: ADMIN_USER_ID,
            title: "Visible RSS post".to_owned(),
            wikitext: "Visible feed body <safe>".to_owned(),
            comments: "create visible forum feed post".to_owned(),
            from_wikidot: false,
        },
    )
    .await
    .expect("visible feed post should be created");
    let mut recent_post_ids = Vec::with_capacity(61);
    for index in 0..61 {
        let post = ForumPostService::create(
            runner.context(),
            CreateForumPost {
                forum_thread_id: visible_thread.forum_thread_id,
                parent_post_id: Some(visible_post.forum_post_id),
                user_id: ADMIN_USER_ID,
                title: format!("Visible RSS reply {index}"),
                wikitext: format!("Visible feed reply body {index}"),
                comments: "create bounded forum feed post".to_owned(),
                from_wikidot: false,
            },
        )
        .await
        .expect("visible feed reply should be created");
        recent_post_ids.push(post.forum_post_id);
    }

    let hidden_group = ForumService::create_group(
        runner.context(),
        CreateForumGroup {
            site_id,
            user_id: ADMIN_USER_ID,
            name: "RSS hidden group".to_owned(),
            description: String::new(),
            visible: false,
            sort_index: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("hidden feed group should be created");
    let hidden_category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: hidden_group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "RSS hidden category".to_owned(),
            description: String::new(),
            sort_index: None,
            max_nest_level: Some(3),
            per_page_discussion: Some(false),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("hidden feed category should be created");
    let hidden_thread = ForumThreadService::create(
        runner.context(),
        CreateForumThread {
            forum_category_id: hidden_category.forum_category_id,
            user_id: ADMIN_USER_ID,
            associated_page_id: None,
            title: "Hidden RSS thread marker".to_owned(),
            description: String::new(),
            sticky: false,
            from_wikidot: false,
        },
    )
    .await
    .expect("hidden feed thread should be created");
    ForumPostService::create(
        runner.context(),
        CreateForumPost {
            forum_thread_id: hidden_thread.forum_thread_id,
            parent_post_id: None,
            user_id: ADMIN_USER_ID,
            title: "Hidden RSS post marker".to_owned(),
            wikitext: "Hidden feed body marker".to_owned(),
            comments: "create hidden forum feed post".to_owned(),
            from_wikidot: false,
        },
    )
    .await
    .expect("hidden feed post should be created");

    let deleted_thread = ForumThreadService::create(
        runner.context(),
        CreateForumThread {
            forum_category_id: visible_category.forum_category_id,
            user_id: ADMIN_USER_ID,
            associated_page_id: None,
            title: "Deleted RSS thread marker".to_owned(),
            description: String::new(),
            sticky: false,
            from_wikidot: false,
        },
    )
    .await
    .expect("deleted feed thread should be created");
    let deleted_thread_post = ForumPostService::create(
        runner.context(),
        CreateForumPost {
            forum_thread_id: deleted_thread.forum_thread_id,
            parent_post_id: None,
            user_id: ADMIN_USER_ID,
            title: "Deleted RSS thread post marker".to_owned(),
            wikitext: "Deleted thread body marker".to_owned(),
            comments: "create deleted forum feed thread post".to_owned(),
            from_wikidot: false,
        },
    )
    .await
    .expect("deleted feed thread post should be created");
    let mut deleted_thread_model =
        ForumThreadTable::find_by_id(deleted_thread.forum_thread_id)
            .one(runner.context().transaction())
            .await
            .expect("deleted feed thread lookup should succeed")
            .expect("deleted feed thread should exist")
            .into_active_model();
    deleted_thread_model.deleted_by = Set(Some(ADMIN_USER_ID));
    deleted_thread_model.deleted_at = Set(Some(time::OffsetDateTime::now_utc()));
    deleted_thread_model.updated_by = Set(Some(ADMIN_USER_ID));
    deleted_thread_model.updated_at = Set(Some(time::OffsetDateTime::now_utc()));
    deleted_thread_model
        .update(runner.context().transaction())
        .await
        .expect("feed thread should be soft-deleted");

    let deleted_post_thread = ForumThreadService::create(
        runner.context(),
        CreateForumThread {
            forum_category_id: visible_category.forum_category_id,
            user_id: ADMIN_USER_ID,
            associated_page_id: None,
            title: "RSS deleted-post-only thread marker".to_owned(),
            description: String::new(),
            sticky: false,
            from_wikidot: false,
        },
    )
    .await
    .expect("deleted-post-only feed thread should be created");
    let deleted_post = ForumPostService::create(
        runner.context(),
        CreateForumPost {
            forum_thread_id: deleted_post_thread.forum_thread_id,
            parent_post_id: None,
            user_id: ADMIN_USER_ID,
            title: "Deleted RSS post marker".to_owned(),
            wikitext: "Deleted post body marker".to_owned(),
            comments: "create deleted forum feed post".to_owned(),
            from_wikidot: false,
        },
    )
    .await
    .expect("feed post should be created before deletion");
    ForumPostService::delete(
        runner.context(),
        DeleteForumPost {
            forum_post_id: deleted_post.forum_post_id,
            user_id: ADMIN_USER_ID,
        },
    )
    .await
    .expect("feed post should be soft-deleted");

    const PRIVATE_PAGE_SLUG: &str = "fixture-forum-feed-private-page";
    const PRIVATE_PAGE_CATEGORY: &str = "fixture-forum-feed-private-category";
    make_listpages_test_category_admin_only(&runner, site_id, PRIVATE_PAGE_CATEGORY)
        .await;
    create_listpages_test_page(
        &mut runner,
        site_id,
        PRIVATE_PAGE_SLUG,
        "Forum Feed Private Page",
        "private forum feed fixture",
    )
    .await;
    set_listpages_test_category_slug(
        &runner,
        site_id,
        PRIVATE_PAGE_SLUG,
        PRIVATE_PAGE_CATEGORY,
    )
    .await;
    let private_page_id =
        listpages_test_page_id(&runner, site_id, PRIVATE_PAGE_SLUG).await;
    let private_page_thread = ForumThreadService::create(
        runner.context(),
        CreateForumThread {
            forum_category_id: visible_category.forum_category_id,
            user_id: ADMIN_USER_ID,
            associated_page_id: Some(private_page_id),
            title: "Private RSS thread marker".to_owned(),
            description: String::new(),
            sticky: false,
            from_wikidot: false,
        },
    )
    .await
    .expect("private-page forum feed thread should be created");
    ForumPostService::create(
        runner.context(),
        CreateForumPost {
            forum_thread_id: private_page_thread.forum_thread_id,
            parent_post_id: None,
            user_id: ADMIN_USER_ID,
            title: "Private RSS post marker".to_owned(),
            wikitext: "Private page feed body marker".to_owned(),
            comments: "create private-page forum feed post".to_owned(),
            from_wikidot: false,
        },
    )
    .await
    .expect("private-page forum feed post should be created");

    runner.set_request_context(RequestContext {
        user_id: None,
        site_id: Some(site_id),
        ..Default::default()
    });
    let threads = run_endpoint!(
        runner,
        wikidot_forum_feed,
        json!({"site_id": site_id, "kind": "threads"}),
    )
    .expect("public thread feed should be viewable");
    assert_eq!(threads.items.len(), 1);
    assert_eq!(
        threads.items[0].forum_thread_id,
        visible_thread.forum_thread_id
    );
    assert_eq!(threads.items[0].thread_title, "Visible RSS thread");
    assert!(threads.items[0].content_html.contains("Visible feed body"));
    assert!(threads.items.iter().all(|item| {
        item.forum_thread_id != deleted_thread.forum_thread_id
            && item.forum_thread_id != deleted_post_thread.forum_thread_id
            && item.forum_thread_id != hidden_thread.forum_thread_id
            && item.forum_thread_id != private_page_thread.forum_thread_id
    }));

    let posts = run_endpoint!(
        runner,
        wikidot_forum_feed,
        json!({"site_id": site_id, "kind": "posts"}),
    )
    .expect("public post feed should be viewable");
    assert_eq!(posts.items.len(), 60);
    assert_eq!(
        posts.items[0].forum_post_id,
        *recent_post_ids.last().unwrap()
    );
    assert!(
        posts
            .items
            .windows(2)
            .all(|items| { items[0].forum_post_id > items[1].forum_post_id })
    );
    assert_eq!(
        posts.items[0].forum_thread_id,
        visible_thread.forum_thread_id
    );
    assert_eq!(posts.items[0].title, "Visible RSS reply 60");
    assert!(
        !posts.items[0]
            .content_html
            .contains("Hidden feed body marker")
    );
    assert!(
        posts
            .items
            .iter()
            .all(|item| item.forum_thread_id == visible_thread.forum_thread_id)
    );
    assert!(posts.items.iter().all(|item| {
        item.forum_post_id != deleted_thread_post.forum_post_id
            && item.forum_post_id != deleted_post.forum_post_id
            && item.forum_thread_id != private_page_thread.forum_thread_id
    }));
}

#[tokio::test]
async fn wikidot_forum_scoped_feeds_filter_to_the_named_object_and_hide_invisible_scopes()
{
    let mut runner = TestRunner::setup().await;
    let site = run_endpoint!(runner, site_get, json!({"site": "test"}))
        .expect("seeded test site should exist");
    let site_id = site.site.site_id;

    let visible_group = ForumService::create_group(
        runner.context(),
        CreateForumGroup {
            site_id,
            user_id: ADMIN_USER_ID,
            name: "Scoped visible group".to_owned(),
            description: String::new(),
            visible: true,
            sort_index: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("visible scoped group should be created");
    let scoped_category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: visible_group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "Scoped category".to_owned(),
            description: String::new(),
            sort_index: None,
            max_nest_level: Some(3),
            per_page_discussion: Some(false),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("scoped category should be created");
    let other_category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: visible_group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "Other category".to_owned(),
            description: String::new(),
            sort_index: None,
            max_nest_level: Some(3),
            per_page_discussion: Some(false),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("other category should be created");
    let hidden_group = ForumService::create_group(
        runner.context(),
        CreateForumGroup {
            site_id,
            user_id: ADMIN_USER_ID,
            name: "Scoped hidden group".to_owned(),
            description: String::new(),
            visible: false,
            sort_index: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("hidden scoped group should be created");
    let hidden_category = ForumService::create_category(
        runner.context(),
        CreateForumCategory {
            forum_group_id: hidden_group.forum_group_id,
            user_id: ADMIN_USER_ID,
            name: "Hidden scoped category".to_owned(),
            description: String::new(),
            sort_index: None,
            max_nest_level: Some(3),
            per_page_discussion: Some(false),
            layout: None,
            from_wikidot: false,
        },
    )
    .await
    .expect("hidden scoped category should be created");

    async fn create_thread(runner: &TestRunner, category_id: i64, title: &str) -> i64 {
        ForumThreadService::create(
            runner.context(),
            CreateForumThread {
                forum_category_id: category_id,
                user_id: ADMIN_USER_ID,
                associated_page_id: None,
                title: title.to_owned(),
                description: String::new(),
                sticky: false,
                from_wikidot: false,
            },
        )
        .await
        .expect("scoped forum thread should be created")
        .forum_thread_id
    }

    async fn create_post(runner: &TestRunner, thread_id: i64, title: &str) -> i64 {
        ForumPostService::create(
            runner.context(),
            CreateForumPost {
                forum_thread_id: thread_id,
                parent_post_id: None,
                user_id: ADMIN_USER_ID,
                title: title.to_owned(),
                wikitext: format!("{title} body"),
                comments: "create scoped forum post".to_owned(),
                from_wikidot: false,
            },
        )
        .await
        .expect("scoped forum post should be created")
        .forum_post_id
    }

    let older_thread =
        create_thread(&runner, scoped_category.forum_category_id, "Older thread").await;
    let older_post = create_post(&runner, older_thread, "Older thread post").await;
    let newer_thread =
        create_thread(&runner, scoped_category.forum_category_id, "Newer thread").await;
    let newer_post = create_post(&runner, newer_thread, "Newer thread post").await;
    let other_thread =
        create_thread(&runner, other_category.forum_category_id, "Other thread").await;
    let other_post = create_post(&runner, other_thread, "Other thread post").await;
    let hidden_thread =
        create_thread(&runner, hidden_category.forum_category_id, "Hidden thread").await;
    create_post(&runner, hidden_thread, "Hidden thread post").await;
    let deleted_thread =
        create_thread(&runner, scoped_category.forum_category_id, "Deleted thread").await;
    create_post(&runner, deleted_thread, "Deleted thread post").await;
    let mut deleted_thread_model = ForumThreadTable::find_by_id(deleted_thread)
        .one(runner.context().transaction())
        .await
        .expect("deleted scoped thread lookup should succeed")
        .expect("deleted scoped thread should exist")
        .into_active_model();
    deleted_thread_model.deleted_by = Set(Some(ADMIN_USER_ID));
    deleted_thread_model.deleted_at = Set(Some(time::OffsetDateTime::now_utc()));
    deleted_thread_model.updated_by = Set(Some(ADMIN_USER_ID));
    deleted_thread_model.updated_at = Set(Some(time::OffsetDateTime::now_utc()));
    deleted_thread_model
        .update(runner.context().transaction())
        .await
        .expect("scoped thread should be soft-deleted");

    runner.set_request_context(RequestContext {
        user_id: None,
        site_id: Some(site_id),
        ..Default::default()
    });

    let category_threads = run_endpoint!(
        runner,
        wikidot_forum_feed,
        json!({"site_id": site_id, "kind": "threads", "category_id": scoped_category.forum_category_id}),
    )
    .expect("category thread feed should be viewable");
    assert_eq!(category_threads.scope, "category");
    assert_eq!(
        category_threads.forum_thread_title, None,
        "a category feed names the category, not a thread"
    );
    assert_eq!(
        category_threads.forum_category_name.as_deref(),
        Some("Scoped category")
    );
    let category_thread_ids: Vec<i64> = category_threads
        .items
        .iter()
        .map(|item| item.forum_thread_id)
        .collect();
    assert_eq!(
        category_thread_ids,
        vec![newer_thread, older_thread],
        "category thread feed is newest first and excludes other scopes"
    );
    assert!(
        category_threads
            .items
            .iter()
            .all(|item| item.forum_category_id == scoped_category.forum_category_id)
    );

    let category_posts = run_endpoint!(
        runner,
        wikidot_forum_feed,
        json!({"site_id": site_id, "kind": "posts", "category_id": scoped_category.forum_category_id}),
    )
    .expect("category post feed should be viewable");
    let category_post_ids: Vec<i64> = category_posts
        .items
        .iter()
        .map(|item| item.forum_post_id)
        .collect();
    assert_eq!(category_post_ids, vec![newer_post, older_post]);
    assert!(
        category_posts
            .items
            .iter()
            .all(|item| item.forum_post_id != other_post)
    );

    let thread_posts = run_endpoint!(
        runner,
        wikidot_forum_feed,
        json!({"site_id": site_id, "kind": "posts", "thread_id": newer_thread}),
    )
    .expect("thread post feed should be viewable");
    assert_eq!(thread_posts.scope, "thread");
    assert_eq!(thread_posts.forum_thread_id, Some(newer_thread));
    assert_eq!(
        thread_posts.forum_thread_title.as_deref(),
        Some("Newer thread")
    );
    let thread_post_ids: Vec<i64> = thread_posts
        .items
        .iter()
        .map(|item| item.forum_post_id)
        .collect();
    assert_eq!(thread_post_ids, vec![newer_post]);

    // Hidden, deleted and unknown scopes are indistinguishable from a missing
    // one: the route answers 404 without leaking that the object exists.
    for missing_scope in [
        json!({"kind": "threads", "category_id": hidden_category.forum_category_id}),
        json!({"kind": "threads", "category_id": deleted_thread}),
        json!({"kind": "threads", "category_id": other_thread}),
        json!({"kind": "posts", "thread_id": hidden_thread}),
        json!({"kind": "posts", "thread_id": deleted_thread}),
        json!({"kind": "posts", "thread_id": other_thread}),
    ] {
        let mut params = json!({"site_id": site_id});
        params
            .as_object_mut()
            .expect("scope params should be an object")
            .extend(missing_scope.as_object().unwrap().clone());
        let scope_label = params.to_string();
        let feed = run_endpoint!(runner, wikidot_forum_feed, params);
        assert!(
            feed.is_none(),
            "invisible or unknown forum feed scope must not be viewable: {scope_label}"
        );
    }
}
