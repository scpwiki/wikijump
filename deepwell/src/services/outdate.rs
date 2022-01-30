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
use crate::models::page_revision::{self, Entity as PageRevision};
use crate::services::{LinkService, PageService, RevisionService};
use crate::web::ConnectionType;
use sea_orm::sea_query::expr::Expr;

#[derive(Debug)]
pub struct OutdateService;

impl OutdateService {
    /// Marks a series of page revisions as outdated.
    ///
    /// Finds the most recent revision for each of the given `(site_id, page_id)`
    /// pairs passed in.
    pub async fn outdate<I>(ctx: &ServiceContext<'_>, ids: I) -> Result<()>
    where
        I: IntoIterator<Item = (i64, i64)>,
    {
        let txn = ctx.transaction();
        let mut revision_ids = vec![];

        for (site_id, page_id) in ids {
            let revision = RevisionService::get_latest(ctx, site_id, page_id).await?;
            revision_ids.push(revision.revision_id);
        }

        PageRevision::update_many()
            .col_expr(page_revision::Column::CompiledOutdated, Expr::value(true))
            .filter(page_revision::Column::RevisionId.is_in(revision_ids))
            .exec(txn)
            .await?;

        Ok(())
    }

    pub async fn outdate_incoming_links(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
    ) -> Result<()> {
        const CONNECTION_TYPES: &[ConnectionType] = &[ConnectionType::Link];

        let result = LinkService::get_to(ctx, page_id, Some(CONNECTION_TYPES)).await?;
        let ids = result
            .connections
            .iter()
            .map(|connection| (site_id, connection.from_page_id))
            .collect::<Vec<_>>();

        Self::outdate(ctx, ids).await
    }

    pub async fn outdate_included_pages(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
    ) -> Result<()> {
        const CONNECTION_TYPES: &[ConnectionType] = &[
            ConnectionType::IncludeMessy,
            ConnectionType::IncludeElements,
            ConnectionType::Component,
        ];

        let result = LinkService::get_to(ctx, page_id, Some(CONNECTION_TYPES)).await?;
        let ids = result
            .connections
            .iter()
            .map(|connection| (site_id, connection.from_page_id))
            .collect::<Vec<_>>();

        Self::outdate(ctx, ids).await
    }

    pub async fn outdate_outgoing_links(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
    ) -> Result<()> {
        const CONNECTION_TYPES: &[ConnectionType] = &[ConnectionType::Link];

        let result =
            LinkService::get_connections_from(ctx, page_id, Some(CONNECTION_TYPES))
                .await?;

        let ids = result
            .present
            .iter()
            .map(|connection| (site_id, connection.to_page_id))
            .collect::<Vec<_>>();

        Self::outdate(ctx, ids).await
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

            Self::outdate(ctx, ids).await?;
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

            Self::outdate(ctx, ids).await?;
        }

        Ok(())
    }
}
