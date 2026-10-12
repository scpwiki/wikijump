/*
 * services/page_query/service/relational_filtering.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Page-query filters that resolve relationships through auxiliary tables.

use super::{
    AuthorSelector, ColumnTrait, Condition, ConnectionType, Expr, ExprTrait,
    PageConnection, PageParent, PageParentSelector, PageService, Query, Reference,
    Result, ServiceContext, SimpleExpr, current_parent_ids,
    normalize_wikidot_author_name, page, page_connection, page_parent,
    postgres_bind_placeholders, wikidot_author_name_sql,
};

pub(super) async fn page_parent_condition(
    ctx: &ServiceContext<'_>,
    current_site_id: i64,
    queried_site_id: i64,
    current_page_id: i64,
    page_parent: PageParentSelector<'_>,
) -> Result<Option<Condition>> {
    if matches!(page_parent, PageParentSelector::All) {
        return Ok(None);
    }
    let mut condition = Condition::all();

    match page_parent {
        PageParentSelector::All => unreachable!(),
        PageParentSelector::NoParent => {
            condition = condition.add(
                page::Column::PageId.not_in_subquery(
                    Query::select()
                        .column(page_parent::Column::ChildPageId)
                        .from(PageParent)
                        .to_owned(),
                ),
            );
        }
        PageParentSelector::SameParents => {
            condition = condition.add(
                page::Column::PageId.in_subquery(
                    Query::select()
                        .column(page_parent::Column::ChildPageId)
                        .from(PageParent)
                        .and_where(
                            page_parent::Column::ParentPageId.is_in(
                                current_parent_ids(ctx, current_site_id, current_page_id)
                                    .await?,
                            ),
                        )
                        .to_owned(),
                ),
            );
        }
        PageParentSelector::DifferentParents => {
            condition = condition
                .add(
                    page::Column::PageId.in_subquery(
                        Query::select()
                            .column(page_parent::Column::ChildPageId)
                            .from(PageParent)
                            .to_owned(),
                    ),
                )
                .add(
                    page::Column::PageId.not_in_subquery(
                        Query::select()
                            .column(page_parent::Column::ChildPageId)
                            .from(PageParent)
                            .and_where(
                                page_parent::Column::ParentPageId.is_in(
                                    current_parent_ids(
                                        ctx,
                                        current_site_id,
                                        current_page_id,
                                    )
                                    .await?,
                                ),
                            )
                            .to_owned(),
                    ),
                );
        }
        PageParentSelector::ChildOf => {
            condition = condition.add(
                page::Column::PageId.in_subquery(
                    Query::select()
                        .column(page_parent::Column::ChildPageId)
                        .from(PageParent)
                        .and_where(page_parent::Column::ParentPageId.eq(current_page_id))
                        .to_owned(),
                ),
            );
        }
        PageParentSelector::HasParents(parents) => {
            // Wikidot's parent selector is any-of rather than all-of.
            let parent_ids = PageService::get_pages(ctx, queried_site_id, parents)
                .await?
                .into_iter()
                .map(|page| page.page_id);
            condition = condition.add(
                page::Column::PageId.in_subquery(
                    Query::select()
                        .column(page_parent::Column::ChildPageId)
                        .from(PageParent)
                        .and_where(page_parent::Column::ParentPageId.is_in(parent_ids))
                        .to_owned(),
                ),
            );
        }
    }

    Ok(Some(condition))
}

pub(super) fn author_condition(author: AuthorSelector<'_>) -> Option<Condition> {
    if matches!(author, AuthorSelector::All) {
        return None;
    }
    let mut condition = Condition::all();

    match author {
        AuthorSelector::All => unreachable!(),
        AuthorSelector::None => {
            condition = condition.add(SimpleExpr::Custom("FALSE".into()));
        }
        AuthorSelector::Any {
            user_ids,
            wikidot_snapshot_names,
        } => {
            let normalized_snapshot_names = wikidot_snapshot_names
                .iter()
                .map(|name| normalize_wikidot_author_name(name))
                .filter(|name| !name.is_empty())
                .collect::<Vec<_>>();
            let mut author_condition = Condition::any();
            let mut has_author_condition = false;

            if !user_ids.is_empty() {
                let placeholders = postgres_bind_placeholders(user_ids.len());
                author_condition = author_condition.add(Expr::cust_with_values(
                    format!(
                        "EXISTS (SELECT 1 FROM page_revision pr WHERE pr.page_id = page.page_id AND pr.user_id IN ({placeholders}) AND pr.revision_id = (SELECT pr2.revision_id FROM page_revision pr2 WHERE pr2.page_id = page.page_id ORDER BY pr2.revision_number ASC, pr2.revision_id ASC LIMIT 1))"
                    ),
                    user_ids.iter().copied(),
                ));
                has_author_condition = true;
            }

            if !normalized_snapshot_names.is_empty() {
                let placeholders =
                    postgres_bind_placeholders(normalized_snapshot_names.len());
                let normalized_name_sql =
                    wikidot_author_name_sql("snapshot.created_by_name");
                author_condition = author_condition.add(Expr::cust_with_values(
                    format!(
                        "EXISTS (SELECT 1 FROM wikidot_page_snapshot snapshot WHERE snapshot.page_id = page.page_id AND {normalized_name_sql} IN ({placeholders}))"
                    ),
                    normalized_snapshot_names,
                ));
                has_author_condition = true;
            }

            if has_author_condition {
                condition = condition.add(author_condition);
            } else {
                condition = condition.add(SimpleExpr::Custom("FALSE".into()));
            }
        }
        AuthorSelector::NotAny {
            user_ids,
            wikidot_snapshot_names,
        } => {
            let normalized_snapshot_names = wikidot_snapshot_names
                .iter()
                .map(|name| normalize_wikidot_author_name(name))
                .filter(|name| !name.is_empty())
                .collect::<Vec<_>>();

            if user_ids.is_empty() && normalized_snapshot_names.is_empty() {
                return None;
            }

            if !user_ids.is_empty() {
                let placeholders = postgres_bind_placeholders(user_ids.len());
                condition = condition.add(Expr::cust_with_values(
                    format!(
                        "NOT EXISTS (SELECT 1 FROM page_revision pr WHERE pr.page_id = page.page_id AND pr.user_id IN ({placeholders}) AND pr.revision_id = (SELECT pr2.revision_id FROM page_revision pr2 WHERE pr2.page_id = page.page_id ORDER BY pr2.revision_number ASC, pr2.revision_id ASC LIMIT 1))"
                    ),
                    user_ids.iter().copied(),
                ));
            }

            if !normalized_snapshot_names.is_empty() {
                let placeholders =
                    postgres_bind_placeholders(normalized_snapshot_names.len());
                let normalized_name_sql =
                    wikidot_author_name_sql("snapshot.created_by_name");
                condition = condition.add(Expr::cust_with_values(
                    format!(
                        "NOT EXISTS (SELECT 1 FROM wikidot_page_snapshot snapshot WHERE snapshot.page_id = page.page_id AND {normalized_name_sql} IN ({placeholders}))"
                    ),
                    normalized_snapshot_names,
                ));
            }
        }
    }

    Some(condition)
}

pub(super) async fn outgoing_link_condition(
    ctx: &ServiceContext<'_>,
    queried_site_id: i64,
    contains_outgoing_links: &[Reference<'_>],
) -> Result<Option<Condition>> {
    if contains_outgoing_links.is_empty() {
        return Ok(None);
    }

    let mut condition = Condition::all();
    let incoming_ids =
        PageService::get_pages(ctx, queried_site_id, contains_outgoing_links)
            .await?
            .into_iter()
            .map(|page| page.page_id);
    condition = condition.add(
        page::Column::PageId.in_subquery(
            Query::select()
                .column(page_connection::Column::FromPageId)
                .from(PageConnection)
                .and_where(page_connection::Column::ToPageId.is_in(incoming_ids))
                .and_where(
                    page_connection::Column::ConnectionType.eq(ConnectionType::Link),
                )
                .and_where(
                    Expr::col(page_connection::Column::FromPageId)
                        .ne(Expr::col(page_connection::Column::ToPageId)),
                )
                .to_owned(),
        ),
    );

    Ok(Some(condition))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unconstrained_author_selectors_do_not_add_conditions() {
        assert!(author_condition(AuthorSelector::All).is_none());
        assert!(
            author_condition(AuthorSelector::NotAny {
                user_ids: &[],
                wikidot_snapshot_names: &[],
            })
            .is_none()
        );
    }
}
