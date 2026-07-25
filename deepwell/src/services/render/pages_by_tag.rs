/*
 * services/render/pages_by_tag.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

//! Wikidot `pagestag/PagesByTagModule` argument parsing and DOM rendering.
//!
//! The emitted markup reproduces a live Wikidot capture byte for byte, tab
//! indentation included, so imported author CSS that targets
//! `#tagged-pages-list .pages-list-item .title` keeps working. Only the
//! `tag` argument is evidenced; every other form stays literal.

use super::compat_html_fragments::CompatHtmlFragments;
use super::prelude::*;
use super::service::{
    RenderService, escape_list_pages_html_attr, escape_list_pages_html_text,
};
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::types::{Action, Permission, Resource};
use ftml::settings::WikitextSettings;
use regex::Regex;
use sea_orm::{ConnectionTrait, FromQueryResult, Statement};
use std::sync::LazyLock;

static PAGES_BY_TAG_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+PagesByTag(?P<head>[^\]]*)\]\]").unwrap()
});

/// Upper bound on rows a single PagesByTag expansion will load.
///
/// Live Wikidot emitted no pager for the largest captured tag, so this is a
/// render-cost guard rather than an emulated page size.
pub(super) const MAX_PAGES_BY_TAG_ROWS: usize = 500;

#[derive(Debug, FromQueryResult)]
pub(super) struct PagesByTagPage {
    pub page_id: i64,
    pub page_category_id: i64,
    pub slug: String,
    pub title: String,
}

/// Extracts the `tag` argument from a `[[module PagesByTag ...]]` head.
///
/// Returns `None` for any head this has no live evidence for, which keeps the
/// module literal instead of guessing a query.
pub(super) fn parse_pages_by_tag_tag(head: &str) -> Option<String> {
    let head = head.trim();
    let rest = head.strip_prefix("tag")?.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();

    let tag = if let Some(rest) = rest.strip_prefix('"') {
        let (tag, rest) = rest.split_once('"')?;
        if !rest.trim().is_empty() {
            return None;
        }
        tag
    } else {
        // A bare value cannot contain whitespace, and nothing may follow it.
        if rest.is_empty() || rest.split_whitespace().count() != 1 {
            return None;
        }
        rest
    };

    let tag = tag.trim();
    if tag.is_empty() {
        return None;
    }

    Some(tag.to_owned())
}

/// Renders the live PagesByTag DOM for `tag` over already-filtered `pages`.
pub(super) fn render_pages_by_tag_module(tag: &str, pages: &[PagesByTagPage]) -> String {
    let mut output =
        String::from("<a name=\"pages\"></a>\n<h2>List of pages tagged with <em>");
    output.push_str(&escape_list_pages_html_text(tag));
    output.push_str(
        "</em>:</h2> \n\n\n<div class=\"pages-list\" id=\"tagged-pages-list\">",
    );

    for page in pages {
        output.push_str("\n\t\t\t<div class=\"pages-list-item\">\n\t\t\t<div class=\"title\">\n\t\t\t\t<a href=\"/");
        output.push_str(&escape_list_pages_html_attr(&page.slug));
        output.push_str("\">");
        output.push_str(&escape_list_pages_html_text(&page.title));
        output.push_str("</a>\n\t\t\t</div>\n\t\t</div>");
    }

    output.push_str("\n\t</div>\n");
    output
}

/// Expands `[[module PagesByTag tag="..."]]` into the live Wikidot DOM.
///
/// Live Wikidot resolves the tag from the module argument first and from a
/// `/tag/<value>` URL argument otherwise. Wikijump routes no URL arguments
/// yet, so only the argument branch can occur here; a module with no `tag`
/// argument renders nothing, which is what live Wikidot does when no URL
/// argument supplies one either.
pub(super) async fn expand_pages_by_tag_modules(
    ctx: &ServiceContext<'_>,
    wikitext: String,
    settings: &WikitextSettings,
    current_site_id: Option<i64>,
    compat_html: &mut CompatHtmlFragments,
) -> Result<String> {
    if !settings.enable_page_syntax || !PAGES_BY_TAG_MODULE_REGEX.is_match(&wikitext) {
        return Ok(wikitext);
    }

    let Some(current_site_id) = current_site_id else {
        return Ok(wikitext);
    };

    let mut expanded = String::with_capacity(wikitext.len());
    let mut cursor = 0;

    for captures in PAGES_BY_TAG_MODULE_REGEX.captures_iter(&wikitext) {
        let mtch = captures.get(0).unwrap();
        expanded.push_str(&wikitext[cursor..mtch.start()]);
        cursor = mtch.end();

        if RenderService::is_inside_wikidot_literal_region(&wikitext, mtch.start()) {
            expanded.push_str(mtch.as_str());
            continue;
        }

        let head = captures.name("head").map_or("", |mtch| mtch.as_str());
        if head.trim().is_empty() {
            // Live emits nothing at all for a tagless module on a URL that
            // carries no tag argument, which is every URL Wikijump serves.
            continue;
        }

        let Some(tag) = parse_pages_by_tag_tag(head) else {
            // Unevidenced argument forms stay literal rather than resolve to
            // a guessed query.
            expanded.push_str(mtch.as_str());
            continue;
        };

        let pages = load_pages_by_tag_pages(ctx, current_site_id, &tag).await?;
        expanded.push_str(
            &compat_html.push_block_html(render_pages_by_tag_module(&tag, &pages)),
        );
    }

    expanded.push_str(&wikitext[cursor..]);
    Ok(expanded)
}

pub(super) async fn load_pages_by_tag_pages(
    ctx: &ServiceContext<'_>,
    current_site_id: i64,
    tag: &str,
) -> Result<Vec<PagesByTagPage>> {
    let make_error = || {
        Error::new(
            format!(
                "failed to load PagesByTag module rows for site ID {current_site_id}",
            ),
            ErrorType::Render,
        )
    };
    let txn = ctx.transaction();
    let statement = Statement::from_sql_and_values(
        txn.get_database_backend(),
        "SELECT p.page_id, p.page_category_id, p.slug, pr.title \
         FROM page p \
         JOIN page_revision pr ON pr.revision_id = p.latest_revision_id \
         WHERE p.site_id = $1 \
           AND p.deleted_at IS NULL \
           AND $2 = ANY(pr.tags) \
         ORDER BY lower(pr.title), p.slug \
         LIMIT $3",
        [
            current_site_id.into(),
            tag.into(),
            (MAX_PAGES_BY_TAG_ROWS as i64).into(),
        ],
    );

    let rows = PagesByTagPage::find_by_statement(statement)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn page(slug: &str, title: &str) -> PagesByTagPage {
        PagesByTagPage {
            page_id: 1,
            page_category_id: 1,
            slug: slug.to_owned(),
            title: title.to_owned(),
        }
    }

    #[test]
    fn parses_the_evidenced_quoted_tag_argument() {
        assert_eq!(
            parse_pages_by_tag_tag(r#" tag="cgmodcat""#).as_deref(),
            Some("cgmodcat"),
        );
        assert_eq!(
            parse_pages_by_tag_tag(r#"tag="cg-pbt-order-20260725""#).as_deref(),
            Some("cg-pbt-order-20260725"),
        );
        assert_eq!(
            parse_pages_by_tag_tag(" tag=cgmodcat").as_deref(),
            Some("cgmodcat")
        );
    }

    #[test]
    fn leaves_unevidenced_argument_forms_unparsed() {
        // No live capture covers a missing tag, extra arguments, an empty
        // value, or a second tag, so each must stay literal.
        for head in [
            "",
            "   ",
            r#" tag="""#,
            r#" tag="a" tag="b""#,
            r#" tag="a" limit="5""#,
            r#" limit="5""#,
            " tag=a b",
            " category=\"scp\"",
        ] {
            assert_eq!(parse_pages_by_tag_tag(head), None, "head: {head:?}");
        }
    }

    #[test]
    fn renders_the_captured_live_dom_byte_for_byte() {
        let pages = [
            page("cg-pbt-order-alpha", "Alpha Probe"),
            page("cg-pbt-order-bravo", "Bravo Probe"),
            page("cg-pbt-order-charlie", "Charlie Probe"),
        ];
        let rendered = render_pages_by_tag_module("cg-pbt-order-20260725", &pages);

        assert_eq!(
            rendered,
            concat!(
                "<a name=\"pages\"></a>\n",
                "<h2>List of pages tagged with <em>cg-pbt-order-20260725</em>:</h2> \n",
                "\n\n<div class=\"pages-list\" id=\"tagged-pages-list\">",
                "\n\t\t\t<div class=\"pages-list-item\">\n\t\t\t<div class=\"title\">\n",
                "\t\t\t\t<a href=\"/cg-pbt-order-alpha\">Alpha Probe</a>\n\t\t\t</div>\n\t\t</div>",
                "\n\t\t\t<div class=\"pages-list-item\">\n\t\t\t<div class=\"title\">\n",
                "\t\t\t\t<a href=\"/cg-pbt-order-bravo\">Bravo Probe</a>\n\t\t\t</div>\n\t\t</div>",
                "\n\t\t\t<div class=\"pages-list-item\">\n\t\t\t<div class=\"title\">\n",
                "\t\t\t\t<a href=\"/cg-pbt-order-charlie\">Charlie Probe</a>\n\t\t\t</div>\n\t\t</div>",
                "\n\t</div>\n",
            ),
        );
    }

    #[test]
    fn escapes_tag_titles_and_slugs() {
        let pages = [page("a<b>", "T<script>alert(1)</script>")];
        let rendered = render_pages_by_tag_module("x\"y<z", &pages);

        assert!(!rendered.contains("<script>"));
        assert!(rendered.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        // The tag and title are element text, where a bare quote is inert; the
        // slug is an attribute value, where it is not.
        assert!(rendered.contains("<em>x\"y&lt;z</em>"));
        assert!(rendered.contains("href=\"/a&lt;b&gt;\""));
        assert!(!rendered.contains("href=\"/a<b>\""));
    }
}
