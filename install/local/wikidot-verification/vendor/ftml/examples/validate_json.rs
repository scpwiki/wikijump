/*
 * examples/validate_json.rs
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

//! Helper script to validate / pretty-print JSON files containing syntax trees.
//!
//! Run via `scripts/validate_json.sh`

extern crate ftml;
extern crate serde_json;

#[macro_use]
extern crate str_macro;
extern crate termcolor;

use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use termcolor::{Buffer, BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

fn main() {
    let json_paths = {
        let mut arguments: Vec<OsString> = env::args_os().collect();
        arguments.remove(0);
        arguments
    };

    if json_paths.is_empty() {
        println!("No files listed.");
        process::exit(0);
    }

    // Read files
    let mut results: Vec<(PathBuf, Outcome)> = Vec::new();

    for json_path in json_paths {
        let path = PathBuf::from(json_path);

        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(error) => {
                let outcome = Outcome::Error(str!(error));
                results.push((path, outcome));
                continue;
            }
        };

        let syntax_tree = match serde_json::from_reader(&mut file) {
            Ok(tree) => tree,
            Err(error) => {
                let outcome = Outcome::Failure(str!(error));
                results.push((path, outcome));
                continue;
            }
        };

        let outcome = Outcome::Success(syntax_tree);
        results.push((path, outcome));
    }

    // Output results
    let mut buf_writer = BufferWriter::stdout(ColorChoice::Auto);
    let mut success = true;
    println!();

    for (path, outcome) in results {
        write_result(&mut buf_writer, &path, &outcome).expect("Unable to write");

        success &= matches!(outcome, Outcome::Success(_));
    }

    // Exit
    if !success {
        process::exit(1);
    }
}

fn write_result(
    buf_writer: &mut BufferWriter,
    path: &Path,
    outcome: &Outcome,
) -> io::Result<()> {
    let mut buffer = buf_writer.buffer();

    outcome.write(&mut buffer, path)?;
    buf_writer.print(&buffer)?;

    Ok(())
}

#[derive(Debug)]
enum Outcome {
    Success(ftml::tree::SyntaxTree<'static>),
    Failure(String),
    Error(String),
}

impl Outcome {
    fn write(&self, buffer: &mut Buffer, path: &Path) -> io::Result<()> {
        buffer.set_color(&ColorSpec::new().set_bold(true).set_fg(Some(self.color())))?;

        match self {
            Outcome::Success(_) => write!(buffer, " pass")?,
            Outcome::Failure(_) => write!(buffer, " fail")?,
            Outcome::Error(_) => write!(buffer, "error")?,
        }

        buffer.set_color(&ColorSpec::new())?;

        write!(buffer, ": {}: ", path.display())?;

        match self {
            Outcome::Success(tree) => writeln!(buffer, "\n\n{:#?}\n", tree)?,
            Outcome::Failure(message) | Outcome::Error(message) => {
                writeln!(buffer, "{}", message)?
            }
        }

        Ok(())
    }

    fn color(&self) -> Color {
        match self {
            Outcome::Success(_) => Color::Green,
            Outcome::Failure(_) => Color::Red,
            Outcome::Error(_) => Color::Yellow,
        }
    }
}
