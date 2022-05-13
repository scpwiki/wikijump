/*
 * web/fetch_limit/mod.rs
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

use super::PageDetailsQuery;

mod de;

/// Represents the number of items to return in this request.
///
/// The default value is 10, and the maximum value is 100.
#[derive(Serialize, Debug, Copy, Clone, PartialEq, Eq)]
pub struct FetchLimit(u16);

impl From<FetchLimit> for u16 {
    #[inline]
    fn from(limit: FetchLimit) -> u16 {
        limit.0
    }
}

impl From<FetchLimit> for u64 {
    #[inline]
    fn from(limit: FetchLimit) -> u64 {
        limit.0.into()
    }
}

impl Default for FetchLimit {
    #[inline]
    fn default() -> Self {
        FetchLimit(10)
    }
}

pub type FetchDetailsQuery = PageDetailsQuery;

#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct FetchLimitQuery {
    /// Include the wikitext in the page output.
    pub wikitext: bool,

    /// Include the compiled HTML in the page output.
    #[serde(alias = "compiled")]
    pub compiled_html: bool,

    /// How many items to pull in this query.
    pub limit: FetchLimit,
}

// NOTE: #[serde(flatten)] on FetchDetailsQuery as a field
//       doesn't seem to work here, so we're just pasting it in.
