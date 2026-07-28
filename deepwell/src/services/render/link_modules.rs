/*
 * services/render/link_modules.rs
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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

//! Wikidot link-list modules backed by the page connection graph.

use super::compat::CompatHtmlFragments;
use super::service::{
    RenderService, escape_list_pages_html_attr, escape_list_pages_html_text,
};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::services::ServiceContext;
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::types::{Action, Permission, Reference, Resource};
use ftml::settings::WikitextSettings;
use regex::Regex;
use sea_orm::{ConnectionTrait, FromQueryResult, Statement};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::sync::LazyLock;

/// The most rows one site-level link module render will load.
pub(super) const MAX_LINK_LISTING_MODULE_ROWS: usize = 2_000;
const WANTED_PAGES_PER_PAGE: usize = 50;
const MAX_WANTED_PAGES_MODULE_SOURCE_ROWS: usize = 10_000;

pub(super) static ORPHANED_PAGES_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+OrphanedPages(?P<head>[^\]]*)\]\]").unwrap()
});
pub(super) static WANTED_PAGES_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+WantedPages(?P<head>[^\]]*)\]\]").unwrap()
});

#[derive(Debug, FromQueryResult)]
struct LinkModulePage {
    page_id: i64,
    page_category_id: i64,
    slug: String,
    title: String,
}

#[derive(Debug, FromQueryResult)]
struct WantedPagesModuleSourceRow {
    to_page_slug: String,
    page_id: i64,
    page_category_id: i64,
    slug: String,
    title: String,
}

#[derive(Debug)]
struct WantedPagesModuleTarget {
    slug: String,
    sources: Vec<LinkModulePage>,
}

fn render_orphaned_pages_module_box(pages: &[LinkModulePage]) -> String {
    let mut output = String::from("\n<h1>List of orphaned pages</h1>\n\n");

    for page in pages {
        output.push_str("\t\t\t<a href=\"/");
        output.push_str(&escape_list_pages_html_attr(&page.slug));
        output.push_str(r#"">"#);
        output.push_str(&escape_list_pages_html_text(&page.title));
        output.push_str(r#"</a> <span style="color: #999">("#);
        output.push_str(&escape_list_pages_html_text(&page.slug));
        output.push_str(")</span>\n\t\t<br/>\n");
    }

    output.push_str("\t\n");
    output
}

fn render_wanted_pages_module_box(targets: &[WantedPagesModuleTarget]) -> String {
    let page_count = targets.len().div_ceil(WANTED_PAGES_PER_PAGE).max(1);
    let paginated = page_count > 1;
    let mut output = String::from("\n        <div class=\"wanted-pages-module\">\n\n\n");

    if paginated {
        push_wanted_pages_pager(&mut output, page_count, "            ");
    }

    output.push_str(concat!(
        "    <table class=\"form grid\" style=\"margin: 1em auto;\">\n",
        "        <tr>\n",
        "            <th>\n",
        "                Linked from            </th>\n",
        "            <th>\n",
        "                Linked to (wanted page name)            </th>\n",
        "        </tr>\n",
    ));

    for target in targets.iter().take(WANTED_PAGES_PER_PAGE) {
        output.push_str(concat!(
            "                    <tr>\n",
            "                <td>\n",
        ));
        for source in &target.sources {
            output.push_str("                                            <a href=\"/");
            output.push_str(&escape_list_pages_html_attr(&source.slug));
            output.push_str("\">");
            output.push_str(&escape_list_pages_html_text(&source.title));
            output.push_str("</a><br/>\n");
        }
        output.push_str(concat!(
            "                                    </td>\n",
            "                <td>\n",
            "                    <a href=\"/",
        ));
        output.push_str(&escape_list_pages_html_attr(&target.slug));
        output.push_str("\" class=\"newpage\">");
        output.push_str(&escape_list_pages_html_text(&target.slug));
        output.push_str(concat!(
            "</a>\n",
            "                </td>\n",
            "            </tr>\n",
        ));
    }

    output.push_str("            </table>\n\n");
    if paginated {
        output.push_str("                ");
        push_wanted_pages_pager(&mut output, page_count, "");
    }
    output.push_str("    </div> \n");
    output
}

fn push_wanted_pages_pager(output: &mut String, page_count: usize, prefix: &str) {
    output.push_str(prefix);
    output.push_str(r#"<div class="pager"><span class="pager-no">page 1 of "#);
    output.push_str(&page_count.to_string());
    output.push_str(r#"</span><span class="current">1</span>"#);

    for page in 2..=page_count {
        push_wanted_pages_pager_target(output, page, &page.to_string());
    }
    push_wanted_pages_pager_target(output, 2, "next &raquo;");
    output.push_str("</div>\n    \n\n");
}

fn push_wanted_pages_pager_target(output: &mut String, page: usize, label: &str) {
    output.push_str(
        r#"<span class="target"><a href="javascript:;" onclick="WIKIDOT.modules.WantedPagesModule.updateList("#,
    );
    output.push_str(&page.to_string());
    output.push_str(", this)\">");
    output.push_str(label);
    output.push_str("</a></span>");
}

impl RenderService {
    pub(super) async fn expand_link_listing_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax {
            return Ok(wikitext);
        }

        let Some(current_site_id) = current_site_id else {
            return Ok(wikitext);
        };

        if !ORPHANED_PAGES_MODULE_REGEX.is_match(&wikitext)
            && !WANTED_PAGES_MODULE_REGEX.is_match(&wikitext)
        {
            return Ok(wikitext);
        }

        let wikitext = Self::expand_orphaned_pages_modules(
            ctx,
            wikitext,
            current_site_id,
            compat_html,
        )
        .await?;
        Self::expand_wanted_pages_modules(ctx, wikitext, current_site_id, compat_html)
            .await
    }

    async fn expand_orphaned_pages_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        current_site_id: i64,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !ORPHANED_PAGES_MODULE_REGEX.is_match(&wikitext) {
            return Ok(wikitext);
        }
        let mut expanded = String::with_capacity(wikitext.len());
        let mut cursor = 0;

        for captures in ORPHANED_PAGES_MODULE_REGEX.captures_iter(&wikitext) {
            let mtch = captures.get(0).unwrap();
            expanded.push_str(&wikitext[cursor..mtch.start()]);

            if Self::is_inside_wikidot_literal_region(&wikitext, mtch.start()) {
                expanded.push_str(mtch.as_str());
                cursor = mtch.end();
                continue;
            }

            let pages =
                Self::load_orphaned_pages_module_pages(ctx, current_site_id).await?;
            expanded.push_str(
                &compat_html.push_block_html(render_orphaned_pages_module_box(&pages)),
            );
            cursor = mtch.end();
        }

        expanded.push_str(&wikitext[cursor..]);
        Ok(expanded)
    }

    async fn expand_wanted_pages_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        current_site_id: i64,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !WANTED_PAGES_MODULE_REGEX.is_match(&wikitext) {
            return Ok(wikitext);
        }
        let mut expanded = String::with_capacity(wikitext.len());
        let mut cursor = 0;

        for captures in WANTED_PAGES_MODULE_REGEX.captures_iter(&wikitext) {
            let mtch = captures.get(0).unwrap();
            expanded.push_str(&wikitext[cursor..mtch.start()]);

            if Self::is_inside_wikidot_literal_region(&wikitext, mtch.start()) {
                expanded.push_str(mtch.as_str());
                cursor = mtch.end();
                continue;
            }

            let targets =
                Self::load_wanted_pages_module_targets(ctx, current_site_id).await?;
            expanded.push_str(
                &compat_html.push_block_html(render_wanted_pages_module_box(&targets)),
            );
            cursor = mtch.end();
        }

        expanded.push_str(&wikitext[cursor..]);
        Ok(expanded)
    }

    async fn load_orphaned_pages_module_pages(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
    ) -> Result<Vec<LinkModulePage>> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to load OrphanedPages module rows for site ID {}",
                    current_site_id,
                ),
                ErrorType::Render,
            )
        };
        let txn = ctx.transaction();
        let statement = Statement::from_string(
            txn.get_database_backend(),
            format!(
                "SELECT p.page_id, p.page_category_id, p.slug, pr.title \
                 FROM page p \
                 JOIN page_revision pr ON pr.revision_id = p.latest_revision_id \
                 WHERE p.site_id = {current_site_id} \
                   AND p.deleted_at IS NULL \
                   AND NOT EXISTS ( \
                     SELECT 1 \
                     FROM page_connection pc \
                     WHERE pc.to_page_id = p.page_id \
                       AND pc.from_page_id <> p.page_id \
                       AND pc.connection_type = 'link' \
                   ) \
                 ORDER BY lower(pr.title), p.slug \
                 LIMIT {MAX_LINK_LISTING_MODULE_ROWS}",
            ),
        );

        let rows = LinkModulePage::find_by_statement(statement)
            .all(txn)
            .await
            .or_raise(make_error)?;

        let mut viewable = Vec::with_capacity(rows.len());
        for row in rows {
            let anonymously_viewable = PermissionService::check_user_can(
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

            if anonymously_viewable {
                viewable.push(row);
            }
        }

        Ok(viewable)
    }

    async fn load_wanted_pages_module_targets(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
    ) -> Result<Vec<WantedPagesModuleTarget>> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to load WantedPages module rows for site ID {}",
                    current_site_id,
                ),
                ErrorType::Render,
            )
        };
        let txn = ctx.transaction();
        let statement = Statement::from_string(
            txn.get_database_backend(),
            format!(
                "SELECT pcm.to_page_slug, p.page_id, p.page_category_id, p.slug, pr.title \
                 FROM page_connection_missing pcm \
                 JOIN page p ON p.page_id = pcm.from_page_id \
                 JOIN page_revision pr ON pr.revision_id = p.latest_revision_id \
                 WHERE pcm.to_site_id = {current_site_id} \
                   AND pcm.connection_type = 'link' \
                   AND p.site_id = {current_site_id} \
                   AND p.deleted_at IS NULL \
                   AND NOT EXISTS ( \
                     SELECT 1 \
                     FROM page existing_page \
                     WHERE existing_page.site_id = pcm.to_site_id \
                       AND existing_page.slug = pcm.to_page_slug \
                       AND existing_page.deleted_at IS NULL \
                   ) \
                 ORDER BY lower(pcm.to_page_slug), pcm.to_page_slug, lower(pr.title), p.slug \
                 LIMIT {MAX_WANTED_PAGES_MODULE_SOURCE_ROWS}",
            ),
        );

        let rows = WantedPagesModuleSourceRow::find_by_statement(statement)
            .all(txn)
            .await
            .or_raise(make_error)?;

        let mut permission_cache = HashMap::new();
        let mut grouped: BTreeMap<String, Vec<LinkModulePage>> = BTreeMap::new();
        for row in rows {
            let anonymously_viewable = match permission_cache.get(&row.page_id) {
                Some(viewable) => *viewable,
                None => {
                    let viewable = PermissionService::check_user_can(
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
                    permission_cache.insert(row.page_id, viewable);
                    viewable
                }
            };

            if anonymously_viewable {
                grouped
                    .entry(row.to_page_slug)
                    .or_default()
                    .push(LinkModulePage {
                        page_id: row.page_id,
                        page_category_id: row.page_category_id,
                        slug: row.slug,
                        title: row.title,
                    });
            }
        }

        let mut targets = grouped
            .into_iter()
            .map(|(slug, mut sources)| {
                sources.sort_by(compare_link_module_pages_by_title);
                WantedPagesModuleTarget { slug, sources }
            })
            .collect::<Vec<_>>();
        targets.sort_by(|left, right| compare_case_folded(&left.slug, &right.slug));
        Ok(targets)
    }
}

fn compare_link_module_pages_by_title(
    left: &LinkModulePage,
    right: &LinkModulePage,
) -> Ordering {
    compare_case_folded(&left.title, &right.title)
        .then_with(|| left.slug.cmp(&right.slug))
}

fn compare_case_folded(left: &str, right: &str) -> Ordering {
    left.to_ascii_lowercase()
        .cmp(&right.to_ascii_lowercase())
        .then_with(|| left.cmp(right))
}

#[cfg(test)]
mod tests {
    use super::{
        LinkModulePage, WantedPagesModuleTarget, render_orphaned_pages_module_box,
        render_wanted_pages_module_box,
    };

    #[test]
    fn orphaned_pages_box_matches_live_wikidot_rows() {
        let html = render_orphaned_pages_module_box(&[LinkModulePage {
            page_id: 1,
            page_category_id: 1,
            slug: "example-page".to_owned(),
            title: "Example Page".to_owned(),
        }]);

        assert_eq!(
            html,
            concat!(
                "\n<h1>List of orphaned pages</h1>\n\n",
                "\t\t\t<a href=\"/example-page\">Example Page</a> ",
                "<span style=\"color: #999\">(example-page)</span>\n",
                "\t\t<br/>\n\t\n",
            ),
        );
    }

    #[test]
    fn wanted_pages_box_matches_live_wikidot_table() {
        let html = render_wanted_pages_module_box(&[WantedPagesModuleTarget {
            slug: "missing-page".to_owned(),
            sources: vec![LinkModulePage {
                page_id: 1,
                page_category_id: 1,
                slug: "source-page".to_owned(),
                title: "Source Page".to_owned(),
            }],
        }]);

        assert!(html.contains(r#"<div class="wanted-pages-module">"#));
        assert!(html.contains(r#"<table class="form grid" style="margin: 1em auto;">"#));
        assert!(html.contains("Linked from"));
        assert!(html.contains(r#"<a href="/source-page">Source Page</a><br/>"#));
        assert!(
            html.contains(r#"<a href="/missing-page" class="newpage">missing-page</a>"#)
        );
        assert!(!html.contains(r#"<div class="pager">"#));
    }
}
