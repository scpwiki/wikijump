/*
 * services/settings/service.rs
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
use crate::services::{PageService, SiteService};
use ftml::layout::Layout;

#[derive(Debug)]
pub struct SettingsService;

impl SettingsService {
    /// Get the layout associated with this page.
    ///
    /// If this page has a specific layout override,
    /// then that is returned. Otherwise, the layout
    /// associated with the site is used.
    ///
    /// If no page ID is specified, then searching
    /// starts with site layout overrides.
    pub async fn get_layout(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: Option<i64>,
    ) -> Result<Layout> {
        fn parse_layout(value: &str) -> Result<Layout> {
            value.parse().map_err(|_| Error::InvalidEnumValue)
        }

        if let Some(page_id) = page_id {
            debug!("Getting page layout for site ID {site_id} page ID {page_id}");
            let page = PageService::get_direct(ctx, page_id, true).await?;
            if let Some(layout) = page.layout {
                debug!("Found page-level layout override: {layout}");
                return parse_layout(&layout);
            }
        }

        debug!("Getting site layout for site ID {site_id}");
        let site = SiteService::get(ctx, Reference::Id(site_id)).await?;
        if let Some(layout) = site.layout {
            debug!("Found site-level layout override: {layout}");
            return parse_layout(&layout);
        }

        debug!("Using platform-level layout");
        Ok(ctx.config().default_page_layout)
    }
}
