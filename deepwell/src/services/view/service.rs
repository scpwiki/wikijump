/*
 * services/view/service.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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

//! The view service, processing high-level requests to Framerail for rendering web routes.
//!
//! This is one of the highest-level services, as it bundles the data from numerous
//! other services into responses which Framerail can use when rendering specific routes.
//! For instance, the `PageView` structure represents a request to any page (i.e. `/slug`),
//! gathering all the relevant data and sending it back in one convenient `PageViewOutput`
//! response.
//!
//! The service also contains the core method `get_view_context()`, which converts the
//! requesting domain and session token into a site and user, respectively.

use super::prelude::*;
use crate::models::site::Model as SiteModel;
use crate::services::{
    PageRevisionService, PageService, SessionService, TextService, UserService,
};

#[derive(Debug)]
pub struct ViewService;

impl ViewService {
    pub async fn page(
        ctx: &ServiceContext<'_>,
        GetPageView {
            hostname,
            route,
            session_token,
        }: GetPageView,
    ) -> Result<GetPageViewOutput> {
        tide::log::info!(
            "Getting page view data for host '{}', route '{:?}'",
            hostname,
            route,
        );

        let site: SiteModel = todo!(); // TODO HostService, get site

        // If None, means the main page for the site.
        let route = match route {
            Some(route) => route,
            None => PageRoute {
                slug: site.default_page.clone(),
                extra: String::new(),
            },
        };

        let options = todo!(); // parse page options (page_extra)

        let page =
            PageService::get(&ctx, site.site_id, Reference::Slug(cow!(route.slug)))
                .await
                .to_api()?;

        let page_revision =
            PageRevisionService::get_latest(&ctx, site.site_id, page.page_id)
                .await
                .to_api()?;

        let (wikitext, compiled_html) = try_join!(
            TextService::get(&ctx, &page_revision.wikitext_hash),
            TextService::get(&ctx, &page_revision.compiled_hash),
        )
        .to_api()?;

        let session = SessionService::get(&ctx, &session_token).await.to_api()?;

        // TODO Check if user-agent and IP match?

        let user = UserService::get(&ctx, Reference::Id(session.user_id))
            .await
            .to_api()?;

        Ok(GetPageViewOutput {
            site,
            page,
            page_revision,
            wikitext,
            compiled_html,
            session,
            user,
        })
    }
}
