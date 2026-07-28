/*
 * services/render/categories.rs
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

//! The Wikidot `Categories` module.

use super::compat::CompatHtmlFragments;
use super::literal_regions::LiteralRegionIndex;
use super::module_arguments::wikidot_module_argument;
use super::service::{RenderService, escape_list_pages_html_text};
use crate::error::prelude::Result;
use crate::services::{CategoryService, ServiceContext};
use ftml::settings::WikitextSettings;
use regex::Regex;
use std::sync::LazyLock;

static CATEGORIES_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+Categories(?P<head>(?:\s+[^\]]*)?)\]\]").unwrap()
});

fn include_hidden_categories(head: &str) -> bool {
    wikidot_module_argument(head, "includeHidden")
        .is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

fn category_is_visible(slug: &str, include_hidden: bool) -> bool {
    slug == "_default" || include_hidden || !slug.starts_with('_')
}

fn wikidot_category_sort_key(slug: &str) -> (String, String) {
    (slug.replace('-', ""), slug.to_owned())
}

fn render_categories_module<'a>(
    categories: impl IntoIterator<Item = (i64, &'a str)>,
    include_hidden: bool,
) -> String {
    let mut output = String::new();

    for (category_id, slug) in categories {
        if !category_is_visible(slug, include_hidden) {
            continue;
        }

        if output.is_empty() {
            output.push('\n');
        }
        let slug = escape_list_pages_html_text(slug);
        output.push_str("<div>\n<h3>");
        output.push_str(&slug);
        output.push_str("</h3>\n<a href=\"javascript:;\" id=\"category-pages-toggler-");
        output.push_str(&category_id.to_string());
        output.push_str("\" onclick=\"WIKIDOT.modules.WikiCategoriesModule.listeners.toggleListPages(event, ");
        output.push_str(&category_id.to_string());
        output.push_str(")\">+ list pages</a>\n<div id=\"category-pages-");
        output.push_str(&category_id.to_string());
        output.push_str("\" style=\"display: none\"></div>\n<div id=\"category-pages-");
        output.push_str(&category_id.to_string());
        output.push_str("-options\" style=\"display: none\"></div>\n</div>\n");
    }

    output
}

impl RenderService {
    pub(super) async fn expand_categories_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax || !CATEGORIES_MODULE_REGEX.is_match(&wikitext) {
            return Ok(wikitext);
        }

        let Some(current_site_id) = current_site_id else {
            return Ok(wikitext);
        };

        let mut categories =
            CategoryService::get_all_active(ctx, current_site_id).await?;
        categories
            .sort_by_cached_key(|category| wikidot_category_sort_key(&category.slug));
        let category_refs = categories
            .iter()
            .map(|category| (category.category_id, category.slug.as_str()))
            .collect::<Vec<_>>();
        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;

        for captures in CATEGORIES_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a Categories capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }

            output.push_str(&wikitext[cursor..matched.start()]);
            let head = captures.name("head").map_or("", |mtch| mtch.as_str());
            let rendered = render_categories_module(
                category_refs.iter().copied(),
                include_hidden_categories(head),
            );
            output.push_str(&compat_html.push_block_html(rendered));
            cursor = matched.end();
        }

        if cursor == 0 {
            return Ok(wikitext);
        }
        output.push_str(&wikitext[cursor..]);
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CATEGORIES_MODULE_REGEX, category_is_visible, include_hidden_categories,
        render_categories_module, wikidot_category_sort_key,
    };

    #[test]
    fn categories_name_must_end_before_arguments() {
        assert!(CATEGORIES_MODULE_REGEX.is_match("[[module Categories]]"));
        assert!(
            CATEGORIES_MODULE_REGEX
                .is_match(r#"[[module categories includeHidden="true"]]"#)
        );
        assert!(!CATEGORIES_MODULE_REGEX.is_match("[[module CategoriesExtra]]"));
    }

    #[test]
    fn include_hidden_is_case_insensitive_and_defaults_to_false() {
        assert!(!include_hidden_categories(""));
        assert!(!include_hidden_categories(r#" includeHidden="false""#));
        assert!(include_hidden_categories(r#" INCLUDEHIDDEN = "TRUE""#));
    }

    #[test]
    fn default_category_is_visible_even_when_hidden_categories_are_not() {
        assert!(category_is_visible("_default", false));
        assert!(!category_is_visible("_admin", false));
        assert!(category_is_visible("_admin", true));
        assert!(category_is_visible("articles", false));
    }

    #[test]
    fn category_order_ignores_hyphens_like_wikidot() {
        let mut slugs = [
            "codexdfcoldfcol04",
            "codex-rating-load-20260715",
            "codexrole1518",
            "codexrateb5t153900z",
            "_default",
        ];
        slugs.sort_by_cached_key(|slug| wikidot_category_sort_key(slug));

        assert_eq!(
            slugs,
            [
                "_default",
                "codexdfcoldfcol04",
                "codexrateb5t153900z",
                "codex-rating-load-20260715",
                "codexrole1518",
            ],
        );
    }

    #[test]
    fn category_dom_matches_wikidot_and_escapes_the_slug() {
        let html = render_categories_module(
            [(17, "_default"), (23, "a<&"), (31, "_hidden")],
            false,
        );

        assert_eq!(
            html,
            concat!(
                "\n<div>\n<h3>_default</h3>\n",
                "<a href=\"javascript:;\" id=\"category-pages-toggler-17\" onclick=\"WIKIDOT.modules.WikiCategoriesModule.listeners.toggleListPages(event, 17)\">+ list pages</a>\n",
                "<div id=\"category-pages-17\" style=\"display: none\"></div>\n",
                "<div id=\"category-pages-17-options\" style=\"display: none\"></div>\n</div>\n",
                "<div>\n<h3>a&lt;&amp;</h3>\n",
                "<a href=\"javascript:;\" id=\"category-pages-toggler-23\" onclick=\"WIKIDOT.modules.WikiCategoriesModule.listeners.toggleListPages(event, 23)\">+ list pages</a>\n",
                "<div id=\"category-pages-23\" style=\"display: none\"></div>\n",
                "<div id=\"category-pages-23-options\" style=\"display: none\"></div>\n</div>\n",
            )
        );
    }
}
