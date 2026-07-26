/*
 * services/link/service.rs
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

use super::resolver::{
    PageExistenceSnapshot, resolve_connection_counts, resolve_page_existence,
};
use super::structs::{
    GetConnectionsFromOutput, GetLinksExternalFromOutput, GetLinksExternalToOutput,
    GetLinksFromOutput, GetLinksToMissingOutput, GetLinksToOutput, ToExternalLink,
};
use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::models::page;
use crate::models::page_connection::{self, Entity as PageConnection};
use crate::models::page_connection_missing::{self, Entity as PageConnectionMissing};
use crate::models::page_link::{self, Entity as PageLink, Model as PageLinkModel};
use crate::services::PageService;
use crate::services::ServiceContext;
use crate::types::ConnectionType;
use crate::types::Reference;
use crate::utils::now;
use ftml::data::Backlinks;
use sea_orm::NotSet;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, JoinType, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set,
};
use std::collections::HashMap;

/// Forms an optional `Condition` from a list of connection types.
///
/// This is used to allow filtering connection queries by what
/// type(s) of connections are desired.
macro_rules! make_contype_condition {
    ($table_name:ident, $connection_types:expr $(,)?) => {
        // Layer 1: Option<&[ConnectionType]> -> Option<Condition>
        $connection_types.map(|connection_types| {
            // Layer 2: &[ConnectionType] -> [&ConnectionType]
            $table_name::Column::ConnectionType.is_in(connection_types.iter().copied())
        })
    };
}

#[derive(Debug)]
pub struct LinkService;

impl LinkService {
    pub(crate) async fn resolve_page_existence(
        ctx: &ServiceContext<'_>,
        source_site_id: i64,
        source_site_slug: &str,
        page_refs: &[ftml::data::PageRef],
    ) -> Result<PageExistenceSnapshot> {
        resolve_page_existence(ctx, source_site_id, source_site_slug, page_refs).await
    }

    pub async fn get_from(
        ctx: &ServiceContext<'_>,
        page_id: i64,
    ) -> Result<GetLinksFromOutput> {
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!("failed to get links from page ID {}", page_id),
                ErrorType::PageLink,
            )
        };

        let (present_result, absent_result, external_result) = join!(
            PageConnection::find()
                .filter(page_connection::Column::FromPageId.eq(page_id))
                .all(txn),
            PageConnectionMissing::find()
                .filter(page_connection_missing::Column::FromPageId.eq(page_id))
                .all(txn),
            PageLink::find()
                .filter(page_link::Column::PageId.eq(page_id))
                .all(txn),
        );

        let (present, absent, external) =
            raise_multiple!(present_result, absent_result, external_result; make_error);

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
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                match connection_types {
                    Some(ctypes) => format!(
                        "failed to get connections from page ID {} (connection types {:?}",
                        page_id, ctypes,
                    ),
                    None => format!(
                        "failed to get connections from page ID {} (all connection types)",
                        page_id,
                    ),
                },
                ErrorType::PageLink,
            )
        };

        let (present_result, absent_result) = join!(
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
        );

        let (present, absent) =
            raise_multiple!(present_result, absent_result; make_error);
        Ok(GetConnectionsFromOutput { present, absent })
    }

    pub async fn get_to(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        connection_types: Option<&[ConnectionType]>,
    ) -> Result<GetLinksToOutput> {
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!("failed to get links to page ID {}", page_id),
                ErrorType::PageLink,
            )
        };

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
            .await
            .or_raise(make_error)?;

        Ok(GetLinksToOutput { connections })
    }

    pub async fn get_to_missing(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_slug: &str,
        connection_types: Option<&[ConnectionType]>,
    ) -> Result<GetLinksToMissingOutput> {
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                match connection_types {
                    Some(ctypes) => format!(
                        "failed to get connections to missing site ID {} page '{}' (connection types {:?}",
                        site_id, page_slug, ctypes,
                    ),
                    None => format!(
                        "failed to get connections to missing site ID {} page '{}'  (all connection types)",
                        site_id, page_slug,
                    ),
                },
                ErrorType::PageLink,
            )
        };

        // Ensure the page doesn't actually exist
        if let Some(page) =
            PageService::get_optional(ctx, site_id, Reference::from(page_slug))
                .await
                .or_raise(make_error)?
        {
            warn!(
                "Requesting missing page connections for page that exists (site id {}, page id {})",
                site_id, page.page_id,
            );
            bail!(Error::new(
                format!(
                    "cannot get missing page connections for page '{}' that exists (site ID {}, page ID {})",
                    page_slug, site_id, page.page_id
                ),
                ErrorType::PageLink
            ));
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
            .await
            .or_raise(make_error)?;

        Ok(GetLinksToMissingOutput { connections })
    }

    pub async fn get_external_from(
        ctx: &ServiceContext<'_>,
        page_id: i64,
    ) -> Result<GetLinksExternalFromOutput> {
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!("failed to get links from page ID {}", page_id),
                ErrorType::PageLink,
            )
        };

        let links = PageLink::find()
            .filter(page_link::Column::PageId.eq(page_id))
            .all(txn)
            .await
            .or_raise(make_error)?;

        Ok(GetLinksExternalFromOutput { links })
    }

    pub async fn get_external_to(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        url: &str,
    ) -> Result<GetLinksExternalToOutput> {
        let txn = ctx.transaction();

        let make_error = || {
            Error::new(
                format!(
                    "failed to get links to external URL {} on site ID {}",
                    url, site_id
                ),
                ErrorType::PageLink,
            )
        };

        // Perform join so we don't leak data from other sites.
        let links = PageLink::find()
            .join(JoinType::InnerJoin, page_link::Relation::Page.def())
            .filter(
                Condition::all()
                    .add(page_link::Column::Url.eq(url))
                    .add(page::Column::SiteId.eq(site_id)),
            )
            .all(txn)
            .await
            .or_raise(make_error)?
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
        let make_error = || {
            Error::new(
                format!(
                    "failed to update links for site ID {} page ID {}",
                    site_id, page_id
                ),
                ErrorType::PageLink,
            )
        };
        let mut connection_counts = resolve_connection_counts(
            ctx,
            site_id,
            [
                (
                    backlinks.included_pages.as_slice(),
                    ConnectionType::IncludeMessy,
                ),
                (backlinks.internal_links.as_slice(), ConnectionType::Link),
            ],
        )
        .await
        .or_raise(make_error)?;
        let mut external_links = HashMap::new();

        // Gather external URL link stats
        for url in &backlinks.external_links {
            let entry = external_links.entry(str!(url)).or_insert(0);
            *entry += 1;
        }

        // Update records
        let (result1, result2, result3) = join!(
            update_connections(ctx, page_id, &mut connection_counts.present),
            update_connections_missing(ctx, page_id, &mut connection_counts.missing),
            update_external_links(ctx, page_id, &mut external_links),
        );
        raise_multiple!(result1, result2, result3; make_error);

        Ok(())
    }
}

// Update link helpers

async fn update_connections(
    ctx: &ServiceContext<'_>,
    from_page_id: i64,
    counts: &mut HashMap<(i64, ConnectionType), i32>,
) -> Result<()> {
    let txn = ctx.transaction();

    let counts_len = counts.len();
    let make_error = || {
        Error::new(
            format!(
                "failed to update connections from page ID {} to {} values",
                from_page_id, counts_len,
            ),
            ErrorType::PageLink,
        )
    };

    // Get existing connections
    let mut connection_chunks = PageConnection::find()
        .filter(page_connection::Column::FromPageId.eq(from_page_id))
        .order_by_asc(page_connection::Column::CreatedAt)
        .paginate(txn, 100);

    // Update and delete connections
    while let Some(connections) = connection_chunks
        .fetch_and_next()
        .await
        .or_raise(make_error)?
    {
        for connection in connections {
            let to_page_id = connection.to_page_id;
            let connection_type = connection.connection_type;

            match counts.remove(&(to_page_id, connection_type)) {
                // Connection exists, count is the same. Do nothing.
                Some(count) if connection.count == count => (),

                // Connection exists, update count.
                Some(count) => {
                    let mut model: page_connection::ActiveModel = connection.into();
                    model.count = Set(count);
                    model.updated_at = Set(Some(now()));
                    model.update(txn).await.or_raise(make_error)?;
                }

                // Connection existed, but has no further counts. Remove it.
                None => {
                    let model: page_connection::ActiveModel = connection.into();
                    model.delete(txn).await.or_raise(make_error)?;
                }
            }
        }
    }

    // Insert new connections
    let updated_at = Some(now());
    let to_insert = counts
        .iter()
        .map(
            |(&(to_page_id, connection_type), count)| page_connection::ActiveModel {
                from_page_id: Set(from_page_id),
                to_page_id: Set(to_page_id),
                connection_type: Set(connection_type),
                created_at: NotSet,
                updated_at: Set(updated_at),
                count: Set(*count),
            },
        )
        .collect::<Vec<_>>();

    if !to_insert.is_empty() {
        PageConnection::insert_many(to_insert)
            .on_conflict(
                OnConflict::columns([
                    page_connection::Column::FromPageId,
                    page_connection::Column::ToPageId,
                    page_connection::Column::ConnectionType,
                ])
                .update_columns([
                    page_connection::Column::Count,
                    page_connection::Column::UpdatedAt,
                ])
                .to_owned(),
            )
            .exec(txn)
            .await
            .or_raise(make_error)?;
    }

    Ok(())
}

async fn update_connections_missing(
    ctx: &ServiceContext<'_>,
    from_page_id: i64,
    counts: &mut HashMap<(i64, String, ConnectionType), i32>,
) -> Result<()> {
    let txn = ctx.transaction();

    let counts_len = counts.len();
    let make_error = || {
        Error::new(
            format!(
                "failed to update missing connections from page ID {} to {} values",
                from_page_id, counts_len,
            ),
            ErrorType::PageLink,
        )
    };

    // Get existing connections
    let mut connection_chunks = PageConnectionMissing::find()
        .filter(page_connection_missing::Column::FromPageId.eq(from_page_id))
        .order_by_asc(page_connection_missing::Column::CreatedAt)
        .paginate(txn, 100);

    // Update and delete connections
    while let Some(connections) = connection_chunks
        .fetch_and_next()
        .await
        .or_raise(make_error)?
    {
        for connection in connections {
            let to_site_id = connection.to_site_id;
            let to_page_slug = connection.to_page_slug.clone();
            let connection_type = connection.connection_type;

            match counts.remove(&(to_site_id, to_page_slug.clone(), connection_type)) {
                // Connection exists, count is the same. Do nothing.
                Some(count) if connection.count == count => (),

                // Connection exists, update count.
                Some(count) => {
                    let mut model: page_connection_missing::ActiveModel =
                        connection.into();
                    model.count = Set(count);
                    model.updated_at = Set(Some(now()));
                    model.update(txn).await.or_raise(make_error)?;
                }

                // Connection existed, but has no further counts. Remove it.
                None => {
                    let model: page_connection_missing::ActiveModel = connection.into();
                    model.delete(txn).await.or_raise(make_error)?;
                }
            }
        }
    }

    // Insert new connections
    let updated_at = Some(now());
    let to_insert = counts
        .iter()
        .map(
            |(&(to_site_id, ref to_page_slug, connection_type), count)| {
                page_connection_missing::ActiveModel {
                    from_page_id: Set(from_page_id),
                    to_site_id: Set(to_site_id),
                    to_page_slug: Set(str!(to_page_slug)),
                    connection_type: Set(connection_type),
                    created_at: NotSet,
                    updated_at: Set(updated_at),
                    count: Set(*count),
                }
            },
        )
        .collect::<Vec<_>>();

    if !to_insert.is_empty() {
        PageConnectionMissing::insert_many(to_insert)
            .on_conflict(
                OnConflict::columns([
                    page_connection_missing::Column::FromPageId,
                    page_connection_missing::Column::ToSiteId,
                    page_connection_missing::Column::ToPageSlug,
                    page_connection_missing::Column::ConnectionType,
                ])
                .update_columns([
                    page_connection_missing::Column::Count,
                    page_connection_missing::Column::UpdatedAt,
                ])
                .to_owned(),
            )
            .exec(txn)
            .await
            .or_raise(make_error)?;
    }

    Ok(())
}

async fn update_external_links(
    ctx: &ServiceContext<'_>,
    from_page_id: i64,
    counts: &mut HashMap<String, i32>,
) -> Result<()> {
    let txn = ctx.transaction();

    let make_error = || {
        Error::new(
            format!(
                "failed to update external links from page ID {}",
                from_page_id
            ),
            ErrorType::PageLink,
        )
    };

    // Get existing links
    let mut link_chunks = PageLink::find()
        .filter(page_link::Column::PageId.eq(from_page_id))
        .order_by_asc(page_link::Column::CreatedAt)
        .paginate(txn, 100);

    // Update and delete connections
    while let Some(links) = link_chunks.fetch_and_next().await.or_raise(make_error)? {
        for link in links {
            match counts.remove(&link.url) {
                // Link exists, count is the same. Do nothing.
                Some(count) if link.count == count => (),

                // Link exists, update count.
                Some(count) => {
                    let mut model: page_link::ActiveModel = link.into();
                    model.count = Set(count);
                    model.updated_at = Set(Some(now()));
                    model.update(txn).await.or_raise(make_error)?;
                }

                // Link existed, but has no further counts. Remove it.
                None => {
                    let model: page_link::ActiveModel = link.into();
                    model.delete(txn).await.or_raise(make_error)?;
                }
            }
        }
    }

    // Insert new links
    let updated_at = Some(now());
    let to_insert = counts
        .iter()
        .map(|(ref url, count)| page_link::ActiveModel {
            page_id: Set(from_page_id),
            url: Set(str!(url)),
            created_at: NotSet,
            updated_at: Set(updated_at),
            count: Set(*count),
        })
        .collect::<Vec<_>>();

    if !to_insert.is_empty() {
        PageLink::insert_many(to_insert)
            .on_conflict(
                OnConflict::columns([page_link::Column::PageId, page_link::Column::Url])
                    .update_columns([
                        page_link::Column::Count,
                        page_link::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(txn)
            .await
            .or_raise(make_error)?;
    }

    Ok(())
}
