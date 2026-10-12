//! Observed default-format slice of Wikidot's FrontForum module.

use std::collections::HashSet;
use std::fmt::Write as _;
use std::sync::LazyLock;

use regex::Regex;
use sea_orm::sea_query::ArrayType;
use sea_orm::{ConnectionTrait, FromQueryResult, Statement, Value};

use super::forum_modules::{forum_user, render_forum_date, render_forum_user};
use super::forum_visibility::ForumPageVisibility;
use super::module_arguments::{WikidotModuleArgumentValueKind, wikidot_module_arguments};
use super::service::{escape_list_pages_html_attr, escape_list_pages_html_text};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::services::forum::GetForumCategory;
use crate::services::text::TextService;
use crate::services::{ForumService, ServiceContext};
use crate::utils::{normalize_page_slug, normalize_slug_without_category_separator};

const DEFAULT_LIMIT: usize = 20;
const MAX_LIMIT: usize = 100;
const CANDIDATE_LIMIT: usize = 1_001;
const MAX_CATEGORY_IDS: usize = 32;
const MAX_THREAD_IDS: usize = 5_000;
pub(super) const MAX_CUSTOM_BODY_BYTES: usize = 64 * 1024;
const MAX_CUSTOM_OUTPUT_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug)]
pub(super) struct FrontForumArguments {
    category_ids: Vec<i64>,
    limit: usize,
    offset: usize,
}

#[derive(Debug)]
pub(super) enum FrontForumArgumentsParse {
    Arguments(FrontForumArguments),
    CategoryError,
}

#[derive(Debug)]
pub(super) enum FrontForumLoad {
    Items(Vec<FrontForumItem>),
    MissingCategory,
    ScanLimit,
}

#[derive(Debug, FromQueryResult)]
struct FrontForumCandidate {
    forum_thread_id: i64,
    forum_category_id: i64,
    title: String,
    description: String,
    user_id: i64,
    created_at: time::OffsetDateTime,
    post_count: i64,
    compiled_html_hash: Vec<u8>,
    group_name: String,
    category_name: String,
    wikidot_user_name: Option<String>,
    wikidot_user_slug: Option<String>,
    local_user_name: Option<String>,
    local_user_slug: Option<String>,
}

#[derive(Debug)]
pub(super) struct FrontForumItem {
    forum_thread_id: i64,
    forum_category_id: i64,
    title: String,
    description: String,
    user: super::forum_modules::ForumUserDisplay,
    created_at: time::OffsetDateTime,
    post_count: i64,
    compiled_html: String,
    group_name: String,
    category_name: String,
}

pub(super) fn parse_arguments(head: &str) -> Option<FrontForumArgumentsParse> {
    let arguments = wikidot_module_arguments(head)?;
    let mut category_ids = None;
    let mut category_error = false;
    let mut limit = DEFAULT_LIMIT;
    let mut limit_seen = false;
    let mut offset = 0;
    let mut offset_seen = false;
    let mut feed_seen = false;
    let mut feed_title_seen = false;
    let mut fix_relative_links_seen = false;
    for argument in arguments {
        if argument.op != "="
            || argument.value_kind != WikidotModuleArgumentValueKind::DoubleQuoted
        {
            return None;
        }
        match argument.key {
            "category" if category_ids.is_none() => {
                let mut parsed = Vec::new();
                let mut seen = HashSet::new();
                for value in argument.value.split(';') {
                    let Some(category_id) = value
                        .parse::<i64>()
                        .ok()
                        .filter(|category_id| *category_id > 0)
                    else {
                        category_error = true;
                        break;
                    };
                    if seen.insert(category_id) {
                        if parsed.len() == MAX_CATEGORY_IDS {
                            category_error = true;
                            break;
                        }
                        parsed.push(category_id);
                    }
                }
                category_ids = Some(parsed);
            }
            "limit" if !limit_seen => {
                limit_seen = true;
                limit = argument
                    .value
                    .parse::<usize>()
                    .ok()
                    .filter(|limit| (1..=MAX_LIMIT).contains(limit))?;
            }
            "offset" if !offset_seen => {
                offset_seen = true;
                offset = argument.value.parse::<usize>().unwrap_or_default();
            }
            "feed" if !feed_seen => feed_seen = true,
            "feedTitle" if !feed_title_seen => feed_title_seen = true,
            // Anonymous PagePreview accepts both observed switch values. The
            // content-bearing rewrite remains unimplemented until a public
            // fixture supplies a relative post link.
            "fixRelativeLinks"
                if !fix_relative_links_seen
                    && matches!(argument.value, "true" | "false") =>
            {
                fix_relative_links_seen = true;
            }
            _ => return None,
        }
    }
    if category_error {
        return Some(FrontForumArgumentsParse::CategoryError);
    }
    Some(FrontForumArgumentsParse::Arguments(FrontForumArguments {
        category_ids: category_ids?,
        limit,
        offset,
    }))
}

