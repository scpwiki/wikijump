/*
 * services/render/forum_mini.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Permission-aware Wikidot mini forum activity modules.

use std::sync::LazyLock;

use regex::Regex;
use sea_orm::sea_query::ArrayType;
use sea_orm::{ConnectionTrait, FromQueryResult, Statement, Value};

use super::compat::CompatHtmlFragments;
use super::forum_visibility::ForumPageVisibility;
use super::literal_regions::LiteralRegionIndex;
use super::module_arguments::wikidot_module_arguments;
use super::service::{
    RenderService, escape_list_pages_html_attr, escape_list_pages_html_text,
    format_wikidot_list_pages_date,
};
use super::wikidot_user_resources::WIKIDOT_USER_INFO_URL;
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::services::ServiceContext;
use crate::services::text::TextService;
use crate::utils::normalize_slug_without_category_separator;
use ftml::settings::WikitextSettings;

const DEFAULT_FORUM_MINI_LIMIT: usize = 5;
const MAX_FORUM_MINI_LIMIT: usize = 100;
const FORUM_MINI_CANDIDATE_LIMIT: usize = 1_001;
const MAX_FORUM_MINI_MODULES_PER_RENDER: usize = 32;

pub(super) static FORUM_MINI_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r#"(?im)^[\t ]*\[\[module\s+(?:"#,
        r#"(?P<recent_threads>MiniRecentThreads)|"#,
        r#"(?P<active_threads>MiniActiveThreads)|MiniRecentPosts)"#,
        r#"\b(?P<head>(?:[^\]"]+|"[^"]*")*)\]\][\t ]*$"#,
    ))
    .expect("mini forum module expression is valid")
});

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ForumMiniModuleKind {
    RecentThreads,
    ActiveThreads,
    RecentPosts,
}

#[derive(Debug, FromQueryResult)]
struct ForumMiniThreadCandidate {
    forum_thread_id: i64,
    title: String,
    created_at: time::OffsetDateTime,
    post_count: i64,
}

#[derive(Debug)]
struct ForumMiniThread {
    forum_thread_id: i64,
    title: String,
    created_at: time::OffsetDateTime,
    post_count: i64,
}

#[derive(Debug, FromQueryResult)]
struct ForumMiniPostCandidate {
    forum_post_id: i64,
    forum_thread_id: i64,
    thread_title: String,
    page_slug: Option<String>,
    user_id: i64,
    created_at: time::OffsetDateTime,
    title: String,
    wikitext_hash: Vec<u8>,
    post_count: i64,
    wikidot_user_name: Option<String>,
    wikidot_user_slug: Option<String>,
    local_user_name: Option<String>,
    local_user_slug: Option<String>,
}

#[derive(Debug)]
struct ForumMiniPost {
    forum_post_id: i64,
    forum_thread_id: i64,
    thread_title: String,
    page_slug: Option<String>,
    user_id: i64,
    created_at: time::OffsetDateTime,
    title: String,
    excerpt: String,
    post_count: i64,
    user_name: String,
    user_slug: Option<String>,
    wikidot_profile: bool,
}

fn parse_module_kind(captures: &regex::Captures<'_>) -> ForumMiniModuleKind {
    if captures.name("recent_threads").is_some() {
        ForumMiniModuleKind::RecentThreads
    } else if captures.name("active_threads").is_some() {
        ForumMiniModuleKind::ActiveThreads
    } else {
        ForumMiniModuleKind::RecentPosts
    }
}

fn parse_forum_mini_limit(head: &str) -> Option<usize> {
    let arguments = wikidot_module_arguments(head)?;
    let limit = arguments
        .iter()
        .rev()
        .find(|argument| argument.key.eq_ignore_ascii_case("limit"))
        .and_then(|argument| argument.value.parse::<usize>().ok())
        .filter(|limit| (1..=MAX_FORUM_MINI_LIMIT).contains(limit))
        .unwrap_or(DEFAULT_FORUM_MINI_LIMIT);
    Some(limit)
}

fn forum_thread_path(thread: &ForumMiniThread) -> String {
    format!(
        "/forum/t-{}/{}",
        thread.forum_thread_id,
        normalize_slug_without_category_separator(&thread.title),
    )
}

fn forum_mini_date(created_at: time::OffsetDateTime) -> String {
    let created_at = created_at.to_offset(time::UtcOffset::UTC);
    format!(
        r#"<span class="odate time_{} format_%25O%20ago">{}</span>"#,
        created_at.unix_timestamp(),
        format_wikidot_list_pages_date(created_at, "%d %b %Y %H:%M"),
    )
}

fn render_forum_mini_threads(threads: &[ForumMiniThread], active: bool) -> String {
    let mut output = String::from("<div class=\"forum-mini-stat\">\n");
    for thread in threads {
        output.push_str("<div class=\"item\">\n<div class=\"title\">\n<a href=\"");
        output.push_str(&escape_list_pages_html_attr(&forum_thread_path(thread)));
        output.push_str("\">");
        output.push_str(&escape_list_pages_html_text(&thread.title));
        output.push_str("</a>\n</div>\n<div class=\"info\">\n(Started ");
        output.push_str(&forum_mini_date(thread.created_at));
        output.push_str(if active { " , Posts: " } else { ", Posts: " });
        output.push_str(&thread.post_count.max(0).to_string());
        output.push_str(")\n</div>\n</div>\n");
    }
    output.push_str("</div>");
    output
}

fn render_forum_mini_user(post: &ForumMiniPost) -> String {
    let name = escape_list_pages_html_text(&post.user_name);
    let Some(slug) = post.user_slug.as_deref().filter(|_| post.wikidot_profile) else {
        return format!(r#"<span class="printuser">{name}</span>"#);
    };
    let slug = escape_list_pages_html_attr(slug);
    format!(
        concat!(
            r#"<span class="printuser"><a href="{user_info}{slug}" "#,
            r#"onclick="WIKIDOT.page.listeners.userInfo({user_id}); return false;" >{name}</a></span>"#,
        ),
        user_info = WIKIDOT_USER_INFO_URL,
        slug = slug,
        user_id = post.user_id,
        name = name,
    )
}

fn forum_post_path(post: &ForumMiniPost) -> String {
    match post.page_slug.as_deref() {
        Some(page_slug) => {
            format!("/{}/comments/show#post-{}", page_slug, post.forum_post_id,)
        }
        None => format!(
            "/forum/t-{}/{}#post-{}",
            post.forum_thread_id,
            normalize_slug_without_category_separator(&post.thread_title),
            post.forum_post_id,
        ),
    }
}

fn render_forum_mini_posts(posts: &[ForumMiniPost]) -> String {
    let mut output = String::from("<div class=\"forum-mini-stat\" >\n");
    for post in posts {
        output.push_str(
            "<div class=\"item\" style=\"padding-bottom: 5px\">\n<div class=\"title\">\n<a href=\"",
        );
        output.push_str(&escape_list_pages_html_attr(&forum_post_path(post)));
        output.push_str("\">");
        output.push_str(&escape_list_pages_html_text(&post.title));
        output.push_str("</a>\n</div>\n<div class=\"info\">\n");
        output.push_str(&escape_list_pages_html_text(&post.excerpt));
        output.push_str("\n<br/>\n(by ");
        output.push_str(&render_forum_mini_user(post));
        output.push(' ');
        output.push_str(&forum_mini_date(post.created_at));
        output.push_str(", posts: ");
        output.push_str(&post.post_count.max(0).to_string());
        output.push_str(")\n</div>\n</div>\n");
    }
    output.push_str("</div>");
    output
}

fn forum_post_excerpt(wikitext: &str) -> String {
    const MAX_EXCERPT_CHARACTERS: usize = 160;
    let normalized = wikitext.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut excerpt: String = normalized.chars().take(MAX_EXCERPT_CHARACTERS).collect();
    if normalized.chars().count() > MAX_EXCERPT_CHARACTERS {
        excerpt.push_str("...");
    }
    excerpt
}

async fn load_forum_mini_threads(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    kind: ForumMiniModuleKind,
    limit: usize,
) -> Result<Option<Vec<ForumMiniThread>>> {
    let make_error =
        || Error::new("failed to load mini forum threads", ErrorType::Render);
    let mut visibility = ForumPageVisibility::new(ctx, ctx.request().user_id);
    if !visibility.site_is_viewable(site_id).await? {
        return Ok(None);
    }
    let Some(visible_threads) = visibility
        .visible_thread_ids(site_id, None, None, true)
        .await?
    else {
        return Ok(None);
    };
    let visibility_complete = visible_threads.complete;
    let visible_thread_ids = visible_threads.ids;
    if visible_thread_ids.is_empty() {
        return Ok(Some(Vec::new()));
    }
    let visible_thread_ids = visible_thread_ids
        .into_iter()
        .map(Value::from)
        .collect::<Vec<_>>();
    let order = match kind {
        ForumMiniModuleKind::RecentThreads => {
            "thread_created_at DESC, forum_thread_id DESC"
        }
        ForumMiniModuleKind::ActiveThreads => concat!(
            "recent_post_count DESC, activity_at DESC, ",
            "thread_created_at DESC, forum_thread_id DESC",
        ),
        ForumMiniModuleKind::RecentPosts => {
            unreachable!("post modules use the post query")
        }
    };
    let active_filter = if kind == ForumMiniModuleKind::ActiveThreads {
        "WHERE recent_post_count > 0"
    } else {
        ""
    };
    let sql = format!(
        "WITH activity AS (\
         SELECT t.forum_thread_id, t.title, \
                t.created_at AS thread_created_at, \
                COALESCE(t.updated_at, t.created_at) AS activity_at, \
                COUNT(fp.forum_post_id) FILTER (\
                    WHERE t.page_id IS NOT NULL OR fp.parent_post_id IS NOT NULL\
                ) AS post_count, \
                COUNT(fp.forum_post_id) FILTER (\
                    WHERE fp.created_at >= NOW() - INTERVAL '7 days' \
                      AND (t.page_id IS NOT NULL OR fp.parent_post_id IS NOT NULL)\
                ) AS recent_post_count \
         FROM forum_thread t \
         JOIN forum_category c ON c.forum_category_id = t.forum_category_id \
                              AND c.site_id = t.site_id \
                              AND c.deleted_at IS NULL \
         JOIN forum_group g ON g.forum_group_id = t.forum_group_id \
                           AND g.site_id = t.site_id \
                           AND g.deleted_at IS NULL \
                           AND g.visible = TRUE \
         LEFT JOIN page p ON p.page_id = t.page_id \
                         AND p.site_id = t.site_id \
                         AND p.deleted_at IS NULL \
         LEFT JOIN forum_post fp ON fp.forum_thread_id = t.forum_thread_id \
                                AND fp.site_id = t.site_id \
                                AND fp.deleted_at IS NULL \
         WHERE t.site_id = $1 \
           AND t.forum_thread_id = ANY($2::BIGINT[]) \
           AND t.deleted_at IS NULL \
           AND (t.page_id IS NULL OR p.page_id IS NOT NULL) \
         GROUP BY t.forum_thread_id, p.page_category_id\
         ) \
         SELECT forum_thread_id, title, \
                thread_created_at AS created_at, post_count \
         FROM activity {active_filter} \
         ORDER BY {order} \
         LIMIT {FORUM_MINI_CANDIDATE_LIMIT}",
    );
    let candidates =
        ForumMiniThreadCandidate::find_by_statement(Statement::from_sql_and_values(
            ctx.transaction().get_database_backend(),
            sql,
            [
                Value::from(site_id),
                Value::Array(ArrayType::BigInt, Some(Box::new(visible_thread_ids))),
            ],
        ))
        .all(ctx.transaction())
        .await
        .or_raise(make_error)?;
    let scan_exhausted = candidates.len() == FORUM_MINI_CANDIDATE_LIMIT;
    let mut threads = Vec::with_capacity(limit);
    for candidate in candidates {
        threads.push(ForumMiniThread {
            forum_thread_id: candidate.forum_thread_id,
            title: candidate.title,
            created_at: candidate.created_at,
            post_count: candidate.post_count,
        });
        if threads.len() == limit {
            return Ok(Some(threads));
        }
    }
    if scan_exhausted && visibility_complete {
        return Ok(None);
    }
    Ok(Some(threads))
}

async fn load_forum_mini_posts(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    limit: usize,
) -> Result<Option<Vec<ForumMiniPost>>> {
    let make_error = || Error::new("failed to load mini forum posts", ErrorType::Render);
    let mut visibility = ForumPageVisibility::new(ctx, ctx.request().user_id);
    if !visibility.site_is_viewable(site_id).await? {
        return Ok(None);
    }
    let Some(visible_threads) = visibility
        .visible_thread_ids(site_id, None, None, true)
        .await?
    else {
        return Ok(None);
    };
    let visibility_complete = visible_threads.complete;
    let visible_thread_ids = visible_threads.ids;
    if visible_thread_ids.is_empty() {
        return Ok(Some(Vec::new()));
    }
    let visible_thread_ids = visible_thread_ids
        .into_iter()
        .map(Value::from)
        .collect::<Vec<_>>();
    let sql = format!(
        "SELECT fp.forum_post_id, fp.forum_thread_id, t.title AS thread_title, \
                p.slug AS page_slug, fp.user_id, \
                fp.created_at, fpr.title, fpr.wikitext_hash, \
                counts.post_count, wu.name AS wikidot_user_name, \
                wu.slug AS wikidot_user_slug, local_user.name AS local_user_name, \
                local_user.slug AS local_user_slug \
         FROM forum_post fp \
         JOIN forum_thread t ON t.forum_thread_id = fp.forum_thread_id \
                            AND t.site_id = fp.site_id \
                            AND t.deleted_at IS NULL \
         JOIN forum_category c ON c.forum_category_id = fp.forum_category_id \
                              AND c.site_id = fp.site_id \
                              AND c.deleted_at IS NULL \
         JOIN forum_group g ON g.forum_group_id = fp.forum_group_id \
                           AND g.site_id = fp.site_id \
                           AND g.deleted_at IS NULL \
                           AND g.visible = TRUE \
         JOIN forum_post_revision fpr ON fpr.forum_post_revision_id = fp.latest_revision_id \
                                     AND fpr.site_id = fp.site_id \
         LEFT JOIN page p ON p.page_id = t.page_id \
                         AND p.site_id = t.site_id \
                         AND p.deleted_at IS NULL \
         LEFT JOIN wikidot_display_user wu ON wu.user_id = fp.user_id \
                                  AND wu.is_deleted = FALSE \
         LEFT JOIN \"user\" local_user ON local_user.user_id = fp.user_id \
                                     AND local_user.deleted_at IS NULL \
         JOIN LATERAL (\
             SELECT COUNT(fp0.forum_post_id) FILTER (\
                        WHERE t.page_id IS NOT NULL OR fp0.parent_post_id IS NOT NULL\
                    ) AS post_count \
             FROM forum_post fp0 \
             WHERE fp0.forum_thread_id = t.forum_thread_id \
               AND fp0.site_id = t.site_id \
               AND fp0.deleted_at IS NULL\
         ) counts ON TRUE \
         WHERE fp.site_id = $1 \
           AND t.forum_thread_id = ANY($2::BIGINT[]) \
           AND fp.deleted_at IS NULL \
           AND (t.page_id IS NULL OR p.page_id IS NOT NULL) \
           AND (t.page_id IS NOT NULL OR fp.parent_post_id IS NOT NULL) \
         ORDER BY fp.created_at DESC, fp.forum_post_id DESC \
         LIMIT {FORUM_MINI_CANDIDATE_LIMIT}",
    );
    let candidates =
        ForumMiniPostCandidate::find_by_statement(Statement::from_sql_and_values(
            ctx.transaction().get_database_backend(),
            sql,
            [
                Value::from(site_id),
                Value::Array(ArrayType::BigInt, Some(Box::new(visible_thread_ids))),
            ],
        ))
        .all(ctx.transaction())
        .await
        .or_raise(make_error)?;
    let scan_exhausted = candidates.len() == FORUM_MINI_CANDIDATE_LIMIT;
    let mut posts = Vec::with_capacity(limit);
    for candidate in candidates {
        let wikitext = TextService::get(ctx, &candidate.wikitext_hash)
            .await
            .or_raise(make_error)?;
        let (user_name, user_slug, wikidot_profile) = if let Some(name) = candidate
            .wikidot_user_name
            .or_else(|| candidate.wikidot_user_slug.clone())
        {
            (name, candidate.wikidot_user_slug, true)
        } else if let Some(name) = candidate
            .local_user_name
            .or_else(|| candidate.local_user_slug.clone())
        {
            (name, candidate.local_user_slug, false)
        } else {
            (candidate.user_id.to_string(), None, false)
        };
        posts.push(ForumMiniPost {
            forum_post_id: candidate.forum_post_id,
            forum_thread_id: candidate.forum_thread_id,
            thread_title: candidate.thread_title,
            page_slug: candidate.page_slug,
            user_id: candidate.user_id,
            created_at: candidate.created_at,
            title: candidate.title,
            excerpt: forum_post_excerpt(&wikitext),
            post_count: candidate.post_count,
            user_name,
            user_slug,
            wikidot_profile,
        });
        if posts.len() == limit {
            return Ok(Some(posts));
        }
    }
    if scan_exhausted && visibility_complete {
        return Ok(None);
    }
    Ok(Some(posts))
}

impl RenderService {
    pub(super) async fn expand_forum_mini_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax || !FORUM_MINI_MODULE_REGEX.is_match(&wikitext) {
            return Ok(wikitext);
        }
        let Some(site_id) = current_site_id else {
            return Ok(wikitext);
        };
        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        let mut expanded_count = 0;
        for captures in FORUM_MINI_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a mini forum capture has a complete match");
            if literal_regions.contains(matched.start())
                || expanded_count == MAX_FORUM_MINI_MODULES_PER_RENDER
            {
                continue;
            }
            let head = captures.name("head").map_or("", |mtch| mtch.as_str());
            let Some(limit) = parse_forum_mini_limit(head) else {
                continue;
            };
            let kind = parse_module_kind(&captures);
            let rendered = match kind {
                ForumMiniModuleKind::RecentPosts => {
                    load_forum_mini_posts(ctx, site_id, limit)
                        .await?
                        .map(|posts| render_forum_mini_posts(&posts))
                }
                ForumMiniModuleKind::RecentThreads
                | ForumMiniModuleKind::ActiveThreads => {
                    load_forum_mini_threads(ctx, site_id, kind, limit)
                        .await?
                        .map(|threads| {
                            render_forum_mini_threads(
                                &threads,
                                kind == ForumMiniModuleKind::ActiveThreads,
                            )
                        })
                }
            };
            let Some(rendered) = rendered else {
                continue;
            };
            output.push_str(&wikitext[cursor..matched.start()]);
            output.push_str(&compat_html.push_block_html(rendered));
            cursor = matched.end();
            expanded_count += 1;
        }
        if cursor == 0 {
            return Ok(wikitext);
        }
        output.push_str(&wikitext[cursor..]);
        Ok(output)
    }
}
