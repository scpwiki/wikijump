/*
 * services/site/structs.rs
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

use crate::models::alias::Model as AliasModel;
use crate::models::site::Model as SiteModel;
use crate::models::site_domain::Model as SiteDomainModel;
use crate::web::{ProvidedValue, Reference};
use ftml::layout::Layout;

#[derive(Deserialize, Debug, Clone)]
pub struct CreateSite {
    pub slug: String,
    pub name: String,
    pub tagline: String,
    pub description: String,
    pub layout: Option<Layout>,
    pub locale: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct CreateSiteOutput {
    pub site_id: i64,
    pub site_user_id: i64,
    pub slug: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetSite<'a> {
    pub site: Reference<'a>,
}

#[derive(Serialize, Debug, Clone)]
pub struct GetSiteOutput {
    #[serde(flatten)]
    pub site: SiteModel,
    pub aliases: Vec<AliasModel>,
    pub domains: Vec<SiteDomainModel>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateSite<'a> {
    pub site: Reference<'a>,
    pub user_id: i64,

    #[serde(flatten)]
    pub body: UpdateSiteBody,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct UpdateSiteBody {
    pub name: ProvidedValue<String>,
    pub slug: ProvidedValue<String>,
    pub tagline: ProvidedValue<String>,
    pub description: ProvidedValue<String>,
    pub locale: ProvidedValue<String>,
    pub layout: ProvidedValue<Option<Layout>>,
}
