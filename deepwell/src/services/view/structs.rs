/*
 * services/view/structs.rs
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

use super::options::PageOptions;
use super::prelude::*;
use crate::models::page::Model as PageModel;
use crate::models::page_revision::Model as PageRevisionModel;
use crate::models::session::Model as SessionModel;
use crate::models::site::Model as SiteModel;
use crate::models::user::Model as WikijumpUserModel;
use crate::services::relation::PageAttribution;
use crate::services::user::User;

// NOTE: Any changes to the output structures here, including the variant names,
//       MUST be reflected in framerail!

#[derive(Deserialize, Debug, Clone)]
pub struct GetPageView {
    pub site_id: i64,
    pub session_token: Option<String>,
    pub route: Option<PageRoute>,
    pub locales: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PageRoute {
    pub slug: String,
    pub extra: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetPreloadView {
    pub site_id: i64,
    pub session_token: Option<String>,
    pub locales: Vec<String>,
}

/// Yield common preload data for any views
///
/// See also framerail src/lib/server/load/preload.ts
#[derive(Serialize, Debug, Clone)]
pub struct GetPreloadViewOutput {
    #[serde(flatten)]
    pub viewer: Viewer,
}

/// Yield information for a page view, depending on the status of the page.
/// For instance, if a page is missing, there is no revision data but we do
/// still need to display the "this page doesn't exist" content.
///
/// See also framerail src/lib/server/load/page.ts and src/routes/+error.svelte
///
/// Note that compiled_xxx_bar_html is Option because None means that this page
/// does not have that nav bar / it is disabled in this context.
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum GetPageViewOutput {
    Found {
        options: PageOptions,
        page: PageModel,
        page_revision: PageRevisionModel,
        attributions: Vec<PageAttribution>,
        redirect_page: Option<String>,
        wikitext: String,
        compiled_body_html: String,
        compiled_top_bar_html: Option<String>,
        compiled_side_bar_html: Option<String>,
    },

    Missing {
        options: PageOptions,
        redirect_page: Option<String>,
        wikitext: String,
        compiled_body_html: String,
        compiled_top_bar_html: Option<String>,
        compiled_side_bar_html: Option<String>,
    },

    Permissions {
        options: PageOptions,
        redirect_page: Option<String>,
        compiled_body_html: String,
        compiled_top_bar_html: Option<String>,
        compiled_side_bar_html: Option<String>,
        banned: bool,
    },
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetUserView<'a> {
    pub site_id: i64,
    pub session_token: Option<String>,
    pub user: Option<Reference<'a>>,
    pub locales: Vec<String>,
}

// See also framerail src/lib/server/load/admin.ts and src/routes/[x+2d]/admin/+error.svelte
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum GetUserViewOutput {
    UserFound { user: User },
    UserMissing,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetAdminView {
    pub site_id: i64,
    pub session_token: Option<String>,
    pub locales: Vec<String>,
}

// See also framerail src/lib/server/load/admin.ts and src/routes/[x+2d]/user/+error.svelte
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum GetAdminViewOutput {
    SiteFound,

    AdminPermissions { html: String },
}

#[derive(Serialize, Debug, Clone)]
pub struct Viewer {
    pub site: SiteModel,
    pub site_file_domain: String,
    pub license_name: String,
    pub license_url: &'static str,
    pub user_session: Option<UserSession>,
}

#[derive(Serialize, Debug, Clone)]
pub struct UserSession {
    pub session: SessionModel,
    pub user: WikijumpUserModel,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViewType {
    Preload,
    Page,
    User,
    Admin,
}