pub(super) async fn load(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    viewer_user_id: Option<i64>,
    arguments: &FrontForumArguments,
) -> Result<FrontForumLoad> {
    let make_error = || Error::new("failed to load FrontForum", ErrorType::Render);
    let mut visibility = ForumPageVisibility::new(ctx, viewer_user_id);
    if !visibility.site_is_viewable(site_id).await? {
        return Ok(FrontForumLoad::ScanLimit);
    }
    let mut category_ids = Vec::with_capacity(arguments.category_ids.len());
    for &category_id in &arguments.category_ids {
        let category_exists = ForumService::get_category_optional(
            ctx,
            GetForumCategory {
                site_id,
                forum_category_id: category_id,
                include_deleted: false,
            },
        )
        .await
        .or_raise(make_error)?
        .is_some();
        if category_exists {
            category_ids.push(category_id);
        }
    }
    if category_ids.is_empty() {
        return Ok(FrontForumLoad::MissingCategory);
    }
    let mut visible_thread_ids = Vec::new();
    let mut visibility_complete = true;
    for &category_id in &category_ids {
        let Some(category_threads) = visibility
            .visible_thread_ids(site_id, Some(category_id), None, true)
            .await?
        else {
            return Ok(FrontForumLoad::ScanLimit);
        };
        visibility_complete &= category_threads.complete;
        let remaining = MAX_THREAD_IDS.saturating_sub(visible_thread_ids.len());
        if category_threads.ids.len() > remaining {
            visibility_complete = false;
        }
        visible_thread_ids.extend(category_threads.ids.into_iter().take(remaining));
        if remaining == 0 {
            break;
        }
    }
    if visible_thread_ids.is_empty() {
        return Ok(FrontForumLoad::Items(Vec::new()));
    }
    let visible_thread_ids = visible_thread_ids
        .into_iter()
        .map(Value::from)
        .collect::<Vec<_>>();
    let category_ids = category_ids
        .into_iter()
        .map(Value::from)
        .collect::<Vec<_>>();

    let candidates = FrontForumCandidate::find_by_statement(
        Statement::from_sql_and_values(
            ctx.transaction().get_database_backend(),
            format!(
                concat!(
                    "SELECT t.forum_thread_id, t.forum_category_id, t.title, t.description, ",
                    "root_post.user_id, root_post.created_at, counts.post_count, ",
                    "root_revision.compiled_html_hash, g.name AS group_name, ",
                    "c.name AS category_name, wu.name AS wikidot_user_name, ",
                    "wu.slug AS wikidot_user_slug, local_user.name AS local_user_name, ",
                    "local_user.slug AS local_user_slug ",
                    "FROM forum_thread t ",
                    "JOIN forum_category c ON c.forum_category_id = t.forum_category_id ",
                    " AND c.site_id = t.site_id AND c.deleted_at IS NULL ",
                    "JOIN forum_group g ON g.forum_group_id = t.forum_group_id ",
                    " AND g.site_id = t.site_id AND g.deleted_at IS NULL AND g.visible = TRUE ",
                    "LEFT JOIN page p ON p.page_id = t.page_id ",
                    " AND p.site_id = t.site_id AND p.deleted_at IS NULL ",
                    "JOIN LATERAL (SELECT fp.user_id, fp.created_at, fp.latest_revision_id ",
                    " FROM forum_post fp WHERE fp.forum_thread_id = t.forum_thread_id ",
                    " AND fp.site_id = t.site_id AND fp.deleted_at IS NULL ",
                    " ORDER BY fp.created_at, fp.forum_post_id LIMIT 1) root_post ON TRUE ",
                    "JOIN forum_post_revision root_revision ",
                    " ON root_revision.forum_post_revision_id = root_post.latest_revision_id ",
                    " AND root_revision.site_id = t.site_id ",
                    "JOIN LATERAL (SELECT COUNT(fp.forum_post_id) AS post_count ",
                    " FROM forum_post fp WHERE fp.forum_thread_id = t.forum_thread_id ",
                    " AND fp.site_id = t.site_id AND fp.deleted_at IS NULL) counts ON TRUE ",
                    "LEFT JOIN wikidot_display_user wu ON wu.user_id = root_post.user_id ",
                    " AND wu.is_deleted = FALSE ",
                    "LEFT JOIN \"user\" local_user ON local_user.user_id = root_post.user_id ",
                    " AND local_user.deleted_at IS NULL ",
                    "WHERE t.site_id = $1 AND t.forum_category_id = ANY($2::BIGINT[]) ",
                    " AND t.forum_thread_id = ANY($3::BIGINT[]) ",
                    " AND t.deleted_at IS NULL AND (t.page_id IS NULL OR p.page_id IS NOT NULL) ",
                    "ORDER BY t.created_at DESC, t.forum_thread_id DESC ",
                    "LIMIT {candidate_limit}",
                ),
                candidate_limit = CANDIDATE_LIMIT,
            ),
            [
                Value::from(site_id),
                Value::Array(ArrayType::BigInt, Some(Box::new(category_ids))),
                Value::Array(ArrayType::BigInt, Some(Box::new(visible_thread_ids))),
            ],
        ),
    )
    .all(ctx.transaction())
    .await
    .or_raise(make_error)?;

    if candidates.len() == CANDIDATE_LIMIT && visibility_complete {
        return Ok(FrontForumLoad::ScanLimit);
    }

    let mut items = Vec::with_capacity(arguments.limit);
    for candidate in candidates
        .into_iter()
        .skip(arguments.offset)
        .take(arguments.limit)
    {
        let compiled_html = TextService::get(ctx, &candidate.compiled_html_hash)
            .await
            .or_raise(make_error)?;
        items.push(FrontForumItem {
            forum_thread_id: candidate.forum_thread_id,
            forum_category_id: candidate.forum_category_id,
            title: candidate.title,
            description: candidate.description,
            user: forum_user(
                candidate.user_id,
                candidate.wikidot_user_name,
                candidate.wikidot_user_slug,
                candidate.local_user_name,
                candidate.local_user_slug,
            ),
            created_at: candidate.created_at,
            post_count: candidate.post_count,
            compiled_html,
            group_name: candidate.group_name,
            category_name: candidate.category_name,
        });
    }
    Ok(FrontForumLoad::Items(items))
}

