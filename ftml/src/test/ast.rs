/*
 * test/ast.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2022 Wikijump Team
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
//!
//! Additionally performs some other tests from the parser which are better
//! in a dedicated test file.

use super::includer::TestIncluder;
use crate::data::PageInfo;
use crate::parsing::ParseWarning;
use crate::render::html::HtmlRender;
use crate::render::text::TextRender;
use crate::render::Render;
use crate::settings::{WikitextMode, WikitextSettings};
use crate::tree::SyntaxTree;
use std::borrow::Cow;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process;
use void::ResultVoidExt;

/// Temporary measure to not run certain tests.
///
/// This is meant to help with development, or in specific circumstances
/// where it is known functionality is broken while alternatives are
/// being developed.
const SKIP_TESTS: &[&str] = &[];

/// Temporary measure to only run certain tests.
///
/// This can assist with development, when you only care about specific
/// tests to check if certain functionality is working as expected.
const ONLY_TESTS: &[&str] = &[];

lazy_static! {
    static ref TEST_DIRECTORY: PathBuf = {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("test");
        path
    };
}

macro_rules! cow {
    ($text:expr) => {
        Cow::Borrowed(&$text)
    };
}

macro_rules! file_name {
    ($entry:expr) => {
        $entry.file_name().to_string_lossy()
    };
}

#[derive(Debug, Copy, Clone)]
pub enum TestResult {
    Pass,
    Fail,
    Skip,
}

// Debugging execution

fn only_test_should_skip(name: &str) -> bool {
    assert!(!ONLY_TESTS.is_empty());

    for pattern in ONLY_TESTS.iter() {
        // Literal test name
        if pattern == &name {
            return false;
        }

        // Test name prefix
        if pattern.ends_with('-') {
            if name.starts_with(pattern) {
                return false;
            }
        }
    }

    true
}

// Newline normalization

#[cfg(not(target_os = "windows"))]
fn process_newlines(_: &mut String) {}

#[cfg(target_os = "windows")]
fn process_newlines(text: &mut String) {
    while let Some(idx) = text.find("\r\n") {
        let range = idx..idx + 2;
        text.replace_range(range, "\n");
    }
}

// Test runner

#[derive(Serialize, Deserialize, Debug)]
struct Test<'a> {
    #[serde(skip)]
    name: String,
    input: String,
    tree: SyntaxTree<'a>,
    warnings: Vec<ParseWarning>,

    #[serde(skip)]
    html: String,

    #[serde(skip)]
    text: String,
}

impl Test<'_> {
    pub fn load(path: &Path, name: &str) -> Self {
        assert!(path.is_absolute());

        macro_rules! open_file {
            ($path:expr) => {
                match File::open(&$path) {
                    Ok(file) => file,
                    Err(error) => {
                        panic!("Unable to open file '{}': {}", $path.display(), error)
                    }
                }
            };
        }

        macro_rules! load_output {
            ($name:expr, $extension:expr) => {{
                let mut path = PathBuf::from(path);
                path.set_extension($extension);

                let mut file = open_file!(path);
                let mut contents = String::new();

                if let Err(error) = file.read_to_string(&mut contents) {
                    panic!(
                        "Unable to read {} file '{}': {}",
                        $name,
                        path.display(),
                        error,
                    );
                }

                process_newlines(&mut contents);

                if contents.ends_with('\n') {
                    contents.pop();
                }

                contents
            }};
        }

        // Load JSON file
        let mut file = open_file!(path);
        let mut test: Self = match serde_json::from_reader(&mut file) {
            Ok(test) => test,
            Err(error) => {
                panic!("Unable to parse JSON file '{}': {}", path.display(), error)
            }
        };

        test.name = str!(name);
        test.html = load_output!("HTML", "html");
        test.text = load_output!("text", "txt");
        test
    }

    pub fn run(&self) -> TestResult {
        if SKIP_TESTS.contains(&&*self.name) {
            println!("+ {} [SKIPPED]", self.name);
            return TestResult::Skip;
        }

        if !ONLY_TESTS.is_empty() && only_test_should_skip(&&*self.name) {
            println!("+ {} [SKIPPED]", self.name);
            return TestResult::Skip;
        }

        info!(
            "Running syntax tree test case {} on {}",
            &self.name, &self.input,
        );

        println!("+ {}", self.name);

        let page_info = PageInfo {
            page: Cow::Owned(format!("page-{}", self.name)),
            category: None,
            site: cow!("test"),
            title: cow!(self.name),
            alt_title: None,
            rating: 0.0,
            tags: vec![cow!("fruit"), cow!("component")],
            language: cow!("default"),
        };

        let settings = WikitextSettings::from_mode(WikitextMode::Page);

        let (mut text, _pages) =
            crate::include(&self.input, &settings, TestIncluder, || unreachable!())
                .void_unwrap();

        crate::preprocess(&mut text);
        let tokens = crate::tokenize(&text);
        let result = crate::parse(&tokens, &page_info, &settings);
        let (tree, warnings) = result.into();
        let html_output = HtmlRender.render(&tree, &page_info, &settings);
        let text_output = TextRender.render(&tree, &page_info, &settings);

        fn json<T>(object: &T) -> String
        where
            T: serde::Serialize,
        {
            let mut output = serde_json::to_string_pretty(object)
                .expect("Unable to serialize JSON to stdout");

            output.insert_str(0, "Generated JSON: ");
            output
        }

        let mut result = TestResult::Pass;

        if tree != self.tree {
            result = TestResult::Fail;
            eprintln!(
                "AST did not match:\nExpected: {:#?}\nActual: {:#?}\n{}\nWarnings: {:#?}",
                self.tree,
                tree,
                json(&tree),
                &warnings,
            );
        }

        if warnings != self.warnings {
            result = TestResult::Fail;
            eprintln!(
                "Warnings did not match:\nExpected: {:#?}\nActual:   {:#?}\n{}\nTree (correct): {:#?}",
                self.warnings,
                warnings,
                json(&warnings),
                &tree,
            );
        }

        if html_output.body != self.html {
            result = TestResult::Fail;
            eprintln!(
                "HTML does not match:\nExpected: {:?}\nActual:   {:?}\n\n{}\n\nTree (correct): {:#?}",
                self.html,
                html_output.body,
                html_output.body,
                &tree,
            );
        }

        if text_output != self.text {
            result = TestResult::Fail;
            eprintln!(
                "Text output does not match:\nExpected: {:?}\nActual:   {:?}\n\n{}\n\nTree (correct): {:#?}",
                self.text,
                text_output,
                text_output,
                &tree,
            );
        }

        result
    }
}

#[test]
fn ast_and_html() {
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

    // Warn if we're only checking certain tests
    if !ONLY_TESTS.is_empty() {
        println!("=========");
        println!(" WARNING ");
        println!("=========");
        println!();
        println!("Only the following tests are being run.");
        println!("All others are being SKIPPED!");

        for test in ONLY_TESTS {
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

        let extension = path.extension().map(|s| s.to_str()).flatten();
        match extension {
            // Load JSON test data
            Some("json") => Some(Test::load(&path, &stem)),

            // We expect these, don't print anything
            Some("html") | Some("txt") => None,

            // Print for other, unexpected files
            _ => {
                println!("Skipping non-JSON file {}", file_name!(entry));
                None
            }
        }
    });

    // Sort tests by name
    let mut tests: Vec<Test> = tests_iter.collect();
    tests.sort_by(|a, b| (a.name).cmp(&b.name));

    // Run tests
    let mut failed = 0;
    let mut skipped = 0;

    println!("Running {} syntax tree tests:", tests.len());
    for test in &tests {
        match test.run() {
            TestResult::Pass => (),
            TestResult::Fail => failed += 1,
            TestResult::Skip => skipped += 1,
        }
    }

    println!();
    println!("Ran a total of {} tests", tests.len());

    if failed > 0 {
        println!("Of these, {} failed", failed);
    }

    if skipped > 0 {
        // Don't allow accidentally committing skipped tests
        println!("Additionally, {} tests are being skipped", skipped);
        println!("Remember to re-enable all tests before committing!");
    }

    process::exit(failed + skipped);
}
