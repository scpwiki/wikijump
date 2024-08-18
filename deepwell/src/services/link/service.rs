/*
 * services/link/service.rs
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
use crate::models::page;
use crate::models::page_connection::{self, Entity as PageConnection};
use crate::models::page_connection_missing::{self, Entity as PageConnectionMissing};
use crate::models::page_link::{self, Entity as PageLink, Model as PageLinkModel};
use crate::services::{PageService, SiteService};
use crate::web::ConnectionType;
use ftml::data::{Backlinks, PageRef};
use sea_orm::NotSet;
use std::collections::HashMap;

/// Forms an optional `Condition` from a list of connection types.
///
/// This is used to allow filtering connection queries by what
/// type(s) of connections are desired.
macro_rules! make_contype_condition {
    ($table_name:ident, $connection_types:expr $(,)?) => {
        // Layer 1: Option<&[ConnectionType]> -> Option<Condition>
        $connection_types.map(|connection_types| {
            // Layer 2: &[ConnectionType] -> [&str]
            $table_name::Column::ConnectionType.is_in(
                // Layer 3: ConnectionType::name -> &str
                connection_types.iter().map(|ctype| ctype.name()),
            )
        })
    };
}

#[derive(Debug)]
pub struct LinkService;

impl LinkService {
    pub async fn get_from(
        ctx: &ServiceContext<'_>,
        page_id: i64,
    ) -> Result<GetLinksFromOutput> {
        let txn = ctx.seaorm_transaction();

        let (present, absent, external) = try_join!(
            PageConnection::find()
                .filter(page_connection::Column::FromPageId.eq(page_id))
                .all(txn),
            PageConnectionMissing::find()
                .filter(page_connection_missing::Column::FromPageId.eq(page_id))
                .all(txn),
            PageLink::find()
                .filter(page_link::Column::PageId.eq(page_id))
                .all(txn),
        )?;

        Ok(GetLinksFromOutput {
            present,
            absent,
            external,
        })
    }

    // TODO
    #[allow(dead_code)]
    pub async fn get_connections_from(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        connection_types: Option<&[ConnectionType]>,
    ) -> Result<GetConnectionsFromOutput> {
        let txn = ctx.seaorm_transaction();

        let (present, absent) = try_join!(
            PageConnection::find()
                .filter(
                    Condition::all()
                        .add(page_connection::Column::FromPageId.eq(page_id))
                        .add_option(make_contype_condition!(
                            page_connection,
                            connection_types,
                        )),
                )
                .all(txn),
            PageConnectionMissing::find()
                .filter(
                    Condition::all()
                        .add(page_connection_missing::Column::FromPageId.eq(page_id))
                        .add_option(make_contype_condition!(
                            page_connection_missing,
                            connection_types,
                        )),
                )
                .all(txn),
        )?;

        Ok(GetConnectionsFromOutput { present, absent })
    }

    pub async fn get_to(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        connection_types: Option<&[ConnectionType]>,
    ) -> Result<GetLinksToOutput> {
        let txn = ctx.seaorm_transaction();

        let connections = PageConnection::find()
            .filter(
                Condition::all()
                    .add(page_connection::Column::ToPageId.eq(page_id))
                    .add_option(make_contype_condition!(
                        page_connection,
                        connection_types,
                    )),
            )
            .all(txn)
            .await?;

        Ok(GetLinksToOutput { connections })
    }

    pub async fn get_to_missing(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_slug: &str,
        connection_types: Option<&[ConnectionType]>,
    ) -> Result<GetLinksToMissingOutput> {
        let txn = ctx.seaorm_transaction();

        // Ensure the page doesn't actually exist
        if let Some(page) =
            PageService::get_optional(ctx, site_id, Reference::from(page_slug)).await?
        {
            warn!(
                "Requesting missing page connections for page that exists (site id {}, page id {})",
                site_id,
                page.page_id,
            );

            return Err(Error::PageExists);
        }

        // Retrieve connections for this slot
        let connections = PageConnectionMissing::find()
            .filter(
                Condition::all()
                    .add(page_connection_missing::Column::ToSiteId.eq(site_id))
                    .add(page_connection_missing::Column::ToPageSlug.eq(page_slug))
                    .add_option(make_contype_condition!(
                        page_connection_missing,
                        connection_types
                    )),
            )
            .all(txn)
            .await?;

        Ok(GetLinksToMissingOutput { connections })
    }

    pub async fn get_external_from(
        ctx: &ServiceContext<'_>,
        page_id: i64,
    ) -> Result<GetLinksExternalFromOutput> {
        let txn = ctx.seaorm_transaction();

        let links = PageLink::find()
            .filter(page_link::Column::PageId.eq(page_id))
            .all(txn)
            .await?;

        Ok(GetLinksExternalFromOutput { links })
    }

    pub async fn get_external_to(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        url: &str,
    ) -> Result<GetLinksExternalToOutput> {
        let txn = ctx.seaorm_transaction();

        // Perform join so we don't leak data from other sites.
        let links = PageLink::find()
            .join(JoinType::InnerJoin, page_link::Relation::Page.def())
            .filter(
                Condition::all()
                    .add(page_link::Column::Url.eq(url))
                    .add(page::Column::SiteId.eq(site_id)),
            )
            .all(txn)
            .await?
            .into_iter()
            .map(
                // Filter out unneeded fields, notably 'url'
                // which is the same for all fields.
                |PageLinkModel {
                     created_at,
                     updated_at,
                     page_id,
                     count,
                     ..
                 }| ToExternalLink {
                    created_at,
                    updated_at,
                    page_id,
                    count,
                },
            )
            .collect();

        Ok(GetLinksExternalToOutput { links })
    }

    pub async fn update(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        backlinks: &Backlinks<'_>,
    ) -> Result<()> {
        let mut connections = HashMap::new();
        let mut connections_missing = HashMap::new();
        let mut external_links = HashMap::new();

        // Get include stats
        for include in &backlinks.included_pages {
            count_connections(
                ctx,
                site_id,
                include,
                // TODO: update Backlinks so that it also tracks other kinds of includes and components
                ConnectionType::IncludeMessy,
                &mut connections,
                &mut connections_missing,
            )
            .await?;
        }

        // Get internal page link stats
        for link in &backlinks.internal_links {
            count_connections(
                ctx,
                site_id,
                link,
                ConnectionType::Link,
                &mut connections,
                &mut connections_missing,
            )
            .await?;
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
}

// Update link helpers

async fn update_connections(
    ctx: &ServiceContext<'_>,
    from_page_id: i64,
    counts: &mut HashMap<(i64, ConnectionType), i32>,
) -> Result<()> {
    let txn = ctx.seaorm_transaction();

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
                created_at: NotSet,
                updated_at: NotSet,
                count: Set(*count),
            },
        )
        .collect::<Vec<_>>();

    if !to_insert.is_empty() {
        PageConnection::insert_many(to_insert).exec(txn).await?;
    }

    Ok(())
}

async fn update_connections_missing(
    ctx: &ServiceContext<'_>,
    from_page_id: i64,
    counts: &mut HashMap<(i64, String, ConnectionType), i32>,
) -> Result<()> {
    let txn = ctx.seaorm_transaction();

    // Get existing connections
    let mut connection_chunks = PageConnectionMissing::find()
        .filter(page_connection_missing::Column::FromPageId.eq(from_page_id))
        .order_by_asc(page_connection_missing::Column::CreatedAt)
        .paginate(txn, 100);

    // Update and delete connections
    while let Some(connections) = connection_chunks.fetch_and_next().await? {
        for connection in connections {
            let to_site_id = connection.to_site_id;
            let to_page_slug = connection.to_page_slug.clone();
            let connection_type = parse_connection_type!(connection);

            match counts.remove(&(to_site_id, to_page_slug.clone(), connection_type)) {
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
        .map(
            |(&(to_site_id, ref to_page_slug, connection_type), count)| {
                page_connection_missing::ActiveModel {
                    from_page_id: Set(from_page_id),
                    to_site_id: Set(to_site_id),
                    to_page_slug: Set(str!(to_page_slug)),
                    connection_type: Set(str!(connection_type.name())),
                    created_at: NotSet,
                    updated_at: NotSet,
                    count: Set(*count),
                }
            },
        )
        .collect::<Vec<_>>();

    if !to_insert.is_empty() {
        PageConnectionMissing::insert_many(to_insert)
            .exec(txn)
            .await?;
    }

    Ok(())
}

async fn update_external_links(
    ctx: &ServiceContext<'_>,
    from_page_id: i64,
    counts: &mut HashMap<String, i32>,
) -> Result<()> {
    let txn = ctx.seaorm_transaction();

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
            created_at: NotSet,
            updated_at: NotSet,
            count: Set(*count),
        })
        .collect::<Vec<_>>();

    if !to_insert.is_empty() {
        PageLink::insert_many(to_insert).exec(txn).await?;
    }

    Ok(())
}

async fn count_connections(
    ctx: &ServiceContext<'_>,
    site_id: i64,
    PageRef {
        site: site_slug,
        page: page_slug,
    }: &PageRef<'_>,
    connection_type: ConnectionType,
    connections: &mut HashMap<(i64, ConnectionType), i32>,
    connections_missing: &mut HashMap<(i64, String, ConnectionType), i32>,
) -> Result<()> {
    let to_site_id = match site_slug {
        None => site_id,
        Some(slug) => {
            let reference = Reference::Slug(cow!(slug));
            SiteService::get(ctx, reference).await?.site_id
        }
    };

    let page = {
        let reference = Reference::Slug(cow!(page_slug));
        PageService::get_optional(ctx, to_site_id, reference).await?
    };

    match page {
        Some(to_page) => {
            let entry = connections
                .entry((to_page.page_id, connection_type))
                .or_insert(0);

            *entry += 1;
        }
        None => {
            let entry = connections_missing
                .entry((to_site_id, str!(page_slug), connection_type))
                .or_insert(0);

            *entry += 1;
        }
    }

    Ok(())
}
