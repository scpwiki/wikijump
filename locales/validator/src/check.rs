/*
 * check.rs
 *
 * wikijump-locales-validator - Validate Wikijump's Fluent localization files
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

use crate::messages::Catalog;
use fluent_syntax::{ast, parser};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use unic_langid::LanguageIdentifier;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ValidationOutcome {
    Valid,
    Invalid,
}

#[derive(Debug)]
pub enum CheckError {
    ReadDirectory { path: PathBuf, source: io::Error },
    ReadEntry { path: PathBuf, source: io::Error },
    ReadFile { path: PathBuf, source: io::Error },
    InvalidUtf8Path { path: PathBuf },
}

impl fmt::Display for CheckError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadDirectory { path, source } => {
                write!(
                    formatter,
                    "unable to read directory {}: {source}",
                    path.display()
                )
            }
            Self::ReadEntry { path, source } => {
                write!(
                    formatter,
                    "unable to read an entry in {}: {source}",
                    path.display()
                )
            }
            Self::ReadFile { path, source } => {
                write!(
                    formatter,
                    "unable to read Fluent file {}: {source}",
                    path.display()
                )
            }
            Self::InvalidUtf8Path { path } => {
                write!(formatter, "path is not valid UTF-8: {}", path.display())
            }
        }
    }
}

impl std::error::Error for CheckError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadDirectory { source, .. }
            | Self::ReadEntry { source, .. }
            | Self::ReadFile { source, .. } => Some(source),
            Self::InvalidUtf8Path { .. } => None,
        }
    }
}

fn read_directory(path: &Path) -> Result<fs::ReadDir, CheckError> {
    fs::read_dir(path).map_err(|source| CheckError::ReadDirectory {
        path: path.to_owned(),
        source,
    })
}

fn read_entry(
    result: io::Result<fs::DirEntry>,
    directory: &Path,
) -> Result<fs::DirEntry, CheckError> {
    result.map_err(|source| CheckError::ReadEntry {
        path: directory.to_owned(),
        source,
    })
}

fn path_text<'a>(path: &Path, value: Option<&'a std::ffi::OsStr>) -> Result<&'a str, CheckError> {
    value
        .and_then(std::ffi::OsStr::to_str)
        .ok_or_else(|| CheckError::InvalidUtf8Path {
            path: path.to_owned(),
        })
}

pub fn run<P: AsRef<Path>>(directory: P) -> Result<ValidationOutcome, CheckError> {
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

    for result in read_directory(directory)? {
        let entry = read_entry(result, directory)?;
        let path = entry.path();
        if !path.is_dir() {
            fail!("Found non-directory in localizations: {}", path.display());
            continue;
        }

        let component = path_text(&path, path.file_name())?;
        println!("+ Reading {}", component);

        for result in read_directory(&path)? {
            let entry = read_entry(result, &path)?;
            let path = entry.path();
            if !path.is_file() {
                fail!("Found non-file in component directory: {}", path.display());
                continue;
            }

            match path.extension() {
                Some(ext) => {
                    let ext = path_text(&path, Some(ext))?;
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

            let locale_name = path_text(&path, path.file_stem())?;
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

            let source = fs::read_to_string(&path).map_err(|source| CheckError::ReadFile {
                path: path.clone(),
                source,
            })?;

            let resource = match parser::parse(source.as_str()) {
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

            for entry in &resource.body {
                match entry {
                    ast::Entry::Message(message) => {
                        if let Err(error) = catalog.add_message(locale.clone(), message) {
                            fail!("{error}");
                        }
                    }
                    ast::Entry::Term(term) => catalog.add_term(term),
                    ast::Entry::Junk { content } => {
                        fail!("Fluent file contains unknown data: {}", content);
                    }
                    _ => (),
                }
            }
        }
    }

    catalog.print_summary();
    success &= catalog.check();

    if success {
        Ok(ValidationOutcome::Valid)
    } else {
        Ok(ValidationOutcome::Invalid)
    }
}

#[cfg(test)]
mod tests {
    use super::{CheckError, ValidationOutcome, run};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_DIRECTORY_ID: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new() -> Self {
            let id = NEXT_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "wikijump-locales-validator-{}-{id}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("create test directory");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.path).expect("remove test directory");
        }
    }

    #[test]
    fn valid_primary_locale_returns_valid_outcome() {
        let directory = TestDirectory::new();
        let component = directory.path().join("core");
        fs::create_dir(&component).expect("create component directory");
        fs::write(component.join("en.ftl"), "hello = Hello\n").expect("write Fluent file");

        assert_eq!(run(directory.path()).unwrap(), ValidationOutcome::Valid);
    }

    #[test]
    fn missing_localization_directory_returns_read_error() {
        let directory = TestDirectory::new();
        let missing = directory.path().join("missing");

        let error = run(&missing).unwrap_err();

        match error {
            CheckError::ReadDirectory { path, .. } => assert_eq!(path, missing),
            other => panic!("expected directory read error, got {other:?}"),
        }
    }
}
