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
extern crate notify;
extern crate wikidot_html;

use clap::{App, Arg};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::ffi::OsStr;
use std::fmt::{self, Debug};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::mpsc;
use std::time::Duration;
use wikidot_html::prelude::*;
use wikidot_html::include::NullIncluder;

type TransformFn = fn(&mut String) -> Result<String>;

struct Context<'a> {
    transform: TransformFn,
    in_paths: Vec<&'a OsStr>,
    output_dir: &'a Path,
}

impl<'a> Debug for Context<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Context")
            .field("transform", &(self.transform as *const TransformFn))
            .field("in_paths", &"[ ... ]")
            .field("output_dir", &self.output_dir)
            .finish()
    }
}

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
                .long("mode")
                .takes_value(true)
                .help("Instead of running the entire transformation process, you can limit it to one of the following operations: 'filter', 'parse', 'transform' (default)."),
        )
        .arg(
            Arg::with_name("watch")
                .short("w")
                .long("watch")
                .help("Watch the input files, and whenever they are modified, rerun the transformation."),
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
        .map(Path::new)
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

    let watch_mode = matches.occurrences_of("watch") > 0;
    let in_paths = matches
        .values_of_os("FILE")
        .expect("No argument(s) for 'FILE'")
        .collect();

    let ctx = Context {
        transform,
        in_paths,
        output_dir,
    };

    if watch_mode {
        run_watch(&ctx);
    } else {
        run_once(&ctx);
    }
}

// Runners
fn run_watch(ctx: &Context) -> ! {
    let (tx, rx) = mpsc::channel();

    let mut watcher = watcher(tx, Duration::from_secs(1)).expect("Unable to create watcher");
    for in_path in &ctx.in_paths {
        if *in_path == OsStr::new("-") {
            eprintln!("Cannot read from stdin in watch mode");
            process::exit(1);
        }

        watcher.watch(in_path, RecursiveMode::NonRecursive).expect("Unable to watch file");
    }

    let run_transform = |path: &Path, action: &str| {
        match do_transform(ctx, path) {
            Ok(_) => {
                println!("{}, converting: '{}'", action, path.display());
            }
            Err(err) => {
                eprintln!("Error transforming from file '{}': {}", path.display(), err);
            }
        }
    };

    // Initial run
    for in_path in &ctx.in_paths {
        let in_path = Path::new(in_path);
        run_transform(&in_path, "Initial pass");
    }

    // Main event loop
    loop {
        use self::DebouncedEvent::*;

        match rx.recv() {
            Err(err) => eprintln!("Error retreiving notify event: {:?}", err),
            Ok(evt) => match evt {
                Create(path) | NoticeWrite(path) | Write(path) | Rename(_, path) => {
                    run_transform(&path, "File updated");
                }
                Error(err, _) => eprintln!("Error received from notify: {:?}", err),
                _ => (),
            }
        }
    }
}

fn run_once(ctx: &Context) -> ! {
    let mut return_code = 0;

    // Process each of the files
    for in_path in &ctx.in_paths {
        if *in_path == OsStr::new("-") {
            if let Err(err) = process_stdin(ctx.transform) {
                eprintln!("Error transforming from stdin: {}", err);
                return_code = 1;
                continue;
            }
        }

        let in_path = Path::new(in_path);
        if let Err(err) = do_transform(ctx, in_path) {
            eprintln!("Error transforming from file '{}': {}", in_path.display(), err);
        }
    }

    process::exit(return_code)
}

// Helper function for running transform and managing paths
fn do_transform(ctx: &Context, in_path: &Path) -> Result<()> {
    let out_path = match in_path.file_stem() {
        Some(stem) => {
            let mut path = PathBuf::from(ctx.output_dir);
            path.push(stem);
            path.set_extension("html");
            path
        }
        None => return Err(Error::Msg(format!("Path \"{}\" does not refer to a file", in_path.display()))),
    };

    process_file(in_path, &out_path, ctx.transform)
}

// Transformation functions
fn prefilter_only(text: &mut String) -> Result<String> {
    let mut text = text.clone();
    prefilter(&mut text, &NullIncluder)?;
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

#[inline]
fn full_transform(text: &mut String) -> Result<String> {
    let mut result = transform::<HtmlRender>(text, &NullIncluder)?;
    result.insert_str(0, "<html><body>\n");
    result.push_str("\n</body></html\n");
    Ok(result)
}

// File handling
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
