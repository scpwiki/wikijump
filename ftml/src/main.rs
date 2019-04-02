/*
 * main.rs
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

#![allow(unknown_lints, large_enum_variant, match_bool)]
#![deny(missing_debug_implementations)]

extern crate clap;
extern crate wikidot_html;

use clap::{App, Arg};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process;
use wikidot_html::prelude::*;

fn main() {
    let matches = App::new("Wikidot to HTML")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Ammon Smith")
        .about("Utility to convert Wikidot code into HTML")
        .max_term_width(110)
        .arg(
            Arg::with_name("output-directory")
                .short("d")
                .long("directory")
                .default_value(".")
                .help("Specify an output directory to place rendered files in. Defaults to the current directory."),
        )
        .arg(
            Arg::with_name("FILE")
                .multiple(true)
                .required(true)
                .help("Input file for the program. Set to \"-\" for stdin."),
        )
        .get_matches();

    let output_dir = matches.value_of_os("output-directory").unwrap();
    if let Err(err) = fs::create_dir_all(&output_dir) {
        let output_dir = Path::new(output_dir);
        eprintln!("Error creating directories for \"{}\": {}", output_dir.display(), &err);
        process::exit(1);
    }

    for in_path in matches.values_of_os("FILE").unwrap() {
        if in_path == "-" {
            if let Err(err) = process_stdin() {
                eprintln!("Error transforming from stdin: {}", &err);
            }

            process::exit(1);
        }

        let in_path = Path::new(in_path);
        let out_path = match in_path.file_stem() {
            Some(stem) => {
                let mut path = PathBuf::from(output_dir);
                path.push(stem);
                path.set_extension("html");
                path
            },
            None => {
                eprintln!("Path \"{}\" does not refer to a file", in_path.display());
                process::exit(1);
            },
        };

        if let Err(err) = process_file(in_path, &out_path) {
            eprintln!("Error transforming \"{}\": {}", in_path.display(), &err);
        }
    }
}

fn process_file(in_path: &Path, out_path: &Path) -> Result<()> {
    let text = {
        let mut contents = String::new();
        let mut file = File::open(in_path)?;
        file.read_to_string(&mut contents)?;
        contents
    };

    let html = transform(text)?;
    let mut file = File::create(out_path)?;
    file.write_all(html.as_bytes())?;
    Ok(())
}

fn process_stdin() -> Result<()> {
    let text = {
        let mut contents = String::new();
        io::stdin().read_to_string(&mut contents)?;
        contents
    };

    let html = transform(text)?;
    println!("{}", &html);
    Ok(())
}
