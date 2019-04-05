/*
 * test.rs
 *
 * wikidot-html - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith for Project Foundation
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

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use super::prelude::*;

lazy_static! {
    static ref TEST_DIRECTORY: PathBuf = {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("test");
        path
    };
}

const SKIP_GEN_TESTS: bool = true;

#[test]
fn test_conversions() {
    if SKIP_GEN_TESTS {
        println!("Generation tests skipped!");
        return;
    }

    // Reuse these buffers for all the tests
    let mut output_file = PathBuf::new();
    let mut expected_html = String::new();

    fn read_file(buffer: &mut String, path: &Path) -> Result<()> {
        buffer.clear();
        let mut file = File::open(path)?;
        file.read_to_string(buffer)?;
        Ok(())
    }

    // Run through all of the test files
    for entry in fs::read_dir(&*TEST_DIRECTORY).expect("Unable to read test directory") {
        let entry = entry.expect("Unable to read entry in directory");
        let ftype = entry
            .file_type()
            .expect("Unable to get file type in directory");
        if !ftype.is_file() {
            continue;
        }

        let input_file = entry.path();
        output_file.clone_from(&input_file);
        output_file.set_extension("html");

        println!(
            "Converting {}...",
            input_file.file_name().unwrap().to_string_lossy()
        );
        let mut input_text = String::new();
        read_file(&mut input_text, &input_file).expect("Unable to read input Wikidot");
        read_file(&mut expected_html, &output_file).expect("Unable to read expected output HTML");

        let output_html = transform(&input_text).expect("Unable to transform Wikidot to HTML");
        assert_eq!(expected_html, output_html);
    }
}
