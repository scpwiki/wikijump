/*
 * services/render/forum_modules.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Permission-aware Wikidot forum index and recent-post modules.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::LazyLock;

use regex::Regex;
use sea_orm::sea_query::ArrayType;
use sea_orm::{ConnectionTrait, FromQueryResult, Statement, Value};

use super::compat::CompatHtmlFragments;
use super::forum_comments::{self, ForumCommentsLoad, ForumCommentsOrder};
use super::forum_front::{self, FrontForumArgumentsParse, FrontForumLoad};
use super::forum_visibility::ForumPageVisibility;
use super::literal_regions::LiteralRegionIndex;
use super::module_arguments::{WikidotModuleArgumentValueKind, wikidot_module_arguments};
use super::service::{
    RenderService, escape_list_pages_html_attr, escape_list_pages_html_text,
    format_wikidot_list_pages_date,
};
use super::url_arguments::UrlArguments;
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::services::ForumService;
use crate::services::ServiceContext;
use crate::services::forum::{ForumGroupStructure, GetForumStructure};
use crate::services::text::TextService;
use crate::utils::{normalize_page_slug, normalize_slug_without_category_separator};
use ftml::settings::WikitextSettings;
use ftml::tree::{Element, Module, SyntaxTree};

const RECENT_POSTS_PER_PAGE: usize = 20;
const RECENT_POSTS_CANDIDATE_LIMIT: usize = 1_001;
const FORUM_START_CANDIDATE_LIMIT: usize = 1_001;
const MAX_RECENT_POSTS_PAGE: u32 = 50;
const MAX_FORUM_MODULES_PER_RENDER: usize = 32;
const FRONT_FORUM_MISSING_CATEGORY_HTML: &str = concat!(
    "<div class=\"error-block\">",
    "Requested forum category does not exist.</div>",
);
const FRONT_FORUM_CATEGORY_ERROR_HTML: &str = concat!(
    "<div class=\"error-block\">",
    "Problem parsing attribute \"category\".</div>",
);

pub(super) static FORUM_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r#"(?im)^[\t ]*\[\[module\s+"#,
        r#"(?P<name>Comments|FrontForum|ForumCategory|ForumNewThread|"#,
        r#"ForumStart|ForumThread|RecentPosts|RecentThreads)\b"#,
        r#"(?P<head>(?:[^\]"]+|"[^"]*")*)\]\][\t ]*$"#,
    ))
    .expect("forum module expression is valid")
});

static MODULE_BOUNDARY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?im)^[\t ]*\[\[(?P<close>/)?module\b")
        .expect("module boundary expression is valid")
});

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ForumModuleKind {
    Comments,
    FrontForum,
    ForumCategory,
    ForumNewThread,
    ForumStart,
    ForumThread,
    RecentPosts,
    RecentThreads,
}

#[derive(Clone, Copy, Debug)]
struct CommentsArguments<'a> {
    title: Option<&'a str>,
    hide: bool,
    hide_form: bool,
    order: ForumCommentsOrder,
    query_safe: bool,
}

impl Default for CommentsArguments<'_> {
    fn default() -> Self {
        Self {
            title: None,
            hide: false,
            hide_form: false,
            order: ForumCommentsOrder::Forward,
            query_safe: true,
        }
    }
}

fn comments_arguments(head: &str) -> CommentsArguments<'_> {
    if head.trim().is_empty() {
        return CommentsArguments::default();
    }
    let Some(arguments) = wikidot_module_arguments(head) else {
        return CommentsArguments {
            query_safe: false,
            ..CommentsArguments::default()
        };
    };
    let mut output = CommentsArguments::default();
    let mut title_seen = false;
    let mut hide_seen = false;
    let mut hide_form_seen = false;
    let mut order_seen = false;
    for argument in arguments {
        if argument.op != "="
            || argument.value_kind != WikidotModuleArgumentValueKind::DoubleQuoted
        {
            output.query_safe = false;
            continue;
        }
        match argument.key {
            "title" if !title_seen && !argument.value.is_empty() => {
                title_seen = true;
                output.title = Some(argument.value);
            }
            "hide" if !hide_seen && matches!(argument.value, "true" | "false") => {
                hide_seen = true;
                output.hide = argument.value == "true";
            }
            "hideForm"
                if !hide_form_seen
                    && matches!(argument.value, "false" | "true" | "yes") =>
            {
                hide_form_seen = true;
                output.hide_form = matches!(argument.value, "true" | "yes");
            }
            "order"
                if !order_seen && matches!(argument.value, "forwards" | "reverse") =>
            {
                order_seen = true;
                output.order = if argument.value == "reverse" {
                    ForumCommentsOrder::Reverse
                } else {
                    ForumCommentsOrder::Forward
                };
            }
            _ => output.query_safe = false,
        }
    }
    if !output.query_safe {
        return CommentsArguments {
            query_safe: false,
            ..CommentsArguments::default()
        };
    }
    output
}

#[derive(Clone, Debug)]
pub(super) struct ForumUserDisplay {
    user_id: i64,
    name: String,
    slug: Option<String>,
    wikidot_profile: bool,
    guest_gravatar_md5: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ForumUserResourceScheme {
    Http,
    Https,
}

impl ForumUserResourceScheme {
    fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct ForumLastPost {
    pub(super) forum_post_id: i64,
    pub(super) forum_thread_id: i64,
    pub(super) user: ForumUserDisplay,
    pub(super) created_at: time::OffsetDateTime,
}

#[derive(Clone, Debug, Default)]
pub(super) struct ForumCategoryActivity {
    thread_count: i64,
    post_count: i64,
    last_post: Option<ForumLastPost>,
}

#[derive(Debug, FromQueryResult)]
struct ForumStartThreadCandidate {
    forum_thread_id: i64,
    forum_category_id: i64,
    post_count: i64,
    last_forum_post_id: Option<i64>,
    last_user_id: Option<i64>,
    last_created_at: Option<time::OffsetDateTime>,
    wikidot_user_name: Option<String>,
    wikidot_user_slug: Option<String>,
    local_user_name: Option<String>,
    local_user_slug: Option<String>,
    guest_name: Option<String>,
    guest_email_md5: Option<String>,
}

#[derive(Debug, FromQueryResult)]
struct RecentPostCandidate {
    forum_post_id: i64,
    forum_thread_id: i64,
    forum_category_id: i64,
    group_name: String,
    category_name: String,
    thread_title: String,
    page_slug: Option<String>,
    page_title: Option<String>,
    user_id: i64,
    created_at: time::OffsetDateTime,
    title: String,
    compiled_html_hash: Vec<u8>,
    wikidot_user_name: Option<String>,
    wikidot_user_slug: Option<String>,
    local_user_name: Option<String>,
    local_user_slug: Option<String>,
    forum_signature: Option<String>,
    guest_name: Option<String>,
    guest_email_md5: Option<String>,
}

#[derive(Debug)]
struct RecentPost {
    forum_post_id: i64,
    forum_thread_id: i64,
    forum_category_id: i64,
    group_name: String,
    category_name: String,
    thread_title: String,
    page_slug: Option<String>,
    page_title: Option<String>,
    user: ForumUserDisplay,
    created_at: time::OffsetDateTime,
    title: String,
    compiled_html: String,
    signature_html: Option<String>,
}

pub(super) async fn render_forum_signature_html(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    source: Option<&str>,
) -> Result<Option<String>> {
    let Some(source) = source.filter(|source| !source.is_empty()) else {
        return Ok(None);
    };
    let output =
        RenderService::render_wikidot_syntax_preview(ctx, site_id, "", source.to_owned())
            .await?;
    Ok(Some(output.html_output.body))
}

#[derive(Debug)]
pub(super) struct RecentPostsPage {
    posts: Vec<RecentPost>,
    page: u32,
    known_page_count: u32,
    page_count_is_lower_bound: bool,
}

fn module_kind(name: &str) -> ForumModuleKind {
    if name.eq_ignore_ascii_case("Comments") {
        ForumModuleKind::Comments
    } else if name.eq_ignore_ascii_case("FrontForum") {
        ForumModuleKind::FrontForum
    } else if name.eq_ignore_ascii_case("ForumCategory") {
        ForumModuleKind::ForumCategory
    } else if name.eq_ignore_ascii_case("ForumNewThread") {
        ForumModuleKind::ForumNewThread
    } else if name.eq_ignore_ascii_case("ForumStart") {
        ForumModuleKind::ForumStart
    } else if name.eq_ignore_ascii_case("ForumThread") {
        ForumModuleKind::ForumThread
    } else if name.eq_ignore_ascii_case("RecentPosts") {
        ForumModuleKind::RecentPosts
    } else {
        ForumModuleKind::RecentThreads
    }
}

fn next_module_boundary_is_closer(wikitext: &str, opener_end: usize) -> bool {
    MODULE_BOUNDARY_REGEX
        .captures(&wikitext[opener_end..])
        .is_some_and(|captures| captures.name("close").is_some())
}

#[derive(Clone, Copy, Debug)]
pub(super) struct OwnedFrontForumBody<'a> {
    pub(super) source: &'a str,
    pub(super) replacement_end: usize,
}

pub(super) fn owned_front_forum_body(
    wikitext: &str,
    opener_end: usize,
) -> Option<OwnedFrontForumBody<'_>> {
    let captures = MODULE_BOUNDARY_REGEX.captures(&wikitext[opener_end..])?;
    captures.name("close")?;
    let boundary = captures.get(0)?;
    let boundary_start = opener_end + boundary.start();
    let tail = &wikitext[boundary_start..];
    let line_end = tail.find('\n').unwrap_or(tail.len());
    let closer = tail[..line_end]
        .strip_suffix('\r')
        .unwrap_or(&tail[..line_end]);
    if closer.trim() != "[[/module]]" {
        return None;
    }
    Some(OwnedFrontForumBody {
        source: &wikitext[opener_end..boundary_start],
        replacement_end: boundary_start + line_end,
    })
}

pub(super) fn render_front_forum_items(
    wikitext: &str,
    opener_end: usize,
    items: &[forum_front::FrontForumItem],
    compat_html: &mut CompatHtmlFragments,
) -> (String, usize, bool) {
    match owned_front_forum_body(wikitext, opener_end) {
        Some(body) => {
            match forum_front::render_custom_body(items, body.source, compat_html) {
                Some(rendered) => (rendered, body.replacement_end, true),
                None => (forum_front::render(items), body.replacement_end, false),
            }
        }
        None => (forum_front::render(items), opener_end, false),
    }
}

pub(super) fn resolve_typed_root_recent_threads_runtime_modules(
    tree: &mut SyntaxTree<'_>,
) {
    for element in &mut tree.elements {
        let Element::Module(Module::Runtime { name, .. }) = element else {
            continue;
        };
        if name.eq_ignore_ascii_case("RecentThreads") {
            *element = Element::Text(Cow::Borrowed(RECENT_THREADS_PLACEHOLDER));
        }
    }
}

pub(super) fn forum_user(
    user_id: i64,
    wikidot_user_name: Option<String>,
    wikidot_user_slug: Option<String>,
    local_user_name: Option<String>,
    local_user_slug: Option<String>,
) -> ForumUserDisplay {
    if let Some(name) = wikidot_user_name.or_else(|| wikidot_user_slug.clone()) {
        ForumUserDisplay {
            user_id,
            name,
            slug: wikidot_user_slug,
            wikidot_profile: true,
            guest_gravatar_md5: None,
        }
    } else if let Some(name) = local_user_name.or_else(|| local_user_slug.clone()) {
        ForumUserDisplay {
            user_id,
            name,
            slug: local_user_slug,
            wikidot_profile: false,
            guest_gravatar_md5: None,
        }
    } else {
        ForumUserDisplay {
            user_id,
            name: user_id.to_string(),
            slug: None,
            wikidot_profile: false,
            guest_gravatar_md5: None,
        }
    }
}

pub(super) fn forum_guest_user(name: String, gravatar_md5: String) -> ForumUserDisplay {
    ForumUserDisplay {
        user_id: crate::constants::ANONYMOUS_USER_ID,
        name,
        slug: None,
        wikidot_profile: false,
        guest_gravatar_md5: Some(gravatar_md5),
    }
}

pub(super) fn render_forum_user(
    user: &ForumUserDisplay,
    avatar_timestamp: i64,
) -> String {
    render_forum_user_with_scheme(user, avatar_timestamp, ForumUserResourceScheme::Http)
}

pub(super) fn render_forum_user_with_scheme(
    user: &ForumUserDisplay,
    avatar_timestamp: i64,
    resource_scheme: ForumUserResourceScheme,
) -> String {
    let name = escape_list_pages_html_text(&user.name);
    if let Some(gravatar_md5) = &user.guest_gravatar_md5 {
        let gravatar_md5 = escape_list_pages_html_attr(gravatar_md5);
        return format!(
            concat!(
                r#"<span class="printuser avatarhover"><a href="javascript:;"><img alt="" class="small" "#,
                r#"src="http://www.gravatar.com/avatar.php?gravatar_id={}&amp;default=http://www.wikidot.com/common--images/avatars/default/a16.png&amp;size=16"/></a>{} (guest)</span>"#,
            ),
            gravatar_md5, name,
        );
    }
    let Some(slug) = user.slug.as_deref().filter(|_| user.wikidot_profile) else {
        return format!(r#"<span class="printuser">{name}</span>"#);
    };
    let slug = escape_list_pages_html_attr(slug);
    let name_attr = escape_list_pages_html_attr(&user.name);
    let resource_scheme = resource_scheme.as_str();
    format!(
        concat!(
            r#"<span class="printuser avatarhover"><a href="http://www.wikidot.com/user:info/{slug}" "#,
            r#"onclick="WIKIDOT.page.listeners.userInfo({user_id}); return false;"  >"#,
            r#"<img class="small" src="{resource_scheme}://www.wikidot.com/avatar.php?userid={user_id}&amp;amp;size=small&amp;amp;timestamp={avatar_timestamp}" "#,
            r#"alt="{name_attr}" style="background-image:url({resource_scheme}://www.wikidot.com/userkarma.php?u={user_id})"/></a>"#,
            r#"<a href="http://www.wikidot.com/user:info/{slug}" onclick="WIKIDOT.page.listeners.userInfo({user_id}); return false;" >{name}</a></span>"#,
        ),
        resource_scheme = resource_scheme,
        slug = slug,
        user_id = user.user_id,
        avatar_timestamp = avatar_timestamp,
        name_attr = name_attr,
        name = name,
    )
}

pub(super) fn render_forum_user_without_avatar(user: &ForumUserDisplay) -> String {
    let name = escape_list_pages_html_text(&user.name);
    if user.guest_gravatar_md5.is_some() {
        return format!(r#"<span class="printuser">{name} (guest)</span>"#);
    }
    let Some(slug) = user.slug.as_deref().filter(|_| user.wikidot_profile) else {
        return format!(r#"<span class="printuser">{name}</span>"#);
    };
    let slug = escape_list_pages_html_attr(slug);
    format!(
        concat!(
            r#"<span class="printuser"><a href="http://www.wikidot.com/user:info/{slug}" "#,
            r#"onclick="WIKIDOT.page.listeners.userInfo({user_id}); return false;" >{name}</a></span>"#,
        ),
        slug = slug,
        user_id = user.user_id,
        name = name,
    )
}

pub(super) fn render_forum_date(
    created_at: time::OffsetDateTime,
    format_class: &str,
    display_format: &str,
) -> String {
    let created_at = created_at.to_offset(time::UtcOffset::UTC);
    format!(
        r#"<span class="odate time_{} {}">{}</span>"#,
        created_at.unix_timestamp(),
        format_class,
        format_wikidot_list_pages_date(created_at, display_format),
    )
}

pub(super) async fn load_forum_start_activity(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    viewer_user_id: Option<i64>,
    include_hidden: bool,
) -> Result<Option<BTreeMap<i64, ForumCategoryActivity>>> {
    let make_error = || Error::new("failed to load forum index", ErrorType::Render);
    let mut visibility = ForumPageVisibility::new(ctx, viewer_user_id);
    if !visibility.site_is_viewable(site_id).await? {
        return Ok(None);
    }
    let Some(visible_threads) = visibility
        .visible_thread_ids(site_id, None, None, !include_hidden)
        .await?
    else {
        return Ok(None);
    };
    let visibility_complete = visible_threads.complete;
    let visible_thread_ids = visible_threads.ids;
    if visible_thread_ids.is_empty() {
        return Ok(Some(BTreeMap::new()));
    }
    let visible_thread_ids = visible_thread_ids
        .into_iter()
        .map(Value::from)
        .collect::<Vec<_>>();
    let candidates = ForumStartThreadCandidate::find_by_statement(
        Statement::from_sql_and_values(
            ctx.transaction().get_database_backend(),
            concat!(
                "SELECT t.forum_thread_id, t.forum_category_id, counts.post_count, ",
                "last_post.forum_post_id AS last_forum_post_id, ",
                "last_post.user_id AS last_user_id, ",
                "last_post.created_at AS last_created_at, ",
                "wu.name AS wikidot_user_name, wu.slug AS wikidot_user_slug, ",
                "local_user.name AS local_user_name, local_user.slug AS local_user_slug, ",
                "last_post.guest_name, last_post.guest_email_md5 ",
                "FROM forum_thread t ",
                "JOIN forum_group g ON g.forum_group_id = t.forum_group_id ",
                " AND g.site_id = t.site_id AND g.deleted_at IS NULL ",
                "JOIN forum_category c ON c.forum_category_id = t.forum_category_id ",
                " AND c.site_id = t.site_id AND c.deleted_at IS NULL ",
                "LEFT JOIN page p ON p.page_id = t.page_id ",
                " AND p.site_id = t.site_id AND p.deleted_at IS NULL ",
                "JOIN LATERAL (SELECT COUNT(fp0.forum_post_id) AS post_count ",
                " FROM forum_post fp0 WHERE fp0.forum_thread_id = t.forum_thread_id ",
                " AND fp0.site_id = t.site_id AND fp0.deleted_at IS NULL) counts ON TRUE ",
                "LEFT JOIN LATERAL (SELECT fp1.forum_post_id, fp1.user_id, fp1.created_at, ",
                " fp1.guest_name, fp1.guest_email_md5 ",
                " FROM forum_post fp1 WHERE fp1.forum_thread_id = t.forum_thread_id ",
                " AND fp1.site_id = t.site_id AND fp1.deleted_at IS NULL ",
                " ORDER BY fp1.created_at DESC, fp1.forum_post_id DESC LIMIT 1) last_post ON TRUE ",
                "LEFT JOIN wikidot_user wu ON wu.user_id = last_post.user_id AND wu.is_deleted = FALSE ",
                "LEFT JOIN \"user\" local_user ON local_user.user_id = last_post.user_id ",
                " AND local_user.deleted_at IS NULL ",
                "WHERE t.site_id = $1 AND t.deleted_at IS NULL ",
                " AND ($2::BOOLEAN OR g.visible = TRUE) ",
                " AND t.forum_thread_id = ANY($3::BIGINT[]) ",
                " AND (t.page_id IS NULL OR p.page_id IS NOT NULL) ",
                "ORDER BY g.sort_index, g.forum_group_id, c.sort_index, ",
                "c.forum_category_id, t.forum_thread_id LIMIT 1001",
            ),
            [
                Value::from(site_id),
                Value::from(include_hidden),
                Value::Array(ArrayType::BigInt, Some(Box::new(visible_thread_ids))),
            ],
        ),
    )
    .all(ctx.transaction())
    .await
    .or_raise(make_error)?;

    if candidates.len() == FORUM_START_CANDIDATE_LIMIT && visibility_complete {
        return Ok(None);
    }
    let mut activity = BTreeMap::<i64, ForumCategoryActivity>::new();
    for candidate in candidates {
        let category = activity.entry(candidate.forum_category_id).or_default();
        category.thread_count += 1;
        category.post_count += candidate.post_count.max(0);
        let (Some(forum_post_id), Some(user_id), Some(created_at)) = (
            candidate.last_forum_post_id,
            candidate.last_user_id,
            candidate.last_created_at,
        ) else {
            continue;
        };
        if category.last_post.as_ref().is_some_and(|last| {
            (last.created_at, last.forum_post_id) >= (created_at, forum_post_id)
        }) {
            continue;
        }
        category.last_post = Some(ForumLastPost {
            forum_post_id,
            forum_thread_id: candidate.forum_thread_id,
            user: match (candidate.guest_name, candidate.guest_email_md5) {
                (Some(name), Some(md5)) => forum_guest_user(name, md5),
                _ => forum_user(
                    user_id,
                    candidate.wikidot_user_name,
                    candidate.wikidot_user_slug,
                    candidate.local_user_name,
                    candidate.local_user_slug,
                ),
            },
            created_at,
        });
    }
    Ok(Some(activity))
}

pub(super) fn render_forum_start(
    structure: &[ForumGroupStructure],
    activity: &BTreeMap<i64, ForumCategoryActivity>,
    show_hidden: bool,
) -> String {
    let avatar_timestamp = time::OffsetDateTime::now_utc().unix_timestamp();
    let mut output = String::from("<div class=\"forum-start-box\">");
    for group in structure
        .iter()
        .filter(|group| show_hidden || group.group.visible)
    {
        output.push_str("<div class=\"forum-group\" style=\"width: 98%\"><div class=\"head\"><div class=\"title\">");
        output.push_str(&escape_list_pages_html_text(&group.group.name));
        output.push_str("</div><div class=\"description\">");
        output.push_str(&escape_list_pages_html_text(&group.group.description));
        output.push_str("</div></div><div ><table><tr class=\"head\"><td>Category name</td><td >Threads</td><td >Posts</td><td >Last post</td></tr>");
        for category in &group.categories {
            let category_activity = activity
                .get(&category.forum_category_id)
                .cloned()
                .unwrap_or_default();
            let category_slug = normalize_page_slug(category.name.clone());
            write!(
                &mut output,
                "<tr><td class=\"name\"><div class=\"title\"><a href=\"/forum/c-{}/{}\">{}</a></div><div class=\"description\">{}</div></td><td class=\"threads\">{}</td><td class=\"posts\">{}</td><td class=\"last\">",
                category.forum_category_id,
                escape_list_pages_html_attr(&category_slug),
                escape_list_pages_html_text(&category.name),
                escape_list_pages_html_text(&category.description),
                category_activity.thread_count,
                category_activity.post_count,
            )
            .expect("writing to a String cannot fail");
            if let Some(last) = category_activity.last_post {
                output.push_str("by&nbsp;");
                output.push_str(&render_forum_user(&last.user, avatar_timestamp));
                output.push_str("<br/>");
                output.push_str(&render_forum_date(
                    last.created_at,
                    "format_%28%25O%20ago%29",
                    "%d %b %Y %H:%M",
                ));
                write!(
                    &mut output,
                    "<a href=\"/forum/t-{}#post-{}\">Jump!</a>",
                    last.forum_thread_id, last.forum_post_id,
                )
                .expect("writing to a String cannot fail");
            }
            output.push_str("</td></tr>");
        }
        output.push_str("</table></div></div>");
    }
    if show_hidden {
        output.push_str("</div><p style=\"text-align: right\"><a href=\"/forum/start\">Hide hidden</a></p>");
    } else {
        output.push_str("</div><p style=\"text-align: right\"><a href=\"/forum/start/hidden/show\">Show hidden</a></p>");
    }
    output.push_str("<p style=\"text-align: right\"><span class=\"rss-icon\"><img src=\"/common--theme/base/images/feed/feed-icon-14x14.png\" alt=\"rss icon\"/></span> RSS: <a href=\"/feed/forum/threads.xml\">New threads</a> | <a href=\"/feed/forum/posts.xml\">New posts</a></p>");
    output
}

pub(super) async fn load_recent_posts_page(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    viewer_user_id: Option<i64>,
    page: u32,
    category_id: Option<i64>,
) -> Result<Option<RecentPostsPage>> {
    if !(1..=MAX_RECENT_POSTS_PAGE).contains(&page) {
        return Ok(None);
    }
    let make_error =
        || Error::new("failed to load recent forum posts", ErrorType::Render);
    let mut visibility = ForumPageVisibility::new(ctx, viewer_user_id);
    if !visibility.site_is_viewable(site_id).await? {
        return Ok(None);
    }
    let Some(visible_threads) = visibility
        .visible_thread_ids(site_id, category_id, None, true)
        .await?
    else {
        return Ok(None);
    };
    let visibility_complete = visible_threads.complete;
    let visible_thread_ids = visible_threads.ids;
    if visible_thread_ids.is_empty() {
        return if page == 1 {
            Ok(Some(RecentPostsPage {
                posts: Vec::new(),
                page,
                known_page_count: 0,
                page_count_is_lower_bound: false,
            }))
        } else {
            Ok(None)
        };
    }
    let visible_thread_ids = visible_thread_ids
        .into_iter()
        .map(Value::from)
        .collect::<Vec<_>>();
    let sql = format!(
        "SELECT fp.forum_post_id, fp.forum_thread_id, fp.forum_category_id, \
                g.name AS group_name, c.name AS category_name, t.title AS thread_title, \
                p.slug AS page_slug, \
                pr.title AS page_title, fp.user_id, fp.created_at, fpr.title, \
                fpr.compiled_html_hash, wu.name AS wikidot_user_name, \
                wu.slug AS wikidot_user_slug, local_user.name AS local_user_name, \
                local_user.slug AS local_user_slug, local_user.forum_signature, \
                fp.guest_name, fp.guest_email_md5 \
         FROM forum_post fp \
         JOIN forum_thread t ON t.forum_thread_id = fp.forum_thread_id \
                            AND t.site_id = fp.site_id AND t.deleted_at IS NULL \
         JOIN forum_category c ON c.forum_category_id = fp.forum_category_id \
                              AND c.site_id = fp.site_id AND c.deleted_at IS NULL \
         JOIN forum_group g ON g.forum_group_id = fp.forum_group_id \
                           AND g.site_id = fp.site_id AND g.deleted_at IS NULL \
                           AND g.visible = TRUE \
         JOIN forum_post_revision fpr ON fpr.forum_post_revision_id = fp.latest_revision_id \
                                     AND fpr.site_id = fp.site_id \
         LEFT JOIN page p ON p.page_id = t.page_id AND p.site_id = t.site_id \
                         AND p.deleted_at IS NULL \
         LEFT JOIN page_revision pr ON pr.revision_id = p.latest_revision_id \
         LEFT JOIN wikidot_user wu ON wu.user_id = fp.user_id AND wu.is_deleted = FALSE \
         LEFT JOIN \"user\" local_user ON local_user.user_id = fp.user_id \
                                      AND local_user.deleted_at IS NULL \
         WHERE fp.site_id = $1 AND fp.deleted_at IS NULL \
           AND ($2::BIGINT IS NULL OR fp.forum_category_id = $2) \
           AND t.forum_thread_id = ANY($3::BIGINT[]) \
           AND (t.page_id IS NULL OR p.page_id IS NOT NULL) \
         ORDER BY fp.created_at DESC, fp.forum_post_id DESC \
         LIMIT {RECENT_POSTS_CANDIDATE_LIMIT}",
    );
    let candidates =
        RecentPostCandidate::find_by_statement(Statement::from_sql_and_values(
            ctx.transaction().get_database_backend(),
            sql,
            [
                Value::from(site_id),
                Value::BigInt(category_id),
                Value::Array(ArrayType::BigInt, Some(Box::new(visible_thread_ids))),
            ],
        ))
        .all(ctx.transaction())
        .await
        .or_raise(make_error)?;
    let page_count_is_lower_bound =
        candidates.len() == RECENT_POSTS_CANDIDATE_LIMIT || !visibility_complete;
    let mut visible = candidates;

    let start = (page as usize - 1) * RECENT_POSTS_PER_PAGE;
    if start >= visible.len() && page_count_is_lower_bound {
        return Ok(None);
    }
    let end = (start + RECENT_POSTS_PER_PAGE).min(visible.len());
    if page_count_is_lower_bound && visible.len() <= end {
        return Ok(None);
    }
    let known_page_count = visible
        .len()
        .div_ceil(RECENT_POSTS_PER_PAGE)
        .max(page as usize) as u32;
    let mut posts = Vec::with_capacity(end.saturating_sub(start));
    let mut signature_cache = BTreeMap::<String, String>::new();
    let drain_start = start.min(visible.len());
    for candidate in visible.drain(drain_start..end) {
        let compiled_html = TextService::get(ctx, &candidate.compiled_html_hash)
            .await
            .or_raise(make_error)?;
        posts.push(RecentPost {
            forum_post_id: candidate.forum_post_id,
            forum_thread_id: candidate.forum_thread_id,
            forum_category_id: candidate.forum_category_id,
            group_name: candidate.group_name,
            category_name: candidate.category_name,
            thread_title: candidate.thread_title,
            page_slug: candidate.page_slug,
            page_title: candidate.page_title,
            user: match (candidate.guest_name, candidate.guest_email_md5) {
                (Some(name), Some(md5)) => forum_guest_user(name, md5),
                _ => forum_user(
                    candidate.user_id,
                    candidate.wikidot_user_name,
                    candidate.wikidot_user_slug,
                    candidate.local_user_name,
                    candidate.local_user_slug,
                ),
            },
            created_at: candidate.created_at,
            title: candidate.title,
            compiled_html,
            signature_html: match candidate.forum_signature.as_deref() {
                Some(source) if !source.is_empty() => {
                    if let Some(rendered) = signature_cache.get(source) {
                        Some(rendered.clone())
                    } else {
                        let rendered =
                            render_forum_signature_html(ctx, site_id, Some(source))
                                .await?
                                .expect("non-empty signature source renders a signature");
                        signature_cache.insert(source.to_owned(), rendered.clone());
                        Some(rendered)
                    }
                }
                _ => None,
            },
        });
    }
    Ok(Some(RecentPostsPage {
        posts,
        page,
        known_page_count,
        page_count_is_lower_bound,
    }))
}

fn recent_post_path(post: &RecentPost, anchor: bool) -> String {
    let anchor_suffix = if anchor {
        format!("#post-{}", post.forum_post_id)
    } else {
        String::new()
    };
    match post.page_slug.as_deref() {
        Some(page_slug) => format!("/{page_slug}/comments/show{anchor_suffix}"),
        None => format!(
            "/forum/t-{}/{}{}",
            post.forum_thread_id,
            normalize_slug_without_category_separator(&post.thread_title),
            anchor_suffix,
        ),
    }
}

fn push_recent_posts_pager(output: &mut String, page: &RecentPostsPage) {
    if page.known_page_count <= 1 && !page.page_count_is_lower_bound {
        return;
    }
    output.push_str("<div class=\"pager\"><span class=\"pager-no\">page ");
    output.push_str(&page.page.to_string());
    output.push_str("</span>");
    if page.page > 1 {
        let previous = page.page - 1;
        write!(output, "<span class=\"target\"><a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumRecentPostsModule.listeners.updateList({previous})\">&laquo; previous</a></span>").expect("writing to a String cannot fail");
    }

    let first = page.page.saturating_sub(2).max(1);
    let last = (page.page + 2).min(page.known_page_count);
    if first > 1 {
        output.push_str("<span class=\"dots\">...</span>");
    }
    for number in first..=last {
        if number == page.page {
            write!(output, "<span class=\"current\">{number}</span>")
                .expect("writing to a String cannot fail");
        } else {
            write!(output, "<span class=\"target\"><a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumRecentPostsModule.listeners.updateList({number})\">{number}</a></span>").expect("writing to a String cannot fail");
        }
    }
    if last < page.known_page_count || page.page_count_is_lower_bound {
        output.push_str("<span class=\"dots\">...</span>");
    }
    if page.page < page.known_page_count || page.page_count_is_lower_bound {
        let next = page.page + 1;
        write!(output, "<span class=\"target\"><a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumRecentPostsModule.listeners.updateList({next})\">next &raquo;</a></span>").expect("writing to a String cannot fail");
    }
    output.push_str("</div>");
}

fn render_recent_posts(
    structure: &[ForumGroupStructure],
    page: &RecentPostsPage,
) -> String {
    let mut output = String::from(
        "<div class=\"forum-recent-posts-box\" ><form onsubmit=\"return false;\" action=\"dummy.html\" method=\"get\"><table class=\"form\"><tr><td>From categories: </td><td><select id=\"recent-posts-category\"><option value=\"\" selected=\"selected\">All categories</option>",
    );
    for group in structure.iter().filter(|group| group.group.visible) {
        for category in &group.categories {
            write!(
                &mut output,
                "<option value=\"{}\">{}: {}</option>",
                category.forum_category_id,
                escape_list_pages_html_text(&group.group.name),
                escape_list_pages_html_text(&category.name),
            )
            .expect("writing to a String cannot fail");
        }
    }
    output.push_str("</select><input class=\"buttons btn btn-primary\" type=\"button\" value=\"Update\" onclick=\"WIKIDOT.modules.ForumRecentPostsModule.listeners.updateList()\"/></td></tr></table></form><div id=\"forum-recent-posts-list\">");
    output.push_str(&render_recent_posts_list(page));
    output.push_str("</div></div>");
    output
}

pub(super) fn render_recent_posts_list(page: &RecentPostsPage) -> String {
    let avatar_timestamp = time::OffsetDateTime::now_utc().unix_timestamp();
    let mut output = String::from("<div id=\"recent-posts-container\">");
    push_recent_posts_pager(&mut output, page);
    output.push_str("<div class=\"thread-container\">");
    for post in &page.posts {
        let post_path = recent_post_path(post, true);
        let discussion_path = recent_post_path(post, false);
        let category_slug = normalize_page_slug(post.category_name.clone());
        let discussion_title = post.page_title.as_deref().unwrap_or(&post.thread_title);
        let rendered_user = render_forum_user(&post.user, avatar_timestamp);
        let date = render_forum_date(
            post.created_at,
            "format_%25e%20%25b%20%25Y%2C%20%25H%3A%25M%7Cagohover",
            "%e %b %Y %H:%M",
        );
        write!(
            &mut output,
            "<div class=\"post\" id=\"post-{}\"><div class=\"long\"><div class=\"head\"><div class=\"title\" id=\"post-title-{}\"><a href=\"{}\">{}</a></div><div class=\"info\">{} {}<br/>in discussion <a href=\"/forum/c-{}/{}\">{} / {}</a> &raquo; <a href=\"{}\">{}</a></div></div><div class=\"content\" id=\"post-content-{}\">{}</div>{}<div class=\"options\"></div><div id=\"post-options-{}\" class=\"options\" style=\"display: none\"></div></div><div class=\"short\"><a class=\"title\" href=\"javascript:;\" >{}</a> by {}, {}</div></div>",
            post.forum_post_id,
            post.forum_post_id,
            escape_list_pages_html_attr(&post_path),
            escape_list_pages_html_text(&post.title),
            rendered_user,
            date,
            post.forum_category_id,
            escape_list_pages_html_attr(&category_slug),
            escape_list_pages_html_text(&post.group_name),
            escape_list_pages_html_text(&post.category_name),
            escape_list_pages_html_attr(&discussion_path),
            escape_list_pages_html_text(discussion_title),
            post.forum_post_id,
            post.compiled_html,
            post.signature_html.as_ref().map_or_else(String::new, |signature| {
                format!(
                    r#"<div class="signature"><hr class="signature-separator"/>{signature}</div>"#,
                )
            }),
            post.forum_post_id,
            escape_list_pages_html_text(&post.title),
            rendered_user,
            date,
        )
        .expect("writing to a String cannot fail");
    }
    output.push_str("</div>");
    push_recent_posts_pager(&mut output, page);
    output.push_str("</div>");
    output
}

fn render_comments_shell(arguments: CommentsArguments<'_>, body: Option<&str>) -> String {
    let mut output = String::from("<div class=\"comments-box\">");
    if let Some(title) = arguments.title {
        output.push_str("<h1>");
        output.push_str(&escape_list_pages_html_text(title));
        output.push_str("</h1>");
    }
    output.push_str("<div class=\"options\" id=\"comments-options-hidden\"");
    if body.is_some() {
        output.push_str(" style=\"display: none\"");
    }
    output.push_str(concat!(
        " >",
        "<a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumCommentsModule.listeners.showComments(event)\">Show Comments</a>",
        "</div><div id=\"thread-container\" class=\"thread-container",
    ));
    if arguments.order == ForumCommentsOrder::Reverse {
        output.push_str(" reverse");
    }
    output.push_str("\" style=\"margin-top: 1em\">");
    if let Some(body) = body {
        output.push_str(body);
    }
    output.push_str("</div></div>");
    output
}

const RECENT_THREADS_PLACEHOLDER: &str = "later.";

fn missing_context_html(kind: ForumModuleKind) -> Option<&'static str> {
    match kind {
        ForumModuleKind::FrontForum => Some(concat!(
            "<div class=\"error-block\">No forum category has been specified. ",
            "Please use attribute category=\"id\" where id is the index number of the category.</div>",
        )),
        ForumModuleKind::ForumCategory | ForumModuleKind::ForumNewThread => {
            Some("<div class=\"error-block\">No forum category has been specified.</div>")
        }
        ForumModuleKind::ForumThread => Some(concat!(
            "<div class=\"error-block\">No thread to show - click Back once or twice ",
            "and try again</div>",
        )),
        _ => None,
    }
}

impl RenderService {
    #[allow(clippy::too_many_arguments)]
    pub(super) async fn expand_forum_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        current_page_id: Option<i64>,
        viewer_user_id: Option<i64>,
        url: UrlArguments<'_>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax || !FORUM_MODULE_REGEX.is_match(&wikitext) {
            return Ok(wikitext);
        }
        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        let mut expanded_count = 0;
        let mut structure_cache = None;
        let mut forum_start_cache = None::<Option<String>>;
        let mut recent_posts_cache = BTreeMap::<u32, Option<String>>::new();
        for captures in FORUM_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a forum module capture has a complete match");
            if matched.start() < cursor
                || literal_regions.contains(matched.start())
                || expanded_count == MAX_FORUM_MODULES_PER_RENDER
            {
                continue;
            }
            let name = captures
                .name("name")
                .expect("a forum module capture has a name")
                .as_str();
            let kind = module_kind(name);
            let head = captures.name("head").map_or("", |head| head.as_str());
            let comments =
                (kind == ForumModuleKind::Comments).then(|| comments_arguments(head));
            let front_forum =
                if kind == ForumModuleKind::FrontForum && !head.trim().is_empty() {
                    let Some(arguments) = forum_front::parse_arguments(head) else {
                        continue;
                    };
                    Some(arguments)
                } else {
                    None
                };
            let front_forum_body = front_forum
                .as_ref()
                .and_then(|_| owned_front_forum_body(&wikitext, matched.end()));
            if next_module_boundary_is_closer(&wikitext, matched.end())
                && front_forum_body.is_none()
            {
                continue;
            }
            if kind != ForumModuleKind::Comments
                && kind != ForumModuleKind::RecentThreads
                && kind != ForumModuleKind::FrontForum
                && !head.trim().is_empty()
            {
                continue;
            }
            let mut replacement_end =
                front_forum_body.map_or(matched.end(), |body| body.replacement_end);
            let mut rendered_is_wikitext = false;
            let rendered = if let Some(arguments) = comments {
                let body = if arguments.query_safe && !arguments.hide {
                    match (current_site_id, current_page_id) {
                        (Some(site_id), Some(page_id)) => {
                            let mut visibility =
                                ForumPageVisibility::new(ctx, viewer_user_id);
                            if !visibility.site_is_viewable(site_id).await? {
                                None
                            } else {
                                match forum_comments::load(
                                    ctx,
                                    site_id,
                                    &mut visibility,
                                    page_id,
                                    arguments.order,
                                    arguments.hide_form,
                                )
                                .await?
                                {
                                    ForumCommentsLoad::Found(output) => Some(output.body),
                                    ForumCommentsLoad::NoPage
                                    | ForumCommentsLoad::Saturated => None,
                                }
                            }
                        }
                        _ => None,
                    }
                } else {
                    None
                };
                Some(render_comments_shell(arguments, body.as_deref()))
            } else if let Some(arguments) = front_forum {
                let Some(site_id) = current_site_id else {
                    continue;
                };
                match arguments {
                    FrontForumArgumentsParse::Arguments(arguments) => {
                        match forum_front::load(ctx, site_id, viewer_user_id, &arguments)
                            .await?
                        {
                            FrontForumLoad::Items(items) => {
                                let (rendered, end, is_wikitext) =
                                    render_front_forum_items(
                                        &wikitext,
                                        matched.end(),
                                        &items,
                                        compat_html,
                                    );
                                replacement_end = end;
                                rendered_is_wikitext = is_wikitext;
                                Some(rendered)
                            }
                            FrontForumLoad::MissingCategory => {
                                Some(FRONT_FORUM_MISSING_CATEGORY_HTML.to_owned())
                            }
                            FrontForumLoad::ScanLimit => None,
                        }
                    }
                    FrontForumArgumentsParse::CategoryError => {
                        Some(FRONT_FORUM_CATEGORY_ERROR_HTML.to_owned())
                    }
                }
            } else if kind == ForumModuleKind::RecentThreads {
                Some(RECENT_THREADS_PLACEHOLDER.to_owned())
            } else if let Some(html) = missing_context_html(kind) {
                Some(html.to_owned())
            } else {
                let Some(site_id) = current_site_id else {
                    continue;
                };
                let mut visibility = ForumPageVisibility::new(ctx, viewer_user_id);
                if !visibility.site_is_viewable(site_id).await? {
                    continue;
                }
                if structure_cache.is_none() {
                    structure_cache = Some(
                        ForumService::get_structure(
                            ctx,
                            GetForumStructure {
                                site_id,
                                include_deleted: false,
                                visible_groups_only: true,
                            },
                        )
                        .await
                        .or_raise(|| {
                            Error::new(
                                "failed to load forum structure",
                                ErrorType::Render,
                            )
                        })?,
                    );
                }
                let structure = structure_cache
                    .as_deref()
                    .expect("forum structure was loaded above");
                match kind {
                    ForumModuleKind::ForumStart => {
                        if forum_start_cache.is_none() {
                            forum_start_cache = Some(
                                load_forum_start_activity(
                                    ctx,
                                    site_id,
                                    viewer_user_id,
                                    false,
                                )
                                .await?
                                .map(|activity| {
                                    render_forum_start(structure, &activity, false)
                                }),
                            );
                        }
                        forum_start_cache.clone().flatten()
                    }
                    ForumModuleKind::RecentPosts => {
                        let page = url.page.unwrap_or(1);
                        if let std::collections::btree_map::Entry::Vacant(entry) =
                            recent_posts_cache.entry(page)
                        {
                            let rendered = load_recent_posts_page(
                                ctx,
                                site_id,
                                viewer_user_id,
                                page,
                                None,
                            )
                            .await?
                            .map(|page| render_recent_posts(structure, &page));
                            entry.insert(rendered);
                        }
                        recent_posts_cache.get(&page).cloned().flatten()
                    }
                    _ => unreachable!("static forum modules are handled above"),
                }
            };
            let Some(rendered) = rendered else {
                continue;
            };
            output.push_str(&wikitext[cursor..matched.start()]);
            if rendered_is_wikitext {
                output.push_str(&rendered);
            } else {
                output.push_str(&compat_html.push_block_html(rendered));
            }
            cursor = replacement_end;
            expanded_count += 1;
        }
        if cursor == 0 {
            return Ok(wikitext);
        }
        output.push_str(&wikitext[cursor..]);
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recent_post_for_path(page_slug: Option<&str>) -> RecentPost {
        RecentPost {
            forum_post_id: 456,
            forum_thread_id: 123,
            forum_category_id: 1,
            group_name: "Group".to_owned(),
            category_name: "Category".to_owned(),
            thread_title: "Thread Title".to_owned(),
            page_slug: page_slug.map(str::to_owned),
            page_title: None,
            user: ForumUserDisplay {
                user_id: 1,
                name: "User".to_owned(),
                slug: None,
                wikidot_profile: false,
                guest_gravatar_md5: None,
            },
            created_at: time::OffsetDateTime::UNIX_EPOCH,
            title: "Post".to_owned(),
            compiled_html: String::new(),
            signature_html: None,
        }
    }

    #[test]
    fn recent_post_paths_use_the_post_anchor_instead_of_boolean_text() {
        let thread_post = recent_post_for_path(None);
        assert_eq!(
            recent_post_path(&thread_post, true),
            "/forum/t-123/thread-title#post-456",
        );
        assert_eq!(
            recent_post_path(&thread_post, false),
            "/forum/t-123/thread-title"
        );

        let page_post = recent_post_for_path(Some("page-comments"));
        assert_eq!(
            recent_post_path(&page_post, true),
            "/page-comments/comments/show#post-456",
        );
    }

    #[test]
    fn forum_start_rss_chrome_uses_the_same_origin_feed_icon() {
        let rendered = render_forum_start(&[], &BTreeMap::new(), false);
        assert!(rendered.contains(
            "<span class=\"rss-icon\"><img src=\"/common--theme/base/images/feed/feed-icon-14x14.png\" alt=\"rss icon\"/></span> RSS: <a href=\"/feed/forum/threads.xml\">New threads</a>"
        ));
        assert!(!rendered.contains("www.wikidot.com"));
    }

    #[test]
    fn frontforum_custom_body_malformed_category_suppresses_body() {
        let source = concat!(
            "[[module FrontForum category=\"8503559;bad\" limit=\"1\"]]\n",
            "OWNER CONTROL %%title%%\n",
            "[[/module]]\ntrailing",
        );
        let opener = FORUM_MODULE_REGEX
            .find(source)
            .expect("FrontForum opener is recognized");
        let body = owned_front_forum_body(source, opener.end())
            .expect("recognized FrontForum owns its exact closer");

        assert_eq!(body.source.trim(), "OWNER CONTROL %%title%%");
        assert_eq!(&source[body.replacement_end..], "\ntrailing");
        assert!(matches!(
            forum_front::parse_arguments(r#" category="8503559;bad" limit="1""#,),
            Some(FrontForumArgumentsParse::CategoryError),
        ));
        let rendered = format!(
            "{}{}",
            FRONT_FORUM_CATEGORY_ERROR_HTML,
            &source[body.replacement_end..],
        );
        assert_eq!(
            rendered,
            "<div class=\"error-block\">Problem parsing attribute \"category\".</div>\ntrailing",
        );
        assert!(!rendered.contains("OWNER CONTROL"));
        assert!(!rendered.contains("%%title%%"));
        assert!(!rendered.contains("[[/module]]"));
    }

    #[test]
    fn frontforum_body_owner_does_not_claim_other_module_closers() {
        let nested_boundary = concat!(
            "[[module FrontForum category=\"8503559\"]]\n",
            "body\n",
            "[[module RecentPosts]]\n",
            "[[/module]]",
        );
        let opener = FORUM_MODULE_REGEX
            .find(nested_boundary)
            .expect("FrontForum opener is recognized");
        assert!(owned_front_forum_body(nested_boundary, opener.end()).is_none());

        let comments = "[[module Comments]]\nbody\n[[/module]]";
        let opener = FORUM_MODULE_REGEX
            .captures(comments)
            .and_then(|captures| captures.get(0))
            .expect("Comments opener is recognized");
        assert_ne!(
            module_kind(
                FORUM_MODULE_REGEX
                    .captures(comments)
                    .and_then(|captures| captures.name("name"))
                    .expect("module name is captured")
                    .as_str(),
            ),
            ForumModuleKind::FrontForum,
        );
        assert!(next_module_boundary_is_closer(comments, opener.end()));
    }

    #[test]
    fn missing_context_errors_route_only_to_their_live_kinds() {
        // Retained #1034 live own-line contracts: each missing-context error
        // routes to exactly its module kinds. Comments, ForumStart,
        // RecentPosts, and RecentThreads have distinct live outputs and must
        // never receive another module's error text.
        for name in [
            "Comments",
            "FrontForum",
            "ForumCategory",
            "ForumNewThread",
            "ForumStart",
            "ForumThread",
            "RecentPosts",
            "RecentThreads",
            "frontforum",
            "FORUMTHREAD",
        ] {
            let kind = module_kind(name);
            let expected = if name.eq_ignore_ascii_case("FrontForum") {
                Some(concat!(
                    "<div class=\"error-block\">No forum category has been specified. ",
                    "Please use attribute category=\"id\" where id is the index number of the category.</div>",
                ))
            } else if name.eq_ignore_ascii_case("ForumCategory")
                || name.eq_ignore_ascii_case("ForumNewThread")
            {
                Some(
                    "<div class=\"error-block\">No forum category has been specified.</div>",
                )
            } else if name.eq_ignore_ascii_case("ForumThread") {
                Some(concat!(
                    "<div class=\"error-block\">No thread to show - click Back once or twice ",
                    "and try again</div>",
                ))
            } else {
                None
            };
            assert_eq!(missing_context_html(kind), expected, "{name}");
        }

        assert_eq!(module_kind("Comments"), ForumModuleKind::Comments);
        assert_eq!(module_kind("ForumStart"), ForumModuleKind::ForumStart);
        assert_eq!(module_kind("RecentPosts"), ForumModuleKind::RecentPosts);
        assert_eq!(module_kind("RecentThreads"), ForumModuleKind::RecentThreads);
    }

    #[test]
    fn comments_hideform_matrix_parses_only_observed_double_quoted_values() {
        // Retained #1367/r7 contract: omitted and exact double-quoted false
        // keep the form path open, while exact double-quoted true/yes close
        // it. Every other spelling fails closed instead of inferring hidden
        // form behavior from an unobserved source shape.
        for (head, hide_form) in [
            ("", false),
            ("hideForm=\"false\"", false),
            ("hideForm=\"true\"", true),
            ("hideForm=\"yes\"", true),
        ] {
            let arguments = comments_arguments(head);
            assert!(arguments.query_safe, "{head:?} must stay query-safe");
            assert_eq!(arguments.hide_form, hide_form, "{head:?}");
        }
        for head in [
            "hideForm='true'",
            "hideForm=true",
            "hideForm",
            "hideForm=\"True\"",
            "hideForm=\"YES\"",
            "hideForm=\"\"",
            "hideForm=\"no\"",
            "hideForm=\"true\" hideForm=\"false\"",
            "unknown=\"x\"",
        ] {
            assert!(
                !comments_arguments(head).query_safe,
                "{head:?} must fail closed",
            );
        }
    }
}
