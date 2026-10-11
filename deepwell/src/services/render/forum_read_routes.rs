//! Sealed read-only forum route and Ajax surfaces observed on Wikidot.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use sea_orm::sea_query::ArrayType;
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, FromQueryResult, QueryFilter, Statement,
    Value,
};
use serde::Serialize;

use super::forum_comments::{self, ForumCommentsLoad, ForumCommentsOrder};
use super::forum_modules::{
    ForumLastPost, ForumUserResourceScheme, forum_user, load_forum_start_activity,
    load_recent_posts_page, render_forum_date, render_forum_signature_html,
    render_forum_start, render_forum_user, render_forum_user_with_scheme,
    render_forum_user_without_avatar, render_recent_posts_list,
};
use super::forum_visibility::ForumPageVisibility;
use super::service::{
    RenderService, escape_list_pages_html_attr, escape_list_pages_html_text,
};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::text;
use crate::services::forum::GetForumStructure;
use crate::services::{ForumService, ServiceContext, SiteService, TextService};
use crate::types::Reference;
use crate::utils::{normalize_page_slug, normalize_slug_without_category_separator};

const THREADS_PER_PAGE: usize = 20;
const THREAD_CANDIDATE_LIMIT: usize = 1_001;
const FORUM_FEED_ITEM_LIMIT: usize = 60;
const FORUM_FEED_BATCH_LIMIT: usize = 250;
const FORUM_FEED_SCAN_LIMIT: usize = 10_000;
const MAX_CATEGORY_PAGE: u32 = 50;
const THREAD_POSTS_SCRIPT: &str = "http://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/forum/ForumViewThreadPostsModule.js";
const THREAD_SCRIPT: &str = "http://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/forum/ForumViewThreadModule.js";

