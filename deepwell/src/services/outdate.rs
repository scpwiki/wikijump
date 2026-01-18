/*
 * services/outdate.rs
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

use super::prelude::*;
use crate::futures::StreamExt;
use crate::models::page::{self, Entity as Page, Model as PageModel};
use crate::models::page_category::{self, Entity as PageCategory};
use crate::services::{JobService, LinkService, PageService, SiteService};
use crate::types::{ConnectionType, PageIdGroup, PageOrder, RerenderDepth};
use crate::types::id::{PageId, SiteId};
use crate::utils::split_category_name;
use ref_map::*;
use sea_orm::FromQueryResult;

#[derive(Debug)]
pub struct OutdateService;

impl OutdateService {
    pub async fn process_page_edit(
        ctx: &ServiceContext<'_>,
        site_id: SiteId,
        page_id: PageId,
        slug: &str,
        depth: RerenderDepth,
    ) -> Result<()> {
        let (category_slug, page_slug) = split_category_name(slug);

        try_join!(
            Self::outdate_outgoing_includes(ctx, page_id, depth),
            Self::outdate_templates(ctx, site_id, category_slug, page_slug, depth),
            Self::outdate_nav_pages(ctx, site_id, slug, depth),
        )?;

        Ok(())
    }

    /// Performs outdating tasks for a page being created or deleted here.
    pub async fn process_page_displace(
        ctx: &ServiceContext<'_>,
        site_id: SiteId,
        page_id: PageId,
        slug: &str,
        depth: RerenderDepth,
    ) -> Result<()> {
        try_join!(
            Self::process_page_edit(ctx, site_id, page_id, slug, depth),
            Self::outdate_incoming_links(ctx, page_id, depth),
        )?;

        Ok(())
    }

    pub async fn process_page_move(
        ctx: &ServiceContext<'_>,
        site_id: SiteId,
        page_id: PageId,
        old_slug: &str,
        new_slug: &str,
        depth: RerenderDepth,
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
        page_id: PageId,
        depth: RerenderDepth,
    ) -> Result<()> {
        let page = PageService::get_direct(ctx, page_id, false).await?;
        let id = PageIdGroup::from_page_model(&page);
        JobService::queue_rerender_page(ctx, id, depth.plus_one()).await
    }

    pub async fn outdate_incoming_links(
        ctx: &ServiceContext<'_>,
        page_id: PageId,
        depth: RerenderDepth,
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
        page_id: PageId,
        depth: RerenderDepth,
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
        depth: RerenderDepth,
    ) -> Result<()> {
        let config = ctx.config();

        // If a template page has been updated,
        // we need to recompile everything in that category.
        if page_slug == config.blueprint_page_template {
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

    /// Determines if the page being updated is used as an nav page anywhere.
    /// If so, all pages using this as a nav page should have their nav pages rebuilt.
    pub async fn outdate_nav_pages(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        slug: &str,
        depth: RerenderDepth,
    ) -> Result<()> {
        // If this is the nav page for the site, then outdate everything
        // Nothing else needs to be done.
        let site = SiteService::get(ctx, Reference::Id(site_id)).await?;
        if site.top_bar_page == slug || site.side_bar_page == slug {
            Self::outdate_nav_site(ctx, site_id, depth).await?;
            return Ok(());
        }

        // If this is the nav page for a category, then outdate all
        // the pages in that category. Note that multiple categories
        // can use the same nav pages.
        let txn = ctx.transaction();
        let category_ids = PageCategory::find()
            .select_only()
            .column(page_category::Column::CategoryId)
            .filter(
                Condition::any()
                    .add(page_category::Column::TopBarPage.eq(slug))
                    .add(page_category::Column::SideBarPage.eq(slug)),
            )
            .into_tuple()
            .all(txn)
            .await?;

        for category_id in category_ids {
            Self::outdate_nav_category(ctx, site_id, category_id, depth).await?;
        }

        Ok(())
    }

    /// Outdates the nav pages of every page on the site.
    pub async fn outdate_nav_site(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        depth: RerenderDepth,
    ) -> Result<()> {
        info!("Outdating all pages on site ID {site_id}");

        #[derive(FromQueryResult)]
        struct Row {
            site_id: i64,
            page_category_id: i64,
            page_id: PageId,
        }

        let txn = ctx.transaction();
        let mut rows = Page::find()
            .select_only()
            .column(page::Column::SiteId)
            .column(page::Column::PageCategoryId)
            .column(page::Column::PageId)
            .filter(
                Condition::all()
                    .add(page::Column::SiteId.eq(site_id))
                    .add(page::Column::DeletedAt.is_null()),
            )
            .into_model::<Row>()
            .stream(txn)
            .await?;

        while let Some(row) = rows.next().await {
            let Row {
                site_id,
                page_category_id: category_id,
                page_id,
            } = row?;

            JobService::queue_rerender_nav_page(
                ctx,
                PageIdGroup {
                    site_id,
                    category_id,
                    page_id,
                },
                depth.plus_one(),
            )
            .await?;
        }

        Ok(())
    }

    /// Outdates the nav pages of all pages in the given category.
    pub async fn outdate_nav_category(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        category_id: i64,
        depth: RerenderDepth,
    ) -> Result<()> {
        let txn = ctx.transaction();
        let mut rows = Page::find()
            .select_only()
            .column(page::Column::PageId)
            .filter(
                Condition::all()
                    .add(page::Column::SiteId.eq(site_id))
                    .add(page::Column::PageCategoryId.eq(category_id))
                    .add(page::Column::DeletedAt.is_null()),
            )
            .into_tuple()
            .stream(txn)
            .await?;

        while let Some(row) = rows.next().await {
            let page_id = row?;

            JobService::queue_rerender_nav_page(
                ctx,
                PageIdGroup {
                    site_id,
                    category_id,
                    page_id,
                },
                depth.plus_one(),
            )
            .await?;
        }

        Ok(())
    }
}
