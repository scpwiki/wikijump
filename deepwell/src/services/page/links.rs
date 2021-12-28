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
use sea_orm::{DatabaseTransaction, Set};
use std::collections::HashMap;

macro_rules! parse_connection_type {
    ($connection:expr) => {
        ConnectionType::try_from($connection.connection_type.as_str())?
    };
}

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

    macro_rules! count_connections {
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
                        .entry((str!(page_slug), $connection_type))
                        .or_insert(0);

                    *entry += 1;
                }
            }
        }};
    }

    // Get include stats (old, so include-messy)
    for include in &backlinks.included_pages {
        count_connections!(include, ConnectionType::IncludeMessy);
    }

    // Get internal page link stats
    for link in &backlinks.internal_links {
        count_connections!(link, ConnectionType::Link);
    }

    // Gather external URL link stats
    for url in &backlinks.external_links {
        let entry = external_links.entry(url.as_ref()).or_insert(0);
        *entry += 1;
    }

    // Update records
    try_join!(
        update_connections(txn, page_id, &mut connections),
        update_connections_missing(txn, page_id, &mut connections_missing),
        update_external_links(txn, page_id, &mut external_links),
    )?;

    Ok(())
}

async fn update_connections(
    txn: &DatabaseTransaction,
    from_page_id: i64,
    counts: &mut HashMap<(i64, ConnectionType), i32>,
) -> Result<()> {
    // Get existing connections
    let mut connection_chunks = PageConnection::find()
        .filter(page_connection::Column::FromPageId.eq(from_page_id))
        .order_by_asc(page_connection::Column::CreatedAt)
        .paginate(txn, 100);

    // Update and delete connections
    while let Some(connections) = connection_chunks.fetch_and_next().await? {
        for connection in connections {
            let to_page_id = connection.to_page_id;
            let connection_type = parse_connection_type!(connection);

            match counts.remove(&(to_page_id, connection_type)) {
                // Connection exists, count is the same. Do nothing.
                Some(count) if connection.count == count => (),

                // Connection exists, update count.
                Some(count) => {
                    let mut model: page_connection::ActiveModel = connection.into();
                    model.count = Set(count);
                    model.updated_at = Set(Some(now()));
                    model.update(txn).await?;
                }

                // Connection existed, but has no further counts. Remove it.
                None => {
                    let model: page_connection::ActiveModel = connection.into();
                    model.delete(txn).await?;
                }
            }
        }
    }

    // Insert new connections
    let mut to_insert = Vec::new();
    for (&(to_page_id, connection_type), count) in counts {
        to_insert.push(page_connection::ActiveModel {
            from_page_id: Set(from_page_id),
            to_page_id: Set(to_page_id),
            connection_type: Set(str!(connection_type.name())),
            created_at: Set(now()),
            updated_at: Set(None),
            count: Set(*count),
        });
    }
    PageConnection::insert_many(to_insert).exec(txn).await?;

    Ok(())
}

async fn update_connections_missing(
    txn: &DatabaseTransaction,
    from_page_id: i64,
    counts: &mut HashMap<(String, ConnectionType), i32>,
) -> Result<()> {
    // Get existing connections
    let mut connection_chunks = PageConnectionMissing::find()
        .filter(page_connection_missing::Column::FromPageId.eq(from_page_id))
        .order_by_asc(page_connection_missing::Column::CreatedAt)
        .paginate(txn, 100);

    // Update and delete connections
    while let Some(connections) = connection_chunks.fetch_and_next().await? {
        for connection in connections {
            let to_page_slug = connection.to_page_slug.clone();
            let connection_type = parse_connection_type!(connection);

            match counts.remove(&(to_page_slug.clone(), connection_type)) {
                // Connection exists, count is the same. Do nothing.
                Some(count) if connection.count == count => (),

                // Connection exists, update count.
                Some(count) => {
                    let mut model: page_connection_missing::ActiveModel =
                        connection.into();
                    model.count = Set(count);
                    model.updated_at = Set(Some(now()));
                    model.update(txn).await?;
                }

                // Connection existed, but has no further counts. Remove it.
                None => {
                    let model: page_connection_missing::ActiveModel = connection.into();
                    model.delete(txn).await?;
                }
            }
        }
    }

    // Insert new connections
    let mut to_insert = Vec::new();
    for (&(ref to_page_slug, connection_type), count) in counts {
        to_insert.push(page_connection_missing::ActiveModel {
            from_page_id: Set(from_page_id),
            to_page_slug: Set(str!(to_page_slug)),
            connection_type: Set(str!(connection_type.name())),
            created_at: Set(now()),
            updated_at: Set(None),
            count: Set(*count),
        });
    }
    PageConnectionMissing::insert_many(to_insert)
        .exec(txn)
        .await?;

    Ok(())
}

async fn update_external_links(
    txn: &DatabaseTransaction,
    from_page_id: i64,
    counts: &mut HashMap<&str, i32>,
) -> Result<()> {
    todo!()
}
