/*
 * services/page_query/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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

use super::prelude::*;
use crate::models::page::{self, Entity as Page};
use crate::models::page_category::{self, Entity as PageCategory};
use crate::models::page_connection::{self, Entity as PageConnection};
use crate::models::page_parent::{self, Entity as PageParent};
use crate::services::{PageService, ParentService};
use sea_query::Query;
use void::Void;

#[derive(Debug)]
pub struct PageQueryService;

impl PageQueryService {
    pub async fn execute(
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
            data_form_fields,
            order,
            pagination,
            variables,
        }: PageQuery<'_>,
    ) -> Result<Void> {
        tide::log::info!("Building ListPages query from specification");

        let txn = ctx.transaction();
        let mut condition = Condition::all();

        // Site ID
        //
        // The site to query from. If not specified, then this is the current site.
        let queried_site_id = queried_site_id.unwrap_or(current_site_id);
        condition = condition.add(page::Column::SiteId.eq(queried_site_id));
        tide::log::debug!("Selecting pages from site ID: {queried_site_id}");

        // Page Type
        // TODO track https://github.com/SeaQL/sea-orm/issues/1746
        let hidden_condition = page::Column::Slug.starts_with("_");
        match page_type {
            PageTypeSelector::Hidden => {
                // Hidden pages are any which have slugs that start with '_'.
                tide::log::debug!("Selecting page slugs starting with '_'");
                condition = condition.add(hidden_condition);
            }
            PageTypeSelector::Normal => {
                // Normal pages are anything not in the above category.
                tide::log::debug!("Selecting page slugs not starting with '_'");
                condition = condition.add(hidden_condition.not());
            }
            PageTypeSelector::All => {
                // If we're getting everything, then do nothing.
                tide::log::debug!("Selecting all page slugs, normal or hidden");
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
                tide::log::debug!("Selecting all categories with exclusions");

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
                tide::log::debug!("Selecting included categories only");

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
                tide::log::debug!("Selecting pages with no parents");

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
                tide::log::debug!("Selecting pages are siblings under the given parents");

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
                tide::log::debug!(
                    "Selecting pages which are not siblings under the given parents",
                );

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
                tide::log::debug!(
                    "Selecting pages which are children of the current page",
                );

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
                tide::log::debug!(
                    "Selecting on pages which have one of the given as parents",
                );

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
            tide::log::debug!("Filtering based on slug {slug}");
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
        // TODO

        // Execute the query!
        // TODO

        // TODO implement query construction
        todo!()
    }
}
