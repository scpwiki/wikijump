/*
 * services/page_query/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2024 Wikijump Team
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
use crate::services::{PageService, ParentService};
use sea_query::{Expr, Query};
use std::convert::Infallible;

#[derive(Debug)]
pub struct PageQueryService;

impl PageQueryService {
    pub async fn execute(
        ctx: &ServiceContext,
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
            data_form_fields,
            order,
            pagination,
            variables,
        }: PageQuery<'_>,
    ) -> Result<Infallible> {
        info!("Building ListPages query from specification");

        let txn = ctx.seaorm_transaction();
        let mut condition = Condition::all();

        // Site ID
        //
        // The site to query from. If not specified, then this is the current site.
        let queried_site_id = queried_site_id.unwrap_or(current_site_id);
        condition = condition.add(page::Column::SiteId.eq(queried_site_id));
        debug!("Selecting pages from site ID: {queried_site_id}");

        // Page Type
        // TODO track https://github.com/SeaQL/sea-orm/issues/1746
        let hidden_condition = page::Column::Slug.starts_with("_");
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
                .await?
                .into_iter()
                .map(|parent| parent.parent_page_id)
            };
        }

        let page_parent_condition = match page_parent {
            // Pages with no parents.
            // This means that there should be no rows in `page_parent`
            // where they are the child page.
            PageParentSelector::NoParent => {
                debug!("Selecting pages with no parents");

                page::Column::PageId.not_in_subquery(
                    Query::select()
                        .column(page_parent::Column::ChildPageId)
                        .from(PageParent)
                        .to_owned(),
                )
            }

            // Pages which are siblings of the current page,
            // i.e., they share parents in common with the current page.
            PageParentSelector::SameParents => {
                debug!("Selecting pages are siblings under the given parents");

                page::Column::PageId.in_subquery(
                    Query::select()
                        .column(page_parent::Column::ChildPageId)
                        .from(PageParent)
                        .and_where(
                            page_parent::Column::ParentPageId.is_in(get_parents!()),
                        )
                        .to_owned(),
                )
            }

            // Pages which are not siblings of the current page,
            // i.e., they do not share any parents with the current page.
            PageParentSelector::DifferentParents => {
                debug!("Selecting pages which are not siblings under the given parents",);

                let parents = ParentService::get_parents(
                    ctx,
                    current_site_id,
                    Reference::Id(current_page_id),
                )
                .await?
                .into_iter()
                .map(|parent| parent.parent_page_id);

                page::Column::PageId.in_subquery(
                    Query::select()
                        .column(page_parent::Column::ChildPageId)
                        .from(PageParent)
                        .and_where(
                            page_parent::Column::ParentPageId.is_not_in(get_parents!()),
                        )
                        .to_owned(),
                )
            }

            // Pages which are children of the current page.
            PageParentSelector::ChildOf => {
                debug!("Selecting pages which are children of the current page",);

                page::Column::PageId.in_subquery(
                    Query::select()
                        .column(page_parent::Column::ChildPageId)
                        .from(PageParent)
                        .and_where(page_parent::Column::ParentPageId.eq(current_page_id))
                        .to_owned(),
                )
            }

            // Pages with any of the specified parents.
            // TODO: Possibly allow either *any* or *all* of specified parents
            //       rather than only any, in the future.
            PageParentSelector::HasParents(parents) => {
                debug!("Selecting on pages which have one of the given as parents",);

                let parent_ids = PageService::get_pages(ctx, queried_site_id, parents)
                    .await?
                    .into_iter()
                    .map(|page| page.page_id);

                page::Column::PageId.in_subquery(
                    Query::select()
                        .column(page_parent::Column::ChildPageId)
                        .from(PageParent)
                        .and_where(page_parent::Column::ParentPageId.is_in(parent_ids))
                        .to_owned(),
                )
            }
        };
        condition = condition.add(page_parent_condition);

        // Slug
        if let Some(slug) = slug {
            let slug = slug.as_ref();
            debug!("Filtering based on slug {slug}");
            condition = condition.add(page::Column::Slug.eq(slug));
        }

        // Contains-link
        //
        // Selects pages that have an outgoing link (`from_page_id`)
        // to a specified page (`to_page_id`).
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
                        .await?
                        .into_iter()
                        .map(|page| page.page_id);

                        page_connection::Column::ToPageId.is_in(incoming_ids)
                    })
                    .to_owned(),
            ),
        );

        // Tag filtering
        // TODO requires joining with most recent revision

        // Build the final query
        let mut query = Page::find().filter(condition);

        // Add necessary joins
        macro_rules! join_revision {
            () => {
                query = query.join(JoinType::Join, page::Relation::PageRevision.def());
            };
        }
        macro_rules! join_text {
            () => {
                query = query.join(JoinType::Join, page_revision::Relation::Text1.def());
            };
        }
        // TODO other joins

        // Add on at the query-level (ORDER BY, LIMIT)
        {
            use sea_orm::query::Order;
            use sea_query::{func::Func, SimpleExpr};

            let OrderBySelector {
                property,
                ascending,
            } = order.unwrap_or_default();

            debug!(
                "Ordering ListPages using {:?} (ascending: {})",
                property, ascending,
            );

            let order = if ascending { Order::Asc } else { Order::Desc };

            // TODO: implement missing properties. these require subqueries or joins or something
            match property {
                OrderProperty::PageSlug => {
                    // idk how to do this, we need to strip off the category part somehow
                    // PgExpr::matches?
                    error!("Ordering by page slug (no category), not yet implemented",);
                    todo!() // TODO
                }
                OrderProperty::FullSlug => {
                    debug!("Ordering by page slug (with category");
                    query = query.order_by(page::Column::Slug, order);
                }
                OrderProperty::Title => {
                    error!("Ordering by title, not yet implemented");
                    join_revision!();
                    query = query.order_by(page_revision::Column::Title, order);
                }
                OrderProperty::AltTitle => {
                    error!("Ordering by alt title, not yet implemented");
                    join_revision!();
                    query = query.order_by(page_revision::Column::AltTitle, order);
                }
                OrderProperty::CreatedBy => {
                    error!("Ordering by author, not yet implemented");
                    todo!() // TODO
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
                    error!("Ordering by page size, not yet implemented");
                    join_revision!();
                    join_text!();
                    let col = Expr::col(text::Column::Contents);
                    let expr = SimpleExpr::FunctionCall(Func::char_length(col));
                    query = query.order_by(expr, order);
                }
                OrderProperty::Score => {
                    error!("Ordering by score, not yet implemented");
                    todo!() // TODO
                }
                OrderProperty::Votes => {
                    error!("Ordering by vote count, not yet implemented");
                    todo!() // TODO
                }
                OrderProperty::Revisions => {
                    error!("Ordering by revision count, not yet implemented");
                    todo!() // TODO
                }
                OrderProperty::Comments => {
                    error!("Ordering by comment count, not yet implemented");
                    todo!() // TODO
                }
                OrderProperty::Random => {
                    debug!("Ordering by random value");
                    let expr = SimpleExpr::FunctionCall(Func::random());
                    query = query.order_by(expr, order);
                }
                OrderProperty::DataFormFieldName => {
                    error!("Ordering by data form field, not yet implemented");
                    todo!() // TODO
                }
            };
        }

        if let Some(limit) = pagination.limit {
            debug!("Limiting ListPages to a maximum of {limit} pages total");
            query = query.limit(limit);
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
        let result = query.all(txn).await?;

        // TODO implement query construction
        todo!()
    }
}
