/*
 * wdtohtml/context.rs
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

use clap::{App, Arg};
use crate::transform::{full_transform, parse_only, prefilter_only, TransformFn};
use std::ffi::OsString;
use std::fmt::{self, Debug};
use std::path::PathBuf;
use std::{fs, process};

pub struct Context {
    pub transform: TransformFn,
    pub in_paths: Vec<OsString>,
    pub output_dir: PathBuf,
    pub wrap: bool,
    pub watch: bool,
}

impl<'a> Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Context")
            .field("transform", &(self.transform as *const TransformFn))
            .field("in_paths", &self.in_paths)
            .field("output_dir", &self.output_dir)
            .finish()
    }
}

pub fn parse_args() -> Context {
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
                .help(concat!(
                    "Specify an output directory to place rendered files in. ",
                    "Defaults to the current directory.",
                )),
        )
        .arg(
            Arg::with_name("mode")
                .short("m")
                .long("mode")
                .takes_value(true)
                .help(concat!(
                    "Instead of running the entire transformation process, you can ",
                    "limit it to one of the following operations: 'filter', 'parse', ",
                    "'transform' (default).",
                )),
        )
        .arg(
            Arg::with_name("watch")
                .short("w")
                .long("watch")
                .help(concat!(
                    "Watch the input files, and whenever they are modified, ",
                    "rerun the transformation.",
                )),
        )
        .arg(Arg::with_name("no-wrap").short("N").long("no-wrap").help(
            "Don't wrap the output HTML with basic document tags. Output exactly as generated.",
        ))
        .arg(
            Arg::with_name("FILE")
                .multiple(true)
                .required(true)
                .help("Input file for the program. Set to \"-\" for stdin."),
        )
        .get_matches();

    let output_dir = matches
        .value_of_os("output-directory")
        .map(PathBuf::from)
        .expect("No argument 'output-directory'");

    if let Err(err) = fs::create_dir_all(&output_dir) {
        eprintln!(
            "Error creating directories for \"{}\": {}",
            output_dir.display(),
            &err
        );
        process::exit(1);
    }

    let transform: TransformFn = match matches.value_of("mode") {
        Some("filter") | Some("prefilter") => prefilter_only,
        Some("parse") | Some("tree") => parse_only,
        Some("transform") | Some("convert") | None => full_transform,
        Some(mode) => {
            eprintln!("Unknown execution mode: '{}'", mode);
            eprintln!("Should be one of: 'filter', 'parse', 'transform'");
            process::exit(1);
        }
    };

    let no_wrap = matches.occurrences_of("no-wrap") > 0;
    let watch_mode = matches.occurrences_of("watch") > 0;

    let in_paths = matches
        .values_of_os("FILE")
        .expect("No argument(s) for 'FILE'")
        .map(OsString::from)
        .collect();

    Context {
        transform,
        in_paths,
        output_dir,
        wrap: !no_wrap,
        watch: watch_mode,
    }
}
