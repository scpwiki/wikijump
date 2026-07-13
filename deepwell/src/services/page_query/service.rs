/*
 * services/page_query/service.rs
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

#![allow(dead_code, unused_variables)] // TEMP

use super::prelude::*;
use crate::models::page::{self, Entity as Page};
use crate::models::page_category::{self, Entity as PageCategory};
use crate::models::page_connection::{self, Entity as PageConnection};
use crate::models::page_parent::{self, Entity as PageParent};
use crate::models::{page_revision, text};
use crate::services::score::ScoreValue;
use crate::services::{PageService, ParentService, ScoreService};
use sea_query::extension::postgres::PgBinOper;
use sea_query::{Expr, Query, SimpleExpr};
use std::cmp::Ordering;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct PageQueryService;

impl PageQueryService {
    pub async fn find(
        ctx: &ServiceContext<'_>,
        query: PageQuery<'_>,
    ) -> Result<FoundPages> {
        Ok(Self::find_with_metadata(ctx, query).await?.pages)
    }

    pub async fn find_with_metadata(
        ctx: &ServiceContext<'_>,
        PageQuery {
            current_page_id,
            current_site_id,
            queried_site_id,
            page_type,
            categories:
                CategoriesSelector {
                    included_categories,
                    excluded_categories,
                },
            tags:
                TagCondition {
                    any_present: any_tags,
                    all_present: all_tags,
                    none_present: no_tags,
                },
            page_parent,
            contains_outgoing_links,
            creation_date,
            update_date,
            author,
            score,
            votes,
            offset,
            range,
            name,
            slug,
            slugs,
            data_form_fields,
            order,
            candidate_limit,
            pagination,
            variables,
            fields,
        }: PageQuery<'_>,
    ) -> Result<PageQueryResultEnvelope> {
        info!("Building ListPages query from specification");

        let make_error =
            || Error::new("failed to create ListPages query", ErrorType::PageQuery);

        let txn = ctx.transaction();
        let mut condition = Condition::all();

        // Site ID
        //
        // The site to query from. If not specified, then this is the current site.
        let queried_site_id = queried_site_id.unwrap_or(current_site_id);
        condition = condition.add(page::Column::SiteId.eq(queried_site_id));
        debug!("Selecting pages from site ID: {queried_site_id}");

        // Page Type
        let hidden_condition = Expr::cust_with_expr(
            r#"regexp_replace($1, '^.*:', '') LIKE '\_%' ESCAPE '\'"#,
            Expr::col((Page, page::Column::Slug)),
        );
        match page_type {
            PageTypeSelector::Hidden => {
                // Hidden pages are any which have slugs that start with '_'.
                debug!("Selecting page slugs starting with '_'");
                condition = condition.add(hidden_condition);
            }
            PageTypeSelector::Normal => {
                // Normal pages are anything not in the above category.
                debug!("Selecting page slugs not starting with '_'");
                condition = condition.add(hidden_condition.not());
            }
            PageTypeSelector::All => {
                // If we're getting everything, then do nothing.
                debug!("Selecting all page slugs, normal or hidden");
            }
        }

        // Categories (included and excluded)
        macro_rules! cat_slugs {
            ($list:expr) => {
                $list.iter().map(|c| c.as_ref())
            };
        }

        let page_category_condition = match included_categories {
            // If all categories are selected (using an asterisk or by only specifying excluded categories),
            // then filter only by site_id and exclude the specified excluded categories.
            IncludedCategories::All => {
                debug!("Selecting all categories with exclusions");

                page::Column::PageCategoryId.in_subquery(
                    Query::select()
                        .column(page_category::Column::CategoryId)
                        .from(PageCategory)
                        .and_where(page_category::Column::SiteId.eq(queried_site_id))
                        .and_where(
                            page_category::Column::Slug
                                .is_not_in(cat_slugs!(excluded_categories)),
                        )
                        .to_owned(),
                )
            }

            // If a specific list of categories is provided, filter by site_id, inclusion in the
            // specified included categories, and exclude the specified excluded categories.
            //
            // NOTE: Exclusion can only have an effect in this query if it is *also* included.
            //       Although by definition this is the same as not including the category in the
            //       included categories to begin with, it is still accounted for to preserve
            //       backwards-compatibility with poorly-constructed ListPages modules.
            IncludedCategories::List(included_categories) => {
                debug!("Selecting included categories only");

                page::Column::PageCategoryId.in_subquery(
                    Query::select()
                        .column(page_category::Column::CategoryId)
                        .from(PageCategory)
                        .and_where(page_category::Column::SiteId.eq(queried_site_id))
                        .and_where(
                            page_category::Column::Slug
                                .is_in(cat_slugs!(included_categories)),
                        )
                        .and_where(
                            page_category::Column::Slug
                                .is_not_in(cat_slugs!(excluded_categories)),
                        )
                        .to_owned(),
                )
            }
        };
        condition = condition.add(page_category_condition);

        // Page Parents
        //
        // Adds constraints based on the presence of parent pages.

        // Convenience macro to pull a list of page IDs which are parents
        // of the current page.
        //
        // In the places where this is used, this could be implemented
        // as a subquery, meaning:
        //
        // SELECT child_page_id FROM page_parent
        // WHERE parent_page_id IN (
        //     SELECT parent_page_id FROM page_parent
        //     WHERE child_page_id = $0
        // )
        //
        // However looking at the query plan, this would be implemented
        // as a self-JOIN, and involve a full sequential scan. So querying
        // the list of parents ahead of time is faster.
        macro_rules! get_parents {
            () => {
                ParentService::get_parents(
                    ctx,
                    current_site_id,
                    Reference::Id(current_page_id),
                )
                .await
                .or_raise(make_error)?
                .into_iter()
                .map(|parent| parent.parent_page_id)
            };
        }

        let page_parent_condition = match page_parent {
            PageParentSelector::All => None,

            // Pages with no parents.
            // This means that there should be no rows in `page_parent`
            // where they are the child page.
            PageParentSelector::NoParent => {
                debug!("Selecting pages with no parents");

                Some(
                    page::Column::PageId.not_in_subquery(
                        Query::select()
                            .column(page_parent::Column::ChildPageId)
                            .from(PageParent)
                            .to_owned(),
                    ),
                )
            }

            // Pages which are siblings of the current page,
            // i.e., they share parents in common with the current page.
            PageParentSelector::SameParents => {
                debug!("Selecting pages are siblings under the given parents");

                Some(
                    page::Column::PageId.in_subquery(
                        Query::select()
                            .column(page_parent::Column::ChildPageId)
                            .from(PageParent)
                            .and_where(
                                page_parent::Column::ParentPageId.is_in(get_parents!()),
                            )
                            .to_owned(),
                    ),
                )
            }

            // Pages which are not siblings of the current page,
            // i.e., they do not share any parents with the current page.
            PageParentSelector::DifferentParents => {
                debug!("Selecting pages which are not siblings under the given parents");

                Some(
                    page::Column::PageId.not_in_subquery(
                        Query::select()
                            .column(page_parent::Column::ChildPageId)
                            .from(PageParent)
                            .and_where(
                                page_parent::Column::ParentPageId.is_in(get_parents!()),
                            )
                            .to_owned(),
                    ),
                )
            }

            // Pages which are children of the current page.
            PageParentSelector::ChildOf => {
                debug!("Selecting pages which are children of the current page");

                Some(
                    page::Column::PageId.in_subquery(
                        Query::select()
                            .column(page_parent::Column::ChildPageId)
                            .from(PageParent)
                            .and_where(
                                page_parent::Column::ParentPageId.eq(current_page_id),
                            )
                            .to_owned(),
                    ),
                )
            }

            // Pages with any of the specified parents.
            // TODO: Possibly allow either *any* or *all* of specified parents
            //       rather than only any, in the future.
            PageParentSelector::HasParents(parents) => {
                debug!("Selecting on pages which have one of the given as parents");

                let parent_ids = PageService::get_pages(ctx, queried_site_id, parents)
                    .await
                    .or_raise(make_error)?
                    .into_iter()
                    .map(|page| page.page_id);

                Some(
                    page::Column::PageId.in_subquery(
                        Query::select()
                            .column(page_parent::Column::ChildPageId)
                            .from(PageParent)
                            .and_where(
                                page_parent::Column::ParentPageId.is_in(parent_ids),
                            )
                            .to_owned(),
                    ),
                )
            }
        };
        if let Some(page_parent_condition) = page_parent_condition {
            condition = condition.add(page_parent_condition);
        }

        // Slug
        if let Some(slug) = slug {
            if !slugs.is_empty() {
                return Err(Error::new(
                    "page query cannot combine singular and plural slug selectors",
                    ErrorType::PageQuery,
                )
                .into());
            }
            let slug = slug.as_ref();
            debug!("Filtering based on slug {slug}");
            condition = condition.add(page::Column::Slug.eq(slug));
        }
        if !slugs.is_empty() {
            debug!("Filtering based on {} exact slugs", slugs.len());
            condition = condition
                .add(page::Column::Slug.is_in(slugs.iter().map(|slug| slug.as_ref())));
        }

        // Initial page author. Local pages use the user ID on their earliest available revision. Corpus imports intentionally keep the Wikidot display name in wikidot_page_snapshot instead of fabricating local users, so the two representations are combined with OR semantics.
        match author {
            AuthorSelector::All => {}
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
                    author_condition = author_condition.add(Expr::cust_with_values(
                        format!(
                            "EXISTS (SELECT 1 FROM wikidot_page_snapshot snapshot WHERE snapshot.page_id = page.page_id AND replace(replace(lower(btrim(snapshot.created_by_name)), '_', '-'), ' ', '-') IN ({placeholders}))"
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
        }

        // Contains-link
        //
        // Selects pages that have an outgoing link (`from_page_id`)
        // to a specified page (`to_page_id`). An empty selector means
        // no link constraint; adding an empty subquery here makes every
        // ordinary ListPages query return no rows.
        if !contains_outgoing_links.is_empty() {
            condition = condition.add(
                page::Column::PageId.in_subquery(
                    Query::select()
                        .column(page_connection::Column::FromPageId)
                        .from(PageConnection)
                        .and_where({
                            let incoming_ids = PageService::get_pages(
                                ctx,
                                queried_site_id,
                                contains_outgoing_links,
                            )
                            .await
                            .or_raise(make_error)?
                            .into_iter()
                            .map(|page| page.page_id);

                            page_connection::Column::ToPageId.is_in(incoming_ids)
                        })
                        .to_owned(),
                ),
            );
        }

        // Build the final query
        let mut query = Page::find()
            .filter(page::Column::DeletedAt.is_null())
            .filter(condition);
        let order = order.unwrap_or_default();
        let needs_tag_filter =
            !all_tags.is_empty() || !any_tags.is_empty() || !no_tags.is_empty();
        let needs_revision_join = needs_tag_filter
            || matches!(
                order.property,
                OrderProperty::Title | OrderProperty::AltTitle | OrderProperty::Size
            );
        if needs_tag_filter {
            query = query.join(JoinType::Join, page::Relation::PageRevision.def());
        } else if needs_revision_join {
            query = query.join(JoinType::LeftJoin, page::Relation::PageRevision.def());
        }

        // Add necessary joins
        macro_rules! join_text {
            () => {
                query = query.join(
                    if needs_tag_filter {
                        JoinType::Join
                    } else {
                        JoinType::LeftJoin
                    },
                    page_revision::Relation::Text1.def(),
                );
            };
        }
        // TODO other joins

        // Tag filtering. Tags live on the current page revision, so this joins through
        // page.latest_revision_id -> page_revision.revision_id before applying array predicates.
        for tag in all_tags {
            query = query.filter(
                Expr::col(page_revision::Column::Tags)
                    .binary(PgBinOper::Contains, Expr::val(vec![tag.to_string()])),
            );
        }

        if !any_tags.is_empty() {
            query = query.filter(
                Expr::col(page_revision::Column::Tags).binary(
                    PgBinOper::Overlap,
                    Expr::val(
                        any_tags
                            .iter()
                            .map(|tag| tag.to_string())
                            .collect::<Vec<_>>(),
                    ),
                ),
            );
        }

        for tag in no_tags {
            query = query.filter(
                Expr::col(page_revision::Column::Tags)
                    .binary(PgBinOper::Contains, Expr::val(vec![tag.to_string()]))
                    .not(),
            );
        }

        // Add on at the query-level (ORDER BY, LIMIT)
        let score_order = matches!(order.property, OrderProperty::Score);
        {
            use sea_orm::query::Order;
            use sea_query::SimpleExpr;
            use sea_query::func::Func;

            let OrderBySelector {
                property,
                ascending,
            } = order;

            debug!("Ordering ListPages using {property:?} (ascending: {ascending})");

            let order = if ascending { Order::Asc } else { Order::Desc };

            match property {
                OrderProperty::PageSlug => {
                    debug!("Ordering by page slug (no category)");
                    let expr = Expr::cust_with_expr(
                        "regexp_replace(regexp_replace($1, '^.*:', ''), '[^[:alnum:]]', '', 'g')",
                        Expr::col((Page, page::Column::Slug)),
                    );
                    query = query
                        .order_by(expr, order.clone())
                        .order_by(Expr::col((Page, page::Column::Slug)), order);
                }
                OrderProperty::FullSlug => {
                    debug!("Ordering by page slug (with category)");
                    query = query.order_by(page::Column::Slug, order);
                }
                OrderProperty::Title => {
                    debug!("Ordering by title");
                    query = query.order_by(page_revision::Column::Title, order);
                }
                OrderProperty::AltTitle => {
                    debug!("Ordering by alt title");
                    query = query.order_by(page_revision::Column::AltTitle, order);
                }
                OrderProperty::CreatedBy => {
                    debug!("Ordering by initial page author");
                    let expr = SimpleExpr::Custom(
                        "(SELECT pr.user_id FROM page_revision pr WHERE pr.page_id = page.page_id ORDER BY pr.revision_number ASC, pr.revision_id ASC LIMIT 1)".into(),
                    );
                    query = query.order_by(expr, order);
                }
                OrderProperty::CreatedAt => {
                    debug!("Ordering by page creation timestamp");
                    query = query.order_by(page::Column::CreatedAt, order);
                }
                OrderProperty::UpdatedAt => {
                    debug!("Ordering by page last update timestamp");
                    query = query.order_by(page::Column::UpdatedAt, order);
                }
                OrderProperty::Size => {
                    debug!("Ordering by page size");
                    join_text!();
                    let col = Expr::col(text::Column::Contents);
                    let expr = SimpleExpr::FunctionCall(Func::char_length(col));
                    query = query.order_by(expr, order);
                }
                OrderProperty::Score => {
                    debug!("Ordering by page score after ScoreService evaluation");
                }
                OrderProperty::Votes => {
                    debug!("Ordering by page vote count");
                    let expr = SimpleExpr::Custom(
                        "COALESCE((SELECT COUNT(*) FROM page_vote pv WHERE pv.page_id = page.page_id AND pv.deleted_at IS NULL AND pv.disabled_at IS NULL), 0)".into(),
                    );
                    query = query.order_by(expr, order);
                }
                OrderProperty::Revisions => {
                    debug!("Ordering by page revision count");
                    let expr = SimpleExpr::Custom(
                        "COALESCE((SELECT COUNT(*) FROM page_revision pr WHERE pr.page_id = page.page_id), 0)".into(),
                    );
                    query = query.order_by(expr, order);
                }
                OrderProperty::Comments => {
                    debug!("Ordering by forum comment count");
                    let expr = SimpleExpr::Custom(
                        "COALESCE((SELECT COUNT(*) FROM forum_post fp JOIN forum_thread ft ON fp.forum_thread_id = ft.forum_thread_id WHERE ft.page_id = page.page_id AND fp.deleted_at IS NULL AND ft.deleted_at IS NULL), 0)".into(),
                    );
                    query = query.order_by(expr, order);
                }
                OrderProperty::Random => {
                    debug!("Ordering by random value");
                    let expr = SimpleExpr::FunctionCall(Func::random());
                    query = query.order_by(expr, order);
                }
                OrderProperty::DataFormFieldName => {
                    debug!("Rejecting unsupported data form field ordering");
                    return Err(Error::new(
                        "ListPages data form field ordering is not implemented",
                        ErrorType::PageQuery,
                    )
                    .into());
                }
            };
            if !matches!(property, OrderProperty::Random | OrderProperty::Score) {
                if !matches!(property, OrderProperty::PageSlug | OrderProperty::FullSlug)
                {
                    query = query.order_by(page::Column::Slug, Order::Asc);
                }
                query = query.order_by(page::Column::PageId, Order::Asc);
            }
        }

        let filtering_deferred_to_rust = !data_form_fields.is_empty();
        let ordering_deferred_to_rust = score_order;
        let defer_offset_limit = ordering_deferred_to_rust || filtering_deferred_to_rust;
        let sql_limit_offset_applied =
            !defer_offset_limit && (offset > 0 || pagination.limit.is_some());
        if !defer_offset_limit {
            if offset > 0 {
                debug!("Offsetting ListPages by {offset} pages");
                query = query.offset(u64::from(offset));
            }
            if let Some(limit) = pagination.limit {
                debug!("Limiting ListPages to a maximum of {limit} pages total");
                query = query.limit(limit);
            }
        } else if !data_form_fields.is_empty()
            && let Some(candidate_limit) = candidate_limit
        {
            debug!(
                "Limiting ListPages data form candidate scan to {candidate_limit} pages"
            );
            query = query.limit(candidate_limit);
        }

        // TODO pagination
        //      the "reverse" field means that, for each page, it is reversed.
        //
        //      this does not affect the overall ORDER BY
        //      for instance, imagine we are selecting from the positive integers
        //      if the pagination limit is 5 and the order is ascending, but reverse = true,
        //      then this means we get pages like:
        //
        //      1. [ 4,  3,  2,  1,  0]
        //      2. [ 9,  8,  7,  6,  5]
        //      3. [14, 13, 12, 11, 10]

        // Execute it!
        let mut pages = query.all(txn).await.or_raise(make_error)?;
        let candidate_count = Some(pages.len());
        let cap_exceeded = filtering_deferred_to_rust
            && candidate_limit
                .and_then(|limit| usize::try_from(limit).ok())
                .is_some_and(|limit| pages.len() >= limit);
        if !data_form_fields.is_empty() {
            pages = filter_pages_by_data_form_fields(ctx, pages, data_form_fields)
                .await
                .or_raise(make_error)?;
        }

        debug!("Query returned {} pages, building FoundPages", pages.len());

        let mut page_ids = pages.iter().map(|page| page.page_id).collect::<Vec<_>>();
        let score_by_page_id: BTreeMap<i64, f32> =
            if (fields.score || score_order) && !page_ids.is_empty() {
                ScoreService::scores_bulk(ctx, &page_ids)
                    .await
                    .or_raise(make_error)?
                    .into_iter()
                    .map(|(page_id, score)| (page_id, score_to_f32(score)))
                    .collect()
            } else {
                BTreeMap::new()
            };

        if defer_offset_limit {
            if score_order {
                pages.sort_by(|left, right| {
                    let left_score =
                        score_by_page_id.get(&left.page_id).copied().unwrap_or(0.0);
                    let right_score =
                        score_by_page_id.get(&right.page_id).copied().unwrap_or(0.0);
                    let ordering = left_score
                        .partial_cmp(&right_score)
                        .unwrap_or(Ordering::Equal);
                    let ordering = if order.ascending {
                        ordering
                    } else {
                        ordering.reverse()
                    };
                    ordering
                        .then_with(|| left.slug.cmp(&right.slug))
                        .then_with(|| left.page_id.cmp(&right.page_id))
                });
            }
            if offset > 0 {
                let skip = (offset as usize).min(pages.len());
                pages.drain(..skip);
            }
            if let Some(limit) = pagination.limit {
                pages.truncate(limit.min(usize::MAX as u64) as usize);
            }
            page_ids = pages.iter().map(|page| page.page_id).collect();
        }

        let revision_fields_requested =
            fields.title || fields.alt_title || fields.tags || fields.updated_by;
        let revisions_by_id: BTreeMap<i64, page_revision::Model> =
            if revision_fields_requested {
                let revision_ids = pages
                    .iter()
                    .filter_map(|page| page.latest_revision_id)
                    .collect::<Vec<_>>();

                if revision_ids.is_empty() {
                    BTreeMap::new()
                } else {
                    page_revision::Entity::find()
                        .filter(page_revision::Column::RevisionId.is_in(revision_ids))
                        .all(txn)
                        .await
                        .or_raise(make_error)?
                        .into_iter()
                        .map(|revision| (revision.revision_id, revision))
                        .collect()
                }
            } else {
                BTreeMap::new()
            };

        let created_by_by_page_id: BTreeMap<i64, i64> =
            if fields.created_by && !page_ids.is_empty() {
                let mut created_by_by_page_id = BTreeMap::new();
                for (page_id, user_id) in page_revision::Entity::find()
                    .select_only()
                    .column(page_revision::Column::PageId)
                    .column(page_revision::Column::UserId)
                    .filter(page_revision::Column::PageId.is_in(page_ids.clone()))
                    .order_by_asc(page_revision::Column::PageId)
                    .order_by_asc(page_revision::Column::RevisionNumber)
                    .order_by_asc(page_revision::Column::RevisionId)
                    .into_tuple::<(i64, i64)>()
                    .all(txn)
                    .await
                    .or_raise(make_error)?
                {
                    created_by_by_page_id.entry(page_id).or_insert(user_id);
                }
                created_by_by_page_id
            } else {
                BTreeMap::new()
            };

        let rows = pages
            .into_iter()
            .map(|page| {
                let revision = page
                    .latest_revision_id
                    .and_then(|revision_id| revisions_by_id.get(&revision_id));

                FoundPageRow {
                    page_id: page.page_id,
                    site_id: page.site_id,
                    slug: if fields.slug { Some(page.slug) } else { None },
                    page_category_id: if fields.page_category_id {
                        Some(page.page_category_id)
                    } else {
                        None
                    },
                    page_revision_id: if fields.page_revision_id {
                        page.latest_revision_id
                    } else {
                        None
                    },
                    created_at: if fields.created_at {
                        Some(page.created_at)
                    } else {
                        None
                    },
                    updated_at: if fields.updated_at {
                        page.updated_at
                    } else {
                        None
                    },
                    title: if fields.title {
                        revision.map(|revision| revision.title.clone())
                    } else {
                        None
                    },
                    alt_title: if fields.alt_title {
                        revision.and_then(|revision| revision.alt_title.clone())
                    } else {
                        None
                    },
                    tags: if fields.tags {
                        revision.map(|revision| revision.tags.clone())
                    } else {
                        None
                    },
                    created_by: if fields.created_by {
                        created_by_by_page_id.get(&page.page_id).copied()
                    } else {
                        None
                    },
                    updated_by: if fields.updated_by {
                        revision.map(|revision| revision.user_id)
                    } else {
                        None
                    },
                    score: if fields.score {
                        score_by_page_id.get(&page.page_id).copied().or(Some(0.0))
                    } else {
                        None
                    },
                }
            })
            .collect();

        let pages = FoundPages { pages: rows };
        if filtering_deferred_to_rust || ordering_deferred_to_rust || cap_exceeded {
            return Ok(PageQueryResultEnvelope::deferred(
                pages,
                candidate_count,
                filtering_deferred_to_rust,
                ordering_deferred_to_rust,
                cap_exceeded,
            ));
        }

        let exact_count_safe =
            !filtering_deferred_to_rust && !ordering_deferred_to_rust && !cap_exceeded;
        Ok(PageQueryResultEnvelope {
            pages,
            metadata: PageQueryResultMetadata {
                candidate_count,
                cap_exceeded,
                sql_limit_offset_applied,
                filtering_deferred_to_rust,
                ordering_deferred_to_rust,
                exact_count_safe,
                unsupported_reason: None,
            },
        })
    }
}

fn postgres_bind_placeholders(count: usize) -> String {
    (1..=count)
        .map(|index| format!("${index}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn score_to_f32(score: ScoreValue) -> f32 {
    match score {
        ScoreValue::Integer(value) => value as f32,
        ScoreValue::Float(value) => value as f32,
    }
}

async fn filter_pages_by_data_form_fields(
    ctx: &ServiceContext<'_>,
    pages: Vec<page::Model>,
    selectors: &[DataFormSelector<'_>],
) -> Result<Vec<page::Model>> {
    let make_error = || {
        Error::new(
            "failed to filter ListPages data form selectors",
            ErrorType::PageQuery,
        )
    };
    let revision_ids = pages
        .iter()
        .filter_map(|page| page.latest_revision_id)
        .collect::<Vec<_>>();
    if revision_ids.is_empty() {
        return Ok(Vec::new());
    }

    let revisions_by_id = page_revision::Entity::find()
        .filter(page_revision::Column::RevisionId.is_in(revision_ids))
        .all(ctx.transaction())
        .await
        .or_raise(make_error)?
        .into_iter()
        .map(|revision| (revision.revision_id, revision.wikitext_hash))
        .collect::<BTreeMap<_, _>>();

    let hashes = revisions_by_id.values().cloned().collect::<Vec<_>>();
    let text_by_hash = text::Entity::find()
        .filter(text::Column::Hash.is_in(hashes))
        .all(ctx.transaction())
        .await
        .or_raise(make_error)?
        .into_iter()
        .map(|text| (text.hash, text.contents))
        .collect::<BTreeMap<_, _>>();

    Ok(pages
        .into_iter()
        .filter(|page| {
            let values = page
                .latest_revision_id
                .and_then(|revision_id| revisions_by_id.get(&revision_id))
                .and_then(|hash| text_by_hash.get(hash))
                .map(|wikitext| parse_static_wikidot_data_form_values(wikitext))
                .unwrap_or_default();

            static_wikidot_data_form_matches(&values, selectors)
        })
        .collect())
}
