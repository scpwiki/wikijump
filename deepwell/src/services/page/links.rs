/*
 * services/page/links.rs
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

use super::super::prelude::*;
use super::PageService;
use ftml::data::{Backlinks, PageRef};
use std::collections::HashMap;

pub async fn update_links(
    page: &PageService<'_>,
    site_id: i64,
    page_id: i64,
    backlinks: &Backlinks<'_>,
) -> Result<()> {
    let mut connections = HashMap::new();
    let mut connections_missing = HashMap::new();
    let mut external_links = HashMap::new();

    for PageRef { site, page: slug } in &backlinks.included_pages {
        let included_from_site_id = match site {
            None => site_id,
            Some(site) => {
                // TODO: get site ID from SiteService
                1
            }
        };

        let exists = page
            .exists(included_from_site_id, ItemReference::Slug(slug))
            .await?;

        let count_map = if exists {
            connections
        } else {
            connections_missing
        };

        *count_map.entry(page_id).or_insert(0) += 1;
    }

    todo!();

    Ok(())
}
