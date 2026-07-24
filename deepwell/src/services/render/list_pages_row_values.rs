/*
 * services/render/list_pages_row_values.rs
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

//! Per-row value resolution for ListPages variables.
//!
//! Each resolver answers one variable for one selected row and returns nothing
//! when the value has no provenance, leaving the caller to preserve the module
//! rather than emit a plausible substitute.

use super::service::{
    ListPagesSnapshotDisplay, WikidotUserDisplay, escape_list_pages_html_attr,
    escape_list_pages_html_text,
};
use crate::services::page_query::FoundPageRow;
use std::collections::BTreeMap;

/// Resolves the revision count for `%%revisions%%`.
///
/// An imported page reports Wikidot's own count, because its local history is
/// the single import revision. A page that was never imported reports its
/// stored revision rows.
pub(super) fn list_pages_revision_count(
    page: &FoundPageRow,
    snapshot_displays: &BTreeMap<i64, ListPagesSnapshotDisplay>,
    revision_counts: &BTreeMap<i64, u64>,
) -> Option<u64> {
    match snapshot_displays.get(&page.page_id) {
        Some(snapshot) => u64::try_from(snapshot.source_revision_count).ok(),
        None => revision_counts.get(&page.page_id).copied(),
    }
}

/// Resolves the parent full name for `%%parent_fullname%%`.
///
/// An imported page reports the parent name Wikidot itself recorded, which is
/// singular. A page that was never imported reports its one live parent; a
/// second live parent is a local shape Wikidot cannot produce, and picking one
/// of them would be a guess, so it resolves to nothing instead.
pub(super) fn list_pages_parent_fullname<'a>(
    page: &FoundPageRow,
    snapshot_displays: &'a BTreeMap<i64, ListPagesSnapshotDisplay>,
    relational_parent_fullnames: &'a BTreeMap<i64, String>,
) -> Option<&'a str> {
    let parent_fullname = match snapshot_displays.get(&page.page_id) {
        Some(snapshot) => snapshot.parent_fullname.as_deref()?,
        None => relational_parent_fullnames.get(&page.page_id)?.as_str(),
    };
    (!parent_fullname.is_empty()).then_some(parent_fullname)
}

/// Resolves the creator's Wikidot unix name for `%%created_by_unix%%`.
///
/// The unix name is stored per account, so it is read from the creator's user
/// row rather than derived from a display name or from an imported snapshot
/// author string, which carries no unix name.
pub(super) fn list_pages_created_by_unix(
    page: &FoundPageRow,
    user_displays: &BTreeMap<i64, WikidotUserDisplay>,
    snapshot_displays: &BTreeMap<i64, ListPagesSnapshotDisplay>,
) -> Option<String> {
    // An imported row's author is the Wikidot account named in its snapshot,
    // while its local creating revision belongs to the account that ran the
    // import. Reading the local account's unix name there would report the
    // importer under the imported author's name, so the row resolves to
    // nothing until Wikidot user identity is imported alongside the page.
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

/// Renders a linked ListPages user cell.
///
/// An imported author has only a snapshot name, which is escaped as text; a
/// resolved account renders Wikidot's two-anchor `printuser` element.
pub(super) fn render_list_pages_wikidot_user(
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
            r#"<span class="printuser avatarhover">"#,
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

pub(super) fn render_list_pages_snapshot_user(name: &str) -> String {
    escape_list_pages_html_text(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_wikidot_list_pages_revision_count_from_import_before_local_history() {
        let page = FoundPageRow {
            page_id: 101,
            site_id: 1,
            title: Some("Devereaux".to_owned()),
            alt_title: None,
            slug: Some("devereaux".to_owned()),
            page_category_id: None,
            page_revision_id: None,
            tags: None,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
            score: None,
        };
        let source_created_at = time::OffsetDateTime::from_unix_timestamp(1_500_000_000)
            .expect("fixture timestamp should be valid");
        let snapshot = ListPagesSnapshotDisplay {
            created_at: source_created_at,
            updated_at: source_created_at,
            created_by_name: None,
            updated_by_name: None,
            comments: 0,
            commented_at: None,
            commented_by_name: None,
            rating_votes: None,
            parent_fullname: None,
            source_revision_count: 37,
        };
        let imported = BTreeMap::from([(101, snapshot.clone())]);
        // An imported page stores exactly one local revision, so local history
        // would report 1 where Wikidot reports the imported count.
        let local_history = BTreeMap::from([(101, 1)]);
        let empty_snapshots = BTreeMap::new();

        assert_eq!(
            list_pages_revision_count(&page, &imported, &local_history),
            Some(37),
            "an imported page reports Wikidot's own revision count",
        );
        assert_eq!(
            list_pages_revision_count(&page, &empty_snapshots, &local_history),
            Some(1),
            "a page that was never imported reports its stored revisions",
        );
        assert_eq!(
            list_pages_revision_count(&page, &empty_snapshots, &BTreeMap::new()),
            None,
            "a page with no history at all has no count to report",
        );

        let negative = BTreeMap::from([(
            101,
            ListPagesSnapshotDisplay {
                source_revision_count: -1,
                ..snapshot
            },
        )]);
        assert_eq!(
            list_pages_revision_count(&page, &negative, &local_history),
            None,
            "an unusable imported count must not fall through to local history",
        );
    }

    #[test]
    fn resolves_wikidot_list_pages_parent_fullname_from_import_before_local_relations() {
        let page = FoundPageRow {
            page_id: 101,
            site_id: 1,
            title: Some("Offset 0".to_owned()),
            alt_title: None,
            slug: Some("fragment:component:offset-timeline-0".to_owned()),
            page_category_id: None,
            page_revision_id: None,
            tags: None,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
            score: None,
        };
        let source_created_at = time::OffsetDateTime::from_unix_timestamp(1_500_000_000)
            .expect("fixture timestamp should be valid");
        let snapshot = ListPagesSnapshotDisplay {
            created_at: source_created_at,
            updated_at: source_created_at,
            created_by_name: None,
            updated_by_name: None,
            comments: 0,
            commented_at: None,
            commented_by_name: None,
            rating_votes: None,
            parent_fullname: Some("component:offset-timeline".to_owned()),
            source_revision_count: 2,
        };
        let imported = BTreeMap::from([(101, snapshot.clone())]);
        let relational =
            BTreeMap::from([(101, "component:some-locally-added-parent".to_owned())]);
        let empty_snapshots = BTreeMap::new();
        let empty_relations = BTreeMap::new();

        assert_eq!(
            list_pages_parent_fullname(&page, &imported, &relational),
            Some("component:offset-timeline"),
            "an imported page reports the parent Wikidot itself recorded",
        );
        assert_eq!(
            list_pages_parent_fullname(&page, &empty_snapshots, &relational),
            Some("component:some-locally-added-parent"),
            "a page that was never imported reports its live parent",
        );
        assert_eq!(
            list_pages_parent_fullname(&page, &empty_snapshots, &empty_relations),
            None,
            "a page with no parent from either source resolves to nothing",
        );

        let parentless_import = BTreeMap::from([(
            101,
            ListPagesSnapshotDisplay {
                parent_fullname: None,
                ..snapshot
            },
        )]);
        assert_eq!(
            list_pages_parent_fullname(&page, &parentless_import, &relational),
            None,
            "an imported page with no recorded parent does not fall through to local relations",
        );
    }
}
