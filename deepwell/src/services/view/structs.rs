/*
 * services/view/structs.rs
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

use super::options::PageOptions;
use super::prelude::*;
use crate::models::page::Model as PageModel;
use crate::models::page_revision::Model as PageRevisionModel;
use crate::models::session::Model as SessionModel;
use crate::models::site::Model as SiteModel;
use crate::models::user::Model as UserModel;

// TODO replace with actual user permissions type
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct UserPermissions;

impl UserPermissions {
    pub fn is_banned(self) -> bool {
        // TODO value from struct
        false
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetPageView {
    pub domain: String,
    pub session_token: Option<String>,
    pub route: Option<PageRoute>,
    pub locales: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PageRoute {
    pub slug: String,
    pub extra: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum GetPageViewOutput {
    PageFound {
        #[serde(flatten)]
        viewer: Viewer,
        options: PageOptions,
        page: PageModel,
        page_revision: PageRevisionModel,
        redirect_page: Option<String>,
        wikitext: String,
        compiled_html: String,
    },

    PageMissing {
        #[serde(flatten)]
        viewer: Viewer,
        options: PageOptions,
        redirect_page: Option<String>,
        wikitext: String,
        compiled_html: String,
    },

    PagePermissions {
        #[serde(flatten)]
        viewer: Viewer,
        options: PageOptions,
        redirect_page: Option<String>,
        compiled_html: String,
        banned: bool,
    },

    SiteMissing {
        html: String,
    },
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetUserView<'a> {
    pub domain: String,
    pub session_token: Option<String>,
    pub user: Option<Reference<'a>>,
    pub locales: Vec<String>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum GetUserViewOutput {
    UserFound {
        #[serde(flatten)]
        viewer: Viewer,
        user: UserModel,
    },

    UserMissing {
        #[serde(flatten)]
        viewer: Viewer,
    },

    SiteMissing {
        html: String,
    },
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetSiteView {
    pub domain: String,
    pub session_token: Option<String>,
    pub locales: Vec<String>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum GetSiteViewOutput {
    SiteFound {
        #[serde(flatten)]
        viewer: Viewer,
    },

    SitePermissions {
        #[serde(flatten)]
        viewer: Viewer,
        html: String,
    },

    SiteMissing {
        html: String,
    },
}

#[derive(Debug, Clone)]
pub enum ViewerResult {
    FoundSite(Viewer),
    MissingSite(String),
}

#[derive(Serialize, Debug, Clone)]
pub struct Viewer {
    pub site: SiteModel,
    pub redirect_site: Option<String>,
    pub user_session: Option<UserSession>,
}

#[derive(Serialize, Debug, Clone)]
pub struct UserSession {
    pub session: SessionModel,
    pub user: UserModel,
    pub user_permissions: UserPermissions,
}
