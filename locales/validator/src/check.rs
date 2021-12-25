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
    let mut success = true;

    macro_rules! fail {
        ($($arg:tt)*) => {{
            success = false;
            eprint!("!! ");
            eprintln!($($arg)*);
        }};
    }

    let mut catalog = Catalog::default();
    println!("Reading all Fluent files...");

    // Walk through all the component directories
    for result in fs::read_dir(directory).expect("Unable to read localization directory") {
        let entry = result.expect("Unable to read directory entry");
        let path = entry.path();
        if !path.is_dir() {
            fail!("Found non-directory in localizations: {}", path.display());
            continue;
        }

        let component = path
            .file_name()
            .expect("No base name for path")
            .to_str()
            .expect("Path is not valid UTF-8");
        println!("+  Reading {}", component);

        // Walk through all the locales for a component
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

            println!("++ {}", locale_name);

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
            for entry in resource.entries() {
                match entry {
                    ast::Entry::Message(message) => catalog.add_message(locale.clone(), message),
                    ast::Entry::Term(term) => catalog.add_term(term),
                    ast::Entry::Junk { content } => {
                        fail!("Fluent file contains unknown data: {}", content);
                    }
                    _ => (),
                }
            }
        }
    }

    // Built catalog, check for validity
    catalog.print_summary();
    success &= catalog.check();

    // Exit with result
    if success {
        println!();
        println!("Everything looks in order.");
        process::exit(0);
    } else {
        eprintln!();
        eprintln!("Some validation issues found! See above.");
        process::exit(1);
    }
}
