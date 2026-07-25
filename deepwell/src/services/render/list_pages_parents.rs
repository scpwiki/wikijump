/*
 * services/render/list_pages_parents.rs
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

//! Parent full names for the ListPages `%%parent_fullname%%` variable.
//!
//! Wikidot gives a page at most one parent, while the Wikijump schema models
//! `page_parent` as a many-to-many relation. A row therefore resolves to a
//! parent full name only when exactly one live parent exists; every other
//! shape is left unresolved for the caller to fail closed on, because live
//! Wikidot output for parentless, multi-parent, and deleted-parent rows has
//! not been captured.

use super::prelude::*;
use crate::services::page_query::FoundPageRow;
use sea_orm::{ConnectionTrait, FromQueryResult, Statement};
use std::collections::{BTreeMap, BTreeSet};

/// The number of direct children each given row has, keyed by page ID.
///
/// Live Wikidot reports this as `%%children%%`: `component:offset-timeline`
/// answers 2 for its two fragment children, and a page with none answers 0.
/// Deleted children are not counted, matching the parent lookup below.
pub(super) async fn load_list_pages_child_counts(
    ctx: &ServiceContext<'_>,
    pages: &[FoundPageRow],
) -> Result<BTreeMap<i64, u64>> {
    #[derive(FromQueryResult, Debug)]
    struct ChildCountRow {
        parent_page_id: i64,
        child_count: i64,
    }

    let page_ids = pages
        .iter()
        .map(|page| page.page_id)
        .collect::<BTreeSet<_>>();
    if page_ids.is_empty() {
        return Ok(BTreeMap::new());
    }

    let make_error = || {
        Error::new(
            "failed to count child pages for ListPages render",
            ErrorType::Render,
        )
    };
    let values = page_ids
        .iter()
        .map(|page_id| format!("({page_id})"))
        .collect::<Vec<_>>()
        .join(", ");
    let txn = ctx.transaction();
    let statement = Statement::from_string(
        txn.get_database_backend(),
        format!(
            "WITH input(page_id) AS (VALUES {values}) \
             SELECT input.page_id AS parent_page_id, count(child.page_id) AS child_count \
             FROM input \
             LEFT JOIN page_parent ON page_parent.parent_page_id = input.page_id \
             LEFT JOIN page child ON child.page_id = page_parent.child_page_id \
                 AND child.deleted_at IS NULL \
             GROUP BY input.page_id",
        ),
    );

    let rows = ChildCountRow::find_by_statement(statement)
        .all(txn)
        .await
        .or_raise(make_error)?;

    Ok(rows
        .into_iter()
        .map(|row| (row.parent_page_id, row.child_count.max(0) as u64))
        .collect())
}

/// The parent full names of the given result rows, keyed by child page ID.
///
/// Rows without exactly one live parent are absent from the map rather than
/// present with an empty value.
pub(super) async fn load_list_pages_parent_fullnames(
    ctx: &ServiceContext<'_>,
    pages: &[FoundPageRow],
) -> Result<BTreeMap<i64, String>> {
    #[derive(FromQueryResult, Debug)]
    struct ParentRow {
        child_page_id: i64,
        parent_slug: String,
    }

    let page_ids = pages
        .iter()
        .map(|page| page.page_id)
        .collect::<BTreeSet<_>>();
    if page_ids.is_empty() {
        return Ok(BTreeMap::new());
    }

    let make_error = || {
        Error::new(
            "failed to load parent page names for ListPages render",
            ErrorType::Render,
        )
    };
    let values = page_ids
        .iter()
        .map(|page_id| format!("({page_id})"))
        .collect::<Vec<_>>()
        .join(", ");
    let txn = ctx.transaction();
    let statement = Statement::from_string(
        txn.get_database_backend(),
        format!(
            "WITH input(page_id) AS (VALUES {values}) \
             SELECT page_parent.child_page_id, page.slug AS parent_slug \
             FROM input \
             JOIN page_parent ON page_parent.child_page_id = input.page_id \
             JOIN page ON page.page_id = page_parent.parent_page_id \
             WHERE page.deleted_at IS NULL",
        ),
    );

    let rows = ParentRow::find_by_statement(statement)
        .all(txn)
        .await
        .or_raise(make_error)?;

    Ok(collapse_parent_rows(rows.into_iter().map(
        |ParentRow {
             child_page_id,
             parent_slug,
         }| (child_page_id, parent_slug),
    )))
}

/// Keeps only the children that resolved to exactly one live parent.
fn collapse_parent_rows(
    rows: impl Iterator<Item = (i64, String)>,
) -> BTreeMap<i64, String> {
    let mut parents = BTreeMap::<i64, Option<String>>::new();
    for (child_page_id, parent_slug) in rows {
        parents
            .entry(child_page_id)
            // A second live parent has no evidenced Wikidot rendering, so the
            // entry collapses to unresolved instead of picking one of them.
            .and_modify(|slot| *slot = None)
            .or_insert(Some(parent_slug));
    }

    parents
        .into_iter()
        .filter_map(|(child_page_id, parent_slug)| Some((child_page_id, parent_slug?)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collapse(rows: Vec<(i64, &str)>) -> BTreeMap<i64, String> {
        collapse_parent_rows(
            rows.into_iter().map(|(child_page_id, parent_slug)| {
                (child_page_id, parent_slug.to_owned())
            }),
        )
    }

    #[test]
    fn resolves_only_rows_with_exactly_one_live_parent() {
        let resolved = collapse(vec![
            (1, "component:offset-timeline"),
            (2, "component:offset-timeline"),
            (3, "first:parent"),
            (3, "second:parent"),
        ]);

        assert_eq!(
            resolved,
            BTreeMap::from([
                (1, "component:offset-timeline".to_owned()),
                (2, "component:offset-timeline".to_owned()),
            ]),
            "a row with two live parents has no evidenced parent full name",
        );
        assert!(
            !resolved.contains_key(&4),
            "a row with no live parent stays unresolved rather than empty",
        );
    }
}
