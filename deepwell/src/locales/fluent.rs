/*
 * locales/fluent.rs
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

use super::error::{fluent_load_err, LocalizationLoadError};
use super::fallback::iterate_locale_fallbacks;
use crate::services::Error as ServiceError;
use fluent::{bundle, FluentArgs, FluentMessage, FluentResource};
use fluent_syntax::ast::Pattern;
use intl_memoizer::concurrent::IntlLangMemoizer;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{self, Debug, Display};
use std::path::{Path, PathBuf};
use tokio::fs;
use unic_langid::LanguageIdentifier;

pub type FluentBundle = bundle::FluentBundle<FluentResource, IntlLangMemoizer>;

pub struct Localizations {
    bundles: HashMap<LanguageIdentifier, FluentBundle>,
}

impl Localizations {
    pub async fn open<P: Into<PathBuf>>(
        directory: P,
    ) -> Result<Self, LocalizationLoadError> {
        debug!("Reading Fluent localization directory...");

        let directory = {
            let mut path = directory.into();
            path.push("fluent");
            path
        };

        let mut bundles = HashMap::new();
        let mut entries = fs::read_dir(&directory).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            Self::load_component(&mut bundles, &path).await?;
        }

        Ok(Localizations { bundles })
    }

    async fn load_component(
        bundles: &mut HashMap<LanguageIdentifier, FluentBundle>,
        directory: &Path,
    ) -> Result<(), LocalizationLoadError> {
        debug!("Reading component at {}", directory.display());
        let mut entries = fs::read_dir(directory).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Get locale from filename
            let locale_name = path
                .file_stem()
                .expect("No base name in locale path")
                .to_str()
                .expect("Path is not valid UTF-8");

            debug!("Loading locale {locale_name}");
            let locale: LanguageIdentifier = locale_name.parse()?;

            // Read and parse localization strings
            let source = fs::read_to_string(&path).await?;
            let resource = FluentResource::try_new(source).map_err(fluent_load_err)?;

            // Create or modify bundle
            let locale2 = locale.clone();
            let bundle = bundles
                .entry(locale)
                .or_insert_with(|| FluentBundle::new_concurrent(vec![locale2]));

            bundle.add_resource(resource).map_err(fluent_load_err)?;
        }

        Ok(())
    }

    /// Parses a message key to split the path from the attribute, if present.
    ///
    /// Fluent does not permit multiple periods in a message key, having multiple
    /// is a logical error.
    pub fn parse_selector(key: &str) -> (&str, Option<&str>) {
        match key.find('.') {
            None => (key, None),
            Some(idx) => {
                let (path, rest) = key.split_at(idx);
                let attribute = &rest[1..]; // This is safe because '.' is one byte
                (path, Some(attribute))
            }
        }
    }

    /// Retrieve the specified Fluent bundle and message.
    fn get_message<'a>(
        &'a self,
        locale: &LanguageIdentifier,
        path: &str,
    ) -> Result<(&'a FluentBundle, FluentMessage), ServiceError> {
        match self.bundles.get(locale) {
            None => Err(ServiceError::LocaleMissing),
            Some(bundle) => match bundle.get_message(path) {
                Some(message) => Ok((bundle, message)),
                None => Err(ServiceError::LocaleMessageMissing),
            },
        }
    }

    /// Retrieve the specified Fluent pattern from the associated bundle.
    fn get_pattern<'a>(
        &'a self,
        locale: &LanguageIdentifier,
        path: &str,
        attribute: Option<&str>,
    ) -> Result<(&'a FluentBundle, &'a Pattern<&'a str>), ServiceError> {
        debug!("Checking for translation patterns in locale {locale}");

        // Get appropriate message and bundle, if found
        let (bundle, message) = self.get_message(locale, path)?;

        // Get pattern from message, if present
        let pattern = match attribute {
            Some(attribute) => match message.get_attribute(attribute) {
                Some(attrib) => attrib.value(),
                None => return Err(ServiceError::LocaleMessageAttributeMissing),
            },
            None => match message.value() {
                Some(pattern) => pattern,
                None => return Err(ServiceError::LocaleMessageValueMissing),
            },
        };

        Ok((bundle, pattern))
    }

    /// Iterate through a list of locales, and try to find the first existing pattern.
    fn get_pattern_locales<'a, L, I>(
        &'a self,
        locales: I,
        path: &str,
        attribute: Option<&str>,
    ) -> Result<(LanguageIdentifier, &'a FluentBundle, &'a Pattern<&'a str>), ServiceError>
    where
        L: AsRef<LanguageIdentifier> + 'a,
        I: IntoIterator<Item = L>,
    {
        let mut last_error = ServiceError::NoLocalesSpecified; // Occurs if locales is empty

        // Iterate through each locale to try
        for locale_ref in locales {
            // Iterate through each fallback locale (e.g. ['fr-BE'] -> ['fr-BE', 'fr'])
            let locale = locale_ref.as_ref();
            let result = iterate_locale_fallbacks(locale.clone(), |locale| {
                // Try and get bundle and pattern, if it exists
                match self.get_pattern(locale, path, attribute) {
                    Err(error) => {
                        debug!("Pattern not found for locale {locale}: {error}");
                        last_error = error;
                        None
                    }
                    Ok((bundle, pattern)) => {
                        info!("Found pattern for locale {locale}");
                        Some((bundle, pattern))
                    }
                }
            });

            if let Some((locale, (bundle, pattern))) = result {
                return Ok((locale, bundle, pattern));
            }
        }

        warn!("Could not find any translation patterns: {last_error}");
        Err(last_error)
    }

    /// Translates the message, given the message key and formatting arguments.
    ///
    /// At least one locale must be specified. If no translation can be found for
    /// the given locale, then progressively more generic forms are attempted. If
    /// no translations can be found even for all fallback locales, an error is
    /// returned.
    pub fn translate<'a, L, I>(
        &'a self,
        locales: I,
        key: &str,
        args: &'a FluentArgs<'a>,
    ) -> Result<Cow<'a, str>, ServiceError>
    where
        L: AsRef<LanguageIdentifier> + Display + 'a,
        I: IntoIterator<Item = L>,
    {
        // Parse translation key
        let (path, attribute) = Self::parse_selector(key);
        info!(
            "Checking message path {}, attribute {} for a matching locale",
            path,
            attribute.unwrap_or("<none>"),
        );

        // Find pattern for translating
        let (locale, bundle, pattern) =
            self.get_pattern_locales(locales, path, attribute)?;

        // Format using pattern
        let mut errors = vec![];
        let output = bundle.format_pattern(pattern, Some(args), &mut errors);

        // Log any errors
        if !errors.is_empty() {
            warn!("Errors formatting message for locale {locale}, message key {key}");

            for (key, value) in args.iter() {
                warn!("Passed formatting argument: {key} -> {value:?}");
            }

            for error in errors {
                warn!("Message formatting error: {error}");
            }
        }

        // We could return the locale used if we wished, but presently we discard this information.
        // Change the return type of this method and its users if you need this information.
        let _ = locale;

        // Done
        Ok(output)
    }

    /// A variant of `translate()` which returns `Option`.
    ///
    /// This way, if a translation cannot be found, instead of returning
    /// an error, it instead returns `Ok(None)`.
    pub fn translate_option<'a, L, I>(
        &'a self,
        locales: I,
        key: &str,
        args: &'a FluentArgs<'a>,
    ) -> Result<Option<Cow<'a, str>>, ServiceError>
    where
        L: AsRef<LanguageIdentifier> + Display + 'a,
        I: IntoIterator<Item = L>,
    {
        match self.translate(locales, key, args) {
            Ok(translation) => Ok(Some(translation)),
            Err(
                ServiceError::LocaleMissing
                | ServiceError::LocaleMessageMissing
                | ServiceError::LocaleMessageValueMissing
                | ServiceError::LocaleMessageAttributeMissing,
            ) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

impl Debug for Localizations {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Localizations")
            .field(
                "bundles",
                &format!(
                    "HashMap {{ LanguageIdentifier => FluentBundle }} ({} items)",
                    self.bundles.len(),
                ),
            )
            .finish()
    }
}
