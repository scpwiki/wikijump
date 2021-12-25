/*
 * locales/fluent.rs
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

use super::error::{fluent_load_err, LocalizationLoadError, LocalizationTranslateError};
use async_std::fs;
use async_std::path::{Path, PathBuf};
use async_std::prelude::*;
use fluent::{bundle, FluentArgs, FluentMessage, FluentResource};
use intl_memoizer::concurrent::IntlLangMemoizer;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use unic_langid::LanguageIdentifier;

pub type FluentBundle = bundle::FluentBundle<FluentResource, IntlLangMemoizer>;

pub struct Localizations {
    bundles: HashMap<LanguageIdentifier, FluentBundle>,
}

impl Localizations {
    pub async fn open<P: Into<PathBuf>>(
        directory: P,
    ) -> Result<Self, LocalizationLoadError> {
        tide::log::debug!("Reading Fluent localization directory...");

        let directory = {
            let mut path = directory.into();
            path.push("fluent");
            path
        };

        let mut bundles = HashMap::new();
        let mut entries = fs::read_dir(&directory).await?;

        while let Some(result) = entries.next().await {
            let entry = result?;
            let path = entry.path();
            if !entry.metadata().await?.is_dir() {
                tide::log::debug!("Skipping non-directory path {}", path.display());
                continue;
            }

            Self::load_component(&mut bundles, &path).await?;
        }

        Ok(Localizations { bundles })
    }

    async fn load_component(
        bundles: &mut HashMap<LanguageIdentifier, FluentBundle>,
        directory: &Path,
    ) -> Result<(), LocalizationLoadError> {
        tide::log::debug!("Reading component at {}", directory.display());
        let mut entries = fs::read_dir(directory).await?;

        while let Some(result) = entries.next().await {
            let entry = result?;
            let path = entry.path();
            if !entry.metadata().await?.is_file() {
                tide::log::debug!("Skipping non-directory path {}", path.display());
                continue;
            }

            // Get locale from filename
            let locale_name = path
                .file_stem()
                .expect("No base name in locale path")
                .to_str()
                .expect("Path is not valid UTF-8");

            tide::log::debug!("Loading locale {}", locale_name);
            let locale = LanguageIdentifier::from_bytes(locale_name.as_bytes())?;

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
    pub fn parse_selector<'a>(key: &'a str) -> (&'a str, Option<&'a str>) {
        match key.find('.') {
            None => (key, None),
            Some(idx) => {
                let (path, attribute) = key.split_at(idx);
                (path, Some(attribute))
            }
        }
    }

    pub fn has_message(&self, locale: &LanguageIdentifier, key: &str) -> bool {
        let (path, attribute) = Self::parse_selector(key);

        self.bundles
            .get(locale)
            .map(|bundle| match attribute {
                None => bundle.has_message(key),
                Some(attribute) => bundle
                    .get_message(path)
                    .map(|message| message.get_attribute(attribute).is_some())
                    .unwrap_or(false),
            })
            .unwrap_or(false)
    }

    fn get_message(
        &self,
        locale: &LanguageIdentifier,
        key: &str,
    ) -> Result<(&FluentBundle, FluentMessage), LocalizationTranslateError> {
        tide::log::info!(
            "Fetching translation for locale {}, message key {}",
            locale,
            key,
        );

        match self.bundles.get(locale) {
            None => Err(LocalizationTranslateError::NoLocale),
            Some(bundle) => match bundle.get_message(key) {
                Some(message) => Ok((bundle, message)),
                None => Err(LocalizationTranslateError::NoMessage),
            },
        }
    }

    pub fn translate(
        &self,
        locale: &LanguageIdentifier,
        key: &str,
        args: &FluentArgs,
    ) -> Result<String, LocalizationTranslateError> {
        // Get appropriate message and bundle
        let (path, attribute) = Self::parse_selector(key);
        let (bundle, message) = self.get_message(locale, path)?;

        // Get pattern from message
        let pattern = match attribute {
            Some(attribute) => match message.get_attribute(attribute) {
                Some(attrib) => attrib.value(),
                None => return Err(LocalizationTranslateError::NoMessageAttribute),
            },
            None => match message.value() {
                Some(pattern) => pattern,
                None => return Err(LocalizationTranslateError::NoMessageValue),
            },
        };

        // Format using pattern
        let mut errors = vec![];
        let output = bundle.format_pattern(pattern, Some(args), &mut errors);

        // Log any errors
        if !errors.is_empty() {
            tide::log::warn!(
                "Errors formatting message for locale {}, message key {}",
                locale,
                key,
            );

            for (key, value) in args.iter() {
                tide::log::warn!("Passed formatting argument: {} -> {:?}", key, value);
            }

            for error in errors {
                tide::log::warn!("Message formatting error: {}", error);
            }
        }

        // Done
        //
        // Passing as owned to avoid complicated FluentBundle lifetimes.
        Ok(str!(output))
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
