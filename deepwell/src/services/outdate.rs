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

use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::futures::StreamExt;
use crate::models::page::{self, Entity as Page};
use crate::models::page_category::{self, Entity as PageCategory};
use crate::services::ServiceContext;
use crate::services::{JobService, LinkService, PageService, SiteService};
use crate::types::{ConnectionType, PageId, PageOrder, Reference, RerenderDepth};
use crate::utils::split_category_name;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, FromQueryResult, QueryFilter, QuerySelect,
};

#[derive(Debug)]
pub struct OutdateService;

impl OutdateService {
    pub async fn process_page_edit(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        slug: &str,
        depth: RerenderDepth,
    ) -> Result<()> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to run outdater for edit of page '{}' (ID {}) on site ID {} (depth {})",
                    slug, page_id, site_id, depth,
                ),
                ErrorType::PageOutdater,
            )
        };

        let (category_slug, page_slug) = split_category_name(slug);
        let (result1, result2, result3) = join!(
            Self::outdate_outgoing_includes(ctx, page_id, depth),
            Self::outdate_templates(ctx, site_id, category_slug, page_slug, depth),
            Self::outdate_nav_pages(ctx, site_id, slug, depth),
        );
        raise_multiple!(result1, result2, result3; make_error);

        Ok(())
    }

    /// Performs outdating tasks for a page being created or deleted here.
    pub async fn process_page_displace(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        slug: &str,
        depth: RerenderDepth,
    ) -> Result<()> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to run outdater for displacement of page '{}' (ID {}) on site ID {} (depth {})",
                    slug, page_id, site_id, depth,
                ),
                ErrorType::PageOutdater,
            )
        };

        let (result1, result2) = join!(
            Self::process_page_edit(ctx, site_id, page_id, slug, depth),
            Self::outdate_incoming_links(ctx, page_id, depth),
        );
        raise_multiple!(result1, result2; make_error);

        Ok(())
    }

    pub async fn process_page_move(
        ctx: &ServiceContext<'_>,
        site_id: i64,
        page_id: i64,
        old_slug: &str,
        new_slug: &str,
        depth: RerenderDepth,
    ) -> Result<()> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to run outdater for move of page ID {} from '{}' to '{}' on site ID {} (depth {})",
                    page_id, old_slug, new_slug, site_id, depth,
                ),
                ErrorType::PageOutdater,
            )
        };

        // In terms of outdating, a move is equivalent to
        // deleting at the old page location and
        // creating at the new page location.
        let (result1, result2) = join!(
            Self::process_page_displace(ctx, site_id, page_id, new_slug, depth),
            Self::process_page_displace(ctx, site_id, page_id, old_slug, depth),
        );
        raise_multiple!(result1, result2; make_error);

        Ok(())
    }

    /// Queues the given pages for re-rendering.
    pub async fn outdate(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        depth: RerenderDepth,
    ) -> Result<()> {
        let make_error = || {
            Error::new(
                format!(
                    "failed to run outdater on page ID {} (depth {})",
                    page_id, depth
                ),
                ErrorType::PageOutdater,
            )
        };

        let Some(page) = PageService::get_direct_optional(ctx, page_id, false)
            .await
            .or_raise(make_error)?
        else {
            debug!(
                "Skipping outdate for missing or deleted page ID {} (depth {})",
                page_id, depth,
            );
            return Ok(());
        };

        let id = PageId::from_page_model(&page);
        JobService::queue_rerender_page(ctx, id, depth.plus_one())
            .await
            .or_raise(make_error)?;

        Ok(())
    }

    pub async fn outdate_incoming_links(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        depth: RerenderDepth,
    ) -> Result<()> {
        const CONNECTION_TYPES: &[ConnectionType] = &[ConnectionType::Link];

        let make_error = || {
            Error::new(
                format!(
                    "failed to run outdater for all pages that link to page ID {} (depth {})",
                    page_id, depth,
                ),
                ErrorType::PageOutdater,
            )
        };

        for id in LinkService::get_to(ctx, page_id, Some(CONNECTION_TYPES))
            .await
            .or_raise(make_error)?
            .connections
            .iter()
            .map(|connection| connection.from_page_id)
            .filter(|id| *id != page_id)
        {
            Self::outdate(ctx, id, depth).await.or_raise(make_error)?;
        }

        Ok(())
    }

    pub async fn outdate_outgoing_includes(
        ctx: &ServiceContext<'_>,
        page_id: i64,
        depth: RerenderDepth,
    ) -> Result<()> {
        const CONNECTION_TYPES: &[ConnectionType] = &[
            ConnectionType::IncludeMessy,
            ConnectionType::IncludeElements,
            ConnectionType::Component,
        ];

        let make_error = || {
            Error::new(
                format!(
                    "failed to run outdater for all pages which include page ID {} (depth {})",
                    page_id, depth,
                ),
                ErrorType::PageOutdater,
            )
        };

        for id in LinkService::get_to(ctx, page_id, Some(CONNECTION_TYPES))
            .await
            .or_raise(make_error)?
            .connections
            .iter()
            .map(|connection| connection.from_page_id)
            .filter(|id| *id != page_id)
        {
            Self::outdate(ctx, id, depth).await.or_raise(make_error)?;
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

        let make_error = || {
            Error::new(
                format!(
                    "failed to run outdater for all pages in category '{}' on site ID {} using page '{}' as a template (depth {})",
                    category_slug, site_id, page_slug, depth,
                ),
                ErrorType::PageOutdater,
            )
        };

        // If a template page has been updated, recompile only its exact
        // category. Named categories do not inherit the default template.
        if let Some(category_select) = exact_template_category(
            &config.blueprint_page_template,
            category_slug,
            page_slug,
        ) {
            let template_slug = if category_select == "_default" {
                page_slug.to_owned()
            } else {
                format!("{category_select}:{page_slug}")
            };

            let pages = PageService::get_all(
                ctx,
                site_id,
                Some(category_select.into()),
                Some(false),
                PageOrder::default(),
            )
            .await
            .or_raise(make_error)?;

            for page in pages {
                if page.slug == template_slug {
                    continue;
                }
                Self::outdate(ctx, page.page_id, depth)
                    .await
                    .or_raise(make_error)?;
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
        let make_error = || {
            Error::new(
                format!(
                    "failed to run nav-only outdater for all pages using page '{}' on site ID {} as a nav page (depth {})",
                    slug, site_id, depth,
                ),
                ErrorType::PageOutdater,
            )
        };

        // If this is the nav page for the site, then outdate everything
        // Nothing else needs to be done.
        let site = SiteService::get(ctx, Reference::Id(site_id))
            .await
            .or_raise(make_error)?;

        if site.top_bar_page == slug || site.side_bar_page == slug {
            Self::outdate_nav_site(ctx, site_id, depth)
                .await
                .or_raise(make_error)?;
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
            .await
            .or_raise(make_error)?;

        for category_id in category_ids {
            Self::outdate_nav_category(ctx, site_id, category_id, depth)
                .await
                .or_raise(make_error)?;
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

        let make_error = || {
            Error::new(
                format!(
                    "failed to run nav-only outdater for all pages on site ID {} (depth {})",
                    site_id, depth,
                ),
                ErrorType::PageOutdater,
            )
        };

        #[derive(FromQueryResult)]
        struct Row {
            site_id: i64,
            page_category_id: i64,
            page_id: i64,
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
            .await
            .or_raise(make_error)?;

        while let Some(row) = rows.next().await {
            let Row {
                site_id,
                page_category_id: category_id,
                page_id,
            } = row.or_raise(make_error)?;

            JobService::queue_rerender_nav_page(
                ctx,
                PageId {
                    site_id,
                    category_id,
                    page_id,
                },
                depth.plus_one(),
            )
            .await
            .or_raise(make_error)?;
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
        let make_error = || {
            Error::new(
                format!(
                    "failed to run nav-only outdater for all pages in category ID {} on site ID {} (depth {})",
                    category_id, site_id, depth,
                ),
                ErrorType::PageOutdater,
            )
        };

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
            .await
            .or_raise(make_error)?;

        while let Some(row) = rows.next().await {
            let page_id = row.or_raise(make_error)?;

            JobService::queue_rerender_nav_page(
                ctx,
                PageId {
                    site_id,
                    category_id,
                    page_id,
                },
                depth.plus_one(),
            )
            .await
            .or_raise(make_error)?;
        }

        Ok(())
    }
}

fn exact_template_category<'a>(
    template_page: &str,
    category: &'a str,
    page: &str,
) -> Option<&'a str> {
    (page == template_page).then_some(category)
}

#[cfg(test)]
mod tests {
    use super::exact_template_category;

    #[test]
    fn template_outdating_stays_in_the_exact_category() {
        assert_eq!(
            exact_template_category("_template", "_default", "_template"),
            Some("_default"),
        );
        assert_eq!(
            exact_template_category("_template", "fragment", "_template"),
            Some("fragment"),
        );
        assert_eq!(
            exact_template_category("_template", "fragment", "article"),
            None,
        );
    }
}
