//! Read-only page comment resolution and first-page rendering.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, FromQueryResult, PaginatorTrait,
    QueryFilter, Statement, Value,
};

use super::forum_modules::ForumUserResourceScheme;
use super::forum_read_routes::{
    ForumThreadPostCandidate, ForumThreadPostView, hydrate_forum_posts,
    render_forum_thread_post,
};
use super::forum_visibility::ForumPageVisibility;
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::{forum_category, forum_group, forum_post, forum_thread, page};
use crate::services::ServiceContext;

const ROOTS_PER_PAGE: usize = 10;
const POST_CANDIDATE_LIMIT: usize = 1_001;
const MAX_COMMENT_DEPTH: usize = 128;
const THREAD_SCRIPT: &str = "https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/forum/ForumViewThreadModule.js";
const POSTS_SCRIPT: &str = "https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/forum/ForumViewThreadPostsModule.js";
const NEW_POST_SCRIPT: &str = "https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/forum/sub/ForumNewPostFormModule.js";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ForumCommentsOrder {
    Forward,
    Reverse,
}

#[derive(Debug)]
pub(super) struct ForumCommentsOutput {
    pub thread_id: i64,
    pub body: String,
    pub scripts: [&'static str; 3],
}

#[derive(Debug)]
pub(super) enum ForumCommentsLoad {
    Found(ForumCommentsOutput),
    NoPage,
    Saturated,
}

#[derive(Debug)]
struct ForumCommentNode {
    post: ForumThreadPostView,
    replies: Vec<ForumCommentNode>,
}

pub(super) async fn load(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    visibility: &mut ForumPageVisibility<'_, '_>,
    page_id: i64,
    order: ForumCommentsOrder,
    hide_form: bool,
) -> Result<ForumCommentsLoad> {
    let Some((thread_id, forum_category_id)) =
        resolve_page_discussion_thread(ctx, site_id, visibility, page_id).await?
    else {
        return Ok(ForumCommentsLoad::NoPage);
    };
    let can_post = visibility
        .forum_category_is_postable(site_id, forum_category_id)
        .await?;
    let root_count = count_comment_roots(ctx, site_id, thread_id).await?;
    let Some(comments) = load_comment_nodes(ctx, site_id, thread_id, order).await? else {
        return Ok(ForumCommentsLoad::Saturated);
    };
    let scripts = match order {
        ForumCommentsOrder::Forward => [THREAD_SCRIPT, POSTS_SCRIPT, NEW_POST_SCRIPT],
        ForumCommentsOrder::Reverse => [THREAD_SCRIPT, NEW_POST_SCRIPT, POSTS_SCRIPT],
    };
    Ok(ForumCommentsLoad::Found(ForumCommentsOutput {
        thread_id,
        body: render_comments(
            thread_id, &comments, root_count, order, can_post, hide_form,
        ),
        scripts,
    }))
}

async fn resolve_page_discussion_thread(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    visibility: &mut ForumPageVisibility<'_, '_>,
    page_id: i64,
) -> Result<Option<(i64, i64)>> {
    let make_error = || {
        Error::new(
            "failed to resolve page discussion thread",
            ErrorType::Render,
        )
    };
    let Some(page) = page::Entity::find_by_id(page_id)
        .filter(page::Column::SiteId.eq(site_id))
        .filter(page::Column::DeletedAt.is_null())
        .one(ctx.transaction())
        .await
        .or_raise(make_error)?
    else {
        return Ok(None);
    };
    if !visibility
        .page_is_viewable(site_id, Some(page.page_id), Some(page.page_category_id))
        .await?
    {
        return Ok(None);
    }

    let mut query = forum_thread::Entity::find()
        .filter(forum_thread::Column::SiteId.eq(site_id))
        .filter(forum_thread::Column::PageId.eq(page.page_id))
        .filter(forum_thread::Column::DeletedAt.is_null());
    if let Some(thread_id) = page.discussion_thread_id {
        query = query.filter(forum_thread::Column::ForumThreadId.eq(thread_id));
    }
    let Some(thread) = query.one(ctx.transaction()).await.or_raise(make_error)? else {
        return Ok(None);
    };
    let category_exists = forum_category::Entity::find_by_id(thread.forum_category_id)
        .filter(forum_category::Column::SiteId.eq(site_id))
        .filter(forum_category::Column::ForumGroupId.eq(thread.forum_group_id))
        .filter(forum_category::Column::DeletedAt.is_null())
        .one(ctx.transaction())
        .await
        .or_raise(make_error)?
        .is_some();
    if !category_exists {
        return Ok(None);
    }
    let group_exists = forum_group::Entity::find_by_id(thread.forum_group_id)
        .filter(forum_group::Column::SiteId.eq(site_id))
        .filter(forum_group::Column::DeletedAt.is_null())
        .one(ctx.transaction())
        .await
        .or_raise(make_error)?
        .is_some();
    if !group_exists {
        return Ok(None);
    }
    Ok(Some((thread.forum_thread_id, thread.forum_category_id)))
}

async fn count_comment_roots(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    thread_id: i64,
) -> Result<u64> {
    forum_post::Entity::find()
        .filter(forum_post::Column::SiteId.eq(site_id))
        .filter(forum_post::Column::ForumThreadId.eq(thread_id))
        .filter(forum_post::Column::ParentPostId.is_null())
        .filter(forum_post::Column::DeletedAt.is_null())
        .count(ctx.transaction())
        .await
        .or_raise(|| Error::new("failed to count page comments", ErrorType::Render))
}

async fn load_comment_nodes(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    thread_id: i64,
    order: ForumCommentsOrder,
) -> Result<Option<Vec<ForumCommentNode>>> {
    let make_error = || Error::new("failed to load page comments", ErrorType::Render);
    let root_order = match order {
        ForumCommentsOrder::Forward => "ASC",
        ForumCommentsOrder::Reverse => "DESC",
    };
    let candidates = ForumThreadPostCandidate::find_by_statement(
        Statement::from_sql_and_values(
            ctx.transaction().get_database_backend(),
            format!(
                concat!(
                    "WITH RECURSIVE selected_posts AS ((",
                    "SELECT root.*, 0::BIGINT AS tree_depth FROM forum_post root ",
                    "WHERE root.site_id = $1 AND root.forum_thread_id = $2 ",
                    " AND root.parent_post_id IS NULL AND root.deleted_at IS NULL ",
                    "ORDER BY root.created_at {root_order}, root.forum_post_id {root_order} ",
                    "LIMIT {root_limit}) UNION ALL SELECT child.*, parent.tree_depth + 1 ",
                    "FROM forum_post child JOIN selected_posts parent ",
                    " ON child.parent_post_id = parent.forum_post_id ",
                    "WHERE child.site_id = $1 AND child.forum_thread_id = $2 ",
                    " AND child.deleted_at IS NULL AND parent.tree_depth < {overflow_depth}) ",
                    "SELECT fp.forum_post_id, fp.parent_post_id, fp.tree_depth, fp.user_id, fp.created_at, ",
                    "revision.revision_number, source_post.source_has_edit, revision.created_at AS revision_created_at, ",
                    "revision.user_id AS revision_user_id, revision.title, ",
                    "revision.compiled_html_hash, wu.name AS wikidot_user_name, ",
                    "wu.slug AS wikidot_user_slug, local_user.name AS local_user_name, ",
                    "local_user.slug AS local_user_slug, local_user.forum_signature, ",
                    "fp.guest_name, fp.guest_email_md5, ",
                    "revision_wu.name AS revision_wikidot_user_name, ",
                    "revision_wu.slug AS revision_wikidot_user_slug, ",
                    "revision_local.name AS revision_local_user_name, ",
                    "revision_local.slug AS revision_local_user_slug FROM selected_posts fp ",
                    "JOIN forum_post_revision revision ",
                    " ON revision.forum_post_revision_id = fp.latest_revision_id ",
                    " AND revision.site_id = fp.site_id ",
                    "LEFT JOIN wikidot_discussion_post_snapshot source_post ON source_post.forum_post_id = fp.forum_post_id ",
                    " AND source_post.revision_id = fp.latest_revision_id ",
                    "LEFT JOIN wikidot_display_user wu ON wu.user_id = fp.user_id AND wu.is_deleted = FALSE ",
                    "LEFT JOIN \"user\" local_user ON local_user.user_id = fp.user_id ",
                    " AND local_user.deleted_at IS NULL ",
                    "LEFT JOIN wikidot_display_user revision_wu ON revision_wu.user_id = revision.user_id ",
                    " AND revision_wu.is_deleted = FALSE ",
                    "LEFT JOIN \"user\" revision_local ON revision_local.user_id = revision.user_id ",
                    " AND revision_local.deleted_at IS NULL LIMIT {candidate_limit}",
                ),
                root_order = root_order,
                root_limit = ROOTS_PER_PAGE,
                overflow_depth = MAX_COMMENT_DEPTH + 1,
                candidate_limit = POST_CANDIDATE_LIMIT,
            ),
            [Value::from(site_id), Value::from(thread_id)],
        ),
    )
    .all(ctx.transaction())
    .await
    .or_raise(make_error)?;
    if candidates.len() == POST_CANDIDATE_LIMIT
        || candidates
            .iter()
            .any(|candidate| candidate.tree_depth > MAX_COMMENT_DEPTH as i64)
    {
        return Ok(None);
    }

    let posts = hydrate_forum_posts(ctx, site_id, candidates).await?;
    let mut posts_by_parent = BTreeMap::<Option<i64>, Vec<ForumThreadPostView>>::new();
    for post in posts {
        posts_by_parent
            .entry(post.parent_post_id)
            .or_default()
            .push(post);
    }
    for (parent_id, posts) in &mut posts_by_parent {
        posts.sort_unstable_by(|left, right| {
            left.created_at
                .cmp(&right.created_at)
                .then(left.forum_post_id.cmp(&right.forum_post_id))
        });
        if parent_id.is_none() && order == ForumCommentsOrder::Reverse {
            posts.reverse();
        }
    }
    Ok(Some(build_comment_nodes(None, &mut posts_by_parent)))
}

fn build_comment_nodes(
    parent_post_id: Option<i64>,
    posts_by_parent: &mut BTreeMap<Option<i64>, Vec<ForumThreadPostView>>,
) -> Vec<ForumCommentNode> {
    posts_by_parent
        .remove(&parent_post_id)
        .unwrap_or_default()
        .into_iter()
        .map(|post| ForumCommentNode {
            replies: build_comment_nodes(Some(post.forum_post_id), posts_by_parent),
            post,
        })
        .collect()
}

fn render_comments(
    thread_id: i64,
    comments: &[ForumCommentNode],
    root_count: u64,
    order: ForumCommentsOrder,
    can_post: bool,
    hide_form: bool,
) -> String {
    let form_open = can_post && !hide_form;
    let new_post = format!(
        concat!(
            "<a href=\"javascript:;\" id=\"new-post-button\" ",
            "onclick=\"WIKIDOT.modules.ForumViewThreadModule.listeners.newPost(event,null)\" ",
            "style=\"display:  {} ; margin-bottom:1em\">Add a New Comment</a>"
        ),
        if form_open { "none" } else { "block" },
    );
    let options = concat!(
        "<div class=\"options\" id=\"comments-options-shown\">",
        "<a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumCommentsModule.listeners.hideComments(event)\" class=\"btn btn-default btn-small btn-sm\">Hide All Comments</a> ",
        "<a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumViewThreadModule.listeners.unfoldAll(event)\" class=\"btn btn-default btn-small btn-sm\">Unfold All</a> ",
        "<a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumViewThreadModule.listeners.foldAll(event)\" class=\"btn btn-default btn-small btn-sm\">Fold All</a></div>",
    );
    let mut pager = String::new();
    push_pager(&mut pager, root_count.div_ceil(ROOTS_PER_PAGE as u64));
    let mut output = format!(
        "<script type=\"text/javascript\">WIKIDOT.forumThreadId = {thread_id};</script>",
    );
    if order == ForumCommentsOrder::Reverse {
        output.push_str(&new_post);
    }
    output.push_str(options);
    output.push_str("<div id=\"thread-container-posts\" style=\"display: none\">");
    output.push_str(&pager);
    output.push_str(&render_comment_nodes(comments));
    output.push_str(&pager);
    output.push_str("</div>");
    if order == ForumCommentsOrder::Forward {
        output.push_str(&new_post);
    }
    if form_open {
        output.push_str(&render_new_post_form(thread_id));
    }
    output.push_str(concat!(
        "<div style=\"display:none\" id=\"post-options-template\">",
        "<a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumViewThreadModule.listeners.showPermalink(event,'%POST_ID%')\" class=\"btn btn-default btn-small btn-sm\">Permanent Link</a> ",
        "<a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumViewThreadModule.listeners.editPost(event,'%POST_ID%')\" class=\"btn btn-default btn-small btn-sm\">Edit</a> ",
        "<a href=\"javascript:;\" onclick=\"WIKIDOT.modules.ForumViewThreadModule.listeners.deletePost(event,'%POST_ID%')\" class=\"btn btn-danger btn-small btn-sm\">Delete</a></div>",
    ));
    output
}

fn render_new_post_form(thread_id: i64) -> String {
    format!(
        concat!(
            "<div id=\"new-post-form-container\"><div id=\"new-post-div\" class=\"well\">",
            "<form id=\"new-post-form\" onkeypress=\"return OZONE.utils.disableEnterKey(event)\">",
            "<input type=\"hidden\" name=\"threadId\" value=\"{}\"/>",
            "<input type=\"hidden\" name=\"parentId\" value=\"\"/>",
            "<span id=\"np-editor-title\">Post title:</span><br />",
            "<input class=\"text form-control\" id=\"np-title\" type=\"text\" name=\"title\" value=\"\" maxlength=\"120\"/>",
            "<div id=\"np-editor-panel\" class=\"wd-editor-toolbar-panel\"></div>",
            "<div><textarea id=\"np-text\" name=\"source\" rows=\"10\" cols=\"50\" class=\"form-control\"></textarea></div>",
            "<div class=\"buttons alignleft\"><input class=\"btn btn-danger btn-small btn-sm\" type=\"button\" value=\"Cancel\" id=\"np-cancel\" onclick=\"WIKIDOT.modules.ForumNewPostFormModule.listeners.cancel(event)\">",
            "<input class=\"btn btn-default btn-small btn-sm\" type=\"button\" value=\"Preview\" id=\"np-preview\" onclick=\"WIKIDOT.modules.ForumNewPostFormModule.listeners.preview(event)\">",
            "<input class=\"btn btn-primary btn-small btn-sm\" type=\"button\" value=\"Post it\" id=\"np-post\" onclick=\"WIKIDOT.modules.ForumNewPostFormModule.listeners.save(event)\"></div>",
            "</form></div></div>",
        ),
        thread_id,
    )
}

fn render_comment_nodes(nodes: &[ForumCommentNode]) -> String {
    fn render(nodes: &[ForumCommentNode], avatar_timestamp: i64) -> String {
        let mut output = String::new();
        for node in nodes {
            let replies = render(&node.replies, avatar_timestamp);
            output.push_str(&render_forum_thread_post(
                &node.post,
                &replies,
                avatar_timestamp,
                true,
                ForumUserResourceScheme::Https,
            ));
        }
        output
    }

    render(nodes, time::OffsetDateTime::now_utc().unix_timestamp())
}

fn push_pager(output: &mut String, page_count: u64) {
    if page_count <= 1 {
        return;
    }
    write!(
        output,
        concat!(
            "<div class=\"pager\"><span class=\"pager-no\">page 1 of {page_count}</span>",
            "<span class=\"current\">1</span>",
        ),
        page_count = page_count,
    )
    .expect("writing to a String cannot fail");
    for page in 2..=page_count.min(3) {
        write!(
            output,
            concat!(
                "<span class=\"target\"><a href=\"javascript:;\" ",
                "onclick=\"WIKIDOT.modules.ForumViewThreadPostsModule.listeners.updateList({page})\">{page}</a></span>",
            ),
            page = page,
        )
        .expect("writing to a String cannot fail");
    }
    if page_count > 5 {
        output.push_str("<span class=\"dots\">...</span>");
    }
    let tail_start = if page_count > 5 { page_count - 1 } else { 4 };
    for page in tail_start..=page_count {
        write!(
            output,
            concat!(
                "<span class=\"target\"><a href=\"javascript:;\" ",
                "onclick=\"WIKIDOT.modules.ForumViewThreadPostsModule.listeners.updateList({page})\">{page}</a></span>",
            ),
            page = page,
        )
        .expect("writing to a String cannot fail");
    }
    output.push_str(concat!(
        "<span class=\"target\"><a href=\"javascript:;\" ",
        "onclick=\"WIKIDOT.modules.ForumViewThreadPostsModule.listeners.updateList(2)\">next &raquo;</a></span></div>",
    ));
}
