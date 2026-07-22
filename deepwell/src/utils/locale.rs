/*
 * utils/locale.rs
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

use crate::error::prelude::*;
use unic_langid::LanguageIdentifier;

const WIKIDOT_JAPANESE_CORRECTIONS_LOCALE: &str = "ja-corrections";

fn parse_locale_identifier(locale_str: &str) -> Option<LanguageIdentifier> {
    let language_identifier = match locale_str {
        WIKIDOT_JAPANESE_CORRECTIONS_LOCALE => "ja",
        _ => locale_str,
    };

    LanguageIdentifier::from_bytes(language_identifier.as_bytes()).ok()
}

/// Ensure the given locale string is valid, returning the parsed locale.
/// If it is invalid, then the appropriate `Error` variant is returned.
pub fn validate_locale(locale_str: &str) -> Result<LanguageIdentifier> {
    parse_locale_identifier(locale_str).ok_or_raise(|| {
        Error::new(
            format!("failed to validate locale for '{locale_str}'"),
            ErrorType::LocaleInvalid {
                locale: str!(locale_str),
            },
        )
    })
}

/// Helper function to convert an array of strings to a list of locales.
///
/// Empty locales lists _are_ allowed, since we have not
/// yet checked the user's locale preferences.
pub fn parse_locales<S: AsRef<str>>(
    locales_str: &[S],
) -> Result<Vec<LanguageIdentifier>> {
    let mut locales = Vec::with_capacity(locales_str.len());
    for locale_str in locales_str {
        let locale_str = locale_str.as_ref();
        let locale = parse_locale_identifier(locale_str).ok_or_raise(|| {
            Error::new(
                format!("failed to parse locale '{locale_str}'"),
                ErrorType::LocaleInvalid {
                    locale: str!(locale_str),
                },
            )
        })?;

        locales.push(locale);
    }

    Ok(locales)
}

#[test]
fn validate_locale_accepts_valid_locale() {
    let locale = validate_locale("en-US").unwrap();

    assert_eq!(locale.to_string(), "en-US");
}

#[test]
fn validate_locale_maps_wikidot_japanese_corrections_to_japanese() {
    let locale = validate_locale("ja-corrections").unwrap();

    assert_eq!(locale.to_string(), "ja");
}

#[test]
fn validate_locale_rejects_invalid_locale() {
    let error = validate_locale("not a locale").unwrap_err();

    assert!(error.to_string().contains("failed to validate locale"));
}

#[test]
fn parse_locales_accepts_empty_and_valid_locale_lists() {
    let empty: Vec<LanguageIdentifier> = parse_locales::<&str>(&[]).unwrap();
    assert!(empty.is_empty());

    let locales = parse_locales(&["en-US", "ja"]).unwrap();
    assert_eq!(
        locales.iter().map(ToString::to_string).collect::<Vec<_>>(),
        vec!["en-US", "ja"],
    );
}

#[test]
fn parse_locales_maps_wikidot_japanese_corrections_to_japanese() {
    let locales = parse_locales(&["ja-corrections", "en"]).unwrap();

    assert_eq!(
        locales.iter().map(ToString::to_string).collect::<Vec<_>>(),
        vec!["ja", "en"],
    );
}

#[test]
fn parse_locales_rejects_invalid_locale() {
    let error = parse_locales(&["en-US", "not a locale"]).unwrap_err();

    assert!(error.to_string().contains("failed to parse locale"));
}
