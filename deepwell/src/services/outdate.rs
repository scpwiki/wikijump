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
use crate::services::PageService;

#[derive(Debug)]
pub struct OutdateService;

impl OutdateService {
    pub async fn outdate(_ctx: &ServiceContext<'_>, _ids: &[(i64, i64)]) -> Result<()> {
        todo!()
    }

    pub async fn outdate_incoming_links(
        _ctx: &ServiceContext<'_>,
        _site_id: i64,
        _page_id: i64,
    ) -> Result<()> {
        todo!()
    }

    pub async fn outdate_outgoing_links(
        _ctx: &ServiceContext<'_>,
        _site_id: i64,
        _page_id: i64,
    ) -> Result<()> {
        todo!()
    }

    pub async fn outdate_included_pages(
        _ctx: &ServiceContext<'_>,
        _site_id: i64,
        _page_id: i64,
    ) -> Result<()> {
        todo!()
    }

    pub async fn outdate_navigation(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        category_slug: &str,
        page_slug: &str,
    ) -> Result<()> {
        // If a navigation page has been updated,
        // we need to recompile everything on that site.
        if matches!((category_slug, page_slug), ("nav", "side" | "top")) {
            let ids = PageService::get_all(ctx, site_id, None, Some(false))
                .await?
                .into_iter()
                .map(|model| (model.site_id, model.page_id))
                .collect::<Vec<_>>();

            Self::outdate(ctx, &ids).await?;
        }

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
            let ids = PageService::get_all(
                ctx,
                site_id,
                Some(category_slug.into()),
                Some(false),
            )
            .await?
            .into_iter()
            .map(|model| (model.site_id, model.page_id))
            .collect::<Vec<_>>();

            Self::outdate(ctx, &ids).await?;
        }

        Ok(())
    }
}
