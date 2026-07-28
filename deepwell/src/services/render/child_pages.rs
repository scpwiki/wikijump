/*
 * services/render/child_pages.rs
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

//! Wikidot `ChildPages` runtime module rendering.

use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;
use sea_orm::{ConnectionTrait, FromQueryResult, Statement};

use super::compat::CompatHtmlFragments;
use super::literal_regions::LiteralRegionIndex;
use super::module_arguments::{module_arguments_are_complete, wikidot_module_arguments};
use super::service::{
    RenderService, escape_list_pages_html_attr, escape_list_pages_html_text,
};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::services::ServiceContext;
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::types::Reference;
use crate::types::{Action, Permission, Resource};
use ftml::settings::WikitextSettings;

pub(super) static CHILD_PAGES_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+ChildPages(?P<head>(?:\s+[^\]]*)?)\]\]").unwrap()
});

#[derive(Debug, FromQueryResult)]
struct ChildPagesModulePage {
    page_id: i64,
    page_category_id: i64,
    slug: String,
    title: String,
}

fn parse_child_pages_module_arguments(head: &str) -> Option<()> {
    if !module_arguments_are_complete(head) {
        return None;
    }

    // Live Wikidot accepts and ignores unknown ChildPages attributes. Running
    // the ordinary parser here still rejects malformed heads so we fail closed
    // instead of widening unverified syntax.
    let _ = wikidot_module_arguments(head)?;
    Some(())
}

fn render_child_pages_module(pages: &[ChildPagesModulePage]) -> String {
    if pages.is_empty() {
        return String::new();
    }

    let mut output = String::from("<div class=\"child-pages-block\">\n<ul>\n");
    for page in pages {
        output.push_str("<li><a href=\"/");
        output.push_str(&escape_list_pages_html_attr(&page.slug));
        output.push_str("\">");
        output.push_str(&escape_list_pages_html_text(&page.title));
        output.push_str("</a></li>\n");
    }
    output.push_str("</ul>\n</div>");
    output
}

fn sort_child_pages_module_pages(pages: &mut [ChildPagesModulePage]) {
    pages.sort_by(|left, right| {
        left.title
            .to_ascii_lowercase()
            .cmp(&right.title.to_ascii_lowercase())
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.slug.cmp(&right.slug))
    });
}

impl RenderService {
    pub(super) async fn expand_child_pages_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        current_page_id: Option<i64>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax || !CHILD_PAGES_MODULE_REGEX.is_match(&wikitext) {
            return Ok(wikitext);
        }
        let (Some(current_site_id), Some(current_page_id)) =
            (current_site_id, current_page_id)
        else {
            return Ok(wikitext);
        };

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut expanded = String::with_capacity(wikitext.len());
        let mut cursor = 0;

        for captures in CHILD_PAGES_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a ChildPages capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }

            let head = captures.name("head").map_or("", |mtch| mtch.as_str());
            if parse_child_pages_module_arguments(head).is_none() {
                continue;
            }

            expanded.push_str(&wikitext[cursor..matched.start()]);
            let pages = Self::load_child_pages_module_pages(
                ctx,
                current_site_id,
                current_page_id,
            )
            .await?;
            expanded.push_str(
                &compat_html.push_block_html(render_child_pages_module(&pages)),
            );
            cursor = matched.end();
        }

        if cursor == 0 {
            return Ok(wikitext);
        }
        expanded.push_str(&wikitext[cursor..]);
        Ok(expanded)
    }

    async fn load_child_pages_module_pages(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        current_page_id: i64,
    ) -> Result<Vec<ChildPagesModulePage>> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to load ChildPages module rows for site ID {current_site_id}, page ID {current_page_id}"
                ),
                ErrorType::Render,
            )
        };
        let txn = ctx.transaction();
        let statement = Statement::from_sql_and_values(
            txn.get_database_backend(),
            "SELECT child.page_id, child.page_category_id, child.slug, child_rev.title \
             FROM page_parent pp \
             JOIN page child ON child.page_id = pp.child_page_id \
             JOIN page_revision child_rev ON child_rev.revision_id = child.latest_revision_id \
             WHERE pp.parent_page_id = $1 \
               AND child.site_id = $2 \
               AND child.deleted_at IS NULL",
            [current_page_id.into(), current_site_id.into()],
        );
        let rows = ChildPagesModulePage::find_by_statement(statement)
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

        sort_child_pages_module_pages(&mut viewable);
        Ok(viewable)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ChildPagesModulePage, parse_child_pages_module_arguments,
        render_child_pages_module, sort_child_pages_module_pages,
    };

    fn page(slug: &str, title: &str) -> ChildPagesModulePage {
        ChildPagesModulePage {
            page_id: 1,
            page_category_id: 1,
            slug: slug.to_owned(),
            title: title.to_owned(),
        }
    }

    #[test]
    fn parser_accepts_empty_and_unknown_complete_arguments() {
        assert_eq!(parse_child_pages_module_arguments(""), Some(()));
        assert_eq!(
            parse_child_pages_module_arguments(r#" foo="bar" data="ignored""#),
            Some(()),
        );
        assert_eq!(
            parse_child_pages_module_arguments(r#" foo="bar" junk"#),
            None
        );
    }

    #[test]
    fn renderer_matches_live_dom_and_escaping_boundaries() {
        let html = render_child_pages_module(&[
            page("category:alpha", "Alpha & One"),
            page("category:quote\"probe", "<Quoted>"),
        ]);

        assert_eq!(
            html,
            concat!(
                "<div class=\"child-pages-block\">\n",
                "<ul>\n",
                "<li><a href=\"/category:alpha\">Alpha &amp; One</a></li>\n",
                "<li><a href=\"/category:quote&quot;probe\">&lt;Quoted&gt;</a></li>\n",
                "</ul>\n",
                "</div>",
            ),
        );
        assert_eq!(render_child_pages_module(&[]), "");
    }

    #[test]
    fn rows_sort_alphabetically_by_live_title_order() {
        let mut pages = [
            page("fixture:zulu", "Zulu"),
            page("fixture:alpha", "alpha"),
            page("fixture:bravo", "Bravo"),
        ];
        sort_child_pages_module_pages(&mut pages);

        assert_eq!(
            pages.into_iter().map(|page| page.slug).collect::<Vec<_>>(),
            ["fixture:alpha", "fixture:bravo", "fixture:zulu"],
        );
    }
}
