/*
 * language.rs
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

use axum::http::header::HeaderMap;

/// Parse the `Accept-Language` header.
/// If there are no languages, or there is no header, then use English.
pub fn parse_accept_language(headers: &HeaderMap) -> Vec<String> {
    const FALLBACK_LANGUAGE: &str = "en";

    fn get_header_value(headers: &HeaderMap) -> Option<&str> {
        match headers.get("accept-language") {
            Some(value) => value.to_str().ok(),
            None => None,
        }
    }

    let header_value = match get_header_value(headers) {
        Some(value) => value,
        None => return vec![str!(FALLBACK_LANGUAGE)],
    };

    let mut languages = accept_language::parse(header_value);
    if languages.is_empty() {
        languages.push(str!(FALLBACK_LANGUAGE));
    }

    languages
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn missing_accept_language_uses_english() {
        let headers = HeaderMap::new();

        assert_eq!(parse_accept_language(&headers), vec!["en"]);
    }

    #[test]
    fn invalid_accept_language_uses_english() {
        let mut headers = HeaderMap::new();
        headers.insert("accept-language", HeaderValue::from_bytes(b"\xFF").unwrap());

        assert_eq!(parse_accept_language(&headers), vec!["en"]);
    }

    #[test]
    fn parses_accept_language_preferences() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "accept-language",
            HeaderValue::from_static("fr-CA, fr;q=0.8, en;q=0.4"),
        );

        assert_eq!(parse_accept_language(&headers), vec!["fr-CA", "fr", "en"]);
    }
}
