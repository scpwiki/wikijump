//! Runtime-backed Wikidot module expansion.

use std::borrow::Cow;
use std::collections::BTreeMap;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use super::compat::CompatHtmlFragments;
use super::compat::text_fragments::CompatTextFragments;
use super::list_pages::{is_tag_cloud_visible_tag, render_tag_cloud_box};
use super::literal_regions::LiteralRegionIndex;
use super::native_list_context::collect_unproven_scope_ranges;
use super::service::{
    RATE_MODULE_REGEX, REGISTRY_MODULE_REGEX, RenderService, TAGCLOUD_MODULE_REGEX,
    escape_list_pages_html_attr, escape_list_pages_html_text, render_clone_module,
    render_members_module_placeholder, render_new_page_module,
    render_read_only_rate_module, wikidot_module_argument,
};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::page::{self, Entity as Page};
use crate::models::page_revision;
use crate::services::ServiceContext;
use crate::services::settings::PageRatingType;
use ftml::data::PageInfo;
use ftml::settings::WikitextSettings;

fn render_join_module(head: &str) -> String {
    let button = wikidot_module_argument(head, "button")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Join");
    let class = wikidot_module_argument(head, "class")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("join-box");
    format!(
        concat!(
            r#"<div class="{class}">"#,
            r#"<a href="javascript:;" onclick="WIKIDOT.page.listeners.join(event, 'unified')">{button}</a>"#,
            "</div>",
        ),
        class = escape_list_pages_html_attr(class),
        button = escape_list_pages_html_text(button),
    )
}

impl RenderService {
    pub(super) fn expand_rate_modules_with_registry(
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        rating_type: PageRatingType,
        compat_html: &mut CompatHtmlFragments,
        compat_text: &mut CompatTextFragments,
    ) -> String {
        if !settings.enable_page_syntax || !RATE_MODULE_REGEX.is_match(&wikitext) {
            return wikitext;
        }

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let footnote_ranges = collect_unproven_scope_ranges(&wikitext, &literal_regions)
            .into_iter()
            .filter(|range| wikidot_scope_head_is(&wikitext, range.start, "footnote"))
            .collect::<Vec<_>>();
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for matched in RATE_MODULE_REGEX.find_iter(&wikitext) {
            if literal_regions.contains(matched.start()) {
                continue;
            }
            let line_start = wikitext[..matched.start()]
                .rfind('\n')
                .map_or(0, |index| index + 1);
            if wikitext[line_start..matched.start()]
                .trim_start()
                .starts_with('>')
            {
                continue;
            }
            output.push_str(&wikitext[cursor..matched.start()]);
            if footnote_ranges
                .iter()
                .any(|range| range.start < matched.start() && matched.end() <= range.end)
            {
                output.push_str(&compat_text.push_escaped_html_text(matched.as_str()));
                cursor = matched.end();
                continue;
            }
            output.push_str(&compat_html.push_block_html(render_read_only_rate_module(
                page_info.score,
                &page_info.language,
                rating_type,
            )));
            cursor = matched.end();
        }
        if cursor == 0 {
            return wikitext;
        }
        output.push_str(&wikitext[cursor..]);
        output
    }

    pub(super) fn expand_registry_modules_with_registry(
        wikitext: String,
        settings: &WikitextSettings,
        compat_html: &mut CompatHtmlFragments,
    ) -> String {
        Self::expand_registry_modules_matching(wikitext, settings, compat_html, |_| true)
    }

    fn expand_registry_modules_matching(
        wikitext: String,
        settings: &WikitextSettings,
        compat_html: &mut CompatHtmlFragments,
        mut should_expand: impl FnMut(&str) -> bool,
    ) -> String {
        if !settings.enable_page_syntax {
            return wikitext;
        }

        // Keep one index over the authored source for the complete pass. A replacement must not expose a later candidate that the original literal, comment, or tag boundaries protected, so malformed cross-boundary input remains fail closed.
        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for captures in REGISTRY_MODULE_REGEX.captures_iter(&wikitext) {
            let matched = captures
                .get(0)
                .expect("a module capture always has a complete match");
            if literal_regions.contains(matched.start()) {
                continue;
            }
            let name = captures
                .name("name")
                .expect("a registry module capture always has a name")
                .as_str();
            if !should_expand(name) {
                continue;
            }
            output.push_str(&wikitext[cursor..matched.start()]);
            let head = captures.name("head").map_or("", |mtch| mtch.as_str());
            let rendered = if name.eq_ignore_ascii_case("Members") {
                let group = wikidot_module_argument(head, "group")
                    .unwrap_or("members")
                    .trim();
                render_members_module_placeholder(group)
            } else if name.eq_ignore_ascii_case("NewPage") {
                render_new_page_module(head)
            } else if name.eq_ignore_ascii_case("Clone") {
                render_clone_module(head)
            } else {
                debug_assert!(name.eq_ignore_ascii_case("Join"));
                render_join_module(head)
            };
            let marker = if name.eq_ignore_ascii_case("Join") {
                compat_html.push_block_html(rendered)
            } else {
                compat_html.push_html(rendered)
            };
            output.push_str(&marker);
            cursor = matched.end();
        }
        if cursor == 0 {
            return wikitext;
        }
        output.push_str(&wikitext[cursor..]);
        output
    }

