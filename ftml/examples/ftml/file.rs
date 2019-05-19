/*
 * ftml/file.rs
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

use crate::transform::TransformFn;
use ftml::prelude::*;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn process_file(
    in_path: &Path,
    out_path: &Path,
    transform: TransformFn,
    wrap: bool,
) -> Result<()> {
    let mut text = String::new();
    let mut file = File::open(in_path)?;
    file.read_to_string(&mut text)?;

    let html = transform(&mut text, wrap)?;
    let mut file = File::create(out_path)?;
    file.write_all(html.as_bytes())?;
    Ok(())
}

pub fn process_stdin(transform: TransformFn, wrap: bool) -> Result<()> {
    let mut text = String::new();
    io::stdin().read_to_string(&mut text)?;

    let html = transform(&mut text, wrap)?;
    println!("{}", &html);
    Ok(())
}
