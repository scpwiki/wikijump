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
use crate::models::page::{self, Model as PageModel};
use void::Void;

#[derive(Debug)]
pub struct PageQueryService;

impl PageQueryService {
    pub async fn build_query(
        ctx: &ServiceContext<'_>,
        PageQuery {
            current_page_id,
            queried_site_id,
            page_type,
            categories,
            tags,
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
        // This value should already be filled in before calling this method (i.e. this
        // field is *not* Option).
        condition = condition.add(page::Column::SiteId.eq(queried_site_id));
        tide::log::debug!("Selecting pages from site ID: {queried_site_id}");

        // Page Type
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
            },
        }

        // Category
        // TODO

        // TODO track https://github.com/SeaQL/sea-orm/issues/1746

        // TODO implement query construction
        todo!()
    }
}
