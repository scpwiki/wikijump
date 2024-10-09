/*
 * services/filter/struct.rs
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

use crate::models::filter;
use crate::types::Maybe;
use sea_orm::{ColumnTrait, Condition};

/// Denotes what class of filter is being selected.
///
/// In the database, a value of `NULL` in the `site_id` column
/// indicates that this is a platform filter, meaning it applies
/// for all sites. If it has a value then
///
/// Previously this value was stored using `Option<i64>` directly
/// mirroring how it was stored in the database. However, this had
/// some issues:
///
/// One is that it is semantically unclear, and similar to `Maybe`,
/// we should make a cheap enum wrapper to provide semantics to what is
/// essentially just an `Option<i64>`.
///
/// It also does not allow a consumer to select both all the global filters
/// as well as the filters for a site. When checking a page edit, for
/// instance, you want both this site's filters, as well as those which
/// apply to all sites.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FilterClass {
    /// This filter applies to all sites on the platform.
    Platform,

    /// This filter applies only to the site with the given ID.
    Site(i64),

    /// This filter combines all platform and site filters.
    ///
    /// It is an optimization which allows the regular expressions
    /// to be merged into one `RegexSet` for improved performance.
    PlatformAndSite(i64),
}

impl FilterClass {
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            FilterClass::Platform => "platform",
            FilterClass::Site(_) => "site",
            FilterClass::PlatformAndSite(_) => "platform and site",
        }
    }

    /// Converts this filter class into a condition which can be used for queries.
    pub fn to_condition(self) -> Condition {
        let mut condition = Condition::any();

        // If we want platform filters
        if matches!(
            self,
            FilterClass::Platform | FilterClass::PlatformAndSite(_),
        ) {
            condition = condition.add(filter::Column::SiteId.is_null());
        }

        // If we want site filters
        if let FilterClass::Site(site_id) | FilterClass::PlatformAndSite(site_id) = self {
            condition = condition.add(filter::Column::SiteId.eq(site_id));
        }

        condition
    }
}

impl From<Option<i64>> for FilterClass {
    #[inline]
    fn from(site_id: Option<i64>) -> FilterClass {
        match site_id {
            None => FilterClass::Platform,
            Some(site_id) => FilterClass::Site(site_id),
        }
    }
}

/// Denotes what kind of object this filter is checking.
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

    /// Filters on user's email address.
    ///
    /// * For a system filter, prevent registration of a user with this email address.
    /// * For a site filter, prevent joining of a user with this email address.
    Email,

    /// Filters on pages.
    /// Prevents a page edit from going through if it trips this filter.
    Page,

    /// Filters on files.
    /// Prevents a file upload or edit from going through if it trips this filter.
    File,

    /// Filters on forum contents.
    /// Prevents a forum post or edit from going through if it trips this filter.
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
            FilterType::Email => filter::Column::AffectsEmail,
            FilterType::Page => filter::Column::AffectsPage,
            FilterType::File => filter::Column::AffectsFile,
            FilterType::Forum => filter::Column::AffectsForum,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct CreateFilter {
    pub affects_user: bool,
    pub affects_email: bool,
    pub affects_page: bool,
    pub affects_file: bool,
    pub affects_forum: bool,
    pub case_sensitive: bool,
    pub regex: String,
    pub description: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateFilter {
    pub filter_id: i64,
    pub affects_user: Maybe<bool>,
    pub affects_email: Maybe<bool>,
    pub affects_page: Maybe<bool>,
    pub affects_file: Maybe<bool>,
    pub affects_forum: Maybe<bool>,
    pub case_sensitive: Maybe<bool>,
    pub regex: Maybe<String>,
    pub description: Maybe<String>,
}
