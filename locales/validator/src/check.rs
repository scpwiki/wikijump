/*
 * check.rs
 *
 * wikijump-locales-validator - Validate Wikijump's Fluent localization files
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

use crate::messages::Catalog;
use fluent_bundle::FluentResource;
use fluent_syntax::ast;
use std::path::Path;
use std::{fs, process};
use unic_langid::LanguageIdentifier;

pub fn run<P: AsRef<Path>>(directory: P) {
    let directory = directory.as_ref();
    let mut return_code = 0;

    macro_rules! fail {
        ($($arg:tt)*) => {{
            return_code = 1;
            eprint!("!! ");
            eprintln!($($arg)*);
        }};
    }

    let mut catalog = Catalog::default();
    print_real_path(directory);

    // Walk through all the component directories
    for result in fs::read_dir(directory).expect("Unable to read localization directory") {
        let entry = result.expect("Unable to read directory entry");
        let path = entry.path();
        if !path.is_dir() {
            fail!("Found non-directory in localizations: {}", path.display());
            continue;
        }

        // Walk through all the locales for a component
        print_real_path(&path);
        for result in fs::read_dir(path).expect("Unable to read component directory") {
            let entry = result.expect("Unable to read directory entry");
            let path = entry.path();
            if !path.is_file() {
                fail!("Found non-file in component directory: {}", path.display());
                continue;
            }

            // Ensure file is Fluent (*.ftl)
            match path.extension() {
                Some(ext) => {
                    let ext = ext.to_str().expect("Path is not valid UTF-8");
                    if !ext.eq_ignore_ascii_case("ftl") {
                        fail!(
                            "Found file with non-Fluent file extension: {} ({})",
                            ext,
                            path.display(),
                        );
                    }
                }
                None => {
                    fail!("Found file with no extension: {}", path.display());
                    continue;
                }
            }

            // Ensure locale is valid
            let locale_name = path
                .file_stem()
                .expect("No base name in locale path")
                .to_str()
                .expect("Path is not valid UTF-8");

            let locale: LanguageIdentifier = match locale_name.parse() {
                Ok(locale) => locale,
                Err(error) => {
                    fail!(
                        "Directory name ({}) is not a valid locale: {}",
                        locale_name,
                        error,
                    );
                    continue;
                }
            };

            // Read and parse Fluent file
            let source = match fs::read_to_string(&path) {
                Ok(source) => source,
                Err(error) => {
                    fail!("Unable to read Fluent file {}: {}", path.display(), error);
                    continue;
                }
            };

            let resource = match FluentResource::try_new(source.clone()) {
                Ok(resource) => resource,
                Err((_, errors)) => {
                    eprintln!("Fluent file source:\n-----\n{}\n-----\n", source);
                    fail!("Unable to parse Fluent source:");

                    for (i, error) in errors.iter().enumerate() {
                        eprintln!("{}. {}", i + 1, error);
                    }

                    continue;
                }
            };

            // Traverse resource, add keys to mapping
            let mut has_resource_comment = false;
            for entry in resource.entries() {
                match entry {
                    ast::Entry::Message(message) => catalog.add_message(&locale, message),
                    ast::Entry::ResourceComment(_) => {
                        has_resource_comment = true;
                    }
                    ast::Entry::Junk { content } => {
                        fail!("Fluent file contains unknown data: {}", content);
                    }
                    _ => (),
                }
            }

            if !has_resource_comment {
                fail!("No resource comments found in {}", path.display());
            }

            todo!();
        }
    }

    process::exit(return_code);
}

fn print_real_path(path: &Path) {
    let real_path = path.canonicalize().expect("Unable to canonicalize path");
    println!("Reading through {}...", real_path.display());
}
