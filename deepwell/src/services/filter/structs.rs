/*
 * services/filter/struct.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2022 Wikijump Team
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

use crate::models::filter;
use crate::web::ProvidedValue;

/// Denotes what kind of filter
///
/// These are stored in the `filter` tables as boolean toggles for each
/// filter entry, but here we imagine them as a separate class or type
/// of filter.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum FilterType {
    /// Filters on user name and slug.
    ///
    /// * For a system filter, prevent registration of a user with this name or slug.
    /// * For a site filter, prevent joining of a user with this name or slug.
    User,

    /// Filters on a page.
    ///
    /// Prevents a page edit from going through if it trips this filter.
    /// Whether it is system or site affects what scope of pages are checked.
    Page,

    /// Filters on a forum.
    ///
    /// Prevents a forum post or edit from going through if it trips this filter.
    /// Whether it is system or site affects what scope of pages are checked.
    Forum,
}

impl FilterType {
    #[inline]
    pub fn into_column(self) -> filter::Column {
        self.into()
    }
}

impl From<FilterType> for filter::Column {
    #[inline]
    fn from(filter_type: FilterType) -> filter::Column {
        match filter_type {
            FilterType::User => filter::Column::AffectsUser,
            FilterType::Page => filter::Column::AffectsPage,
            FilterType::Forum => filter::Column::AffectsForum,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct CreateFilter {
    pub site_id: Option<i64>,
    pub affects_user: bool,
    pub affects_page: bool,
    pub affects_forum: bool,
    pub regex: String,
    pub reason: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateFilter {
    pub filter_id: i64,
    pub affects_user: ProvidedValue<bool>,
    pub affects_page: ProvidedValue<bool>,
    pub affects_forum: ProvidedValue<bool>,
    pub regex: ProvidedValue<String>,
    pub reason: ProvidedValue<String>,
}