    #[cfg(test)]
    pub(super) fn expand_members_modules(
        wikitext: String,
        settings: &WikitextSettings,
    ) -> String {
        let mut fragments = CompatHtmlFragments::new(&wikitext);
        let protected = Self::expand_registry_modules_matching(
            wikitext,
            settings,
            &mut fragments,
            |name| name.eq_ignore_ascii_case("Members"),
        );
        fragments.restore(&protected)
    }

    #[cfg(test)]
    pub(super) fn expand_new_page_modules(
        wikitext: String,
        settings: &WikitextSettings,
    ) -> String {
        let mut fragments = CompatHtmlFragments::new(&wikitext);
        let protected = Self::expand_registry_modules_matching(
            wikitext,
            settings,
            &mut fragments,
            |name| name.eq_ignore_ascii_case("NewPage"),
        );
        fragments.restore(&protected)
    }

    #[cfg(test)]
    pub(super) fn expand_clone_modules(
        wikitext: String,
        settings: &WikitextSettings,
    ) -> String {
        let mut fragments = CompatHtmlFragments::new(&wikitext);
        let protected = Self::expand_registry_modules_matching(
            wikitext,
            settings,
            &mut fragments,
            |name| name.eq_ignore_ascii_case("Clone"),
        );
        fragments.restore(&protected)
    }

    #[cfg(test)]
    pub(super) fn expand_join_modules(
        wikitext: String,
        settings: &WikitextSettings,
    ) -> String {
        let mut fragments = CompatHtmlFragments::new(&wikitext);
        let protected = Self::expand_registry_modules_matching(
            wikitext,
            settings,
            &mut fragments,
            |name| name.eq_ignore_ascii_case("Join"),
        );
        fragments.restore(&protected)
    }

    pub(super) async fn expand_tag_cloud_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        page_info: &PageInfo<'_>,
        current_site_id: Option<i64>,
        current_page_id: Option<i64>,
    ) -> Result<String> {
        if !TAGCLOUD_MODULE_REGEX.is_match(&wikitext) {
            return Ok(wikitext);
        }

        let (Some(current_site_id), Some(current_page_id)) =
            (current_site_id, current_page_id)
        else {
            return Ok(wikitext);
        };

        let current_branch_tag = page_info
            .tags
            .iter()
            .find(|tag| tag.starts_with("branch-"))
            .map(Cow::as_ref);
        let tags = Self::load_tag_cloud_counts(
            ctx,
            current_site_id,
            current_page_id,
            current_branch_tag,
        )
        .await?;
        let replacement = render_tag_cloud_box(&tags);
        Ok(TAGCLOUD_MODULE_REGEX
            .replace_all(&wikitext, replacement.as_str())
            .into_owned())
    }

    async fn load_tag_cloud_counts(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        _current_page_id: i64,
        current_branch_tag: Option<&str>,
    ) -> Result<Vec<(String, usize)>> {
        let make_error =
            || Error::new("failed to render TagCloud module", ErrorType::Render);
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
        if revision_ids.is_empty() {
            return Ok(Vec::new());
        }

        let revisions = page_revision::Entity::find()
            .filter(page_revision::Column::RevisionId.is_in(revision_ids))
            .all(txn)
            .await
            .or_raise(make_error)?;
        let mut counts = BTreeMap::<String, usize>::new();
        for revision in revisions {
            if let Some(branch_tag) = current_branch_tag
                && !revision.tags.iter().any(|tag| tag == branch_tag)
            {
                continue;
            }
            for tag in revision.tags {
                if !is_tag_cloud_visible_tag(&tag) {
                    continue;
                }
                *counts.entry(tag).or_default() += 1;
            }
        }

        Ok(counts.into_iter().collect())
    }
}

fn wikidot_scope_head_is(source: &str, start: usize, expected: &str) -> bool {
    let Some(tail) = source.get(start + 2..) else {
        return false;
    };
    let Some(end) = tail.find("]]") else {
        return false;
    };
    tail[..end].trim().eq_ignore_ascii_case(expected)
}
