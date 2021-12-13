/*
 * locales.rs
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

use anyhow::Result;
use gettext::Catalog;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display};
use std::fs::{self, File};
use std::path::Path;

#[derive(Debug)]
pub struct Localizations {
    catalogs: HashMap<String, Catalog>,
}

impl Localizations {
    pub fn open(directory: &Path) -> Result<Self> {
        let mut catalogs = HashMap::new();

        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();

            // Only read files
            if !path.is_file() {
                continue;
            }

            // Skip regular files
            let extension = match path.extension() {
                Some(ext) => ext,
                None => continue,
            };

            // Only read message catalogs (*.mo)
            if extension.eq_ignore_ascii_case("mo") {
                let catalog = load_catalog(&path)?;
                let name = path
                    .file_stem()
                    .expect("Path has extension but no file stem")
                    .to_str()
                    .expect("File stem is not valid UTF-8")
                    .to_owned();

                catalogs.insert(name, catalog);
            }
        }

        Ok(Localizations { catalogs })
    }

    /// Returns the translation for the given message key.
    /// If no translation exists, then the message key itself is returned.
    pub fn translate(&self, locale: &str, key: &str) -> Result<String> {
        if key.is_empty() {
            tide::log::warn!("Empty message key passed");
            return Err(EmptyMessageKey.into());
        }

        let message = match self.catalogs.get(locale) {
            Some(catalog) => catalog.gettext(key),
            None => return Err(NoSuchLocale::new(locale).into()),
        };

        todo!()
    }
}

fn load_catalog(path: &Path) -> Result<Catalog> {
    let file = File::open(path)?;
    let catalog = Catalog::parse(file)?;
    Ok(catalog)
}

#[derive(Debug, Copy, Clone)]
pub struct EmptyMessageKey;

impl Display for EmptyMessageKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Passed message key is empty")
    }
}

impl Error for EmptyMessageKey {}

#[derive(Debug, Clone)]
pub struct NoSuchLocale {
    locale: String,
}

impl NoSuchLocale {
    #[inline]
    pub fn new<S: Into<String>>(locale: S) -> Self {
        NoSuchLocale {
            locale: locale.into(),
        }
    }
}

impl Display for NoSuchLocale {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No locale found in catalog: {}", self.locale)
    }
}

impl Error for NoSuchLocale {}