pub(super) fn render(items: &[FrontForumItem]) -> String {
    let avatar_timestamp = time::OffsetDateTime::now_utc().unix_timestamp();
    let mut output = String::from("<div class=\"front-forum-box\">");
    for item in items {
        let thread_slug = normalize_slug_without_category_separator(&item.title);
        let category_slug = normalize_page_slug(item.category_name.clone());
        write!(
            &mut output,
            "<div><h1><span><a href=\"/forum/t-{}/{}\">{}</a></span></h1><p>by {} {}</p><div>{}</div><p><a href=\"/forum/t-{}/{}\">Comments: {}</a> | category: <a href=\"/forum/c-{}/{}\">{} / {}</a></p></div>",
            item.forum_thread_id,
            escape_list_pages_html_attr(&thread_slug),
            escape_list_pages_html_text(&item.title),
            render_forum_user(&item.user, avatar_timestamp),
            render_forum_date(
                item.created_at,
                "format_%25O%20ago%20%28%25e%20%25b%20%25Y%2C%20%25H%3A%25M%29",
                "%e %b %Y %H:%M",
            ),
            item.compiled_html,
            item.forum_thread_id,
            escape_list_pages_html_attr(&thread_slug),
            item.post_count.saturating_sub(1),
            item.forum_category_id,
            escape_list_pages_html_attr(&category_slug),
            escape_list_pages_html_text(&item.group_name),
            escape_list_pages_html_text(&item.category_name),
        )
        .expect("writing to a String cannot fail");
    }
    output.push_str("</div>");
    output
}

