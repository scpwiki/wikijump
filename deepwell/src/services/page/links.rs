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

//! This helper module updates the various kinds of backlinks supported in Wikijump.
//!
//! This includes "page connections" (a generic term for a relation between pages,
//! such as includes, links, redirects, etc), which can either be present or missing,
//! as well as external links, which are string URLs.
//!
//! Whenever a page is updated, a list of its backlinks is gathered by the parser,
//! which is then presented here for processing. A diff is needed:
//! * Any links no longer present are deleted.
//! * Any links not previously present are inesrted.
//! * Any links present but changed in quantity are updated.
//! * Else, the link is up-to-date and left alone.
//!
//! While the logic here is similar for each case, the slight differences in keys,
//! types, and tables make it hard to modularize. Instead, the logic is hopefully
//! clear enough to be acceptable when repeated over a few slightly distinct cases.

use super::{super::prelude::*, PageService};
use crate::models::page_connection::{self, Entity as PageConnection};
use crate::models::page_connection_missing::{self, Entity as PageConnectionMissing};
use crate::models::page_link::{self, Entity as PageLink};
use crate::web::ConnectionType;
use ftml::data::{Backlinks, PageRef};
use std::collections::HashMap;

macro_rules! parse_connection_type {
    ($connection:expr) => {
        ConnectionType::try_from($connection.connection_type.as_str())?
    };
}

pub async fn update_links(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    page_id: i64,
    backlinks: &Backlinks<'_>,
) -> Result<()> {
    let mut connections = HashMap::new();
    let mut connections_missing = HashMap::new();
    let mut external_links = HashMap::new();

    macro_rules! count_connections {
        ($ctx:expr, $page_ref:expr, $connection_type:expr) => {{
            let PageRef {
                site: site_slug,
                page: page_slug,
            } = $page_ref;

            let to_site_id = match site_slug {
                None => site_id,
                Some(_slug) => {
                    // TODO: get site ID from SiteService
                    1
                }
            };

            let page =
                PageService::get_optional($ctx, to_site_id, Reference::Slug(page_slug))
                    .await?;
            match page {
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
        count_connections!(ctx, include, ConnectionType::IncludeMessy);
    }

    // Get internal page link stats
    for link in &backlinks.internal_links {
        count_connections!(ctx, link, ConnectionType::Link);
    }

    // Gather external URL link stats
    for url in &backlinks.external_links {
        let entry = external_links.entry(str!(url)).or_insert(0);
        *entry += 1;
    }

    // Update records
    try_join!(
        update_connections(ctx, page_id, &mut connections),
        update_connections_missing(ctx, page_id, &mut connections_missing),
        update_external_links(ctx, page_id, &mut external_links),
    )?;

    Ok(())
}

async fn update_connections(
    ctx: &ServiceContext<'_>,
    from_page_id: i64,
    counts: &mut HashMap<(i64, ConnectionType), i32>,
) -> Result<()> {
    let txn = ctx.transaction();

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
    let to_insert = counts
        .iter()
        .map(
            |(&(to_page_id, connection_type), count)| page_connection::ActiveModel {
                from_page_id: Set(from_page_id),
                to_page_id: Set(to_page_id),
                connection_type: Set(str!(connection_type.name())),
                created_at: Set(now()),
                updated_at: Set(None),
                count: Set(*count),
            },
        )
        .collect::<Vec<_>>();

    PageConnection::insert_many(to_insert).exec(txn).await?;

    Ok(())
}

async fn update_connections_missing(
    ctx: &ServiceContext<'_>,
    from_page_id: i64,
    counts: &mut HashMap<(String, ConnectionType), i32>,
) -> Result<()> {
    let txn = ctx.transaction();

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
    let to_insert = counts
        .iter()
        .map(|(&(ref to_page_slug, connection_type), count)| {
            page_connection_missing::ActiveModel {
                from_page_id: Set(from_page_id),
                to_page_slug: Set(str!(to_page_slug)),
                connection_type: Set(str!(connection_type.name())),
                created_at: Set(now()),
                updated_at: Set(None),
                count: Set(*count),
            }
        })
        .collect::<Vec<_>>();

    PageConnectionMissing::insert_many(to_insert)
        .exec(txn)
        .await?;

    Ok(())
}

async fn update_external_links(
    ctx: &ServiceContext<'_>,
    from_page_id: i64,
    counts: &mut HashMap<String, i32>,
) -> Result<()> {
    let txn = ctx.transaction();

    // Get existing links
    let mut link_chunks = PageLink::find()
        .filter(page_link::Column::PageId.eq(from_page_id))
        .order_by_asc(page_link::Column::CreatedAt)
        .paginate(txn, 100);

    // Update and delete connections
    while let Some(links) = link_chunks.fetch_and_next().await? {
        for link in links {
            match counts.remove(&link.url) {
                // Link exists, count is the same. Do nothing.
                Some(count) if link.count == count => (),

                // Link exists, update count.
                Some(count) => {
                    let mut model: page_link::ActiveModel = link.into();
                    model.count = Set(count);
                    model.updated_at = Set(Some(now()));
                    model.update(txn).await?;
                }

                // Link existed, but has no further counts. Remove it.
                None => {
                    let model: page_link::ActiveModel = link.into();
                    model.delete(txn).await?;
                }
            }
        }
    }

    // Insert new links
    let to_insert = counts
        .iter()
        .map(|(ref url, count)| page_link::ActiveModel {
            page_id: Set(from_page_id),
            url: Set(str!(url)),
            created_at: Set(now()),
            updated_at: Set(None),
            count: Set(*count),
        })
        .collect::<Vec<_>>();
    PageLink::insert_many(to_insert).exec(txn).await?;

    Ok(())
}
