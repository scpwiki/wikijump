/*
 * services/render/page_tree.rs
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

//! The Wikidot `PageTree` runtime module.

use std::collections::{HashMap, HashSet};
use std::num::NonZeroU32;
use std::sync::LazyLock;

use regex::Regex;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use super::compat::CompatHtmlFragments;
use super::literal_regions::LiteralRegionIndex;
use super::module_arguments::{module_arguments_are_complete, wikidot_module_arguments};
use super::service::{
    RenderService, escape_list_pages_html_attr, escape_list_pages_html_text,
};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::page::{self, Entity as Page};
use crate::models::page_parent::{self, Entity as PageParent};
use crate::models::page_revision::{self, Entity as PageRevision};
use crate::services::ServiceContext;
use ftml::settings::WikitextSettings;

static PAGE_TREE_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)\[\[module\s+PageTree(?P<head>(?:\s+[^\]]*)?)\]\]").unwrap()
});

#[derive(Debug)]
struct PageTreeNode {
    slug: String,
    title: String,
}

#[derive(Debug)]
struct PageTree {
    nodes: HashMap<i64, PageTreeNode>,
    page_ids_by_slug: HashMap<String, i64>,
    child_ids_by_parent: HashMap<i64, Vec<i64>>,
}

#[derive(Debug, PartialEq, Eq)]
struct PageTreeArguments<'a> {
    root: Option<&'a str>,
    show_root: bool,
    depth: Option<NonZeroU32>,
}

fn page_tree_argument<'a>(head: &'a str, name: &str) -> Option<&'a str> {
    wikidot_module_arguments(head)?
        .into_iter()
        .rev()
        .find(|argument| argument.key == name)
        .map(|argument| argument.value)
}

fn parse_page_tree_arguments(head: &str) -> Option<PageTreeArguments<'_>> {
    if !module_arguments_are_complete(head) {
        return None;
    }

    Some(PageTreeArguments {
        root: page_tree_argument(head, "root"),
        show_root: page_tree_argument(head, "showRoot") == Some("true"),
        depth: page_tree_argument(head, "depth").and_then(|value| value.parse().ok()),
    })
}

fn root_is_supported(root: Option<&str>) -> bool {
    root.is_none_or(|root| {
        !root.starts_with(':')
            && !root.contains("://")
            && !root.starts_with("//")
            && !root.contains('@')
    })
}

fn render_page_tree(
    tree: &PageTree,
    root_id: i64,
    show_root: bool,
    depth: Option<NonZeroU32>,
) -> String {
    let mut output = String::new();
    let mut visited = HashSet::new();
    visited.insert(root_id);

    if show_root {
        output.push_str("\n<ul>\n");
        render_page_tree_node(
            &mut output,
            tree,
            root_id,
            0,
            depth.map(NonZeroU32::get),
            &mut visited,
        );
        output.push_str("</ul>\n");
    } else {
        render_page_tree_children(
            &mut output,
            tree,
            root_id,
            1,
            depth.map(NonZeroU32::get),
            &mut visited,
        );
    }

    output
}

fn render_page_tree_children(
    output: &mut String,
    tree: &PageTree,
    parent_id: i64,
    depth: u32,
    max_depth: Option<u32>,
    visited: &mut HashSet<i64>,
) -> bool {
    let Some(child_ids) = tree.child_ids_by_parent.get(&parent_id) else {
        return false;
    };
    let child_ids = child_ids
        .iter()
        .copied()
        .filter(|child_id| !visited.contains(child_id))
        .collect::<Vec<_>>();
    if child_ids.is_empty() {
        return false;
    }

    output.push_str("\n<ul>\n");
    for child_id in child_ids {
        visited.insert(child_id);
        render_page_tree_node(output, tree, child_id, depth, max_depth, visited);
    }
    output.push_str("</ul>\n");
    true
}

fn render_page_tree_node(
    output: &mut String,
    tree: &PageTree,
    page_id: i64,
    depth: u32,
    max_depth: Option<u32>,
    visited: &mut HashSet<i64>,
) {
    let Some(node) = tree.nodes.get(&page_id) else {
        return;
    };

    output.push_str("<li>\n<a href=\"/");
    output.push_str(&escape_list_pages_html_attr(&node.slug));
    output.push_str("\">");
    output.push_str(&escape_list_pages_html_text(&node.title));
    output.push_str("</a>");
    let rendered_children = if max_depth.is_none_or(|max_depth| depth < max_depth) {
        render_page_tree_children(
            output,
            tree,
            page_id,
            depth.saturating_add(1),
            max_depth,
            visited,
        )
    } else {
        false
    };
    if !rendered_children {
        output.push('\n');
    }
    output.push_str("</li>\n");
}

impl RenderService {
    pub(super) async fn expand_categories_and_page_tree_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        current_page: (Option<i64>, Option<i64>),
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        let (current_site_id, current_page_id) = current_page;
        let wikitext = Self::expand_categories_modules(
            ctx,
            wikitext,
            settings,
            current_site_id,
            compat_html,
        )
        .await?;
        Self::expand_page_tree_modules(
            ctx,
            wikitext,
            settings,
            current_site_id,
            current_page_id,
            compat_html,
        )
        .await
    }

    async fn expand_page_tree_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        current_page_id: Option<i64>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax || !PAGE_TREE_MODULE_REGEX.is_match(&wikitext) {
            return Ok(wikitext);
        }
        let (Some(current_site_id), Some(current_page_id)) =
            (current_site_id, current_page_id)
        else {
            return Ok(wikitext);
        };

        let tree = Self::load_page_tree(ctx, current_site_id).await?;
        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;

        for captures in PAGE_TREE_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a PageTree capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }
            let head = captures.name("head").map_or("", |mtch| mtch.as_str());
            let Some(arguments) = parse_page_tree_arguments(head) else {
                continue;
            };
            if !root_is_supported(arguments.root) {
                continue;
            }

            output.push_str(&wikitext[cursor..matched.start()]);
            let root_id = arguments.root.map_or(Some(current_page_id), |root| {
                tree.page_ids_by_slug.get(root).copied()
            });
            if let Some(root_id) = root_id {
                output.push_str(&compat_html.push_block_html(render_page_tree(
                    &tree,
                    root_id,
                    arguments.show_root,
                    arguments.depth,
                )));
            }
            cursor = matched.end();
        }

        if cursor == 0 {
            return Ok(wikitext);
        }
        output.push_str(&wikitext[cursor..]);
        Ok(output)
    }

    async fn load_page_tree(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
    ) -> Result<PageTree> {
        let make_error =
            || Error::new("failed to render PageTree module", ErrorType::Render);
        let txn = ctx.transaction();
        let pages = Page::find()
            .filter(page::Column::SiteId.eq(current_site_id))
            .filter(page::Column::DeletedAt.is_null())
            .all(txn)
            .await
            .or_raise(make_error)?;
        let revision_ids = pages
            .iter()
            .filter_map(|page| page.latest_revision_id)
            .collect::<Vec<_>>();
        let revisions = PageRevision::find()
            .filter(page_revision::Column::RevisionId.is_in(revision_ids))
            .all(txn)
            .await
            .or_raise(make_error)?;
        let titles = revisions
            .into_iter()
            .map(|revision| (revision.revision_id, revision.title))
            .collect::<HashMap<_, _>>();

        let mut nodes = HashMap::with_capacity(pages.len());
        let mut page_ids_by_slug = HashMap::with_capacity(pages.len());
        for page in pages {
            let Some(revision_id) = page.latest_revision_id else {
                continue;
            };
            let Some(title) = titles.get(&revision_id) else {
                continue;
            };
            page_ids_by_slug.insert(page.slug.clone(), page.page_id);
            nodes.insert(
                page.page_id,
                PageTreeNode {
                    slug: page.slug,
                    title: title.clone(),
                },
            );
        }

        let page_ids = nodes.keys().copied().collect::<Vec<_>>();
        let relationships = PageParent::find()
            .filter(page_parent::Column::ParentPageId.is_in(page_ids.clone()))
            .filter(page_parent::Column::ChildPageId.is_in(page_ids))
            .order_by_asc(page_parent::Column::CreatedAt)
            .order_by_asc(page_parent::Column::ChildPageId)
            .all(txn)
            .await
            .or_raise(make_error)?;
        let mut child_ids_by_parent = HashMap::<i64, Vec<i64>>::new();
        for relationship in relationships {
            child_ids_by_parent
                .entry(relationship.parent_page_id)
                .or_default()
                .push(relationship.child_page_id);
        }

        Ok(PageTree {
            nodes,
            page_ids_by_slug,
            child_ids_by_parent,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::{
        PageTree, PageTreeArguments, PageTreeNode, parse_page_tree_arguments,
        render_page_tree, root_is_supported,
    };
    use std::collections::HashMap;

    fn fixture_tree() -> PageTree {
        let nodes = [
            (1, "root", "Root"),
            (2, "alpha", "Alpha"),
            (3, "beta", "Beta"),
            (4, "grandchild", "Grandchild"),
            (5, "great-grandchild", "Great <Child>"),
        ]
        .into_iter()
        .map(|(page_id, slug, title)| {
            (
                page_id,
                PageTreeNode {
                    slug: slug.to_owned(),
                    title: title.to_owned(),
                },
            )
        })
        .collect::<HashMap<_, _>>();
        PageTree {
            page_ids_by_slug: nodes
                .iter()
                .map(|(page_id, node)| (node.slug.clone(), *page_id))
                .collect(),
            nodes,
            child_ids_by_parent: HashMap::from([
                (1, vec![2, 3]),
                (2, vec![4]),
                (4, vec![5]),
                (5, vec![1]),
            ]),
        }
    }

    #[test]
    fn arguments_match_live_case_sensitive_semantics() {
        assert_eq!(
            parse_page_tree_arguments(
                r#" root="first" root="root" showRoot="true" depth="2""#,
            ),
            Some(PageTreeArguments {
                root: Some("root"),
                show_root: true,
                depth: NonZeroU32::new(2),
            }),
        );
        for head in [
            r#" Root="root" Showroot="true" Depth="2""#,
            r#" showRoot="TRUE" depth="0""#,
            r#" showRoot="yes" depth="many""#,
        ] {
            assert_eq!(
                parse_page_tree_arguments(head),
                Some(PageTreeArguments {
                    root: None,
                    show_root: false,
                    depth: None,
                }),
            );
        }
        assert_eq!(parse_page_tree_arguments(r#" root="x" garbage"#), None);
    }

    #[test]
    fn live_dom_depth_and_cycle_boundaries_are_preserved() {
        let html = render_page_tree(&fixture_tree(), 1, false, NonZeroU32::new(2));
        assert_eq!(
            html,
            concat!(
                "\n<ul>\n",
                "<li>\n<a href=\"/alpha\">Alpha</a>\n",
                "<ul>\n<li>\n<a href=\"/grandchild\">Grandchild</a>\n</li>\n</ul>\n",
                "</li>\n",
                "<li>\n<a href=\"/beta\">Beta</a>\n</li>\n",
                "</ul>\n",
            ),
        );

        let html = render_page_tree(&fixture_tree(), 1, true, None);
        assert!(html.starts_with("\n<ul>\n<li>\n<a href=\"/root\">Root</a>"));
        assert!(html.contains("Great &lt;Child&gt;"));
        assert_eq!(html.matches(r#"href="/root""#).count(), 1);
    }

    #[test]
    fn url_and_cross_site_root_forms_fail_closed() {
        assert!(root_is_supported(None));
        assert!(root_is_supported(Some("category:page")));
        assert!(!root_is_supported(Some(":other-site:page")));
        assert!(!root_is_supported(Some("https://other.example/page")));
        assert!(!root_is_supported(Some("//other.example/page")));
        assert!(!root_is_supported(Some("page@other-site")));
    }
}
