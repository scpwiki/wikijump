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
use crate::models::page_connection::{
    self, Entity as PageConnection, Model as PageConnectionModel,
};
use crate::models::page_connection_missing::{
    self, Entity as PageConnectionMissing, Model as PageConnectionMissingModel,
};
use crate::models::page_link::{self, Entity as PageLink, Model as PageLinkModel};
use crate::web::ConnectionType;
use ftml::data::{Backlinks, PageRef};
use sea_orm::DatabaseTransaction;
use std::collections::HashMap;

pub async fn update_links(
    txn: &DatabaseTransaction,
    page: &PageService<'_>,
    site_id: i64,
    page_id: i64,
    backlinks: &Backlinks<'_>,
) -> Result<()> {
    let mut connections = HashMap::new();
    let mut connections_missing = HashMap::new();
    let mut external_links = HashMap::new();

    macro_rules! update_connections {
        ($page_ref:expr, $connection_type:expr) => {{
            let PageRef {
                site: site_slug,
                page: page_slug,
            } = $page_ref;

            let to_site_id = match site_slug {
                None => site_id,
                Some(slug) => {
                    // TODO: get site ID from SiteService
                    1
                }
            };

            match page
                .get_optional(to_site_id, ItemReference::Slug(page_slug))
                .await?
            {
                Some(to_page) => {
                    let entry = connections
                        .entry((to_page.page_id, $connection_type))
                        .or_insert(0);

                    *entry += 1;
                }
                None => {
                    let entry = connections_missing
                        .entry((page_slug.as_ref(), $connection_type))
                        .or_insert(0);

                    *entry += 1;
                }
            }
        }};
    }

    // Get include stats (old, so include-messy)
    for include in &backlinks.included_pages {
        update_connections!(include, ConnectionType::IncludeMessy);
    }

    // Get internal page link stats
    for link in &backlinks.internal_links {
        update_connections!(link, ConnectionType::Link);
    }

    // Gather external URL link stats
    for url in &backlinks.external_links {
        let entry = external_links.entry(url.as_ref()).or_insert(0);
        *entry += 1;
    }

    // Update records
    try_join!(
        update_connections(txn, page_id, &connections),
        update_connections_missing(txn, page_id, &connections_missing),
        update_external_links(txn, page_id, &external_links),
    )?;

    Ok(())
}

async fn update_connections(
    txn: &DatabaseTransaction,
    from_page_id: i64,
    counts: &HashMap<(i64, ConnectionType), i32>,
) -> Result<()> {
    todo!()
}

async fn update_connections_missing(
    txn: &DatabaseTransaction,
    from_page_id: i64,
    counts: &HashMap<(&str, ConnectionType), i32>,
) -> Result<()> {
    todo!()
}

async fn update_external_links(
    txn: &DatabaseTransaction,
    from_page_id: i64,
    counts: &HashMap<&str, i32>,
) -> Result<()> {
    todo!()
}
