/*
 * license.rs
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

//! Constant data for licenses usable by Wikijump sites.

use crate::error::prelude::*;
use crate::locales::Localizations;
use fluent::FluentArgs;
use unic_langid::LanguageIdentifier;

pub use crate::types::License;

impl License {
    pub fn url(self) -> &'static str {
        match self {
            License::CcBySa40 => "https://creativecommons.org/licenses/by-sa/4.0/",
            License::CcBy40 => "https://creativecommons.org/licenses/by/4.0/",
            License::CcByNd40 => "https://creativecommons.org/licenses/by-nd/4.0/",
            License::CcByNc40 => "https://creativecommons.org/licenses/by-nc/4.0/",
            License::CcByNcSa40 => "https://creativecommons.org/licenses/by-nc-sa/4.0/",
            License::CcByNcNd40 => "https://creativecommons.org/licenses/by-nc-nd/4.0/",
            License::CcBySa30 => "https://creativecommons.org/licenses/by-sa/3.0/",
            License::CcBy30 => "https://creativecommons.org/licenses/by/3.0/",
            License::CcByNd30 => "https://creativecommons.org/licenses/by-nd/3.0/",
            License::CcByNc30 => "https://creativecommons.org/licenses/by-nc/3.0/",
            License::CcByNcSa30 => "https://creativecommons.org/licenses/by-nc-sa/3.0/",
            License::CcByNcNd30 => "https://creativecommons.org/licenses/by-nc-nd/3.0/",
            License::CcBySa25 => "https://creativecommons.org/licenses/by-sa/2.5/",
            License::CcBy25 => "https://creativecommons.org/licenses/by/2.5/",
            License::CcByNd25 => "https://creativecommons.org/licenses/by-nd/2.5/",
            License::CcByNc25 => "https://creativecommons.org/licenses/by-nc/2.5/",
            License::CcByNcSa25 => "https://creativecommons.org/licenses/by-nc-sa/2.5/",
            License::CcByNcNd25 => "https://creativecommons.org/licenses/by-nc-nd/2.5/",
            License::GnuFdl13 => "https://www.gnu.org/licenses/fdl-1.3.html",
            License::GnuFdl12 => "https://www.gnu.org/licenses/old-licenses/fdl-1.2.html",
            License::GnuFdl11 => "https://www.gnu.org/licenses/old-licenses/fdl-1.1.html",
            License::Cc0 => "https://creativecommons.org/public-domain/cc0/",
        }
    }

    fn fluent_key(self) -> &'static str {
        match self {
            // Creative Commons 4.0
            License::CcBySa40 => "license.cc-by-sa-4-0",
            License::CcBy40 => "license.cc-by-4-0",
            License::CcByNd40 => "license.cc-by-nd-4-0",
            License::CcByNc40 => "license.cc-by-nc-4-0",
            License::CcByNcSa40 => "license.cc-by-nc-sa-4-0",
            License::CcByNcNd40 => "license.cc-by-nc-nd-4-0",

            // Creative Commons 3.0
            License::CcBySa30 => "license.cc-by-sa-3-0",
            License::CcBy30 => "license.cc-by-3-0",
            License::CcByNd30 => "license.cc-by-nd-3-0",
            License::CcByNc30 => "license.cc-by-nc-3-0",
            License::CcByNcSa30 => "license.cc-by-nc-sa-3-0",
            License::CcByNcNd30 => "license.cc-by-nc-nd-3-0",

            // Creative Commons 2.5
            License::CcBySa25 => "license.cc-by-sa-2-5",
            License::CcBy25 => "license.cc-by-2-5",
            License::CcByNd25 => "license.cc-by-nd-2-5",
            License::CcByNc25 => "license.cc-by-nc-2-5",
            License::CcByNcSa25 => "license.cc-by-nc-sa-2-5",
            License::CcByNcNd25 => "license.cc-by-nc-nd-2-5",

            // GNU Free Documentation License
            License::GnuFdl13 => "license.gnu-fdl-1-3",
            License::GnuFdl12 => "license.gnu-fdl-1-2",
            License::GnuFdl11 => "license.gnu-fdl-1-1",

            // Public Domain
            License::Cc0 => "license.cc0",
        }
    }

    pub fn translate(
        self,
        localization: &Localizations,
        locales: &[LanguageIdentifier],
    ) -> Result<String> {
        assert!(!locales.is_empty(), "No languages specified");
        let args = FluentArgs::new();
        let name = localization
            .translate(locales, self.fluent_key(), &args)
            .or_raise(|| {
                Error::new("failed to translate license name", ErrorType::License)
            })?;

        Ok(name.to_string())
    }
}

#[test]
fn license_urls_are_stable() {
    assert_eq!(
        License::CcBySa40.url(),
        "https://creativecommons.org/licenses/by-sa/4.0/",
    );
    assert_eq!(
        License::CcBy40.url(),
        "https://creativecommons.org/licenses/by/4.0/",
    );
    assert_eq!(
        License::CcByNd40.url(),
        "https://creativecommons.org/licenses/by-nd/4.0/",
    );
    assert_eq!(
        License::CcByNc40.url(),
        "https://creativecommons.org/licenses/by-nc/4.0/",
    );
    assert_eq!(
        License::CcByNcSa40.url(),
        "https://creativecommons.org/licenses/by-nc-sa/4.0/",
    );
    assert_eq!(
        License::CcByNcNd40.url(),
        "https://creativecommons.org/licenses/by-nc-nd/4.0/",
    );
    assert_eq!(
        License::CcBySa30.url(),
        "https://creativecommons.org/licenses/by-sa/3.0/",
    );
    assert_eq!(
        License::CcBy30.url(),
        "https://creativecommons.org/licenses/by/3.0/",
    );
    assert_eq!(
        License::CcByNd30.url(),
        "https://creativecommons.org/licenses/by-nd/3.0/",
    );
    assert_eq!(
        License::CcByNc30.url(),
        "https://creativecommons.org/licenses/by-nc/3.0/",
    );
    assert_eq!(
        License::CcByNcSa30.url(),
        "https://creativecommons.org/licenses/by-nc-sa/3.0/",
    );
    assert_eq!(
        License::CcByNcNd30.url(),
        "https://creativecommons.org/licenses/by-nc-nd/3.0/",
    );
    assert_eq!(
        License::CcBySa25.url(),
        "https://creativecommons.org/licenses/by-sa/2.5/",
    );
    assert_eq!(
        License::CcBy25.url(),
        "https://creativecommons.org/licenses/by/2.5/",
    );
    assert_eq!(
        License::CcByNd25.url(),
        "https://creativecommons.org/licenses/by-nd/2.5/",
    );
    assert_eq!(
        License::CcByNc25.url(),
        "https://creativecommons.org/licenses/by-nc/2.5/",
    );
    assert_eq!(
        License::CcByNcSa25.url(),
        "https://creativecommons.org/licenses/by-nc-sa/2.5/",
    );
    assert_eq!(
        License::CcByNcNd25.url(),
        "https://creativecommons.org/licenses/by-nc-nd/2.5/",
    );
    assert_eq!(
        License::GnuFdl13.url(),
        "https://www.gnu.org/licenses/fdl-1.3.html",
    );
    assert_eq!(
        License::GnuFdl12.url(),
        "https://www.gnu.org/licenses/old-licenses/fdl-1.2.html",
    );
    assert_eq!(
        License::GnuFdl11.url(),
        "https://www.gnu.org/licenses/old-licenses/fdl-1.1.html",
    );
    assert_eq!(
        License::Cc0.url(),
        "https://creativecommons.org/public-domain/cc0/",
    );
}

#[test]
fn license_fluent_keys_are_stable() {
    assert_eq!(License::CcBySa40.fluent_key(), "license.cc-by-sa-4-0");
    assert_eq!(License::CcBy40.fluent_key(), "license.cc-by-4-0");
    assert_eq!(License::CcByNd40.fluent_key(), "license.cc-by-nd-4-0");
    assert_eq!(License::CcByNc40.fluent_key(), "license.cc-by-nc-4-0");
    assert_eq!(License::CcByNcSa40.fluent_key(), "license.cc-by-nc-sa-4-0",);
    assert_eq!(License::CcByNcNd40.fluent_key(), "license.cc-by-nc-nd-4-0",);
    assert_eq!(License::CcBySa30.fluent_key(), "license.cc-by-sa-3-0");
    assert_eq!(License::CcBy30.fluent_key(), "license.cc-by-3-0");
    assert_eq!(License::CcByNd30.fluent_key(), "license.cc-by-nd-3-0");
    assert_eq!(License::CcByNc30.fluent_key(), "license.cc-by-nc-3-0");
    assert_eq!(License::CcByNcSa30.fluent_key(), "license.cc-by-nc-sa-3-0",);
    assert_eq!(License::CcByNcNd30.fluent_key(), "license.cc-by-nc-nd-3-0",);
    assert_eq!(License::CcBySa25.fluent_key(), "license.cc-by-sa-2-5");
    assert_eq!(License::CcBy25.fluent_key(), "license.cc-by-2-5");
    assert_eq!(License::CcByNd25.fluent_key(), "license.cc-by-nd-2-5");
    assert_eq!(License::CcByNc25.fluent_key(), "license.cc-by-nc-2-5");
    assert_eq!(License::CcByNcSa25.fluent_key(), "license.cc-by-nc-sa-2-5",);
    assert_eq!(License::CcByNcNd25.fluent_key(), "license.cc-by-nc-nd-2-5",);
    assert_eq!(License::GnuFdl13.fluent_key(), "license.gnu-fdl-1-3");
    assert_eq!(License::GnuFdl12.fluent_key(), "license.gnu-fdl-1-2");
    assert_eq!(License::GnuFdl11.fluent_key(), "license.gnu-fdl-1-1");
    assert_eq!(License::Cc0.fluent_key(), "license.cc0");
}
