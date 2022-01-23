/*
 * web/category.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2021 Wikijump Team
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

/// Splits a normalized slug into the category and page portions.
///
/// This finds the first `:` in the full slug and returns everything
/// up to that as the category slug.
///
/// Normal slugs do not have an explicit `_default`, so they
/// should lack a `:` entirely.
pub fn split_category(slug: &str) -> (Option<&str>, &str) {
    match slug.find(':') {
        None => (None, slug),
        Some(idx) => {
            let (category, page) = slug.split_at(idx);
            (Some(category), page)
        }
    }
}

/// Retrieves the category portion of a slug, if it exists.
#[inline]
pub fn get_category(slug: &str) -> Option<&str> {
    split_category(slug).0
}

/// Retrieves the category name for a slug.
#[inline]
pub fn get_category_name(slug: &str) -> &str {
    get_category(slug).unwrap_or("_default")
}

/// Trims off the `_default:` category if present.
pub fn trim_default(slug: &str) -> &str {
    slug.strip_prefix("_default:").unwrap_or(slug)
}
