/*
 * services/user/locale.rs
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

use crate::error::prelude::{Error, ErrorType, Result, ResultExt};
use crate::types::UserType;
use crate::utils::validate_locale;
use std::collections::HashSet;

const MAX_LOCALE_PREFERENCES: usize = 16;
const MAX_LOCALE_LENGTH: usize = 64;

pub(super) fn validate_locales<S: AsRef<str>>(
    user_type: UserType,
    locales: &[S],
) -> Result<()> {
    debug!(
        "Validating locales ({}) for user type {:?}",
        locales.len(),
        user_type,
    );

    if locales.len() > MAX_LOCALE_PREFERENCES {
        bail!(Error::new(
            "too many locale preferences",
            ErrorType::BadRequest
        ));
    }

    let make_error = || Error::new("failed to validate list of locales", ErrorType::User);
    let mut seen = HashSet::with_capacity(locales.len());

    for locale in locales {
        let locale = locale.as_ref();
        if locale.len() > MAX_LOCALE_LENGTH || !seen.insert(locale) {
            bail!(Error::new(
                "one or more locales are invalid",
                ErrorType::BadRequest
            ));
        }

        validate_locale(locale).or_raise(make_error)?;
    }

    let valid = match user_type {
        UserType::System => locales.is_empty(),
        UserType::Site => locales.len() == 1,
        _ => !locales.is_empty(),
    };

    if valid {
        Ok(())
    } else {
        bail!(Error::new(
            "one or more locales are invalid",
            ErrorType::BadRequest
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIXTEEN_LOCALES: [&str; MAX_LOCALE_PREFERENCES] = [
        "en", "ja", "fr", "de", "es", "it", "pt", "nl", "ru", "zh", "ko", "pl", "sv",
        "da", "fi", "no",
    ];

    #[test]
    fn accepts_bounded_distinct_regular_user_locales() {
        validate_locales(UserType::Regular, &SIXTEEN_LOCALES).unwrap();
    }

    #[test]
    fn rejects_too_many_locales_before_parsing_them() {
        let locales = ["not a locale"; MAX_LOCALE_PREFERENCES + 1];
        let error = validate_locales(UserType::Regular, &locales).unwrap_err();

        assert!(matches!(error.error_type, ErrorType::BadRequest));
        assert_eq!(error.message, "too many locale preferences");
    }

    #[test]
    fn rejects_oversized_and_duplicate_locales() {
        let oversized = "x".repeat(MAX_LOCALE_LENGTH + 1);
        let oversized_error =
            validate_locales(UserType::Regular, &[oversized.as_str()]).unwrap_err();
        let duplicate_error =
            validate_locales(UserType::Regular, &["en", "en"]).unwrap_err();

        assert!(matches!(oversized_error.error_type, ErrorType::BadRequest));
        assert!(matches!(duplicate_error.error_type, ErrorType::BadRequest));
    }

    #[test]
    fn preserves_user_type_locale_invariants() {
        validate_locales::<&str>(UserType::System, &[]).unwrap();
        validate_locales(UserType::Site, &["en"]).unwrap();

        assert!(validate_locales(UserType::System, &["en"]).is_err());
        assert!(validate_locales::<&str>(UserType::Site, &[]).is_err());
        assert!(validate_locales(UserType::Site, &["en", "ja"]).is_err());
        assert!(validate_locales::<&str>(UserType::Regular, &[]).is_err());
    }
}
