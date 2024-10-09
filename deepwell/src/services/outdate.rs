/*
 * services/outdate.rs
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

use super::prelude::*;
use crate::models::page::Model as PageModel;
use crate::services::{JobService, LinkService, PageService};
use crate::utils::split_category_name;
use crate::types::{ConnectionType, PageOrder};

#[derive(Debug)]
pub struct OutdateService;

impl OutdateService {
    pub async fn process_page_edit(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        slug: &str,
        depth: u32,
    ) -> Result<()> {
        let (category_slug, page_slug) = split_category_name(slug);

        try_join!(
            OutdateService::outdate_outgoing_includes(ctx, page_id, depth),
            OutdateService::outdate_templates(
                ctx,
                site_id,
                category_slug,
                page_slug,
                depth,
            ),
        )?;

        Ok(())
    }

    /// Performs outdating tasks for a page being created or deleted here.
    pub async fn process_page_displace(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        slug: &str,
        depth: u32,
    ) -> Result<()> {
        try_join!(
            Self::process_page_edit(ctx, site_id, page_id, slug, depth),
            Self::outdate_incoming_links(ctx, page_id, depth),
        )?;

        Ok(())
    }

    pub async fn process_page_move(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        old_slug: &str,
        new_slug: &str,
        depth: u32,
    ) -> Result<()> {
        // In terms of outdating, a move is equivalent to
        // deleting at the old page location and
        // creating at the new page location.
        try_join!(
            Self::process_page_displace(ctx, site_id, page_id, new_slug, depth),
            Self::process_page_displace(ctx, site_id, page_id, old_slug, depth),
        )?;

        Ok(())
    }

    /// Queues the given pages for re-rendering.
    pub async fn outdate(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        depth: u32,
    ) -> Result<()> {
        let PageModel { site_id, .. } =
            PageService::get_direct(ctx, page_id, false).await?;

        JobService::queue_rerender_page(ctx, site_id, page_id, depth + 1).await
    }

    pub async fn outdate_incoming_links(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        depth: u32,
    ) -> Result<()> {
        const CONNECTION_TYPES: &[ConnectionType] = &[ConnectionType::Link];

        for id in LinkService::get_to(ctx, page_id, Some(CONNECTION_TYPES))
            .await?
            .connections
            .iter()
            .map(|connection| connection.from_page_id)
            .filter(|id| *id != page_id)
        {
            Self::outdate(ctx, id, depth).await?;
        }
        Ok(())
    }

    pub async fn outdate_outgoing_includes(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        depth: u32,
    ) -> Result<()> {
        const CONNECTION_TYPES: &[ConnectionType] = &[
            ConnectionType::IncludeMessy,
            ConnectionType::IncludeElements,
            ConnectionType::Component,
        ];

        for id in LinkService::get_to(ctx, page_id, Some(CONNECTION_TYPES))
            .await?
            .connections
            .iter()
            .map(|connection| connection.from_page_id)
            .filter(|id| *id != page_id)
        {
            Self::outdate(ctx, id, depth).await?;
        }
        Ok(())
    }

    pub async fn outdate_templates(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        category_slug: &str,
        page_slug: &str,
        depth: u32,
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

            let pages = PageService::get_all(
                ctx,
                site_id,
                category_select,
                Some(false),
                PageOrder::default(),
            )
            .await?;

            for page in pages {
                Self::outdate(ctx, page.page_id, depth).await?;
            }
        }

        Ok(())
    }
}
