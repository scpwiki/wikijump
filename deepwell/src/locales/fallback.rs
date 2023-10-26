/*
 * locales/fallback.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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

//! Module to implement locale fallbacks.
//!
//! This is different than having a list of locales and simply trying each one.
//! Beyond that, this is another important component to finding a proper locale.
//!
//! Given some locale, it iterates through increasingly generic forms of it
//! until a match can be found (or not).
//!
//! The order followed is:
//! * Language, script, region, and variant (unmodified)
//! * Language, script, and region
//! * Language and script
//! * Language, region, and variant
//! * Language and region
//! * Language only
//!
//! For an already-generic locale (say just `ko`) this will emit that same locale
//! multiple times for each step. This is up to the caller to ignore; in
//! `iterate_locale_fallbacks()`; the first output to return `Ok(Some(_))` or `Err(_)`
//! will terminate iteration.

use unic_langid::LanguageIdentifier;

pub fn iterate_locale_fallbacks<F, T>(
    mut locale: LanguageIdentifier,
    mut f: F,
) -> Option<(LanguageIdentifier, T)>
where
    F: FnMut(&LanguageIdentifier) -> Option<T>,
{
    debug!("Iterating through locale fallbacks for {locale}");

    macro_rules! try_iter {
        () => {
            if let Some(result) = f(&locale) {
                return Some((locale, result));
            }
        };
    }

    // Storage of temporarily removed fields.
    let variants: Vec<_> = locale.variants().cloned().collect();
    let region;

    // Unmodified locale
    try_iter!();

    // Remove variant
    locale.clear_variants();
    try_iter!();

    // Remove region, keep value
    region = locale.region.take();
    try_iter!();

    // Re-add region and variant
    locale.region = region;
    locale.set_variants(&variants);
    try_iter!();

    // Remove variant
    locale.clear_variants();
    try_iter!();

    // Remove region
    locale.region = None;
    try_iter!();

    // No results
    None
}

#[test]
fn fallbacks() {
    fn check(locale: &str, expected: &[&str]) {
        let locale = locale.parse().expect("Unable to parse locale");
        let mut actual = Vec::new();

        iterate_locale_fallbacks::<_, ()>(locale, |locale| {
            actual.push(str!(locale));
            None
        });

        assert!(
            actual.iter().eq(expected),
            "Actual fallback locale list doesn't match expected\nactual:   {:?}\nexpected: {:?}",
            actual,
            expected,
        );
    }

    check("en", &["en", "en", "en", "en", "en", "en"]);
    check("en-US", &["en-US", "en-US", "en", "en-US", "en-US", "en"]);
}
