/*
 * actions/locales.rs
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

use crate::config::Config;
use crate::locales::Localizations;
use unic_langid::LanguageIdentifier;

lazy_static! {
    /// The "primary" locale, to compare other locales against.
    ///
    /// This is defined as one which is always complete, containing
    /// every message key used by the application.
    ///
    /// Thus, we can compare all other locales to it, ensuring they
    /// are equal or subsets, raising errors on any new message keys,
    /// as they are either typos or removed keys.
    static ref PRIMARY_LOCALE: LanguageIdentifier = "en".parse().unwrap();
}

pub async fn validate_localization(config: &Config) {
    let localizations = Localizations::open(&config.localization_path)
        .await
        .expect("Unable to read localization files");

    todo!();
}