#[derive(Clone, Debug)]
pub struct WikidotForumModuleRequest {
    pub module_name: String,
    pub parameters: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct WikidotForumModuleResponse {
    pub status: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<i64>,
    pub js_include: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WikidotForumFeedKind {
    Threads,
    Posts,
}

#[derive(Clone, Debug, Serialize)]
pub struct WikidotForumFeedItem {
    pub forum_post_id: i64,
    pub forum_thread_id: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
    pub title: String,
    pub thread_title: String,
    pub thread_slug: String,
    pub page_slug: Option<String>,
    pub forum_group_id: i64,
    pub forum_group_name: String,
    pub forum_category_id: i64,
    pub forum_category_name: String,
    pub forum_category_slug: String,
    pub author_user_id: i64,
    pub author_name: String,
    pub content_html: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct WikidotForumFeedOutput {
    pub site_name: String,
    pub site_description: String,
    pub items: Vec<WikidotForumFeedItem>,
}

#[derive(Debug, FromQueryResult)]
struct ForumFeedCandidate {
    forum_post_id: i64,
    forum_thread_id: i64,
    forum_category_id: i64,
    forum_group_id: i64,
    page_id: Option<i64>,
    page_category_id: Option<i64>,
    page_slug: Option<String>,
    created_at: time::OffsetDateTime,
    title: String,
    thread_title: String,
    group_name: String,
    category_name: String,
    author_user_id: i64,
    author_wikidot_name: Option<String>,
    author_wikidot_slug: Option<String>,
    author_local_name: Option<String>,
    author_local_slug: Option<String>,
    guest_name: Option<String>,
    compiled_html_hash: Vec<u8>,
}

#[derive(Debug, FromQueryResult)]
struct ForumThreadCandidate {
    forum_thread_id: i64,
    forum_category_id: i64,
    group_name: String,
    category_name: String,
    title: String,
    description: String,
    created_at: time::OffsetDateTime,
    post_count: i64,
    last_forum_post_id: Option<i64>,
    last_created_at: Option<time::OffsetDateTime>,
    creator_user_id: i64,
    creator_wikidot_name: Option<String>,
    creator_wikidot_slug: Option<String>,
    creator_local_name: Option<String>,
    creator_local_slug: Option<String>,
    last_user_id: Option<i64>,
    last_wikidot_name: Option<String>,
    last_wikidot_slug: Option<String>,
    last_local_name: Option<String>,
    last_local_slug: Option<String>,
}

#[derive(Debug)]
struct ForumThreadView {
    forum_thread_id: i64,
    forum_category_id: i64,
    group_name: String,
    category_name: String,
    title: String,
    description: String,
    created_at: time::OffsetDateTime,
    post_count: i64,
    creator: super::forum_modules::ForumUserDisplay,
    last_post: Option<ForumLastPost>,
}

#[derive(Debug, FromQueryResult)]
pub(super) struct ForumThreadPostCandidate {
    pub(super) forum_post_id: i64,
    pub(super) parent_post_id: Option<i64>,
    pub(super) tree_depth: i64,
    pub(super) user_id: i64,
    pub(super) created_at: time::OffsetDateTime,
    pub(super) revision_number: i32,
    pub(super) revision_created_at: time::OffsetDateTime,
    pub(super) revision_user_id: i64,
    pub(super) title: String,
    pub(super) compiled_html_hash: Vec<u8>,
    pub(super) wikidot_user_name: Option<String>,
    pub(super) wikidot_user_slug: Option<String>,
    pub(super) local_user_name: Option<String>,
    pub(super) local_user_slug: Option<String>,
    pub(super) forum_signature: Option<String>,
    pub(super) guest_name: Option<String>,
    pub(super) guest_email_md5: Option<String>,
    pub(super) revision_wikidot_user_name: Option<String>,
    pub(super) revision_wikidot_user_slug: Option<String>,
    pub(super) revision_local_user_name: Option<String>,
    pub(super) revision_local_user_slug: Option<String>,
}

#[derive(Debug)]
struct ForumPostEditView {
    user: super::forum_modules::ForumUserDisplay,
    created_at: time::OffsetDateTime,
}

#[derive(Debug)]
pub(super) struct ForumThreadPostView {
    pub(super) forum_post_id: i64,
    pub(super) parent_post_id: Option<i64>,
    user: super::forum_modules::ForumUserDisplay,
    pub(super) created_at: time::OffsetDateTime,
    title: String,
    compiled_html: String,
    signature_html: Option<String>,
    edit: Option<ForumPostEditView>,
}

async fn load_forum_threads(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    viewer_user_id: Option<i64>,
    category_id: Option<i64>,
    thread_id: Option<i64>,
) -> Result<Option<Vec<ForumThreadView>>> {
    let make_error = || Error::new("failed to load forum threads", ErrorType::Render);
    let mut visibility = ForumPageVisibility::new(ctx, viewer_user_id);
    if !visibility.site_is_viewable(site_id).await? {
        return Ok(None);
    }
    let Some(visible_threads) = visibility
        .visible_thread_ids(site_id, category_id, thread_id, false)
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
    let candidates = ForumThreadCandidate::find_by_statement(
        Statement::from_sql_and_values(
            ctx.transaction().get_database_backend(),
            concat!(
                "SELECT t.forum_thread_id, t.forum_category_id, g.name AS group_name, ",
                "c.name AS category_name, ",
                "t.title, t.description, t.created_at, ",
                "counts.post_count, last_post.forum_post_id AS last_forum_post_id, ",
                "last_post.created_at AS last_created_at, t.created_by AS creator_user_id, ",
                "creator_wu.name AS creator_wikidot_name, ",
                "creator_wu.slug AS creator_wikidot_slug, ",
                "creator_local.name AS creator_local_name, ",
                "creator_local.slug AS creator_local_slug, ",
                "last_post.user_id AS last_user_id, last_wu.name AS last_wikidot_name, ",
                "last_wu.slug AS last_wikidot_slug, last_local.name AS last_local_name, ",
                "last_local.slug AS last_local_slug ",
                "FROM forum_thread t ",
                "JOIN forum_category c ON c.forum_category_id = t.forum_category_id ",
                " AND c.site_id = t.site_id AND c.deleted_at IS NULL ",
                "JOIN forum_group g ON g.forum_group_id = t.forum_group_id ",
                " AND g.site_id = t.site_id AND g.deleted_at IS NULL ",
                "LEFT JOIN page p ON p.page_id = t.page_id ",
                " AND p.site_id = t.site_id AND p.deleted_at IS NULL ",
                "JOIN LATERAL (SELECT COUNT(fp.forum_post_id) AS post_count ",
                " FROM forum_post fp WHERE fp.forum_thread_id = t.forum_thread_id ",
                " AND fp.site_id = t.site_id AND fp.deleted_at IS NULL) counts ON TRUE ",
                "LEFT JOIN LATERAL (SELECT fp.forum_post_id, fp.user_id, fp.created_at ",
                " FROM forum_post fp WHERE fp.forum_thread_id = t.forum_thread_id ",
                " AND fp.site_id = t.site_id AND fp.deleted_at IS NULL ",
                " ORDER BY fp.created_at DESC, fp.forum_post_id DESC LIMIT 1) last_post ON TRUE ",
                "LEFT JOIN wikidot_user creator_wu ON creator_wu.user_id = t.created_by ",
                " AND creator_wu.is_deleted = FALSE ",
                "LEFT JOIN \"user\" creator_local ON creator_local.user_id = t.created_by ",
                " AND creator_local.deleted_at IS NULL ",
                "LEFT JOIN wikidot_user last_wu ON last_wu.user_id = last_post.user_id ",
                " AND last_wu.is_deleted = FALSE ",
                "LEFT JOIN \"user\" last_local ON last_local.user_id = last_post.user_id ",
                " AND last_local.deleted_at IS NULL ",
                "WHERE t.site_id = $1 AND t.deleted_at IS NULL ",
                " AND ($2::BIGINT IS NULL OR t.forum_category_id = $2) ",
                " AND ($3::BIGINT IS NULL OR t.forum_thread_id = $3) ",
                " AND t.forum_thread_id = ANY($4::BIGINT[]) ",
                " AND (t.page_id IS NULL OR p.page_id IS NOT NULL) ",
                "ORDER BY t.sticky DESC, COALESCE(last_post.created_at, t.created_at) DESC, ",
                "t.created_at DESC, t.forum_thread_id DESC LIMIT 1001",
            ),
            [
                Value::from(site_id),
                Value::BigInt(category_id),
                Value::BigInt(thread_id),
                Value::Array(ArrayType::BigInt, Some(Box::new(visible_thread_ids))),
            ],
        ),
    )
    .all(ctx.transaction())
    .await
    .or_raise(make_error)?;

    if candidates.len() == THREAD_CANDIDATE_LIMIT && visibility_complete {
        return Ok(None);
    }

    let mut threads = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let last_post = match (
            candidate.last_forum_post_id,
            candidate.last_user_id,
            candidate.last_created_at,
        ) {
            (Some(forum_post_id), Some(user_id), Some(created_at)) => {
                Some(ForumLastPost {
                    forum_post_id,
                    forum_thread_id: candidate.forum_thread_id,
                    user: forum_user(
                        user_id,
                        candidate.last_wikidot_name,
                        candidate.last_wikidot_slug,
                        candidate.last_local_name,
                        candidate.last_local_slug,
                    ),
                    created_at,
                })
            }
            _ => None,
        };
        threads.push(ForumThreadView {
            forum_thread_id: candidate.forum_thread_id,
            forum_category_id: candidate.forum_category_id,
            group_name: candidate.group_name,
            category_name: candidate.category_name,
            title: candidate.title,
            description: candidate.description,
            created_at: candidate.created_at,
            post_count: candidate.post_count,
            creator: forum_user(
                candidate.creator_user_id,
                candidate.creator_wikidot_name,
                candidate.creator_wikidot_slug,
                candidate.creator_local_name,
                candidate.creator_local_slug,
            ),
            last_post,
        });
    }
    Ok(Some(threads))
}

fn render_forum_category(
    category_id: i64,
    group_name: &str,
    category_name: &str,
    category_description: &str,
    threads: &[ForumThreadView],
    page: u32,
) -> String {
    let thread_count = threads.len();
    let post_count: i64 = threads.iter().map(|thread| thread.post_count.max(0)).sum();
    let page_count = thread_count.div_ceil(THREADS_PER_PAGE);
    let page_start = (page as usize - 1).saturating_mul(THREADS_PER_PAGE);
    let page_end = (page_start + THREADS_PER_PAGE).min(thread_count);
    let page_threads = threads.get(page_start..page_end).unwrap_or_default();
    let avatar_timestamp = time::OffsetDateTime::now_utc().unix_timestamp();
    let mut output = String::from(
        "<div class=\"forum-category-box\"><div class=\"forum-breadcrumbs\"><a href=\"/forum/start\">Forum</a> &raquo; ",
    );
    output.push_str(&escape_list_pages_html_text(group_name));
    output.push_str(" / ");
    output.push_str(&escape_list_pages_html_text(category_name));
    write!(
        &mut output,
        "</div><div class=\"description-block well\"><div class=\"statistics\">Number of threads: {thread_count}<br/>Number of posts: {post_count}<br/><span class=\"rss-icon\"><img src=\"/common--theme/base/images/feed/feed-icon-14x14.png\" alt=\"rss icon\"/></span> RSS: <a href=\"/feed/forum/ct-{category_id}.xml\">New threads</a> | <a href=\"/feed/forum/cp-{category_id}.xml\">New posts</a></div>{}</div>",
        escape_list_pages_html_text(category_description),
    )
    .expect("writing to a String cannot fail");
    write!(
        &mut output,
        "<div class=\"options\">Order by: <div class=\"btn btn-primary disabled btn-small btn-sm\"><strong>Last post date</strong></div> <a href=\"/forum/c-{category_id}/sort/start\" class=\"btn btn-primary btn-small btn-sm\">Thread starting date</a></div>",
    )
    .expect("writing to a String cannot fail");
    push_forum_category_pager(&mut output, category_id, page, page_count);
    output.push_str("<table style=\"width: 98%\" class=\"table\"><tr class=\"head\"><td>Thread name</td><td>Started</td><td>Posts</td><td>Recent post</td></tr>");
    for thread in page_threads {
        let thread_slug = normalize_slug_without_category_separator(&thread.title);
        write!(
            &mut output,
            "<tr><td class=\"name\"><div class=\"title\"><a href=\"/forum/t-{}/{}\">{}</a><br/></div><div class=\"description\">{}</div></td><td class=\"started\">by: {}<br/>{}</td><td class=\"posts\">{}</td><td class=\"last\">",
            thread.forum_thread_id,
            escape_list_pages_html_attr(&thread_slug),
            escape_list_pages_html_text(&thread.title),
            escape_list_pages_html_text(&thread.description),
            render_forum_user(&thread.creator, avatar_timestamp),
            render_forum_date(
                thread.created_at,
                "format_%25e%20%25b%20%25Y%2C%20%25H%3A%25M%7Cagohover",
                "%e %b %Y %H:%M",
            ),
            thread.post_count,
        )
        .expect("writing to a String cannot fail");
        if let Some(last) = &thread.last_post {
            output.push_str("by ");
            output.push_str(&render_forum_user(&last.user, avatar_timestamp));
            output.push_str("<br/>");
            output.push_str(&render_forum_date(
                last.created_at,
                "format_%28%25O%20%25A%29",
                "%e %b %Y %H:%M",
            ));
            write!(
                &mut output,
                "<a href=\"/forum/t-{}/{}#post-{}\">Jump!</a>",
                thread.forum_thread_id,
                escape_list_pages_html_attr(&thread_slug),
                last.forum_post_id,
            )
            .expect("writing to a String cannot fail");
        } else {
            output.push_str("&nbsp;");
        }
        output.push_str("</td></tr>");
    }
    output.push_str("</table>");
    if page_threads.len() == THREADS_PER_PAGE {
        push_forum_category_pager(&mut output, category_id, page, page_count);
    }
    output.push_str("</div>");
    output
}

fn push_forum_category_pager(
    output: &mut String,
    category_id: i64,
    page: u32,
    page_count: usize,
) {
    if page_count <= 1 && page == 1 {
        return;
    }
    write!(
        output,
        "<div class=\"pager\"><span class=\"pager-no\">page {page} of {page_count}</span>",
    )
    .expect("writing to a String cannot fail");
    if page > 1 {
        write!(
            output,
            "<span class=\"target\"><a href=\"/forum/c-{category_id}/p/{}\">&laquo; previous</a></span>",
            page - 1,
        )
        .expect("writing to a String cannot fail");
    }

    let page_count = page_count as u32;
    let upper = page_count.max(page);
    let mut pages = Vec::with_capacity(9);
    pages.extend(1..=2.min(upper));
    pages.extend(page.saturating_sub(2).max(1)..=(page + 2).min(upper));
    pages.extend(page_count.saturating_sub(1).max(1)..=page_count);
    pages.sort_unstable();
    pages.dedup();
    let mut previous = 0;
    for number in pages {
        if number == 0 {
            continue;
        }
        if previous > 0 && number > previous + 1 {
            output.push_str("<span class=\"dots\">...</span>");
        }
        if number == page {
            write!(output, "<span class=\"current\">{number}</span>")
                .expect("writing to a String cannot fail");
        } else {
            write!(
                output,
                "<span class=\"target\"><a href=\"/forum/c-{category_id}/p/{number}\">{number}</a></span>",
            )
            .expect("writing to a String cannot fail");
        }
        previous = number;
    }
    if page_count > 0 && page != page_count {
        write!(
            output,
            "<span class=\"target\"><a href=\"/forum/c-{category_id}/p/{}\">next &raquo;</a></span>",
            page + 1,
        )
        .expect("writing to a String cannot fail");
    }
    output.push_str("</div>");
}

fn render_forum_thread(thread: &ForumThreadView, posts: &str) -> String {
    let avatar_timestamp = time::OffsetDateTime::now_utc().unix_timestamp();
    let category_slug = normalize_page_slug(thread.category_name.clone());
    let thread_slug = normalize_page_slug(thread.title.clone());
    format!(
        concat!(
            "<div class=\"forum-thread-box \"><div class=\"forum-breadcrumbs\">",
            "<a href=\"/forum/start\">Forum</a> &raquo; ",
            "<a href=\"/forum/c-{category_id}/{category_slug}\">{group_name} / {category_name}</a>",
            " &raquo; {title}</div><div class=\"description-block well\">",
            "<div class=\"statistics\">Started by: {creator}<br/>Date: {date}<br/>",
            "Number of posts: {post_count}<br/><span class=\"rss-icon\">",
            "<img src=\"/common--theme/base/images/feed/feed-icon-14x14.png\" alt=\"rss icon\"/>",
            "</span> RSS: <a href=\"/feed/forum/t-{thread_id}.xml\">New posts</a></div>",
            "<div class=\"head\">Summary:</div>{description}</div>",
            "<div class=\"options\"><a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumViewThreadModule.listeners.unfoldAll(event)\" class=\"btn btn-default btn-small btn-sm\">Unfold All</a> ",
            "<a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumViewThreadModule.listeners.foldAll(event)\" class=\"btn btn-default btn-small btn-sm\">Fold All</a> ",
            "<a href=\"javascript:;\" id=\"thread-toggle-options\" onclick=\"WIKIDOT.modules.ForumViewThreadModule.listeners.toggleThreadOptions(event)\" class=\"btn btn-default btn-small btn-sm\"><i class=\"icon-plus\"></i> More Options</a></div>",
            "<div id=\"thread-options-2\" class=\"options\" style=\"display: none\"><a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumViewThreadModule.listeners.editThreadMeta(event)\" class=\"btn btn-default btn-small btn-sm\">Edit Title &amp; Description</a> ",
            "<a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumViewThreadModule.listeners.editThreadStickiness(event)\" class=\"btn btn-default btn-small btn-sm\">Stickness</a> ",
            "<a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumViewThreadModule.listeners.editThreadBlock(event)\" class=\"btn btn-default btn-small btn-sm\">Lock Thread</a> ",
            "<a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumViewThreadModule.listeners.moveThread(event)\" class=\"btn btn-default btn-small btn-sm\">Move Thread</a></div>",
            "<div id=\"thread-action-area\" class=\"action-area well\" style=\"display: none\"></div>",
            "<div id=\"thread-container\" class=\"thread-container\">",
            "<div id=\"thread-container-posts\" style=\"display: none\">{posts}</div>",
            "</div><div class=\"new-post\"><a href=\"javascript:;\" id=\"new-post-button\" onclick=\"WIKIDOT.modules.ForumViewThreadModule.listeners.newPost(event,null)\" class=\"btn btn-default\">New Post</a></div>",
            "<div style=\"display:none\" id=\"post-options-template\"><a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumViewThreadModule.listeners.showPermalink(event,'%POST_ID%')\" class=\"btn btn-default btn-small btn-sm\">Permanent Link</a> ",
            "<a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumViewThreadModule.listeners.editPost(event,'%POST_ID%')\" class=\"btn btn-default btn-small btn-sm\">Edit</a> ",
            "<a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumViewThreadModule.listeners.deletePost(event,'%POST_ID%')\" class=\"btn btn-danger btn-small btn-sm\">Delete</a></div>",
            "<div style=\"display:none\" id=\"post-options-permalink-template\">/forum/t-{thread_id}/{thread_slug}#post-</div>",
            "</div><script type=\"text/javascript\">",
            "WIKIDOT.forumThreadId = {thread_id};</script>",
        ),
        category_id = thread.forum_category_id,
        category_slug = escape_list_pages_html_attr(&category_slug),
        thread_slug = escape_list_pages_html_text(&thread_slug),
        group_name = escape_list_pages_html_text(&thread.group_name),
        category_name = escape_list_pages_html_text(&thread.category_name),
        title = escape_list_pages_html_text(&thread.title),
        creator = render_forum_user(&thread.creator, avatar_timestamp),
        date = render_forum_date(
            thread.created_at,
            "format_%25e%20%25b%20%25Y%2C%20%25H%3A%25M%7Cagohover",
            "%e %b %Y %H:%M",
        ),
        post_count = thread.post_count,
        thread_id = thread.forum_thread_id,
        description = escape_list_pages_html_text(&thread.description),
        posts = posts,
    )
}

async fn load_forum_thread_posts(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    forum_thread_id: i64,
) -> Result<Vec<ForumThreadPostView>> {
    let make_error =
        || Error::new("failed to load forum thread posts", ErrorType::Render);
    let order_clause = "ORDER BY fp.created_at ASC, fp.forum_post_id ASC LIMIT 20";
    let candidates = ForumThreadPostCandidate::find_by_statement(
        Statement::from_sql_and_values(
            ctx.transaction().get_database_backend(),
            format!(
                "{} {order_clause}",
                concat!(
                "SELECT fp.forum_post_id, fp.parent_post_id, 0::BIGINT AS tree_depth, fp.user_id, fp.created_at, ",
                "revision.revision_number, revision.created_at AS revision_created_at, ",
                "revision.user_id AS revision_user_id, revision.title, ",
                "revision.compiled_html_hash, wu.name AS wikidot_user_name, ",
                "wu.slug AS wikidot_user_slug, local_user.name AS local_user_name, ",
                "local_user.slug AS local_user_slug, local_user.forum_signature, ",
                "fp.guest_name, fp.guest_email_md5, ",
                "revision_wu.name AS revision_wikidot_user_name, ",
                "revision_wu.slug AS revision_wikidot_user_slug, ",
                "revision_local.name AS revision_local_user_name, ",
                "revision_local.slug AS revision_local_user_slug FROM forum_post fp ",
                "JOIN forum_post_revision revision ",
                " ON revision.forum_post_revision_id = fp.latest_revision_id ",
                " AND revision.site_id = fp.site_id ",
                "LEFT JOIN wikidot_user wu ON wu.user_id = fp.user_id AND wu.is_deleted = FALSE ",
                "LEFT JOIN \"user\" local_user ON local_user.user_id = fp.user_id ",
                " AND local_user.deleted_at IS NULL ",
                "LEFT JOIN wikidot_user revision_wu ON revision_wu.user_id = revision.user_id ",
                " AND revision_wu.is_deleted = FALSE ",
                "LEFT JOIN \"user\" revision_local ON revision_local.user_id = revision.user_id ",
                " AND revision_local.deleted_at IS NULL ",
                "WHERE fp.site_id = $1 AND fp.forum_thread_id = $2 ",
                " AND fp.deleted_at IS NULL",
                ),
            ),
            [Value::from(site_id), Value::from(forum_thread_id)],
        ),
    )
    .all(ctx.transaction())
    .await
    .or_raise(make_error)?;

    hydrate_forum_posts(ctx, site_id, candidates).await
}

pub(super) async fn hydrate_forum_posts(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    candidates: Vec<ForumThreadPostCandidate>,
) -> Result<Vec<ForumThreadPostView>> {
    let make_error =
        || Error::new("failed to load forum thread posts", ErrorType::Render);
    let hashes = candidates
        .iter()
        .map(|candidate| candidate.compiled_html_hash.clone())
        .collect::<BTreeSet<_>>();
    let compiled_html_by_hash = if hashes.is_empty() {
        BTreeMap::new()
    } else {
        text::Entity::find()
            .filter(text::Column::Hash.is_in(hashes.iter().cloned()))
            .all(ctx.transaction())
            .await
            .or_raise(make_error)?
            .into_iter()
            .map(|text| (text.hash, text.contents))
            .collect::<BTreeMap<_, _>>()
    };
    if compiled_html_by_hash.len() != hashes.len() {
        return Err(make_error().into());
    }

    let mut posts = Vec::with_capacity(candidates.len());
    let mut signature_cache = BTreeMap::<String, String>::new();
    for candidate in candidates {
        let Some(compiled_html) = compiled_html_by_hash
            .get(&candidate.compiled_html_hash)
            .cloned()
        else {
            return Err(make_error().into());
        };
        posts.push(ForumThreadPostView {
            forum_post_id: candidate.forum_post_id,
            parent_post_id: candidate.parent_post_id,
            user: match (
                candidate.guest_name.clone(),
                candidate.guest_email_md5.clone(),
            ) {
                (Some(name), Some(md5)) => {
                    super::forum_modules::forum_guest_user(name, md5)
                }
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
            edit: (candidate.revision_number > 0).then(|| ForumPostEditView {
                user: forum_user(
                    candidate.revision_user_id,
                    candidate.revision_wikidot_user_name,
                    candidate.revision_wikidot_user_slug,
                    candidate.revision_local_user_name,
                    candidate.revision_local_user_slug,
                ),
                created_at: candidate.revision_created_at,
            }),
        });
    }
    Ok(posts)
}

fn render_forum_thread_posts(posts: &[ForumThreadPostView]) -> String {
    let avatar_timestamp = time::OffsetDateTime::now_utc().unix_timestamp();
    let mut output = String::new();
    for post in posts {
        output.push_str(&render_forum_thread_post(
            post,
            "",
            avatar_timestamp,
            false,
            ForumUserResourceScheme::Http,
        ));
    }
    output
}

pub(super) fn render_forum_thread_post(
    post: &ForumThreadPostView,
    replies: &str,
    avatar_timestamp: i64,
    include_reply: bool,
    resource_scheme: ForumUserResourceScheme,
) -> String {
    let user =
        render_forum_user_with_scheme(&post.user, avatar_timestamp, resource_scheme);
    let date = render_forum_date(
        post.created_at,
        "format_%25e%20%25b%20%25Y%2C%20%25H%3A%25M%7Cagohover",
        "%e %b %Y %H:%M",
    );
    let changes = post.edit.as_ref().map_or_else(String::new, |edit| {
        format!(
            concat!(
                "<div class=\"changes\">Last edited on {date} by {user} ",
                "<a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumViewThreadModule.listeners.showHistory(event,{post_id})\"><i class=\"icon-plus\"></i> Show more</a></div>",
                "<div class=\"revisions\" style=\"display: none\"></div>",
            ),
            date = render_forum_date(
                edit.created_at,
                "format_%25e%20%25b%20%25Y%2C%20%25H%3A%25M%7Cagohover",
                "%e %b %Y %H:%M",
            ),
            user = render_forum_user_without_avatar(&edit.user),
            post_id = post.forum_post_id,
        )
    });
    let reply = include_reply.then(|| {
        format!(
            "<strong><a href=\"javascript:;\" onclick=\"postReply(event,{})\" class=\"btn btn-primary btn-small btn-sm\">Reply</a></strong> ",
            post.forum_post_id,
        )
    });
    format!(
        "<div class=\"post-container\" id=\"fpc-{post_id}\"><div class=\"post\" id=\"post-{post_id}\"><div class=\"long\"><div class=\"head\"><div class=\"options\"><a href=\"javascript:;\" onclick=\"togglePostFold(event,{post_id})\" class=\"btn btn-default btn-small btn-sm\">Fold</a></div><div class=\"title\" id=\"post-title-{post_id}\">{title}</div><div class=\"info\">{user} {date}</div></div><div class=\"content\" id=\"post-content-{post_id}\">{compiled_html}</div>{signature}{changes}<div class=\"options\">{reply}<a href=\"javascript:;\" onclick=\"togglePostOptions(event,{post_id})\" class=\"btn btn-default btn-small btn-sm\">Options</a></div><div id=\"post-options-{post_id}\" class=\"options\" style=\"display: none\"></div></div><div class=\"short\"><a class=\"options btn btn-default btn-mini btn-xs\" href=\"javascript:;\" onclick=\"togglePostFold(event,{post_id})\">Unfold</a><a class=\"title\" href=\"javascript:;\" onclick=\"togglePostFold(event,{post_id})\">{title}</a> by {user}, {date}</div></div>{replies}</div>",
        post_id = post.forum_post_id,
        title = escape_list_pages_html_text(&post.title),
        compiled_html = post.compiled_html.as_str(),
        signature = post.signature_html.as_ref().map_or_else(String::new, |signature| {
            format!(
                r#"<div class="signature"><hr class="signature-separator"/>{signature}</div>"#,
            )
        }),
        reply = reply.as_deref().unwrap_or_default(),
    )
}

fn response(status: &str, body: String) -> WikidotForumModuleResponse {
    WikidotForumModuleResponse {
        status: status.to_owned(),
        body,
        thread_id: None,
        js_include: Vec::new(),
    }
}

fn response_with_scripts(
    status: &str,
    body: String,
    scripts: &[&str],
) -> WikidotForumModuleResponse {
    WikidotForumModuleResponse {
        status: status.to_owned(),
        body,
        thread_id: None,
        js_include: scripts.iter().map(|script| (*script).to_owned()).collect(),
    }
}

fn comments_response(
    forum_thread_id: i64,
    body: String,
    scripts: &[&str],
) -> WikidotForumModuleResponse {
    WikidotForumModuleResponse {
        status: "ok".to_owned(),
        body,
        thread_id: Some(forum_thread_id),
        js_include: scripts.iter().map(|script| (*script).to_owned()).collect(),
    }
}

fn parameters_are(parameters: &BTreeMap<String, String>, required: &[&str]) -> bool {
    parameters.len() == required.len()
        && parameters
            .keys()
            .all(|parameter| required.contains(&parameter.as_str()))
}

fn wikidot_positive_decimal<T>(value: &str) -> Option<T>
where
    T: std::str::FromStr,
{
    let mut bytes = value.bytes();
    if !matches!(bytes.next(), Some(b'1'..=b'9'))
        || !bytes.all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    value.parse().ok()
}

fn positive_decimal_parameter<T>(
    parameters: &BTreeMap<String, String>,
    name: &str,
) -> Option<T>
where
    T: std::str::FromStr,
{
    parameters
        .get(name)
        .and_then(|value| wikidot_positive_decimal(value))
}

async fn load_category_context(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    category_id: i64,
) -> Result<Option<(String, String, String)>> {
    let structure = ForumService::get_structure(
        ctx,
        GetForumStructure {
            site_id,
            include_deleted: false,
            visible_groups_only: false,
        },
    )
    .await
    .or_raise(|| Error::new("failed to load forum category", ErrorType::Render))?;
    Ok(structure.into_iter().find_map(|group| {
        group.categories.into_iter().find_map(|category| {
            (category.forum_category_id == category_id).then(|| {
                (
                    group.group.name.clone(),
                    category.name,
                    category.description,
                )
            })
        })
    }))
}

impl RenderService {
    pub async fn wikidot_forum_feed(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        kind: WikidotForumFeedKind,
    ) -> Result<Option<WikidotForumFeedOutput>> {
        let viewer_user_id = ctx.request().user_id().ok();
        let mut visibility = ForumPageVisibility::new(ctx, viewer_user_id);
        if !visibility.site_is_viewable(site_id).await? {
            return Ok(None);
        }

        let site = SiteService::get(ctx, Reference::Id(site_id)).await?;
        let mut items = Vec::with_capacity(FORUM_FEED_ITEM_LIMIT);
        let mut offset = 0usize;
        let make_error =
            || Error::new("failed to load public forum feed", ErrorType::Render);

        while offset < FORUM_FEED_SCAN_LIMIT && items.len() < FORUM_FEED_ITEM_LIMIT {
            let candidates =
                ForumFeedCandidate::find_by_statement(Statement::from_sql_and_values(
                    ctx.transaction().get_database_backend(),
                    forum_feed_candidate_sql(kind),
                    [
                        Value::from(site_id),
                        Value::from(FORUM_FEED_BATCH_LIMIT as i64),
                        Value::from(offset as i64),
                    ],
                ))
                .all(ctx.transaction())
                .await
                .or_raise(make_error)?;
            if candidates.is_empty() {
                break;
            }
            let batch_len = candidates.len();
            for candidate in candidates {
                if !visibility
                    .page_is_viewable(
                        site_id,
                        candidate.page_id,
                        candidate.page_category_id,
                    )
                    .await?
                {
                    continue;
                }
                let author_name = candidate
                    .guest_name
                    .or(candidate.author_wikidot_name)
                    .or(candidate.author_wikidot_slug)
                    .or(candidate.author_local_name)
                    .or(candidate.author_local_slug)
                    .unwrap_or_else(|| candidate.author_user_id.to_string());
                let content_html = TextService::get(ctx, &candidate.compiled_html_hash)
                    .await
                    .or_raise(make_error)?;
                items.push(WikidotForumFeedItem {
                    forum_post_id: candidate.forum_post_id,
                    forum_thread_id: candidate.forum_thread_id,
                    created_at: candidate.created_at,
                    title: candidate.title,
                    thread_title: candidate.thread_title.clone(),
                    thread_slug: normalize_slug_without_category_separator(
                        &candidate.thread_title,
                    ),
                    page_slug: candidate.page_slug,
                    forum_group_id: candidate.forum_group_id,
                    forum_group_name: candidate.group_name,
                    forum_category_id: candidate.forum_category_id,
                    forum_category_slug: normalize_page_slug(
                        candidate.category_name.clone(),
                    ),
                    forum_category_name: candidate.category_name,
                    author_user_id: candidate.author_user_id,
                    author_name,
                    content_html,
                });
                if items.len() == FORUM_FEED_ITEM_LIMIT {
                    break;
                }
            }
            offset += batch_len;
            if batch_len < FORUM_FEED_BATCH_LIMIT {
                break;
            }
        }

        // If a site has an unusual concentration of hidden rows, do not return
        // an incomplete newest-first feed or continue scanning without a bound.
        if offset >= FORUM_FEED_SCAN_LIMIT && items.len() < FORUM_FEED_ITEM_LIMIT {
            return Ok(None);
        }

        Ok(Some(WikidotForumFeedOutput {
            site_name: site.name,
            site_description: site.description,
            items,
        }))
    }

    pub async fn render_wikidot_forum_module(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        request: WikidotForumModuleRequest,
    ) -> Result<WikidotForumModuleResponse> {
        let viewer_user_id = ctx.request().user_id().ok();
        let mut visibility = ForumPageVisibility::new(ctx, viewer_user_id);
        if !visibility.site_is_viewable(site_id).await? {
            return Ok(response("not_ok", String::new()));
        }
        match request.module_name.as_str() {
            "forum/ForumStartModule"
                if request.parameters.is_empty()
                    || parameters_are(&request.parameters, &["hidden"]) =>
            {
                let show_hidden = request
                    .parameters
                    .get("hidden")
                    .is_some_and(|hidden| hidden == "true");
                if request.parameters.contains_key("hidden") && !show_hidden {
                    return Ok(response("not_ok", String::new()));
                }
                let Some(activity) =
                    load_forum_start_activity(ctx, site_id, viewer_user_id, show_hidden)
                        .await?
                else {
                    return Ok(response("not_ok", String::new()));
                };
                let structure = ForumService::get_structure(
                    ctx,
                    GetForumStructure {
                        site_id,
                        include_deleted: false,
                        visible_groups_only: !show_hidden,
                    },
                )
                .await
                .or_raise(|| {
                    Error::new("failed to load forum structure", ErrorType::Render)
                })?;
                Ok(response(
                    "ok",
                    render_forum_start(&structure, &activity, show_hidden),
                ))
            }
            "forum/ForumCommentsListModule"
                if parameters_are(&request.parameters, &["pageId"])
                    || parameters_are(&request.parameters, &["pageId", "order"]) =>
            {
                let Some(page_id) =
                    positive_decimal_parameter::<i64>(&request.parameters, "pageId")
                else {
                    return Ok(response("no_page", String::new()));
                };
                let order = match request.parameters.get("order").map(String::as_str) {
                    None | Some("forwards") => ForumCommentsOrder::Forward,
                    Some("reverse") => ForumCommentsOrder::Reverse,
                    Some(_) => return Ok(response("not_ok", String::new())),
                };
                match forum_comments::load(
                    ctx,
                    site_id,
                    &mut visibility,
                    page_id,
                    order,
                    false,
                )
                .await?
                {
                    ForumCommentsLoad::Found(output) => Ok(comments_response(
                        output.thread_id,
                        output.body,
                        &output.scripts,
                    )),
                    ForumCommentsLoad::NoPage => Ok(response("no_page", String::new())),
                    ForumCommentsLoad::Saturated => Ok(response("not_ok", String::new())),
                }
            }
            "forum/ForumViewCategoryModule"
                if parameters_are(&request.parameters, &["c", "p"]) =>
            {
                let Some(category_id) =
                    positive_decimal_parameter::<i64>(&request.parameters, "c")
                else {
                    return Ok(response("no_category", String::new()));
                };
                let Some(page) =
                    positive_decimal_parameter::<u32>(&request.parameters, "p")
                else {
                    return Ok(response("not_ok", String::new()));
                };
                if page > MAX_CATEGORY_PAGE {
                    return Ok(response("not_ok", String::new()));
                }
                let Some((group_name, category_name, category_description)) =
                    load_category_context(ctx, site_id, category_id).await?
                else {
                    return Ok(response("no_category", String::new()));
                };
                let Some(threads) = load_forum_threads(
                    ctx,
                    site_id,
                    viewer_user_id,
                    Some(category_id),
                    None,
                )
                .await?
                else {
                    return Ok(response("not_ok", String::new()));
                };
                if threads.is_empty() && page > 1 {
                    return Ok(response("not_ok", String::new()));
                }
                Ok(response(
                    "ok",
                    render_forum_category(
                        category_id,
                        &group_name,
                        &category_name,
                        &category_description,
                        &threads,
                        page,
                    ),
                ))
            }
            "forum/ForumViewThreadModule"
                if parameters_are(&request.parameters, &["t"]) =>
            {
                let Some(thread_id) =
                    positive_decimal_parameter::<i64>(&request.parameters, "t")
                else {
                    return Ok(response("no_thread", String::new()));
                };
                let Some(mut threads) = load_forum_threads(
                    ctx,
                    site_id,
                    viewer_user_id,
                    None,
                    Some(thread_id),
                )
                .await?
                else {
                    return Ok(response("not_ok", String::new()));
                };
                let Some(thread) = threads.pop() else {
                    return Ok(response("no_thread", String::new()));
                };
                let posts = load_forum_thread_posts(ctx, site_id, thread_id).await?;
                let rendered_posts = render_forum_thread_posts(&posts);
                Ok(response_with_scripts(
                    "ok",
                    render_forum_thread(&thread, &rendered_posts),
                    &[THREAD_POSTS_SCRIPT, THREAD_SCRIPT],
                ))
            }
            "forum/ForumViewThreadPostsModule"
                if parameters_are(&request.parameters, &["t", "pageNo"]) =>
            {
                let Some(thread_id) =
                    positive_decimal_parameter::<i64>(&request.parameters, "t")
                else {
                    return Ok(response("no_thread", String::new()));
                };
                if positive_decimal_parameter::<u32>(&request.parameters, "pageNo")
                    != Some(1)
                {
                    return Ok(response("not_ok", String::new()));
                }
                let Some(threads) = load_forum_threads(
                    ctx,
                    site_id,
                    viewer_user_id,
                    None,
                    Some(thread_id),
                )
                .await?
                else {
                    return Ok(response("not_ok", String::new()));
                };
                if threads.is_empty() {
                    return Ok(response("no_thread", String::new()));
                }
                let posts = load_forum_thread_posts(ctx, site_id, thread_id).await?;
                Ok(response_with_scripts(
                    "ok",
                    render_forum_thread_posts(&posts),
                    &[THREAD_POSTS_SCRIPT],
                ))
            }
            "forum/ForumRecentPostsListModule"
                if parameters_are(&request.parameters, &["page", "categoryId"]) =>
            {
                let Some(page_number) =
                    positive_decimal_parameter::<u32>(&request.parameters, "page")
                else {
                    return Ok(response("not_ok", String::new()));
                };
                let category_id = match request.parameters.get("categoryId") {
                    None => None,
                    Some(category_id) if category_id.is_empty() => None,
                    Some(category_id) => {
                        match wikidot_positive_decimal::<i64>(category_id) {
                            Some(category_id) => Some(category_id),
                            None => return Ok(response("not_ok", String::new())),
                        }
                    }
                };
                let Some(page) = load_recent_posts_page(
                    ctx,
                    site_id,
                    viewer_user_id,
                    page_number,
                    category_id,
                )
                .await?
                else {
                    return Ok(response("not_ok", String::new()));
                };
                Ok(response("ok", render_recent_posts_list(&page)))
            }
            _ => Ok(response("not_ok", String::new())),
        }
    }
}

fn forum_feed_candidate_sql(kind: WikidotForumFeedKind) -> String {
    let (site_column, created_at, order_id, title, revision_hash, body_post_join) =
        match kind {
            WikidotForumFeedKind::Posts => (
                "fp.site_id",
                "fp.created_at",
                "fp.forum_post_id",
                "fpr.title",
                "fpr.compiled_html_hash",
                concat!(
                    "JOIN forum_post fp ON fp.forum_thread_id = t.forum_thread_id ",
                    " AND fp.site_id = t.site_id AND fp.deleted_at IS NULL ",
                    "JOIN forum_post_revision fpr ON fpr.forum_post_revision_id = fp.latest_revision_id ",
                    " AND fpr.site_id = fp.site_id ",
                    "LEFT JOIN wikidot_user wu ON wu.user_id = fp.user_id AND wu.is_deleted = FALSE ",
                    "LEFT JOIN \"user\" local_user ON local_user.user_id = fp.user_id ",
                    " AND local_user.deleted_at IS NULL ",
                ),
            ),
            WikidotForumFeedKind::Threads => (
                "t.site_id",
                "t.created_at",
                "t.forum_thread_id",
                "fp.title",
                "fp.compiled_html_hash",
                concat!(
                    "JOIN LATERAL (SELECT fp.forum_post_id, fp.user_id, fp.guest_name, ",
                    "fp.created_at, fpr.title, fpr.compiled_html_hash ",
                    "FROM forum_post fp JOIN forum_post_revision fpr ",
                    "ON fpr.forum_post_revision_id = fp.latest_revision_id ",
                    "AND fpr.site_id = fp.site_id ",
                    "WHERE fp.forum_thread_id = t.forum_thread_id ",
                    "AND fp.site_id = t.site_id AND fp.deleted_at IS NULL ",
                    "ORDER BY fp.created_at ASC, fp.forum_post_id ASC LIMIT 1) fp ON TRUE ",
                    "LEFT JOIN wikidot_user wu ON wu.user_id = fp.user_id AND wu.is_deleted = FALSE ",
                    "LEFT JOIN \"user\" local_user ON local_user.user_id = fp.user_id ",
                    " AND local_user.deleted_at IS NULL ",
                ),
            ),
        };
    format!(
        "SELECT fp.forum_post_id, t.forum_thread_id, t.forum_category_id, \
                t.forum_group_id, t.page_id, p.page_category_id, p.slug AS page_slug, \
                {created_at} AS created_at, {title} AS title, t.title AS thread_title, \
                g.name AS group_name, c.name AS category_name, fp.user_id AS author_user_id, \
                wu.name AS author_wikidot_name, wu.slug AS author_wikidot_slug, \
                local_user.name AS author_local_name, local_user.slug AS author_local_slug, \
                fp.guest_name, {revision_hash} AS compiled_html_hash \
         FROM forum_thread t \
         JOIN forum_category c ON c.forum_category_id = t.forum_category_id \
                              AND c.site_id = t.site_id AND c.deleted_at IS NULL \
         JOIN forum_group g ON g.forum_group_id = t.forum_group_id \
                           AND g.site_id = t.site_id AND g.deleted_at IS NULL AND g.visible = TRUE \
         {body_post_join} \
         LEFT JOIN page p ON p.page_id = t.page_id AND p.site_id = t.site_id AND p.deleted_at IS NULL \
         WHERE {site_column} = $1 AND t.deleted_at IS NULL \
           AND (t.page_id IS NULL OR p.page_id IS NOT NULL) \
         ORDER BY {created_at} DESC, {order_id} DESC \
         LIMIT $2 OFFSET $3"
    )
}

#[cfg(test)]
mod tests {
    use super::{
        ForumThreadView, push_forum_category_pager, render_forum_category,
        render_forum_thread,
    };
    use crate::services::render::forum_modules::forum_guest_user;

    fn pager(page: u32) -> String {
        let mut output = String::new();
        push_forum_category_pager(&mut output, 1_113_520, page, 11);
        output
    }

    #[test]
    fn category_pager_matches_the_four_sealed_live_sequences() {
        assert_eq!(
            pager(1),
            concat!(
                r#"<div class="pager"><span class="pager-no">page 1 of 11</span>"#,
                r#"<span class="current">1</span><span class="target"><a href="/forum/c-1113520/p/2">2</a></span>"#,
                r#"<span class="target"><a href="/forum/c-1113520/p/3">3</a></span><span class="dots">...</span>"#,
                r#"<span class="target"><a href="/forum/c-1113520/p/10">10</a></span><span class="target"><a href="/forum/c-1113520/p/11">11</a></span>"#,
                r#"<span class="target"><a href="/forum/c-1113520/p/2">next &raquo;</a></span></div>"#,
            ),
        );
        assert_eq!(
            pager(2),
            concat!(
                r#"<div class="pager"><span class="pager-no">page 2 of 11</span><span class="target"><a href="/forum/c-1113520/p/1">&laquo; previous</a></span>"#,
                r#"<span class="target"><a href="/forum/c-1113520/p/1">1</a></span><span class="current">2</span>"#,
                r#"<span class="target"><a href="/forum/c-1113520/p/3">3</a></span><span class="target"><a href="/forum/c-1113520/p/4">4</a></span>"#,
                r#"<span class="dots">...</span><span class="target"><a href="/forum/c-1113520/p/10">10</a></span>"#,
                r#"<span class="target"><a href="/forum/c-1113520/p/11">11</a></span><span class="target"><a href="/forum/c-1113520/p/3">next &raquo;</a></span></div>"#,
            ),
        );
        assert_eq!(
            pager(11),
            concat!(
                r#"<div class="pager"><span class="pager-no">page 11 of 11</span><span class="target"><a href="/forum/c-1113520/p/10">&laquo; previous</a></span>"#,
                r#"<span class="target"><a href="/forum/c-1113520/p/1">1</a></span><span class="target"><a href="/forum/c-1113520/p/2">2</a></span>"#,
                r#"<span class="dots">...</span><span class="target"><a href="/forum/c-1113520/p/9">9</a></span>"#,
                r#"<span class="target"><a href="/forum/c-1113520/p/10">10</a></span><span class="current">11</span></div>"#,
            ),
        );
        assert_eq!(
            pager(12),
            concat!(
                r#"<div class="pager"><span class="pager-no">page 12 of 11</span><span class="target"><a href="/forum/c-1113520/p/11">&laquo; previous</a></span>"#,
                r#"<span class="target"><a href="/forum/c-1113520/p/1">1</a></span><span class="target"><a href="/forum/c-1113520/p/2">2</a></span>"#,
                r#"<span class="dots">...</span><span class="target"><a href="/forum/c-1113520/p/10">10</a></span>"#,
                r#"<span class="target"><a href="/forum/c-1113520/p/11">11</a></span><span class="current">12</span>"#,
                r#"<span class="target"><a href="/forum/c-1113520/p/13">next &raquo;</a></span></div>"#,
            ),
        );
    }

    #[test]
    fn category_pager_is_absent_for_a_single_page() {
        let mut output = String::new();
        push_forum_category_pager(&mut output, 1_113_520, 1, 1);
        assert_eq!(output, "", "a single-page category must not render a pager",);
    }

    #[test]
    fn forum_category_and_thread_rss_chrome_uses_the_same_origin_feed_icon() {
        const ICON: &str = "<img src=\"/common--theme/base/images/feed/feed-icon-14x14.png\" alt=\"rss icon\"/>";

        let category = render_forum_category(7, "Group", "Category", "", &[], 1);
        assert!(category.contains(ICON));

        let thread = ForumThreadView {
            forum_thread_id: 11,
            forum_category_id: 7,
            group_name: "Group".to_owned(),
            category_name: "Category".to_owned(),
            title: "Thread".to_owned(),
            description: String::new(),
            created_at: time::OffsetDateTime::UNIX_EPOCH,
            post_count: 0,
            creator: forum_guest_user("Guest".to_owned(), "0".repeat(32)),
            last_post: None,
        };
        let thread_html = render_forum_thread(&thread, "");
        assert!(thread_html.contains(ICON));

        // Forum user chrome still links to Wikidot profiles; only the RSS icon is pinned local here.
        for rendered in [category, thread_html] {
            assert!(!rendered.contains("www.wikidot.com/common--theme"));
        }
    }
}
