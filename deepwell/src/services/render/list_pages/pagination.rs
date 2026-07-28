/*
 * services/render/list_pages/pagination.rs
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

//! Pager targets and RSS-link presentation for ListPages.

use super::super::percent_encoding::percent_encode_path_segment;
use super::super::service::{
    MAX_LISTPAGES_RENDER_LIMIT, RenderService, escape_list_pages_html_attr,
};
use super::super::url_arguments::list_pages_page_argument_key;
use super::ListPagesArguments;
use crate::services::render::UrlArguments;
use ftml::data::PageInfo;
use std::collections::BTreeSet;

pub(in crate::services::render) fn push_list_pages_pager(
    output: &mut String,
    page_info: &PageInfo<'_>,
    url: UrlArguments<'_>,
    url_attr_prefix: Option<&str>,
    offset: u32,
    per_page: u64,
    total_selected: usize,
) {
    let per_page = per_page
        .min(MAX_LISTPAGES_RENDER_LIMIT)
        .min(usize::MAX as u64) as usize;
    if per_page == 0 || total_selected <= per_page {
        return;
    }

    let page_count = total_selected.div_ceil(per_page);
    let current_page = (offset as usize / per_page).saturating_add(1);
    if current_page > page_count {
        return;
    }

    output.push_str("[[div class=\"pager\"]]\n");
    output.push_str(&format!(
        r#"[[span class="pager-no"]]page {current_page} of {page_count}[[/span]]"#
    ));

    let mut pages = BTreeSet::from([1, current_page, page_count]);
    if current_page > 1 {
        pages.insert(current_page - 1);
    }
    if current_page < page_count {
        pages.insert(current_page + 1);
    }
    if current_page <= 2 && page_count >= 3 {
        pages.insert(3);
    }
    if current_page + 1 >= page_count && page_count > 2 {
        pages.insert(page_count - 2);
    }
    if page_count > 1 {
        pages.insert(page_count - 1);
    }

    let mut previous = 0;
    for page in pages {
        if previous != 0 && page > previous + 1 {
            output.push_str(r#"[[span class="dots"]]...[[/span]]"#);
        }
        if page == current_page {
            output.push_str(&format!(r#"[[span class="current"]]{page}[[/span]]"#));
        } else {
            push_list_pages_pager_target(
                output,
                page_info,
                url,
                url_attr_prefix,
                page,
                &page.to_string(),
            );
        }
        previous = page;
    }

    if current_page < page_count {
        push_list_pages_pager_target(
            output,
            page_info,
            url,
            url_attr_prefix,
            current_page + 1,
            "next »",
        );
    }

    output.push_str("\n[[/div]]\n");
}

pub(in crate::services::render) fn push_list_pages_pager_target(
    output: &mut String,
    page_info: &PageInfo<'_>,
    url: UrlArguments<'_>,
    url_attr_prefix: Option<&str>,
    target_page: usize,
    label: &str,
) {
    output.push_str(r#"[[span class="target"]][[[/"#);
    output.push_str(&list_pages_pager_href_path(
        page_info,
        url,
        url_attr_prefix,
        target_page,
    ));
    output.push('|');
    output.push_str(label);
    output.push_str("]]][[/span]]");
}

pub(in crate::services::render) fn list_pages_feed_info_html(
    page_info: &PageInfo<'_>,
    arguments: &ListPagesArguments,
) -> Option<String> {
    let title = arguments
        .rss_title
        .as_deref()
        .filter(|title| !title.is_empty())?;
    let mut url = format!("http://{}.wikidot.com/feed/pages", page_info.site);

    if let Some(pagetype) = arguments.rss_path.pagetype.as_deref() {
        push_list_pages_feed_path_argument(&mut url, "pagetype", pagetype);
    }
    match arguments.rss_path.category.as_deref() {
        Some("*") => {}
        Some(category) => {
            push_list_pages_feed_path_argument(&mut url, "category", category);
        }
        None => {
            push_list_pages_feed_path_argument(
                &mut url,
                "category",
                &RenderService::page_info_category_slug(page_info),
            );
        }
    }
    if let Some(tags) = arguments.rss_path.tags.as_deref() {
        push_list_pages_feed_path_argument(&mut url, "tags", tags);
    }
    if let Some(parent) = arguments.rss_path.parent.as_deref() {
        push_list_pages_feed_path_argument(&mut url, "parent", parent);
    }
    if let Some(created_by) = arguments.rss_path.created_by.as_deref() {
        push_list_pages_feed_path_argument(&mut url, "created_by", created_by);
    }
    if let Some(offset) = arguments
        .rss_path
        .offset
        .as_deref()
        .filter(|offset| *offset != "0")
    {
        push_list_pages_feed_path_argument(&mut url, "offset", offset);
    }
    if let Some(rating) = arguments.rss_path.rating.as_deref() {
        push_list_pages_feed_path_argument(&mut url, "rating", rating);
    }
    if let Some(range) = arguments.rss_path.range.as_deref() {
        push_list_pages_feed_path_argument(&mut url, "range", range);
    }
    if let Some(order) = arguments.rss_path.order.as_deref() {
        push_list_pages_feed_path_argument(&mut url, "order", order);
    }
    if let Some(limit) = list_pages_feed_limit(arguments) {
        push_list_pages_feed_path_argument(&mut url, "limit", limit);
    }
    push_list_pages_feed_path_argument(&mut url, "t", title);
    if let Some(description) = arguments
        .rss_description
        .as_deref()
        .filter(|description| !description.is_empty())
    {
        push_list_pages_feed_path_argument(&mut url, "d", description);
    }
    if let Some(home) = arguments
        .rss_home
        .as_deref()
        .filter(|home| !home.is_empty())
    {
        push_list_pages_feed_path_argument(&mut url, "h", home);
    }

    Some(format!(
        concat!(
            "\n\n",
            r#"<div class="feedinfo" data-wikijump-compat-listpages-feed="1">"#,
            r#"<span class="rss-icon"><img src="/common--theme/base/images/feed/feed-icon-14x14.png" alt="rss icon"/></span>"#,
            r#"<a href="{url}">RSS feed</a></div>"#,
            "\n\n",
        ),
        url = escape_list_pages_html_attr(&url),
    ))
}

fn push_list_pages_feed_path_argument(url: &mut String, name: &str, value: &str) {
    url.push('/');
    url.push_str(name);
    url.push('/');
    url.push_str(&encode_list_pages_feed_path_segment(value));
}

fn list_pages_feed_limit(arguments: &ListPagesArguments) -> Option<&str> {
    if let Some(limit) = arguments.rss_limit.as_deref() {
        return Some(limit);
    }

    let limit = arguments
        .rss_path
        .limit
        .as_deref()
        .filter(|value| *value != "0");
    let per_page = arguments
        .rss_path
        .per_page
        .as_deref()
        .filter(|value| *value != "0");
    match (limit, per_page) {
        (Some(limit), Some(per_page)) => {
            Some(if list_pages_feed_limit_is_lower(limit, per_page) {
                limit
            } else {
                per_page
            })
        }
        (Some(limit), None) => Some(limit),
        (None, Some(per_page)) => Some(per_page),
        (None, None) => None,
    }
}

fn list_pages_feed_limit_is_lower(left: &str, right: &str) -> bool {
    match (left.parse::<f64>(), right.parse::<f64>()) {
        (Ok(left), Ok(right)) => left <= right,
        _ => left <= right,
    }
}

fn encode_list_pages_feed_path_segment(value: &str) -> String {
    percent_encode_path_segment(value).replace("%20", "+")
}

fn list_pages_pager_href_path(
    page_info: &PageInfo<'_>,
    url: UrlArguments<'_>,
    url_attr_prefix: Option<&str>,
    target_page: usize,
) -> String {
    let key = list_pages_page_argument_key(url_attr_prefix);
    let mut segments = Vec::with_capacity(1 + url.path_arguments.len() * 2 + 2);
    segments.push(percent_encode_path_segment(page_info.page.as_ref()));
    let mut replaced = false;

    for argument in url.path_arguments {
        segments.push(percent_encode_path_segment(&argument.name));
        if !replaced
            && argument.name.eq_ignore_ascii_case(key.as_ref())
            && argument
                .value
                .as_deref()
                .is_some_and(|value| value.parse::<u32>().is_ok())
        {
            segments.push(target_page.to_string());
            replaced = true;
        } else if let Some(value) = argument.value.as_deref() {
            segments.push(percent_encode_path_segment(value));
        }
    }

    if !replaced {
        segments.push(percent_encode_path_segment(key.as_ref()));
        segments.push(target_page.to_string());
    }

    segments.join("/")
}
