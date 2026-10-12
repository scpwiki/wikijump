/*
 * services/render/list_pages/presentation.rs
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

//! Shared ListPages variable and presentation helpers.

use super::super::compat::CompatHtmlFragments;
use super::super::compat::preparation::neutralize_authored_markers;
#[cfg(test)]
use super::super::compat::text_fragments::CompatTextFragments;
use super::super::percent_encoding::percent_encode_path_segment;
use super::super::service::{
    RenderService, escape_list_pages_html_attr, escape_list_pages_html_text,
    format_wikidot_list_pages_date,
};
use super::super::wikidot_user_resources::{WIKIDOT_AVATAR_BASE, WIKIDOT_USER_INFO_URL};
use super::parents::ListPagesParentDisplay;
use super::substitution::{ListPagesSnapshotDisplay, WikidotUserDisplay};
#[cfg(test)]
use super::substitution::{
    ListPagesSubstitutionContext, substitute_list_pages_variables_inner,
};
use super::template::LISTPAGES_VARIABLE_REGEX;
use crate::services::page_query::FoundPageRow;
use std::collections::BTreeMap;

#[cfg(test)]
pub(in crate::services::render) fn substitute_list_pages_variables(
    template: &str,
    page: &FoundPageRow,
    index: usize,
    total: usize,
    context: &ListPagesSubstitutionContext<'_>,
) -> String {
    let mut compat_html = CompatHtmlFragments::new(template);
    let mut compat_text = CompatTextFragments::new(template);
    let protected = substitute_list_pages_variables_inner(
        template,
        page,
        index,
        total,
        context,
        &mut compat_html,
        &mut compat_text,
        None,
        None,
        None,
        false,
    );
    compat_text.restore(&compat_html.restore(&protected))
}

pub(in crate::services::render) fn substitute_count_pages_variables(
    template: &str,
    total: usize,
) -> String {
    let total = total.to_string();
    let substituted = LISTPAGES_VARIABLE_REGEX
        .replace_all(template, |captures: &regex::Captures<'_>| {
            match captures["name"].to_ascii_lowercase().as_str() {
                "total" | "count" => total.clone(),
                _ => captures
                    .get(0)
                    .map_or("", |matched| matched.as_str())
                    .to_owned(),
            }
        })
        .into_owned();
    let mut substituted = RenderService::resolve_wikidot_parser_functions(&substituted);
    neutralize_authored_markers(&mut substituted);
    substituted
}

pub(in crate::services::render) fn render_list_pages_tags(
    tags: &[String],
    path_prefix: Option<&str>,
    render_as_html: bool,
    compat_html: &mut CompatHtmlFragments,
) -> String {
    let whitespace_target =
        path_prefix.is_some_and(|prefix| !prefix.is_empty() && prefix.trim().is_empty());
    let path_prefix = path_prefix
        .filter(|prefix| !prefix.trim().is_empty())
        .unwrap_or("/system:page-tags/tag/");
    tags.iter()
        .map(|tag| {
            let href = list_pages_tag_link_href(path_prefix, tag);
            let label = compat_html.push_plain(tag);
            if whitespace_target {
                let linked_label =
                    format!("/tag/{} {label}", escape_list_pages_html_text(tag));
                return if render_as_html {
                    format!(r#"<a href="/">{linked_label}</a>"#)
                } else {
                    format!("[/ {linked_label}]")
                };
            }
            if render_as_html {
                format!(
                    r#"<a href="{href}">{label}</a>"#,
                    href = escape_list_pages_html_attr(&href),
                )
            } else {
                format!("[{href} {label}]")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(in crate::services::render) fn list_pages_tag_target_prefix(
    target: &str,
) -> Option<String> {
    if target.is_empty() {
        None
    } else if target.trim().is_empty() {
        Some(target.to_owned())
    } else if target.starts_with("http://") || target.starts_with("https://") {
        Some(format!("{target}/tag/"))
    } else if target.starts_with("//") {
        Some(format!("/{}/tag/", target.trim_start_matches('/')))
    } else if target.starts_with(['/', '.']) {
        Some(format!("{target}/tag/"))
    } else {
        Some(format!("/{target}/tag/"))
    }
}

pub(in crate::services::render) fn list_pages_tag_link_href(
    path_prefix: &str,
    tag: &str,
) -> String {
    let path_prefix = percent_encode_list_pages_href_prefix(path_prefix.trim());
    let tag = percent_encode_list_pages_path_segment(tag.trim());
    if path_prefix.starts_with("http://")
        || path_prefix.starts_with("https://")
        || path_prefix.starts_with('/')
    {
        format!("{path_prefix}{tag}")
    } else {
        format!("/{path_prefix}{tag}")
    }
}

pub(in crate::services::render) fn percent_encode_list_pages_href_prefix(
    value: &str,
) -> String {
    percent_encode_list_pages_href_bytes(value, |byte| {
        matches!(
            byte,
            b':' | b'/' | b'?' | b'&' | b'=' | b',' | b'@' | b'%' | b'+' | b';'
        )
    })
}

pub(in crate::services::render) fn percent_encode_list_pages_path_segment(
    value: &str,
) -> String {
    percent_encode_list_pages_href_bytes(value, |_| false)
}

pub(in crate::services::render) fn percent_encode_list_pages_href_bytes(
    value: &str,
    preserve_reserved: impl Fn(u8) -> bool,
) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            _ if preserve_reserved(byte) => encoded.push(byte as char),
            _ => {
                use std::fmt::Write as _;
                write!(&mut encoded, "%{byte:02X}")
                    .expect("writing to a String cannot fail");
            }
        }
    }
    encoded
}

pub(in crate::services::render) fn is_list_pages_visible_tag(tag: &str) -> bool {
    let tag = tag.trim();
    !tag.is_empty() && !tag.starts_with('_')
}

pub(in crate::services::render) fn is_list_pages_hidden_tag(tag: &str) -> bool {
    let tag = tag.trim();
    !tag.is_empty() && tag.starts_with('_')
}

pub(in crate::services::render) fn render_list_pages_wikidot_user(
    user_id: i64,
    user: Option<&WikidotUserDisplay>,
) -> String {
    let Some(user) = user else {
        return user_id.to_string();
    };
    if !user.wikidot_profile {
        return escape_list_pages_html_text(&user.name);
    }
    let slug = user.slug.as_deref().unwrap_or(&user.name);
    let avatar_timestamp = time::OffsetDateTime::now_utc().unix_timestamp();
    format!(
        concat!(
            r#"<span class="printuser avatarhover" data-wikijump-compat-listpages-user="1">"#,
            r#"<a href="{user_info}{slug}" onclick="WIKIDOT.page.listeners.userInfo({user_id}); return false;">"#,
            r#"<img class="small" src="{avatar_base}/avatar.php?userid={user_id}&amp;amp;size=small&amp;amp;timestamp={avatar_timestamp}" "#,
            r#"alt="{name_attr}" style="background-image:url({avatar_base}/userkarma.php?u={user_id})" />"#,
            r#"</a><a href="{user_info}{slug}" onclick="WIKIDOT.page.listeners.userInfo({user_id}); return false;">{name_text}</a>"#,
            r#"</span>"#
        ),
        user_info = WIKIDOT_USER_INFO_URL,
        avatar_base = WIKIDOT_AVATAR_BASE,
        slug = escape_list_pages_html_attr(slug),
        user_id = user.user_id,
        avatar_timestamp = avatar_timestamp,
        name_attr = escape_list_pages_html_attr(&user.name),
        name_text = escape_list_pages_html_text(&user.name),
    )
}

pub(in crate::services::render) fn render_list_pages_wikidot_feed_user(
    user_id: i64,
    user: Option<&WikidotUserDisplay>,
    avatar_timestamp: i64,
) -> String {
    let Some(user) = user else {
        return user_id.to_string();
    };
    if !user.wikidot_profile {
        return escape_list_pages_html_text(&user.name);
    }
    let slug = user.slug.as_deref().unwrap_or(&user.name);
    format!(
        concat!(
            r#"<span class="printuser avatarhover">"#,
            r#"<a href="{user_info}{slug}" >"#,
            r#"<img class="small" src="{avatar_base}/avatar.php?userid={user_id}&amp;amp;size=small&amp;amp;timestamp={avatar_timestamp}" "#,
            r#"alt="{name_attr}" style="background-image:url({avatar_base}/userkarma.php?u={user_id})" />"#,
            r#"</a><a href="{user_info}{slug}" >{name_text}</a>"#,
            r#"</span>"#
        ),
        user_info = WIKIDOT_USER_INFO_URL,
        avatar_base = WIKIDOT_AVATAR_BASE,
        slug = escape_list_pages_html_attr(slug),
        user_id = user.user_id,
        avatar_timestamp = avatar_timestamp,
        name_attr = escape_list_pages_html_attr(&user.name),
        name_text = escape_list_pages_html_text(&user.name),
    )
}

pub(in crate::services::render) fn render_list_pages_snapshot_user(name: &str) -> String {
    escape_list_pages_html_text(name)
}

pub(in crate::services::render) fn render_list_pages_snapshot_wikidot_user(
    name: &str,
    user_id: Option<i64>,
    slug: Option<&str>,
) -> String {
    let Some(user_id) = user_id else {
        return render_list_pages_snapshot_user(name);
    };
    let display = WikidotUserDisplay {
        user_id,
        name: name.to_owned(),
        slug: slug.map(str::to_owned),
        wikidot_profile: true,
    };
    render_list_pages_wikidot_user(user_id, Some(&display))
}

pub(in crate::services::render) fn list_pages_parent_fullname<'a>(
    page: &FoundPageRow,
    snapshot_displays: &'a BTreeMap<i64, ListPagesSnapshotDisplay>,
    relational_parent_displays: &'a BTreeMap<i64, ListPagesParentDisplay>,
) -> Option<&'a str> {
    let parent_fullname = match snapshot_displays.get(&page.page_id) {
        Some(snapshot) => snapshot.parent_fullname.as_deref()?,
        None => relational_parent_displays
            .get(&page.page_id)?
            .fullname
            .as_str(),
    };
    (!parent_fullname.is_empty()).then_some(parent_fullname)
}

pub(in crate::services::render) fn list_pages_created_by_slug(
    page: &FoundPageRow,
    user_displays: &BTreeMap<i64, WikidotUserDisplay>,
    snapshot_displays: &BTreeMap<i64, ListPagesSnapshotDisplay>,
) -> Option<String> {
    if let Some(snapshot) = snapshot_displays.get(&page.page_id)
        && snapshot
            .created_by_name
            .as_deref()
            .is_some_and(|created_by_name| !created_by_name.is_empty())
    {
        return snapshot
            .created_by_slug
            .as_deref()
            .filter(|slug| !slug.is_empty())
            .map(str::to_owned);
    }
    let user = user_displays.get(&page.created_by?)?;
    let slug = user.slug.as_deref()?;
    if slug.is_empty() {
        return None;
    }
    Some(slug.to_owned())
}

pub(in crate::services::render) fn format_list_pages_created_at(
    created_at: Option<time::OffsetDateTime>,
    format: Option<&str>,
    render_as_html: bool,
    page_preview: bool,
) -> String {
    let Some(created_at) = created_at else {
        return String::new();
    };
    // PagePreview/ListPages responses carry the Wikidot server text in UTC,
    // without the comma used by the saved-page ODate phase. Saved generated
    // rows retain that later phase's JST text while the requested format
    // remains in the class for browser-side ODate handling.
    let saved_page_html = !page_preview;
    let created_at = if saved_page_html {
        created_at
            .to_offset(time::UtcOffset::from_hms(9, 0, 0).expect("valid JST offset"))
    } else {
        created_at.to_offset(time::UtcOffset::UTC)
    };
    const SERVER_FORMAT: &str = "%d %b %Y %H:%M";
    const SAVED_PAGE_FORMAT: &str = "%e %b %Y, %H:%M";
    const DEFAULT_ODATE_FORMAT: &str = "%e %b %Y, %H:%M|agohover";
    let format = format.unwrap_or(DEFAULT_ODATE_FORMAT);
    // Wikidot's Ajax response always carries the default server-rendered text.
    // A requested display format belongs to the later ODate client phase and
    // is transported only through the `format_*` class.
    let text = format_wikidot_list_pages_date(
        created_at,
        if saved_page_html {
            SAVED_PAGE_FORMAT
        } else {
            SERVER_FORMAT
        },
    );
    let mut normalized_format = String::with_capacity(format.len());
    let mut previous_was_space = false;
    for character in format.chars() {
        if character == ' ' && previous_was_space {
            continue;
        }
        normalized_format.push(character);
        previous_was_space = character == ' ';
    }
    let encoded_format = percent_encode_path_segment(&normalized_format);
    if render_as_html {
        format!(
            r#"<span class="odate time_{} format_{}">{}</span>"#,
            created_at.unix_timestamp(),
            encoded_format,
            escape_list_pages_html_text(&text),
        )
    } else {
        format!(
            r#"<span class="odate time_{} format_{}" data-wikijump-compat-date="1">{}</span>"#,
            created_at.unix_timestamp(),
            encoded_format,
            escape_list_pages_html_text(&text),
        )
    }
}

pub(in crate::services::render) fn protect_list_pages_generated_html(
    html: String,
    rendered_inside_generated_html: bool,
    compat_html: &mut CompatHtmlFragments,
) -> String {
    if html.is_empty() || rendered_inside_generated_html {
        html
    } else {
        compat_html.push_html(html)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FoundPageRow, ListPagesSnapshotDisplay, WikidotUserDisplay,
        is_list_pages_hidden_tag, is_list_pages_visible_tag, list_pages_created_by_slug,
        list_pages_tag_link_href, list_pages_tag_target_prefix,
        percent_encode_list_pages_href_prefix, percent_encode_list_pages_path_segment,
        protect_list_pages_generated_html, render_list_pages_tags,
        render_list_pages_wikidot_feed_user, render_list_pages_wikidot_user,
    };
    use crate::services::render::compat::CompatHtmlFragments;
    use std::collections::BTreeMap;

    #[test]
    fn tag_target_builds_live_paths_without_admitting_protocol_relative_urls() {
        for (target, expected) in [
            ("landing", Some("/landing/tag/")),
            ("/landing", Some("/landing/tag/")),
            (".", Some("./tag/")),
            (
                "https://example.com/tags/",
                Some("https://example.com/tags//tag/"),
            ),
            ("//example.com", Some("/example.com/tag/")),
            ("", None),
            (" ", Some(" ")),
        ] {
            assert_eq!(list_pages_tag_target_prefix(target).as_deref(), expected);
        }
    }

    #[test]
    fn whitespace_tag_target_escapes_raw_tag_names_in_generated_html() {
        let mut compat_html = CompatHtmlFragments::new("");
        let output = render_list_pages_tags(
            &["<svg/onload=alert(1)>".to_owned()],
            Some(" "),
            true,
            &mut compat_html,
        );

        assert!(!output.contains("<svg"), "raw tag markup leaked: {output}");
        assert!(
            output.contains("&lt;svg/onload=alert(1)&gt;"),
            "escaped tag missing: {output}"
        );
    }

    #[test]
    fn imported_user_names_are_safe_in_avatar_alt_attributes() {
        let user = WikidotUserDisplay {
            user_id: 42,
            name: r##"quoted" onload="alert(1)""##.to_owned(),
            slug: Some("quoted-user".to_owned()),
            wikidot_profile: true,
        };

        let html = render_list_pages_wikidot_user(42, Some(&user));
        let feed_html =
            render_list_pages_wikidot_feed_user(42, Some(&user), 1_700_000_000);

        assert!(
            html.contains(r##"alt="quoted&quot; onload=&quot;alert(1)&quot;""##),
            "unexpected user markup: {html}"
        );
        assert!(!html.contains(r#"alt="quoted" onload="alert(1)""#));
        assert!(html.contains("https://www.wikidot.com/avatar.php?userid=42"));
        assert!(html.contains(
            "background-image:url(https://www.wikidot.com/userkarma.php?u=42)"
        ));
        assert!(html.contains("https://www.wikidot.com/user:info/quoted-user"));
        assert!(feed_html.contains("https://www.wikidot.com/user:info/quoted-user"));
        assert!(
            feed_html.contains(r##"alt="quoted&quot; onload=&quot;alert(1)&quot;""##),
            "unexpected feed user markup: {feed_html}"
        );
        assert!(!feed_html.contains(r#"alt="quoted" onload="alert(1)""#));
    }

    #[test]
    fn percent_encoding_keeps_only_unreserved_and_selected_reserved_bytes() {
        // Path segments keep only unreserved bytes.
        assert_eq!(
            percent_encode_list_pages_path_segment("a-b.c_d~e"),
            "a-b.c_d~e"
        );
        assert_eq!(percent_encode_list_pages_path_segment("a b/c"), "a%20b%2Fc");
        assert_eq!(percent_encode_list_pages_path_segment("é"), "%C3%A9");

        // Href prefixes keep the reserved bytes Wikidot preserves.
        assert_eq!(
            percent_encode_list_pages_href_prefix("/landing/tag/?a=b&c,d@e%f+g;h"),
            "/landing/tag/?a=b&c,d@e%f+g;h"
        );
        assert_eq!(percent_encode_list_pages_href_prefix("a b"), "a%20b");
    }

    #[test]
    fn tag_link_href_keeps_absolute_prefixes_and_encodes_the_tag() {
        assert_eq!(
            list_pages_tag_link_href("/landing/tag/", "my tag"),
            "/landing/tag/my%20tag"
        );
        assert_eq!(
            list_pages_tag_link_href("https://example.com/tag/", "foo"),
            "https://example.com/tag/foo"
        );
        assert_eq!(
            list_pages_tag_link_href("http://example.com/tag/", "foo"),
            "http://example.com/tag/foo"
        );
        assert_eq!(
            list_pages_tag_link_href("landing/tag/", "foo"),
            "/landing/tag/foo"
        );
    }

    #[test]
    fn visible_and_hidden_tag_predicates_partition_trimmed_tags() {
        for visible in ["alpha", "Alpha", "a-b"] {
            assert!(is_list_pages_visible_tag(visible), "{visible}");
            assert!(!is_list_pages_hidden_tag(visible), "{visible}");
        }
        for hidden in ["_hidden", " _x "] {
            assert!(!is_list_pages_visible_tag(hidden), "{hidden}");
            assert!(is_list_pages_hidden_tag(hidden), "{hidden}");
        }
        for blank in ["", "   "] {
            assert!(!is_list_pages_visible_tag(blank), "{blank:?}");
            assert!(!is_list_pages_hidden_tag(blank), "{blank:?}");
        }
    }

    #[test]
    fn generated_html_is_protected_only_outside_generated_html() {
        let mut fragments = CompatHtmlFragments::new("");
        let protected = protect_list_pages_generated_html(
            "<b>row</b>".to_owned(),
            false,
            &mut fragments,
        );
        assert_ne!(protected, "<b>row</b>");
        assert_eq!(fragments.restore(&protected), "<b>row</b>");

        let mut fragments = CompatHtmlFragments::new("");
        assert_eq!(
            protect_list_pages_generated_html(
                "<b>row</b>".to_owned(),
                true,
                &mut fragments,
            ),
            "<b>row</b>"
        );
        assert_eq!(
            protect_list_pages_generated_html(String::new(), false, &mut fragments),
            ""
        );
    }

    fn row(page_id: i64, created_by: Option<i64>) -> FoundPageRow {
        FoundPageRow {
            page_id,
            site_id: 1,
            title: None,
            alt_title: None,
            slug: None,
            page_category_id: None,
            page_revision_id: None,
            tags: None,
            created_at: None,
            created_by,
            updated_at: None,
            updated_by: None,
            score: None,
            revision_count: None,
        }
    }

    fn snapshot(
        created_by_name: Option<&str>,
        created_by_slug: Option<&str>,
    ) -> ListPagesSnapshotDisplay {
        ListPagesSnapshotDisplay {
            title_shown: None,
            created_at: time::OffsetDateTime::UNIX_EPOCH,
            updated_at: time::OffsetDateTime::UNIX_EPOCH,
            created_by_user_id: None,
            created_by_name: created_by_name.map(str::to_owned),
            created_by_slug: created_by_slug.map(str::to_owned),
            updated_by_user_id: None,
            updated_by_name: None,
            updated_by_slug: None,
            comments: 0,
            commented_at: None,
            commented_by_name: None,
            rating_votes: None,
            parent_fullname: None,
            source_revision_count: 0,
        }
    }

    #[test]
    fn created_by_slug_prefers_a_snapshot_then_falls_back_to_the_user_display() {
        let page = row(1, Some(42));
        let mut users = BTreeMap::new();
        users.insert(
            42,
            WikidotUserDisplay {
                user_id: 42,
                name: "Alice".to_owned(),
                slug: Some("alice-user".to_owned()),
                wikidot_profile: false,
            },
        );

        // A snapshot with a non-empty name wins and uses its own slug.
        let mut snapshots = BTreeMap::new();
        snapshots.insert(1, snapshot(Some("Alice"), Some("alice")));
        assert_eq!(
            list_pages_created_by_slug(&page, &users, &snapshots),
            Some("alice".to_owned()),
        );

        // A snapshot without a name falls back to the user display slug.
        let mut snapshots = BTreeMap::new();
        snapshots.insert(1, snapshot(None, Some("alice")));
        assert_eq!(
            list_pages_created_by_slug(&page, &users, &snapshots),
            Some("alice-user".to_owned()),
        );

        // No snapshot and no user display resolves to no slug.
        assert_eq!(
            list_pages_created_by_slug(&page, &BTreeMap::new(), &BTreeMap::new()),
            None,
        );
    }

    #[test]
    fn tag_rendering_uses_the_whitespace_target_only_for_a_whitespace_prefix() {
        // A normal prefix renders an ordinary tag link.
        let mut fragments = CompatHtmlFragments::new("");
        let rendered = render_list_pages_tags(
            &["alpha".to_owned()],
            Some("/system:page-tags/tag/"),
            true,
            &mut fragments,
        );
        let normal = fragments.restore(&rendered);
        assert!(
            normal.contains(r#"<a href="/system:page-tags/tag/alpha">alpha</a>"#),
            "{normal}"
        );
        assert!(!normal.contains("/tag/alpha "), "{normal}");

        // An empty prefix falls back to the default ordinary tag link.
        let mut fragments = CompatHtmlFragments::new("");
        let rendered =
            render_list_pages_tags(&["alpha".to_owned()], Some(""), true, &mut fragments);
        let empty = fragments.restore(&rendered);
        assert!(
            empty.contains(r#"<a href="/system:page-tags/tag/alpha">alpha</a>"#),
            "{empty}"
        );
        assert!(!empty.contains("/tag/alpha "), "{empty}");

        // Only a whitespace prefix uses the whitespace target form.
        let mut fragments = CompatHtmlFragments::new("");
        let rendered = render_list_pages_tags(
            &["alpha".to_owned()],
            Some(" "),
            true,
            &mut fragments,
        );
        let whitespace = fragments.restore(&rendered);
        assert!(whitespace.contains("/tag/alpha "), "{whitespace}");
    }
}
