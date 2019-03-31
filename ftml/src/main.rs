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

use clap::{Arg, App};
use std::fs::File;
use std::io::{self, Read};
use wikidot_html::prelude::*;

fn main() -> Result<()> {
    let matches = App::new("Wikidot to HTML")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Ammon Smith")
        .about("Utility to convert Wikidot code into HTML")
        .max_term_width(110)
        .arg(
            Arg::with_name("FILE")
                .required(true)
                .help("Input file for the program. Set to \"-\" for stdin.")
        )
        .get_matches();

    let text = {
        let mut contents = String::new();
        let mut file: Box<io::Read> = match matches.value_of("FILE").unwrap() {
            "-" => Box::new(io::stdin()),
            path => Box::new(File::open(path)?),
        };

        file.read_to_string(&mut contents)?;
        contents
    };

    let html = transform(&text)?;
    println!("{}", &html);

    Ok(())
}
