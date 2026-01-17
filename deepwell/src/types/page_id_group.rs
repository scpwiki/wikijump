/*
 * types/page_id_group.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2025 Wikijump Team
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

use crate::models::page::Model as PageModel;

/// A structure storing the three IDs for a page.
///
/// This is needed to represent not just a particular page, but also
/// its various levels of silo (site, category, then the page itself).
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct PageIdGroup {
    pub site_id: i64,
    pub category_id: i64,
    pub page_id: i64,
}

impl PageIdGroup {
    #[inline]
    pub fn from_page_model(model: &PageModel) -> Self {
        model.into()
    }
}

impl From<&'_ PageModel> for PageIdGroup {
    fn from(page: &PageModel) -> PageIdGroup {
        PageIdGroup {
            site_id: page.site_id,
            category_id: page.page_category_id,
            page_id: page.page_id,
        }
    }
}
