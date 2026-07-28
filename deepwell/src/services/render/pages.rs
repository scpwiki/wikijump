/*
 * services/render/pages.rs
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

//! Wikidot `Pages` site-index rendering.

use super::compat::CompatHtmlFragments;
use super::diagnostics::{
    CorpusRenderScope, CorpusRenderStage, CorpusRenderTrace, StageGuard,
};
use super::list_pages::{WikidotUserDisplay, render_list_pages_wikidot_user};
use super::module_arguments::wikidot_module_arguments;
use super::pages_by_tag::expand_pages_by_tag_modules;
use super::percent_encoding::percent_encode_path_segment;
use super::service::{
    RenderService, escape_list_pages_html_attr, escape_list_pages_html_text,
    format_wikidot_list_pages_date,
};
use super::url_arguments::UrlArguments;
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::services::ServiceContext;
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::types::Reference;
use crate::types::{Action, Permission, Resource};
use ftml::data::PageInfo;
use ftml::settings::WikitextSettings;
use regex::Regex;
use sea_orm::{ConnectionTrait, FromQueryResult, Statement};
use std::collections::{BTreeSet, HashMap};
use std::sync::LazyLock;

pub(super) static PAGES_MODULE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)\[\[module\s+Pages(?P<head>[^\]]*)\]\]").unwrap());

const PAGES_PER_PAGE: usize = 20;

#[derive(Debug, FromQueryResult)]
pub(super) struct PagesModulePage {
    pub page_id: i64,
    pub page_category_id: i64,
    pub slug: String,
    pub title: String,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
    pub revision_number: i32,
    pub updated_by: i64,
    pub wikidot_user_name: Option<String>,
    pub wikidot_user_slug: Option<String>,
    pub local_user_name: Option<String>,
    pub local_user_slug: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PagesModuleOrder {
    DateCreatedAsc,
    DateCreatedDesc,
    DateEditedAsc,
    DateEditedDesc,
    TitleAsc,
    TitleDesc,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PagesModuleArguments {
    category: Option<String>,
    details: bool,
    order: PagesModuleOrder,
    limit: Option<usize>,
}

impl Default for PagesModuleArguments {
    fn default() -> Self {
        Self {
            category: None,
            details: false,
            order: PagesModuleOrder::TitleAsc,
            limit: None,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn expand_page_index_modules(
    ctx: &ServiceContext<'_>,
    wikitext: String,
    page_info: &PageInfo<'_>,
    settings: &WikitextSettings,
    current_site_id: Option<i64>,
    url: UrlArguments<'_>,
    trace: Option<(&CorpusRenderTrace, CorpusRenderScope)>,
    compat_html: &mut CompatHtmlFragments,
) -> Result<String> {
    let wikitext = {
        let _stage = StageGuard::new(trace, CorpusRenderStage::Pages);
        expand_pages_modules(
            ctx,
            wikitext,
            page_info,
            settings,
            current_site_id,
            url.page,
            compat_html,
        )
        .await?
    };
    let _stage = StageGuard::new(trace, CorpusRenderStage::PagesByTag);
    expand_pages_by_tag_modules(
        ctx,
        wikitext,
        page_info,
        settings,
        current_site_id,
        url,
        compat_html,
    )
    .await
}

pub(super) async fn expand_pages_modules(
    ctx: &ServiceContext<'_>,
    wikitext: String,
    page_info: &PageInfo<'_>,
    settings: &WikitextSettings,
    current_site_id: Option<i64>,
    requested_page: Option<u32>,
    compat_html: &mut CompatHtmlFragments,
) -> Result<String> {
    if !settings.enable_page_syntax || !PAGES_MODULE_REGEX.is_match(&wikitext) {
        return Ok(wikitext);
    }

    let Some(current_site_id) = current_site_id else {
        return Ok(wikitext);
    };

    let mut expanded = String::with_capacity(wikitext.len());
    let mut cursor = 0;

    for captures in PAGES_MODULE_REGEX.captures_iter(&wikitext) {
        let mtch = captures.get(0).unwrap();
        expanded.push_str(&wikitext[cursor..mtch.start()]);
        cursor = mtch.end();

        let Some(arguments) = parse_pages_module_arguments(
            captures.name("head").map_or("", |head| head.as_str()),
        ) else {
            expanded.push_str(mtch.as_str());
            continue;
        };

        if RenderService::is_inside_wikidot_literal_region(&wikitext, mtch.start()) {
            expanded.push_str(mtch.as_str());
            continue;
        }

        let pages = load_pages_module_pages(ctx, current_site_id, &arguments).await?;
        let html = render_pages_module(
            page_info,
            &pages,
            requested_page.unwrap_or(1),
            arguments.details,
        );
        expanded.push_str(&compat_html.push_block_html(html));
    }

    expanded.push_str(&wikitext[cursor..]);
    Ok(expanded)
}

async fn load_pages_module_pages(
    ctx: &ServiceContext<'_>,
    current_site_id: i64,
    arguments: &PagesModuleArguments,
) -> Result<Vec<PagesModulePage>> {
    let make_error = || {
        Error::new(
            format!("failed to load Pages module rows for site ID {current_site_id}"),
            ErrorType::Render,
        )
    };
    let txn = ctx.transaction();
    let mut values = vec![current_site_id.into()];
    let category_filter = if let Some(category) = &arguments.category {
        values.push(category.as_str().into());
        " AND pc.slug = $2"
    } else {
        ""
    };
    let statement = Statement::from_sql_and_values(
        txn.get_database_backend(),
        format!(
            "SELECT p.page_id, p.page_category_id, p.slug, pr.title, \
                p.created_at, COALESCE(p.updated_at, pr.created_at) AS updated_at, \
                pr.revision_number, pr.user_id AS updated_by, \
                wu.name AS wikidot_user_name, wu.slug AS wikidot_user_slug, \
                local_user.name AS local_user_name, local_user.slug AS local_user_slug \
         FROM page p \
         JOIN page_revision pr ON pr.revision_id = p.latest_revision_id \
         JOIN page_category pc ON pc.category_id = p.page_category_id \
         LEFT JOIN wikidot_user wu ON wu.user_id = pr.user_id AND wu.is_deleted = FALSE \
         LEFT JOIN \"user\" local_user ON local_user.user_id = pr.user_id \
         WHERE p.site_id = $1 \
           AND p.deleted_at IS NULL \
           {category_filter}",
        ),
        values,
    );
    let rows = PagesModulePage::find_by_statement(statement)
        .all(txn)
        .await
        .or_raise(make_error)?;

    let mut category_permissions = HashMap::new();
    let mut viewable = Vec::with_capacity(rows.len());
    for row in rows {
        let can_view =
            if let Some(can_view) = category_permissions.get(&row.page_category_id) {
                *can_view
            } else {
                let can_view = PermissionService::check_user_can(
                    ctx,
                    &CheckPermissionContext {
                        user_id: None,
                        site_id: current_site_id,
                        page_reference: Some(Reference::Id(row.page_id)),
                    },
                    Permission {
                        resource_type: Resource::Page,
                        resource_category: Some(Reference::Id(row.page_category_id)),
                        action: Action::View,
                    },
                )
                .await
                .or_raise(make_error)?;
                category_permissions.insert(row.page_category_id, can_view);
                can_view
            };
        if can_view {
            viewable.push(row);
        }
    }

    sort_pages_module_pages(&mut viewable, arguments.order);
    if let Some(limit) = arguments.limit {
        viewable.truncate(limit);
    }

    Ok(viewable)
}

pub(super) fn render_pages_module(
    page_info: &PageInfo<'_>,
    pages: &[PagesModulePage],
    requested_page: u32,
    details: bool,
) -> String {
    let page_count = pages.len().div_ceil(PAGES_PER_PAGE).max(1);
    let current_page = (requested_page.max(1) as usize).min(page_count);
    let start = (current_page - 1) * PAGES_PER_PAGE;
    let end = (start + PAGES_PER_PAGE).min(pages.len());

    let mut output = String::from("<div class=\"list-pages-box\">    ");
    for page in &pages[start..end] {
        if details {
            push_pages_details_row(&mut output, page);
        } else {
            push_pages_title_row(&mut output, page);
        }
    }

    if page_count > 1 {
        output.push_str("    \n    ");
        push_pages_pager(&mut output, page_info, current_page, page_count);
        output.push_str("\n    ");
    }
    output.push_str("</div>");
    output
}

fn parse_pages_module_arguments(head: &str) -> Option<PagesModuleArguments> {
    let mut arguments = PagesModuleArguments::default();
    for argument in wikidot_module_arguments(head)? {
        match argument.key.to_ascii_lowercase().as_str() {
            "category" => {
                arguments.category = Some(argument.value.trim().to_owned());
            }
            "details" => {
                arguments.details = argument.value.eq_ignore_ascii_case("true");
            }
            "preview" => {
                // Live Wikidot accepts this documented flag but currently ignores it.
            }
            "order" => {
                arguments.order = match argument.value {
                    "dateCreatedAsc" => PagesModuleOrder::DateCreatedAsc,
                    "dateCreatedDesc" => PagesModuleOrder::DateCreatedDesc,
                    "dateEditedAsc" => PagesModuleOrder::DateEditedAsc,
                    "dateEditedDesc" => PagesModuleOrder::DateEditedDesc,
                    "titleAsc" => PagesModuleOrder::TitleAsc,
                    "titleDesc" => PagesModuleOrder::TitleDesc,
                    _ => arguments.order,
                };
            }
            "limit" => {
                arguments.limit = Some(argument.value.parse::<usize>().ok()?);
            }
            _ => {
                // Live Wikidot ignores unknown Pages arguments while applying
                // recognized arguments from the same module invocation.
            }
        }
    }
    Some(arguments)
}

fn sort_pages_module_pages(pages: &mut [PagesModulePage], order: PagesModuleOrder) {
    match order {
        PagesModuleOrder::DateCreatedAsc => {
            pages.sort_by(|left, right| {
                left.created_at
                    .cmp(&right.created_at)
                    .then_with(|| left.page_id.cmp(&right.page_id))
                    .then_with(|| title_asc(left, right))
            });
        }
        PagesModuleOrder::DateCreatedDesc => {
            pages.sort_by(|left, right| {
                right
                    .created_at
                    .cmp(&left.created_at)
                    .then_with(|| right.page_id.cmp(&left.page_id))
                    .then_with(|| title_asc(left, right))
            });
        }
        PagesModuleOrder::DateEditedAsc => {
            pages.sort_by(|left, right| {
                left.updated_at
                    .cmp(&right.updated_at)
                    .then_with(|| left.page_id.cmp(&right.page_id))
                    .then_with(|| title_asc(left, right))
            });
        }
        PagesModuleOrder::DateEditedDesc => {
            pages.sort_by(|left, right| {
                right
                    .updated_at
                    .cmp(&left.updated_at)
                    .then_with(|| right.page_id.cmp(&left.page_id))
                    .then_with(|| title_asc(left, right))
            });
        }
        PagesModuleOrder::TitleAsc => pages.sort_by(title_asc),
        PagesModuleOrder::TitleDesc => {
            pages.sort_by(|left, right| title_asc(right, left))
        }
    }
}

fn title_asc(left: &PagesModulePage, right: &PagesModulePage) -> std::cmp::Ordering {
    left.title
        .to_ascii_lowercase()
        .cmp(&right.title.to_ascii_lowercase())
        .then_with(|| left.title.cmp(&right.title))
        .then_with(|| left.slug.cmp(&right.slug))
}

fn push_pages_title_row(output: &mut String, page: &PagesModulePage) {
    output.push_str(
        "<div class=\"list-pages-item\">\n\n\n<div class=\"title\">\n<p><a href=\"/",
    );
    output.push_str(&escape_list_pages_html_attr(&page.slug));
    output.push_str("\">");
    output.push_str(&escape_list_pages_html_text(&page.title));
    output.push_str("</a></p>\n</div>\n</div>\n");
}

fn push_pages_details_row(output: &mut String, page: &PagesModulePage) {
    output.push_str(
        "<div class=\"list-pages-item\">\n<table>\n<tr>\n<td class=\"title\"><a href=\"/",
    );
    output.push_str(&escape_list_pages_html_attr(&page.slug));
    output.push_str("\">");
    output.push_str(&escape_list_pages_html_text(&page.title));
    output.push_str("</a></td>\n<td class=\"last-mod-by\">");
    let display = page.user_display();
    output.push_str(&render_list_pages_wikidot_user(
        page.updated_by,
        display.as_ref(),
    ));
    output.push_str("</td>\n<td class=\"revision-no\">rev. ");
    output.push_str(&page.revision_number.to_string());
    output.push_str("</td>\n<td class=\"last-mod-date\">");
    output.push_str(&pages_details_date(page.updated_at));
    output.push_str("</td>\n</tr>\n</table>\n</div>\n");
}

fn pages_details_date(date: time::OffsetDateTime) -> String {
    let format = "%e %b %Y - %H:%M|agohover";
    let display_format = "%e %b %Y %H:%M";
    let text = format_wikidot_list_pages_date(date, display_format);
    let encoded_format = percent_encode_path_segment(format);
    format!(
        r#"<span class="odate time_{} format_{}">{}</span>"#,
        date.unix_timestamp(),
        encoded_format,
        escape_list_pages_html_text(&text),
    )
}

impl PagesModulePage {
    fn user_display(&self) -> Option<WikidotUserDisplay> {
        if let Some(name) = self
            .wikidot_user_name
            .as_ref()
            .or(self.wikidot_user_slug.as_ref())
        {
            return Some(WikidotUserDisplay {
                user_id: self.updated_by,
                name: name.clone(),
                slug: self.wikidot_user_slug.clone(),
                wikidot_profile: true,
            });
        }

        self.local_user_name
            .as_ref()
            .or(self.local_user_slug.as_ref())
            .map(|name| WikidotUserDisplay {
                user_id: self.updated_by,
                name: name.clone(),
                slug: self.local_user_slug.clone(),
                wikidot_profile: false,
            })
    }
}

fn push_pages_pager(
    output: &mut String,
    page_info: &PageInfo<'_>,
    current_page: usize,
    page_count: usize,
) {
    output.push_str("<div class=\"pager\"><span class=\"pager-no\">page ");
    output.push_str(&current_page.to_string());
    output.push_str(" of ");
    output.push_str(&page_count.to_string());
    output.push_str("</span>");

    if current_page > 1 {
        push_pages_pager_target(output, page_info, current_page - 1, "« previous");
    }

    let mut pages = BTreeSet::from([1, current_page, page_count]);
    for distance in 1..=2 {
        if current_page > distance {
            pages.insert(current_page - distance);
        }
        if current_page + distance <= page_count {
            pages.insert(current_page + distance);
        }
    }
    if page_count > 1 {
        pages.insert(page_count - 1);
    }

    let mut previous = 0;
    for page in pages {
        if previous != 0 && page > previous + 1 {
            output.push_str("<span class=\"dots\">...</span>");
        }
        if page == current_page {
            output.push_str("<span class=\"current\">");
            output.push_str(&page.to_string());
            output.push_str("</span>");
        } else {
            push_pages_pager_target(output, page_info, page, &page.to_string());
        }
        previous = page;
    }

    if current_page < page_count {
        push_pages_pager_target(output, page_info, current_page + 1, "next »");
    }
    output.push_str("</div>");
}

fn push_pages_pager_target(
    output: &mut String,
    page_info: &PageInfo<'_>,
    target_page: usize,
    label: &str,
) {
    output.push_str("<span class=\"target\"><a href=\"/");
    output.push_str(&percent_encode_path_segment(
        &RenderService::page_info_full_slug(page_info),
    ));
    output.push_str("/p/");
    output.push_str(&target_page.to_string());
    output.push_str("\">");
    output.push_str(label);
    output.push_str("</a></span>");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    fn page(number: usize) -> PagesModulePage {
        PagesModulePage {
            page_id: number as i64,
            page_category_id: 1,
            slug: format!("page-{number}"),
            title: format!("Page {number:02}"),
            created_at: time::OffsetDateTime::UNIX_EPOCH,
            updated_at: time::OffsetDateTime::UNIX_EPOCH,
            revision_number: 0,
            updated_by: 0,
            wikidot_user_name: None,
            wikidot_user_slug: None,
            local_user_name: None,
            local_user_slug: None,
        }
    }

    fn page_info() -> PageInfo<'static> {
        PageInfo {
            page: Cow::Borrowed("site-index"),
            category: None,
            site: Cow::Borrowed("sandbox"),
            title: Cow::Borrowed("Site Index"),
            alt_title: None,
            score: ftml::data::ScoreValue::Integer(0),
            tags: Vec::new(),
            language: Cow::Borrowed("en"),
        }
    }

    #[test]
    fn renders_twenty_rows_with_the_captured_dom_shape() {
        let pages = (1..=21).map(page).collect::<Vec<_>>();
        let rendered = render_pages_module(&page_info(), &pages, 1, false);

        assert_eq!(rendered.matches("class=\"list-pages-item\"").count(), 20);
        assert!(rendered.starts_with(
            "<div class=\"list-pages-box\">    <div class=\"list-pages-item\">\n\n\n<div class=\"title\">\n<p><a href=\"/page-1\">Page 01</a></p>"
        ));
        assert!(rendered.contains("<span class=\"pager-no\">page 1 of 2</span>"));
        assert!(rendered.contains(
            "<span class=\"target\"><a href=\"/site-index/p/2\">next »</a></span>"
        ));
    }

    #[test]
    fn renders_previous_and_clamps_an_out_of_range_page() {
        let pages = (1..=41).map(page).collect::<Vec<_>>();
        let rendered = render_pages_module(&page_info(), &pages, 999, false);

        assert_eq!(rendered.matches("class=\"list-pages-item\"").count(), 1);
        assert!(rendered.contains("<span class=\"pager-no\">page 3 of 3</span>"));
        assert!(rendered.contains(
            "<span class=\"target\"><a href=\"/site-index/p/2\">« previous</a></span>"
        ));
        assert!(rendered.contains("<span class=\"current\">3</span>"));
        assert!(!rendered.contains("next »"));
    }

    #[test]
    fn renders_the_captured_second_page_pager_targets_in_order() {
        let pages = (1..=6_520).map(page).collect::<Vec<_>>();
        let rendered = render_pages_module(&page_info(), &pages, 2, false);
        let pager = rendered
            .split_once("<div class=\"pager\">")
            .expect("pager should render")
            .1
            .split_once("</div>")
            .expect("pager should close")
            .0;

        assert_eq!(
            pager,
            concat!(
                "<span class=\"pager-no\">page 2 of 326</span>",
                "<span class=\"target\"><a href=\"/site-index/p/1\">« previous</a></span>",
                "<span class=\"target\"><a href=\"/site-index/p/1\">1</a></span>",
                "<span class=\"current\">2</span>",
                "<span class=\"target\"><a href=\"/site-index/p/3\">3</a></span>",
                "<span class=\"target\"><a href=\"/site-index/p/4\">4</a></span>",
                "<span class=\"dots\">...</span>",
                "<span class=\"target\"><a href=\"/site-index/p/325\">325</a></span>",
                "<span class=\"target\"><a href=\"/site-index/p/326\">326</a></span>",
                "<span class=\"target\"><a href=\"/site-index/p/3\">next »</a></span>",
            ),
        );
    }

    #[test]
    fn escapes_page_titles_and_slugs() {
        let pages = [PagesModulePage {
            page_id: 1,
            page_category_id: 1,
            slug: "unsafe\"<slug>".to_owned(),
            title: "<script>alert(1)</script>".to_owned(),
            created_at: time::OffsetDateTime::UNIX_EPOCH,
            updated_at: time::OffsetDateTime::UNIX_EPOCH,
            revision_number: 0,
            updated_by: 0,
            wikidot_user_name: None,
            wikidot_user_slug: None,
            local_user_name: None,
            local_user_slug: None,
        }];
        let rendered = render_pages_module(&page_info(), &pages, 1, false);

        assert!(!rendered.contains("<script>"));
        assert!(rendered.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(rendered.contains("href=\"/unsafe&quot;&lt;slug&gt;\""));
    }
}
