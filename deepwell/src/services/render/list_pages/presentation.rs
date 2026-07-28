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
use super::super::percent_encoding::percent_encode_path_segment;
use super::super::service::{
    RenderService, WIKIDOT_LISTPAGES_LITERAL_ELLIPSIS_SENTINEL_PREFIX,
    WIKIDOT_LISTPAGES_LITERAL_ELLIPSIS_SENTINEL_REGEX, escape_list_pages_html_attr,
    escape_list_pages_html_text, format_wikidot_list_pages_date,
};
use super::parents::ListPagesParentDisplay;
use super::substitution::{ListPagesSnapshotDisplay, WikidotUserDisplay};
#[cfg(test)]
use super::substitution::{
    ListPagesSubstitutionContext, substitute_list_pages_variables_with_fragments,
};
use super::template::LISTPAGES_VARIABLE_REGEX;
use crate::services::page_query::FoundPageRow;
use std::collections::BTreeMap;
use uuid::Uuid;

#[cfg(test)]
pub(in crate::services::render) fn substitute_list_pages_variables(
    template: &str,
    page: &FoundPageRow,
    index: usize,
    total: usize,
    context: &ListPagesSubstitutionContext<'_>,
) -> String {
    let mut compat_html = CompatHtmlFragments::new(template);
    let protected = substitute_list_pages_variables_with_fragments(
        template,
        page,
        index,
        total,
        context,
        &mut compat_html,
    );
    compat_html.restore(&protected)
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
    let path_prefix = path_prefix
        .filter(|prefix| !prefix.trim().is_empty())
        .unwrap_or("/system:page-tags/tag/");
    tags.iter()
        .map(|tag| {
            let href = list_pages_tag_link_href(path_prefix, tag);
            let label = compat_html.push_plain(tag);
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
    format!(
        concat!(
            r#"<span class="printuser avatarhover" data-wikijump-compat-listpages-user="1">"#,
            r#"<a href="http://www.wikidot.com/user:info/{slug}" onclick="WIKIDOT.page.listeners.userInfo({user_id}); return false;">"#,
            r#"<img alt="{name}" class="small" src="http://www.wikidot.com/avatar.php?userid={user_id}&amp;size=small"/>"#,
            r#"</a><a href="http://www.wikidot.com/user:info/{slug}" onclick="WIKIDOT.page.listeners.userInfo({user_id}); return false;">{name}</a>"#,
            r#"</span>"#
        ),
        slug = escape_list_pages_html_attr(slug),
        user_id = user.user_id,
        name = escape_list_pages_html_text(&user.name),
    )
}

pub(in crate::services::render) fn render_list_pages_snapshot_user(name: &str) -> String {
    escape_list_pages_html_text(name)
}

pub(in crate::services::render) fn list_pages_revision_count(
    page: &FoundPageRow,
    snapshot_displays: &BTreeMap<i64, ListPagesSnapshotDisplay>,
    revision_counts: &BTreeMap<i64, u64>,
) -> Option<u64> {
    match snapshot_displays.get(&page.page_id) {
        Some(snapshot) => u64::try_from(snapshot.source_revision_count).ok(),
        None => revision_counts.get(&page.page_id).copied(),
    }
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

pub(in crate::services::render) fn list_pages_created_by_unix(
    page: &FoundPageRow,
    user_displays: &BTreeMap<i64, WikidotUserDisplay>,
    snapshot_displays: &BTreeMap<i64, ListPagesSnapshotDisplay>,
) -> Option<String> {
    if snapshot_displays
        .get(&page.page_id)
        .and_then(|snapshot| snapshot.created_by_name.as_deref())
        .is_some_and(|created_by_name| !created_by_name.is_empty())
    {
        return None;
    }
    let user = user_displays.get(&page.created_by?)?;
    let slug = user.slug.as_deref()?;
    if slug.is_empty() {
        return None;
    }
    Some(slug.to_owned())
}

pub(in crate::services::render) fn preserve_list_pages_generated_text_typography(
    value: &str,
) -> String {
    if !value.contains("...") {
        return value.to_owned();
    }
    let marker = list_pages_literal_ellipsis_marker();
    value.replace("...", &marker)
}

pub(in crate::services::render) fn list_pages_literal_ellipsis_marker() -> String {
    format!(
        "{WIKIDOT_LISTPAGES_LITERAL_ELLIPSIS_SENTINEL_PREFIX}{}X",
        Uuid::new_v4().as_simple(),
    )
}

pub(in crate::services::render) fn restore_list_pages_literal_ellipsis_markers(
    html: &str,
) -> String {
    WIKIDOT_LISTPAGES_LITERAL_ELLIPSIS_SENTINEL_REGEX
        .replace_all(html, "...")
        .into_owned()
}

pub(in crate::services::render) fn format_list_pages_created_at(
    created_at: Option<time::OffsetDateTime>,
    format: Option<&str>,
    render_as_html: bool,
) -> String {
    let Some(created_at) = created_at else {
        return String::new();
    };
    let created_at = created_at
        .to_offset(time::UtcOffset::from_hms(9, 0, 0).expect("valid JST offset"));
    let format = format.unwrap_or("%e %b %Y, %H:%M");
    let display_format = format.split('|').next().unwrap_or(format);
    let text = format_wikidot_list_pages_date(created_at, display_format);
    let encoded_format = percent_encode_path_segment(format);
    if render_as_html {
        format!(
            r#"<span class="odate time_{} format_{}" style="cursor: help; display: inline;">{}</span>"#,
            created_at.unix_timestamp(),
            encoded_format,
            escape_list_pages_html_text(&text),
        )
    } else {
        format!(
            r#"<span class="odate time_{} format_{}" data-wikijump-compat-date="1" style="cursor: help; display: inline;">{}</span>"#,
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
