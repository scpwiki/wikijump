/*
 * path.rs
 *
 * Wilson's Web Server - Serves a zoo of user-generated content
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

use axum::http::Uri;

/// Extracts the path and query from a URI.
///
/// Since `Uri::path_and_query()` returns an `Option`,
/// we need a match statement to get the path if there
/// is no query string portion.
pub fn get_path(uri: &Uri) -> &str {
    match uri.path_and_query() {
        Some(path_and_query) => path_and_query.as_str(),
        None => uri.path(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_without_query_returns_path() {
        let uri: Uri = "/scp-173".parse().unwrap();

        assert_eq!(get_path(&uri), "/scp-173");
    }

    #[test]
    fn path_with_query_preserves_query() {
        let uri: Uri = "/scp-173?theme=classic".parse().unwrap();

        assert_eq!(get_path(&uri), "/scp-173?theme=classic");
    }

    #[test]
    fn authority_form_uri_falls_back_to_path() {
        let uri: Uri = "https://example.wikijump.com".parse().unwrap();

        assert_eq!(get_path(&uri), "/");
    }
}
