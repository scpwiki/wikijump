//! Runtime-backed Wikidot module expansion.

use std::borrow::Cow;
use std::collections::BTreeMap;

use sea_orm::{ColumnTrait, EntityTrait, FromQueryResult, QueryFilter, Statement};

use super::compat::CompatHtmlFragments;
use super::list_pages::{
    BacklinksModulePage, is_tag_cloud_visible_tag, render_tag_cloud_box,
};
use super::literal_regions::LiteralRegionIndex;
use super::service::{
    BACKLINKS_MODULE_REGEX, MAX_BACKLINKS_MODULE_ROWS, RATE_MODULE_REGEX,
    REGISTRY_MODULE_REGEX, RenderService, TAGCLOUD_MODULE_REGEX,
    render_backlinks_module_box, render_clone_module, render_members_module_placeholder,
    render_new_page_module, render_read_only_rate_module, wikidot_module_argument,
};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::page::{self, Entity as Page};
use crate::models::page_revision;
use crate::services::ServiceContext;
use crate::services::permission::{CheckPermissionContext, PermissionService};
use crate::types::Reference;
use crate::types::{Action, Permission, Resource};
use ftml::data::PageInfo;
use ftml::settings::WikitextSettings;
use sea_orm::ConnectionTrait;

impl RenderService {
    pub(super) fn expand_rate_modules_with_registry(
        wikitext: String,
        page_info: &PageInfo<'_>,
        settings: &WikitextSettings,
        compat_html: &mut CompatHtmlFragments,
    ) -> String {
        if !settings.enable_page_syntax {
            return wikitext;
        }

        let literal_regions =
            LiteralRegionIndex::new_wikidot_module_recognition(&wikitext);
        let mut output = String::with_capacity(wikitext.len());
        let mut cursor = 0;
        for matched in RATE_MODULE_REGEX.find_iter(&wikitext) {
            if literal_regions.contains(matched.start()) {
                continue;
            }
            output.push_str(&wikitext[cursor..matched.start()]);
            output.push_str(&compat_html.push_block_html(render_read_only_rate_module(
                page_info.score,
                &page_info.language,
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
            } else {
                debug_assert!(name.eq_ignore_ascii_case("Clone"));
                render_clone_module(head)
            };
            output.push_str(&compat_html.push_html(rendered));
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

    pub(super) async fn expand_backlinks_modules(
        ctx: &ServiceContext<'_>,
        wikitext: String,
        settings: &WikitextSettings,
        current_site_id: Option<i64>,
        current_page_id: Option<i64>,
        compat_html: &mut CompatHtmlFragments,
    ) -> Result<String> {
        if !settings.enable_page_syntax || !BACKLINKS_MODULE_REGEX.is_match(&wikitext) {
            return Ok(wikitext);
        }

        let (Some(current_site_id), Some(current_page_id)) =
            (current_site_id, current_page_id)
        else {
            return Ok(wikitext);
        };

        let mut expanded = String::with_capacity(wikitext.len());
        let mut cursor = 0;

        for captures in BACKLINKS_MODULE_REGEX.captures_iter(&wikitext) {
            let mtch = captures.get(0).unwrap();
            expanded.push_str(&wikitext[cursor..mtch.start()]);

            if Self::is_inside_wikidot_literal_region(&wikitext, mtch.start()) {
                expanded.push_str(mtch.as_str());
                cursor = mtch.end();
                continue;
            }

            let head = captures.name("head").map_or("", |mtch| mtch.as_str());
            if !head.trim().is_empty() {
                expanded.push_str(mtch.as_str());
                cursor = mtch.end();
                continue;
            }

            let pages =
                Self::load_backlinks_module_pages(ctx, current_site_id, current_page_id)
                    .await?;
            expanded
                .push_str(&compat_html.push_html(render_backlinks_module_box(&pages)));
            cursor = mtch.end();
        }

        expanded.push_str(&wikitext[cursor..]);
        Ok(expanded)
    }

    async fn load_backlinks_module_pages(
        ctx: &ServiceContext<'_>,
        current_site_id: i64,
        current_page_id: i64,
    ) -> Result<Vec<BacklinksModulePage>> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to load Backlinks module rows for page ID {} in site ID {}",
                    current_page_id, current_site_id,
                ),
                ErrorType::Render,
            )
        };
        let txn = ctx.transaction();
        let statement = Statement::from_string(
            txn.get_database_backend(),
            format!(
                "SELECT p.page_id, p.page_category_id, p.slug, pr.title \
                 FROM page_connection pc \
                 JOIN page p ON p.page_id = pc.from_page_id \
                 JOIN page_revision pr ON pr.revision_id = p.latest_revision_id \
                 WHERE pc.to_page_id = {current_page_id} \
                   AND pc.connection_type = 'link' \
                   AND p.site_id = {current_site_id} \
                   AND p.deleted_at IS NULL \
                 ORDER BY lower(pr.title), p.slug \
                 LIMIT {MAX_BACKLINKS_MODULE_ROWS}",
            ),
        );

        let rows = BacklinksModulePage::find_by_statement(statement)
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
