/*
 * test.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2020 Ammon Smith
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

//! Retrieves tests from JSON in the root `/test` directory, and runs them.

use crate::parse::ParseError;
use crate::tree::SyntaxTree;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

const SKIP_TESTS: &[&str] = &[
    "code",
    "code-empty",
    "code-language",
    "code-language-empty",
    "code-spaces",
    "code-uppercase",
    // TODO add div tests
];

lazy_static! {
    static ref TEST_DIRECTORY: PathBuf = {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("test");
        path
    };
}

macro_rules! file_name {
    ($entry:expr) => {
        $entry.file_name().to_string_lossy()
    };
}

#[derive(Serialize, Deserialize, Debug)]
struct Test<'a> {
    #[serde(skip)]
    name: String,
    input: String,
    tree: SyntaxTree<'a>,
    errors: Vec<ParseError>,
}

impl Test<'_> {
    pub fn load(path: &Path, name: &str) -> Self {
        assert!(path.is_absolute());

        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(error) => panic!("Unable to open file '{}': {}", path.display(), error),
        };

        let mut test: Self = match serde_json::from_reader(&mut file) {
            Ok(test) => test,
            Err(error) => {
                panic!("Unable to parse JSON file '{}': {}", path.display(), error)
            }
        };

        test.name = str!(name);
        test
    }

    pub fn run(&self, log: &slog::Logger) {
        info!(
            &log,
            "Running syntax tree test case";
            "name" => &self.name,
            "input" => &self.input,
        );

        if SKIP_TESTS.contains(&&*self.name) {
            println!("+ {} [SKIPPED]", self.name);
            return;
        }

        println!("+ {}", self.name);

        let tokens = crate::tokenize(log, &self.input);
        let result = crate::parse(log, &tokens);
        let (tree, errors) = result.into();

        fn json<T>(object: &T) -> String
        where
            T: serde::Serialize,
        {
            let mut output = serde_json::to_string_pretty(object)
                .expect("Unable to serialize JSON to stdout");

            output.insert_str(0, "Generated JSON: ");
            output
        }

        if tree != self.tree {
            panic!(
                "Running test '{}' failed! AST did not match:\nExpected: {:#?}\nActual: {:#?}\n{}\nErrors: {:#?}",
                self.name,
                self.tree,
                tree,
                json(&tree),
                &errors,
            );
        }

        if errors != self.errors {
            panic!(
                "Running test '{}' failed! Errors did not match:\nExpected: {:#?}\nActual: {:#?}\n{}\nTree (correct): {:#?}",
                self.name,
                self.errors,
                errors,
                json(&errors),
                &tree,
            );
        }
    }
}

#[test]
fn ast() {
    let log = crate::build_logger();

    // Warn if any test are being skipped
    if !SKIP_TESTS.is_empty() {
        println!("=========");
        println!(" WARNING ");
        println!("=========");
        println!();
        println!("The following tests are being SKIPPED:");

        for test in SKIP_TESTS {
            println!("- {}", test);
        }

        println!();
    }

    // Load tests from JSON files
    let entries = fs::read_dir(&*TEST_DIRECTORY) //
        .expect("Unable to read directory");

    let tests_iter = entries.filter_map(|entry| {
        let entry = entry.expect("Unable to read directory entry");
        let ftype = entry.file_type().expect("Unable to get file type");
        if !ftype.is_file() {
            println!("Skipping non-file {}", file_name!(entry));
            return None;
        }

        let path = entry.path();
        let stem = path
            .file_stem()
            .expect("Unable to get file stem")
            .to_string_lossy();

        let ext = path.extension().expect("Unable to get file extension");
        if ext != "json" {
            println!("Skipping non-JSON file {}", file_name!(entry));
            return None;
        }

        Some(Test::load(&path, &stem))
    });

    // Sort tests by name
    let mut tests: Vec<Test> = tests_iter.collect();
    tests.sort_by(|a, b| (a.name).cmp(&b.name));

    // Run tests
    println!("Running tests:");
    for test in tests {
        test.run(&log);
    }
}
