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
//! The service also contains the core method `ViewService::get_viewer()`, which converts the
//! requesting domain and session token into a site and user, respectively.

use super::prelude::*;
use crate::models::site::Model as SiteModel;
use crate::services::{
    DomainService, PageRevisionService, PageService, SessionService, TextService,
    UserService,
};

#[derive(Debug)]
pub struct ViewService;

impl ViewService {
    pub async fn page(
        ctx: &ServiceContext<'_>,
        GetPageView {
            domain,
            route,
            session_token,
        }: GetPageView,
    ) -> Result<GetPageViewOutput> {
        tide::log::info!(
            "Getting page view data for domain '{}', route '{:?}'",
            domain,
            route,
        );

        let Viewer {
            site,
            session,
            user,
            user_permissions,
        } = Self::get_viewer(ctx, &domain, &session_token).await?;

        // If None, means the main page for the site. Pull from site data.
        let (page_slug, page_extra): (&str, &str) = match route {
            None => (&site.default_page, ""),
            Some(PageRoute {
                ref slug,
                ref extra,
            }) => (slug, extra),
        };

        let options = todo!(); // parse page options (page_extra)

        let page = PageService::get(&ctx, site.site_id, Reference::Slug(cow!(page_slug)))
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

        // TODO Check if user-agent and IP match?

        let user = UserService::get(&ctx, Reference::Id(session.user_id))
            .await
            .to_api()?;

        Ok(GetPageViewOutput {
            viewer: Viewer {
                site,
                session,
                user,
                user_permissions,
            },
            page,
            page_revision,
            wikitext,
            compiled_html,
        })
    }

    pub async fn get_viewer(
        ctx: &ServiceContext<'_>,
        domain: &str,
        session_token: &str,
    ) -> Result<Viewer> {
        tide::log::info!("Getting view context from domain '{domain}' and session token");

        let (site, session) = try_join!(
            DomainService::site_from_domain(&ctx, domain),
            SessionService::get(&ctx, session_token),
        )
        .to_api()?;

        let user = UserService::get(&ctx, Reference::Id(session.user_id)).await?;
        let user_permissions = (); // TODO add user permissions, get schem for user and site

        Ok(Viewer {
            site,
            session,
            user,
            user_permissions: (),
        })
    }
}
