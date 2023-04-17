/*
 * services/view/structs.rs
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

use super::options::PageOptions;
use crate::models::page::Model as PageModel;
use crate::models::page_revision::Model as PageRevisionModel;
use crate::models::session::Model as SessionModel;
use crate::models::site::Model as SiteModel;
use crate::models::user::Model as UserModel;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetPageView {
    pub domain: String,
    pub session_token: String,
    pub route: Option<PageRoute>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PageRoute {
    pub slug: String,
    pub extra: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetPageViewOutput {
    #[serde(flatten)]
    pub viewer: Viewer,
    pub options: PageOptions,

    pub page: PageModel,
    pub page_revision: PageRevisionModel,
    pub wikitext: String,
    pub compiled_html: String,
    pub redirect_page: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Viewer {
    pub site: SiteModel,
    pub session: SessionModel,
    pub user: UserModel,
    pub user_permissions: (),
    pub redirect_site: Option<String>,
}