static FRONT_FORUM_VARIABLE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Custom date formats pass through opaquely: any nonempty format without
    // a double-percent sequence is accepted and echoed percent-encoded into
    // the odate class, matching live behavior for observed formats such as
    // %e %b %Y, %H:%M, and unknown %Q%Q alike. An empty format or a %%
    // sequence keeps the variable literal (fail closed).
    Regex::new(concat!(
        r"%%(?P<name>title|linked_title|title_linked|link|author|date|comments|category|",
        r"description|short|summary|content|text|long|body)",
        r"(?:\|(?P<format>(?:[^%]|%[^%])+))?%%",
    ))
    .expect("FrontForum variable expression is valid")
});

/// Percent-encode a custom date format for the odate class exactly as live
/// Wikidot does: ASCII alphanumerics and `-_.~` pass through, every other
/// byte becomes uppercase `%XX`. The encoded value cannot break out of the
/// class attribute.
fn encode_forum_date_format_class(format: &str) -> String {
    let mut class = String::with_capacity("format_".len() + format.len());
    class.push_str("format_");
    for byte in format.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            class.push(byte as char);
        } else {
            class.push_str(&format!("%{byte:02X}"));
        }
    }
    class
}

fn thread_path(item: &FrontForumItem) -> String {
    format!(
        "/forum/t-{}/{}",
        item.forum_thread_id,
        normalize_slug_without_category_separator(&item.title),
    )
}

fn render_custom_variable(
    captures: &regex::Captures<'_>,
    item: &FrontForumItem,
    avatar_timestamp: i64,
    compat_html: &mut super::compat::CompatHtmlFragments,
) -> String {
    let name = captures
        .name("name")
        .expect("FrontForum variable has a name")
        .as_str();
    let format = captures.name("format").map(|matched| matched.as_str());
    if format.is_some() && name != "date" {
        return captures
            .get(0)
            .expect("FrontForum variable match is complete")
            .as_str()
            .to_owned();
    }
    let path = thread_path(item);
    match name {
        "title" => compat_html.push_html(escape_list_pages_html_text(&item.title)),
        "linked_title" | "title_linked" => compat_html.push_html(format!(
            r#"<a href="{}">{}</a>"#,
            escape_list_pages_html_attr(&path),
            escape_list_pages_html_text(&item.title),
        )),
        "link" => compat_html.push_plain(&path),
        "author" => {
            compat_html.push_html(render_forum_user(&item.user, avatar_timestamp))
        }
        "date" => {
            let format_class = match format {
                Some(custom) => encode_forum_date_format_class(custom),
                None => "format_%25O%20ago%20%28%25e%20%25b%20%25Y%2C%20%25H%3A%25M%29"
                    .to_owned(),
            };
            compat_html.push_html(render_forum_date(
                item.created_at,
                &format_class,
                "%e %b %Y %H:%M",
            ))
        }
        "comments" => compat_html.push_html(format!(
            r#"<a href="{}">Comments: {}</a>"#,
            escape_list_pages_html_attr(&path),
            item.post_count.saturating_sub(1),
        )),
        "category" => {
            let category_slug = normalize_page_slug(item.category_name.clone());
            compat_html.push_html(format!(
                r#"<a href="/forum/c-{}/{}">{} / {}</a>"#,
                item.forum_category_id,
                escape_list_pages_html_attr(&category_slug),
                escape_list_pages_html_text(&item.group_name),
                escape_list_pages_html_text(&item.category_name),
            ))
        }
        "description" | "short" | "summary" => compat_html.push_block_html(format!(
            "<div>{}</div>",
            escape_list_pages_html_text(&item.description)
        )),
        "content" | "text" | "long" | "body" => {
            compat_html.push_block_html(format!("<div>{}</div>", item.compiled_html))
        }
        _ => unreachable!("FrontForum variable regex limits variable names"),
    }
}

