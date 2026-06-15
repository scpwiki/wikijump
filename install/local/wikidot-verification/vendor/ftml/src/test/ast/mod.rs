/*
 * test/ast/mod.rs
 *
 * ftml - Library to parse Wikidot text
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

//! Runs AST tests, stored in `/test`, where a given input wikitext file
//! is processed and a variety of assertions can be done on its output.

mod loader;
mod runner;

use crate::parsing::ParseError;
use crate::tree::SyntaxTree;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::{env, process};

// Debug settings

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

/// Temporary measure to update tests instead of checking them.
///
/// This should be used when adding or changing functionality,
/// provided you also carefully check the output is as expected.
const UPDATE_TESTS: bool = false;

// Constants

/// The directory where all test files are located.
/// This is the directory `test` under the repository root.
static TEST_DIRECTORY: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test");
    path
});

// Structs

/// Represents a particular result from a test execution.
#[derive(Debug, Copy, Clone)]
pub enum TestResult {
    Pass,
    Fail,
    Skip,
}

/// Represents the cumulative stats from a test execution.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct TestStats {
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
}

impl TestStats {
    #[inline]
    pub fn new() -> TestStats {
        TestStats::default()
    }

    pub fn add(&mut self, result: TestResult) {
        match result {
            TestResult::Pass => self.passed += 1,
            TestResult::Fail => self.failed += 1,
            TestResult::Skip => self.skipped += 1,
        }
    }

    pub fn print(self) {
        let total = self.passed + self.failed + self.skipped;

        if self.failed + self.skipped == 0 {
            println!("Ran a total of {total} tests, all of which passed.");
        } else {
            let percent = |value| (value as f32) / (total as f32) * 100.0;
            println!("Ran a total of {total} tests. Of these:");
            println!("* {} passed ({:.1}%)", self.passed, percent(self.passed));

            if self.failed != 0 {
                println!("* {} failed ({:.1}%)", self.failed, percent(self.failed));
            }

            if self.skipped != 0 {
                println!("* {} skipped ({:.1}%)", self.skipped, percent(self.skipped));
            }
        }
    }

    /// Get an exit code for the test.
    ///
    /// This way, if we skip any tests, or if tests fail, then the overall
    /// Rust test does not pass.
    pub fn exit_code(self) -> i32 {
        (self.failed + self.skipped).try_into().ok().unwrap_or(-1)
    }

    pub fn exit(self) -> ! {
        process::exit(self.exit_code());
    }
}

/// Represents one AST unit test case.
#[derive(Debug)]
pub struct Test {
    /// The name of this test.
    /// This is composed of two parts joined with a `/`.
    /// This is unique among all AST tests in the universe.
    pub name: String,

    /// The wikitext input for this test.
    /// Read from `input.ftml`. This file is required.
    pub input: String,

    /// The abstract syntax tree to check the output against.
    /// Read from `tree.json`.
    pub tree: Option<SyntaxTree<'static>>,

    /// The list of expected errors to be produced from this input.
    /// Read from `errors.json`.
    pub errors: Option<Vec<ParseError>>,

    /// The Wikidot-layout HTML expected to be generated from this input.
    /// Read from `wikidot.html`.
    pub wikidot_output: Option<String>,

    /// The Wikijump-layout HTML expected to be generated from this input.
    /// Read from `output.html`.
    pub html_output: Option<String>,

    /// The Wikijump-layout text expected to be generated from this input.
    /// This refers to the "text renderer" present in ftml.
    /// Read from `output.txt`.
    pub text_output: Option<String>,

    /// The locale for this test.
    /// Read from `locale.txt`.
    pub locale: Option<String>,
}

/// Represents the universe of all AST unit tests read from the filesystem.
#[derive(Debug)]
pub struct TestUniverse {
    pub tests: BTreeMap<String, Test>,
}

// Environment flags

fn env_update_tests() -> bool {
    match env::var("FTML_UPDATE_TESTS").ok() {
        Some(value) => matches!(value.as_str(), "true" | "1"),
        _ => false,
    }
}

// Test runner

#[test]
fn ast() {
    // If running in update mode, then run that and don't do anything else
    if UPDATE_TESTS || env_update_tests() {
        let tests = TestUniverse::load_permissive(&TEST_DIRECTORY);

        println!("=========");
        println!(" WARNING ");
        println!("=========");
        println!();
        println!("You are running in UPDATE MODE!");
        println!();
        println!(
            "This will run tests and save whatever results as the new \"expected\" value."
        );
        println!("Carefully inspect the diff and only save changes that are correct.");
        println!();

        tests.update(&TEST_DIRECTORY, SKIP_TESTS, ONLY_TESTS);

        // Never allow tests to pass with this option
        println!();
        println!("Failing test, you must unset update mode to let CI pass");
        println!("This is either:");
        println!("* The constant UPDATE_TESTS");
        println!("* The environment variable FTML_UPDATE_TESTS");
        process::exit(-1);
    }

    // Load all tests
    let tests = TestUniverse::load(&TEST_DIRECTORY);

    // Warn if any tests are being skipped
    #[allow(clippy::const_is_empty)]
    if !SKIP_TESTS.is_empty() {
        println!("=========");
        println!(" WARNING ");
        println!("=========");
        println!();
        println!("Tests matching the following are being SKIPPED:");

        for test in SKIP_TESTS {
            println!("- {}", test);
        }

        println!();
    }

    // Warn if we're only running certain tests
    #[allow(clippy::const_is_empty)]
    if !ONLY_TESTS.is_empty() {
        println!("=========");
        println!(" WARNING ");
        println!("=========");
        println!();
        println!("Only tests matching the following will being run.");
        println!("All others are being SKIPPED!");

        for test in ONLY_TESTS {
            println!("> {}", test);
        }

        println!();
    }

    // Test execution
    let stats = tests.run(SKIP_TESTS, ONLY_TESTS);
    stats.print();
    stats.exit();
}
