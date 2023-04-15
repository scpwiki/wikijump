/*
 * methods/view.rs
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

use super::prelude::*;
use crate::models::page::Model as PageModel;
use crate::models::page_revision::Model as PageRevisionModel;
use crate::models::session::Model as SessionModel;
use crate::models::site::Model as SiteModel;
use crate::models::user::Model as UserModel;
use crate::web::Reference;

/// Returns relevant context for rendering a page from a processed web request.
pub async fn view_page(mut req: ApiRequest) -> ApiResponse {
    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct WebPageGet {
        hostname: String,
        page: Option<Page>,
        session_token: String,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct Page {
        slug: String,
        extra: String,
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct WebPageGetOutput {
        site: SiteModel,
        page: PageModel,
        page_revision: PageRevisionModel,
        wikitext: String,
        compiled_html: String,
        session: SessionModel,
        user: UserModel,
    }

    let txn = req.database().begin().await?;
    let ctx = ServiceContext::new(&req, &txn);
    let WebPageGet {
        hostname,
        page: page_route,
        session_token,
    } = req.body_json().await?;

    let site: SiteModel = todo!(); // TODO HostService, get site
    let page_route = match page_route {
        Some(page_route) => page_route,
        None => Page {
            slug: site.default_page.clone(),
            extra: String::new(),
        },
    };

    let options = todo!(); // parse page options (page_extra)

    let page =
        PageService::get(&ctx, site.site_id, Reference::Slug(cow!(page_route.slug)))
            .await
            .to_api()?;

    let page_revision = PageRevisionService::get_latest(&ctx, site.site_id, page.page_id)
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

    let body = Body::from_json(&WebPageGetOutput {
        site,
        page,
        page_revision,
        wikitext,
        compiled_html,
        session,
        user,
    })?;

    Ok(body.into())
}
