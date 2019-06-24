/*
 * ftml/main.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith
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

#![deny(missing_debug_implementations)]

extern crate clap;
extern crate ftml;
extern crate notify;

#[macro_use]
extern crate str_macro;

mod context;
mod file;
mod runner;
mod transform;

use self::context::parse_args;
use self::runner::{run_once, run_watch};

fn main() {
    let ctx = parse_args();

    if ctx.watch {
        run_watch(&ctx);
    } else {
        run_once(&ctx);
    }
}
