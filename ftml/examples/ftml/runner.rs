/*
 * ftml/runner.rs
 *
 * ftml - Convert Wikidot code to HTML
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

use crate::context::Context;
use crate::file::{process_file, process_stdin};
use ftml::prelude::*;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::ffi::OsStr;
use std::path::Path;
use std::process;
use std::sync::mpsc;
use std::time::Duration;

pub fn run_watch(ctx: &Context) -> ! {
    let (tx, rx) = mpsc::channel();

    let mut watcher = watcher(tx, Duration::from_secs(1)).expect("Unable to create watcher");
    for in_path in &ctx.in_paths {
        if *in_path == OsStr::new("-") {
            eprintln!("Cannot read from stdin in watch mode");
            process::exit(1);
        }

        watcher
            .watch(in_path, RecursiveMode::NonRecursive)
            .expect("Unable to watch file");
    }

    let run_transform = |path: &Path, action: &str| match do_transform(ctx, path) {
        Ok(_) => {
            println!("{}, converting: '{}'", action, path.display());
        }
        Err(err) => {
            eprintln!("Error transforming from file '{}': {}", path.display(), err);
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
            },
        }
    }
}

pub fn run_once(ctx: &Context) -> ! {
    let mut return_code = 0;

    // Process each of the files
    for in_path in &ctx.in_paths {
        if *in_path == OsStr::new("-") {
            if let Err(err) = process_stdin(ctx.transform, ctx.wrap) {
                eprintln!("Error transforming from stdin: {}", err);
                return_code = 1;
            }
            continue;
        }

        let in_path = Path::new(in_path);
        if let Err(err) = do_transform(ctx, in_path) {
            eprintln!(
                "Error transforming from file '{}': {}",
                in_path.display(),
                err
            );
        }
    }

    process::exit(return_code)
}

// Helper function for running transform and managing paths
fn do_transform(ctx: &Context, in_path: &Path) -> Result<()> {
    let out_path = match in_path.file_stem() {
        Some(stem) => {
            let mut path = ctx.output_dir.clone();
            path.push(stem);
            path.set_extension("html");
            path
        }
        None => {
            return Err(Error::Msg(format!(
                "Path \"{}\" does not refer to a file",
                in_path.display()
            )))
        }
    };

    process_file(in_path, &out_path, ctx.transform, ctx.wrap)
}
