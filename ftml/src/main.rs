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

type TransformFn = fn(&mut String) -> Result<String>;

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
            Arg::with_name("mode")
                .short("m")
                .long("execution-mode")
                .takes_value(true)
                .help("Instead of running the entire transformation process, you can limit it to one of the following operations: 'filter', 'parse'."),
        )
        .arg(
            Arg::with_name("FILE")
                .multiple(true)
                .required(true)
                .help("Input file for the program. Set to \"-\" for stdin."),
        )
        .get_matches();

    let output_dir = matches
        .value_of_os("output-directory")
        .expect("No argument 'output-directory'");
    if let Err(err) = fs::create_dir_all(&output_dir) {
        let output_dir = Path::new(output_dir);
        eprintln!(
            "Error creating directories for \"{}\": {}",
            output_dir.display(),
            &err
        );
        process::exit(1);
    }

    let transform_fn: TransformFn = match matches.value_of("mode") {
        None => transform::<HtmlRender>,
        Some("filter") | Some("prefilter") => prefilter_only,
        Some("parse") | Some("tree") => parse_only,
        Some(mode) => {
            eprintln!("Unknown execution mode: '{}'", mode);
            eprintln!("Should be one of: 'filter', 'parse'");
            process::exit(1);
        },
    };

    let mut return_code = 0;
    for in_path in matches.values_of_os("FILE").expect("No argument(s) 'FILE'") {
        if in_path == "-" {
            if let Err(err) = process_stdin(transform_fn) {
                eprintln!("Error transforming from stdin: {}", &err);
            }

            return_code = 1;
            continue;
        }

        let in_path = Path::new(in_path);
        let out_path = match in_path.file_stem() {
            Some(stem) => {
                let mut path = PathBuf::from(output_dir);
                path.push(stem);
                path.set_extension("html");
                path
            }
            None => {
                eprintln!("Path \"{}\" does not refer to a file", in_path.display());
                process::exit(1);
            }
        };

        if let Err(err) = process_file(in_path, &out_path, transform_fn) {
            eprintln!("Error transforming \"{}\": {}", in_path.display(), &err);
            return_code = 1;
        }
    }

    process::exit(return_code);
}

fn prefilter_only(text: &mut String) -> Result<String> {
    let mut text = text.clone();
    prefilter(&mut text);
    Ok(text)
}

fn parse_only(text: &mut String) -> Result<String> {
    let tree = parse(text)?;
    let result = format!(
        "<html><body><pre><code>\n{:#?}\n</code></pre></body></html>\n",
        &tree
    );
    Ok(result)
}

fn process_file(in_path: &Path, out_path: &Path, transform: TransformFn) -> Result<()> {
    let mut text = String::new();
    let mut file = File::open(in_path)?;
    file.read_to_string(&mut text)?;

    let html = transform(&mut text)?;
    let mut file = File::create(out_path)?;
    file.write_all(html.as_bytes())?;
    Ok(())
}

fn process_stdin(transform: TransformFn) -> Result<()> {
    let mut text = String::new();
    io::stdin().read_to_string(&mut text)?;

    let html = transform(&mut text)?;
    println!("{}", &html);
    Ok(())
}
