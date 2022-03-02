/*
 * services/outdate.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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
use crate::services::{JobService, LinkService, PageService};
use crate::web::ConnectionType;

#[derive(Debug)]
pub struct OutdateService;

impl OutdateService {
    pub async fn process_page_create(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        slug: &str,
    ) -> Result<()> {
        todo!()
    }

    pub async fn process_page_delete(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        slug: &str,
    ) -> Result<()> {
        todo!()
    }

    pub async fn process_page_move(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        old_slug: &str,
        new_slug: &str,
    ) -> Result<()> {
        // In terms of outdating, a move is equivalent to
        // deleting at the old page location and
        // creating at the new page location.
        try_join!(
            Self::process_page_create(ctx, site_id, page_id, new_slug),
            Self::process_page_delete(ctx, site_id, page_id, old_slug),
        )?;

        Ok(())
    }

    /// Queues the given pages for re-rendering.
    ///
    /// Finds the most recent revision for each of the given `(site_id, page_id)`
    /// pairs passed in.
    pub fn outdate<I: IntoIterator<Item = (i64, i64)>>(ids: I) {
        for (site_id, page_id) in ids {
            JobService::queue_rerender_page(site_id, page_id);
        }
    }

    pub async fn outdate_incoming_links(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
    ) -> Result<()> {
        const CONNECTION_TYPES: &[ConnectionType] = &[ConnectionType::Link];

        let result = LinkService::get_to(ctx, page_id, Some(CONNECTION_TYPES)).await?;
        let ids = result
            .connections
            .iter()
            .map(|connection| (site_id, connection.from_page_id))
            .filter(|&(_, to_page_id)| to_page_id != page_id)
            .collect::<Vec<_>>();

        Self::outdate(ids);
        Ok(())
    }

    pub async fn outdate_outgoing_includes(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
    ) -> Result<()> {
        const CONNECTION_TYPES: &[ConnectionType] = &[
            ConnectionType::IncludeMessy,
            ConnectionType::IncludeElements,
            ConnectionType::Component,
        ];

        let result = LinkService::get_to(ctx, page_id, Some(CONNECTION_TYPES)).await?;
        let ids = result
            .connections
            .iter()
            .map(|connection| (site_id, connection.from_page_id))
            .filter(|&(_, to_page_id)| to_page_id != page_id)
            .collect::<Vec<_>>();

        Self::outdate(ids);
        Ok(())
    }

    pub async fn outdate_templates(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        category_slug: &str,
        page_slug: &str,
    ) -> Result<()> {
        // If a template page has been updated,
        // we need to recompile everything in that category.
        if page_slug == "_template" {
            let category_select = if category_slug == "_default" {
                // If the category is _default, we need to recompile everything.
                // All other categories may inherit from _default.
                //
                // Specifying "None" here means that we aren't filtering by category.
                None
            } else {
                // Otherwise, filter by whatever category slug we have here.
                Some(category_slug.into())
            };

            let ids = PageService::get_all(ctx, site_id, category_select, Some(false))
                .await?
                .into_iter()
                .map(|model| (model.site_id, model.page_id))
                .collect::<Vec<_>>();

            Self::outdate(ids);
        }

        Ok(())
    }
}
