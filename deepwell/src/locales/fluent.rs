/*
 * locales/fluent.rs
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

use super::fallback::iterate_locale_fallbacks;
use crate::error::prelude::*;
use fluent::{FluentArgs, FluentMessage, FluentResource, bundle};
use fluent_syntax::ast::Pattern;
use intl_memoizer::concurrent::IntlLangMemoizer;
use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{self, Debug, Display},
    path::{Path, PathBuf},
};
use tokio::fs;
use unic_langid::LanguageIdentifier;

pub type FluentBundle = bundle::FluentBundle<FluentResource, IntlLangMemoizer>;

pub struct Localizations {
    bundles: HashMap<LanguageIdentifier, FluentBundle>,
}

impl Localizations {
    pub async fn open<P: Into<PathBuf>>(directory: P) -> Result<Self> {
        debug!("Reading Fluent localization directory...");

        let directory = {
            let mut path = directory.into();
            path.push("fluent");
            path
        };

        let make_error = || {
            Error::new(
                format!(
                    "failed to open localization directory at {}",
                    directory.display(),
                ),
                ErrorType::Localization,
            )
        };

        let mut bundles = HashMap::new();
        let mut entries = fs::read_dir(&directory).await.or_raise(make_error)?;

        while let Some(entry) = entries.next_entry().await.or_raise(make_error)? {
            let path = entry.path();
            Self::load_component(&mut bundles, &path)
                .await
                .or_raise(make_error)?;
        }

        Ok(Localizations { bundles })
    }

    async fn load_component(
        bundles: &mut HashMap<LanguageIdentifier, FluentBundle>,
        directory: &Path,
    ) -> Result<()> {
        debug!("Reading component at {}", directory.display());

        let make_error = || {
            Error::new(
                format!("failed to load component at {}", directory.display()),
                ErrorType::Localization,
            )
        };

        let mut entries = fs::read_dir(directory).await.or_raise(make_error)?;

        while let Some(entry) = entries.next_entry().await.or_raise(make_error)? {
            let path = entry.path();

            // Get locale from filename
            let locale_name = path
                .file_stem()
                .expect("No base name in locale path")
                .to_str()
                .expect("Path is not valid UTF-8");

            debug!("Loading locale {locale_name}");
            let locale = {
                let result: StdResult<LanguageIdentifier, _> = locale_name.parse();
                result.or_raise(make_error)?
            };

            // Read and parse localization strings
            let source = fs::read_to_string(&path).await.or_raise(make_error)?;

            let resource =
                FluentResource::try_new(source).map_err(|(_resource, errors)| {
                    Error::new(
                        format!("failed to load resource from {}", path.display()),
                        ErrorType::FluentParser(errors),
                    )
                })?;

            // Create or modify bundle
            let locale2 = locale.clone();
            let bundle = bundles
                .entry(locale)
                .or_insert_with(|| FluentBundle::new_concurrent(vec![locale2]));

            bundle.add_resource(resource).map_err(|errors| {
                Error::new(
                    format!("failed to add resources from {} to bundle", path.display()),
                    ErrorType::Fluent(errors),
                )
            })?;
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
    fn get_message(
        &self,
        locale: &LanguageIdentifier,
        path: &str,
    ) -> Result<(&'_ FluentBundle, FluentMessage<'_>)> {
        match self.bundles.get(locale) {
            Some(bundle) => match bundle.get_message(path) {
                Some(message) => Ok((bundle, message)),
                None => bail!(Error::new(
                    "no such message in locale",
                    ErrorType::LocaleMessageMissing {
                        message_key: str!(path)
                    }
                )),
            },
            None => bail!(Error::new(
                "cannot get message, no such locale",
                ErrorType::LocaleMissing {
                    locale: str!(locale)
                }
            )),
        }
    }

    /// Retrieve the specified Fluent pattern from the associated bundle.
    fn get_pattern(
        &self,
        locale: &LanguageIdentifier,
        path: &str,
        attribute: Option<&str>,
    ) -> Result<(&FluentBundle, &Pattern<&str>)> {
        debug!("Checking for translation patterns in locale {locale}");

        // Get appropriate message and bundle, if found
        let (bundle, message) = self.get_message(locale, path).or_raise(|| {
            Error::new(
                format!(
                    "failed to get Fluent pattern for path '{}' (attribute {:?})",
                    path, attribute,
                ),
                ErrorType::Localization,
            )
        })?;

        // Get pattern from message, if present
        let pattern = match attribute {
            Some(attribute) => match message.get_attribute(attribute) {
                Some(attrib) => attrib.value(),
                None => bail!(Error::new(
                    format!("locale message '{}' has no attribute '{}'", path, attribute),
                    ErrorType::LocaleMessageAttributeMissing {
                        message_key: str!(path),
                        attribute: str!(attribute),
                    },
                )),
            },
            None => match message.value() {
                Some(pattern) => pattern,
                None => bail!(Error::new(
                    format!("locale message '{}' with no attribute does not exist", path),
                    ErrorType::LocaleMessageValueMissing {
                        message_key: str!(path),
                    },
                )),
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
    ) -> Result<(LanguageIdentifier, &'a FluentBundle, &'a Pattern<&'a str>)>
    where
        L: AsRef<LanguageIdentifier> + 'a,
        I: IntoIterator<Item = L>,
    {
        let mut last_error = None; // Occurs if locales is empty

        // Iterate through each locale to try
        for locale_ref in locales {
            // Iterate through each fallback locale (e.g. ['fr-BE'] -> ['fr-BE', 'fr'])
            let locale = locale_ref.as_ref();
            let result = iterate_locale_fallbacks(locale.clone(), |locale| {
                // Try and get bundle and pattern, if it exists
                match self.get_pattern(locale, path, attribute) {
                    Err(error) => {
                        debug!("Pattern not found for locale {locale}: {error}");
                        last_error = Some(error);
                        None
                    }
                    Ok((bundle, pattern)) => {
                        debug!("Found pattern for locale {locale}");
                        Some((bundle, pattern))
                    }
                }
            });

            // Found a match, return this
            if let Some((locale, (bundle, pattern))) = result {
                return Ok((locale, bundle, pattern));
            }
        }

        let last_error = last_error.unwrap_or_else(|| {
            Error::new(
                "failed to get patterns for locales, no locales specified",
                ErrorType::NoLocalesSpecified,
            )
            .into()
        });

        warn!("Could not find any translation patterns: {last_error}");
        bail!(last_error);
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
    ) -> Result<Cow<'a, str>>
    where
        L: AsRef<LanguageIdentifier> + Display + 'a,
        I: IntoIterator<Item = L>,
    {
        // Parse translation key
        let (path, attribute) = Self::parse_selector(key);
        match attribute {
            Some(attribute) => {
                debug!(
                    "Checking message path {}, attribute {} for a matching locale",
                    path, attribute,
                );
            }
            None => {
                debug!(
                    "Checking message path {}, no attribute for a matching locale",
                    path,
                );
            }
        }

        // Find pattern for translating
        let (locale, bundle, pattern) = self
            .get_pattern_locales(locales, path, attribute)
            .or_raise(|| {
                Error::new("failed to translate message", ErrorType::Localization)
            })?;

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
    ) -> Result<Option<Cow<'a, str>>>
    where
        L: AsRef<LanguageIdentifier> + Display + 'a,
        I: IntoIterator<Item = L>,
    {
        match self.translate(locales, key, args) {
            Ok(translation) => Ok(Some(translation)),
            Err(error) => match error.error_type {
                ErrorType::LocaleMissing { .. }
                | ErrorType::LocaleMessageMissing { .. }
                | ErrorType::LocaleMessageValueMissing { .. }
                | ErrorType::LocaleMessageAttributeMissing { .. } => Ok(None),
                _ => Err(error),
            },
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