pub(super) fn render_custom_body(
    items: &[FrontForumItem],
    body: &str,
    compat_html: &mut super::compat::CompatHtmlFragments,
) -> Option<String> {
    if body.len() > MAX_CUSTOM_BODY_BYTES {
        return None;
    }
    let avatar_timestamp = time::OffsetDateTime::now_utc().unix_timestamp();
    let mut output = String::from("[[div class=\"front-forum-box\"]]\n");
    for item in items {
        output.push_str("[[div]]\n");
        output.push_str(&FRONT_FORUM_VARIABLE_REGEX.replace_all(
            body,
            |captures: &regex::Captures<'_>| {
                render_custom_variable(captures, item, avatar_timestamp, compat_html)
            },
        ));
        output.push_str("\n[[/div]]\n");
        if output.len() > MAX_CUSTOM_OUTPUT_BYTES {
            return None;
        }
    }
    output.push_str("[[/div]]");
    Some(output)
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use ftml::data::{PageInfo, ScoreValue};
    use ftml::layout::Layout;
    use ftml::render::{Render, html::HtmlRender};
    use ftml::settings::{WikitextMode, WikitextSettings};

    use super::*;
    use crate::services::render::compat::CompatHtmlFragments;
    use crate::services::render::forum_modules::{
        FORUM_MODULE_REGEX, render_front_forum_items,
    };

    #[test]
    fn frontforum_category_fanout_is_bounded() {
        let categories = (1..=MAX_CATEGORY_IDS + 1)
            .map(|category_id| category_id.to_string())
            .collect::<Vec<_>>()
            .join(";");
        assert!(matches!(
            parse_arguments(&format!(r#" category="{categories}""#)),
            Some(FrontForumArgumentsParse::CategoryError)
        ));
    }

    #[test]
    fn frontforum_custom_body_is_bounded() {
        let mut fragments = CompatHtmlFragments::new("");
        let body = "x".repeat(MAX_CUSTOM_BODY_BYTES + 1);
        assert!(render_custom_body(&[], &body, &mut fragments).is_none());
    }

    fn item(title: &str, description: &str, compiled_html: &str) -> FrontForumItem {
        FrontForumItem {
            forum_thread_id: 18029831,
            forum_category_id: 8503559,
            title: title.to_owned(),
            user: forum_user(
                8955132,
                Some("Sandbox Author".to_owned()),
                Some("sandbox-author".to_owned()),
                None,
                None,
            ),
            created_at: time::OffsetDateTime::from_unix_timestamp(1_781_693_034)
                .expect("test timestamp is valid"),
            post_count: 2,
            compiled_html: compiled_html.to_owned(),
            group_name: "Community & Friends".to_owned(),
            category_name: "Open <Topic>".to_owned(),
            description: description.to_owned(),
        }
    }

    fn render_owned_source(source: &str, items: &[FrontForumItem]) -> String {
        let captures = FORUM_MODULE_REGEX
            .captures(source)
            .expect("test source contains FrontForum");
        let matched = captures.get(0).expect("module match is complete");
        let mut fragments = CompatHtmlFragments::new(source);
        let (mut expanded, replacement_end, is_wikitext) =
            render_front_forum_items(source, matched.end(), items, &mut fragments);
        assert!(is_wikitext, "test source has an owned FrontForum body");
        expanded.push_str(&source[replacement_end..]);

        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let page_info = PageInfo {
            page: Cow::Borrowed("frontforum-test"),
            category: None,
            site: Cow::Borrowed("sandbox"),
            title: Cow::Borrowed("FrontForum test"),
            alt_title: None,
            score: ScoreValue::Integer(0),
            tags: Vec::new(),
            language: Cow::Borrowed("en"),
        };
        ftml::preprocess_for_layout(&mut expanded, settings.layout);
        let tokens = ftml::tokenize(&expanded);
        let (tree, errors) = ftml::parse(&tokens, &page_info, &settings).into();
        assert!(errors.is_empty(), "{errors:#?}");
        fragments.restore(&HtmlRender.render(&tree, &page_info, &settings).body)
    }

    #[test]
    fn frontforum_custom_body_canonical_public_render() {
        let source = concat!(
            "[[module FrontForum category=\"8503559\" limit=\"1\"]]\n",
            "[[div class=\"title\"]]\n%%title%%\n[[/div]]\n",
            "[[div class=\"linked\"]]\n%%linked_title%%\n[[/div]]\n",
            "[[div class=\"author\"]]\n%%author%%\n[[/div]]\n",
            "[[div class=\"date\"]]\n%%date|%Y-%m-%d%%\n[[/div]]\n",
            "[[div class=\"comments\"]]\n%%comments%%\n[[/div]]\n",
            "[[div class=\"category\"]]\n%%category%%\n[[/div]]\n",
            "[[div class=\"description\"]]\n%%description%%\n[[/div]]\n",
            "[[div class=\"content\"]]\n%%content%%\n[[/div]]\n",
            "[[/module]]",
        );
        let rendered = render_owned_source(
            source,
            &[item(
                "Title <unsafe>",
                "Description <unsafe>",
                "<p>Trusted post</p>",
            )],
        );

        assert!(rendered.contains(r#"<div class="front-forum-box">"#));
        assert!(
            rendered.contains("Title &lt;unsafe&gt;"),
            "rendered FrontForum body:\n{rendered}",
        );
        assert!(rendered.contains(
            r#"<a href="/forum/t-18029831/title-unsafe">Title &lt;unsafe&gt;</a>"#,
        ));
        assert!(rendered.contains("format_%25Y-%25m-%25d"));
        assert!(rendered.contains("Comments: 1"));
        assert!(rendered.contains("Community &amp; Friends / Open &lt;Topic&gt;"));
        assert!(rendered.contains("<div>Description &lt;unsafe&gt;</div>"));
        assert!(rendered.contains("<div><p>Trusted post</p></div>"));
        assert!(!rendered.contains("[[/module]]"));
    }

    #[test]
    fn frontforum_accepts_observed_relative_link_switch_values() {
        for value in ["true", "false"] {
            assert!(matches!(
                parse_arguments(&format!(
                    r#" category="8503559" limit="1" fixRelativeLinks="{value}""#,
                )),
                Some(FrontForumArgumentsParse::Arguments(_)),
            ));
        }
        for value in ["", "yes", "TRUE"] {
            assert!(
                parse_arguments(&format!(
                    r#" category="8503559" limit="1" fixRelativeLinks="{value}""#,
                ))
                .is_none()
            );
        }
    }

    #[test]
    fn frontforum_custom_body_alias_offset_multi_public_render() {
        let source = concat!(
            "[[module FrontForum category=\"8503561;8503559\" limit=\"1\" offset=\"1\"]]\n",
            "[[div class=\"linked\"]]\n%%title_linked%%\n[[/div]]\n",
            "[[div class=\"link\"]]\n%%link%%\n[[/div]]\n",
            "[[div class=\"short\"]]\n%%short%%\n[[/div]]\n",
            "[[div class=\"summary\"]]\n%%summary%%\n[[/div]]\n",
            "[[div class=\"text\"]]\n%%text%%\n[[/div]]\n",
            "[[div class=\"long\"]]\n%%long%%\n[[/div]]\n",
            "[[div class=\"body\"]]\n%%body%%\n[[/div]]\n",
            "[[/module]]",
        );
        let rendered = render_owned_source(
            source,
            &[item("Alias title", "Alias summary", "<p>Alias body</p>")],
        );

        assert_eq!(
            rendered.matches("Alias title").count(),
            1,
            "rendered FrontForum body:\n{rendered}",
        );
        assert!(rendered.contains("/forum/t-18029831/alias-title"));
        assert_eq!(rendered.matches("Alias summary").count(), 2);
        assert_eq!(rendered.matches("Alias body").count(), 3);
    }

    #[test]
    fn frontforum_custom_body_unknown_stays_literal() {
        let source = concat!(
            "[[module FrontForum category=\"8503559\" limit=\"1\"]]\n",
            "%%unknown%%\n[[/module]]",
        );
        let rendered =
            render_owned_source(source, &[item("Title", "Summary", "<p>Body</p>")]);

        assert!(rendered.contains("%%unknown%%"));
    }

    #[test]
    fn frontforum_custom_body_date_formats_match_live_odate_classes() {
        // Live sandbox probes honor custom strftime formats opaquely: the
        // odate class echoes the requested format percent-encoded while the
        // server text stays the fallback rendering. Unknown formats pass
        // through the same way (no validation); an empty format and a format
        // on a non-date variable stay literal.
        let rendered = render_owned_source(
            concat!(
                "[[module FrontForum category=\"8503559\" limit=\"1\"]]\n",
                "[[div class=\"day\"]]\n%%date|%e %b %Y%%\n[[/div]]\n",
                "[[div class=\"clock\"]]\n%%date|%H:%M%%\n[[/div]]\n",
                "[[div class=\"mystery\"]]\n%%date|%Q%Q%%\n[[/div]]\n",
                "[[/module]]",
            ),
            &[item("Title", "Summary", "<p>Body</p>")],
        );
        for expected in [
            "format_%25e%20%25b%20%25Y",
            "format_%25H%3A%25M",
            "format_%25Q%25Q",
        ] {
            assert!(
                rendered.contains(expected),
                "custom date format should echo {expected:?}:\n{rendered}",
            );
        }
        // Fail-closed boundaries: an empty format and a format on a
        // non-date variable stay literal. A double-percent sequence ends
        // the honored prefix compositionally (the evidenced %e renders,
        // the unestablished remainder stays literal) rather than
        // suppressing the whole variable.
        let literal = render_owned_source(
            concat!(
                "[[module FrontForum category=\"8503559\" limit=\"1\"]]\n",
                "[[div class=\"empty\"]]\n%%date|%%\n[[/div]]\n",
                "[[div class=\"doubled\"]]\n%%date|%e%%Y%%\n[[/div]]\n",
                "[[div class=\"titled\"]]\n%%title|%e %b %Y%%\n[[/div]]\n",
                "[[/module]]",
            ),
            &[item("Title", "Summary", "<p>Body</p>")],
        );
        for expected in ["%%date|%%", "%%title|%e %b %Y%%"] {
            assert!(
                literal.contains(expected),
                "unestablished date shape should stay literal ({expected:?}):\n{literal}",
            );
        }
        assert!(
            literal.contains("format_%25e") && literal.contains("Y%%"),
            "doubled percent should split honored prefix from literal remainder:\n{literal}",
        );
        assert!(!literal.contains("format_%25e%25Y"));
    }

    #[test]
    fn frontforum_default_format_remains_unchanged() {
        let rendered = render(&[item("Title", "unused", "<p>Body</p>")]);

        assert!(rendered.starts_with("<div class=\"front-forum-box\">"));
        assert!(rendered.contains("<h1><span><a"));
        assert!(rendered.contains("Comments: 1"));
        assert!(!rendered.contains("unused"));
    }
}
