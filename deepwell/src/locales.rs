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

use gettext::{Catalog, Error};
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::Path;

#[derive(Debug)]
pub struct Localizations {
    catalogs: HashMap<String, Catalog>,
}

impl Localizations {
    pub fn open(directory: &Path) -> Result<Self, Error> {
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
}

fn load_catalog(path: &Path) -> Result<Catalog, Error> {
    let file = File::open(path)?;
    let catalog = Catalog::parse(file)?;
    Ok(catalog)
}
