/*
 * config/special_action.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2023 Wikijump Team
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

//! Perform a "special action" instead of normal server execution.
//!
//! This is useful in contexts such as CI, where we want DEEPWELL to
//! not run as a daemon, but instead perform a special action or check,
//! as if motivated by a script.

use super::Config;
use std::path::PathBuf;
use std::{env, process};

pub fn run_special_action() {
    // Get action name, if specified.
    // Otherwise return and perform normal execution.
    let Ok(action_name) = env::var("DEEPWELL_SPECIAL_ACTION") else {
        return;
    };

    // Run appropriate special action.
    let return_code = match action_name.as_str() {
        "config" | "validate-config" => validate_config(),
        _ => {
            eprintln!("Unknown special action: {action_name}");
            process::exit(1);
        }
    };

    // Exit, don't perform normal server execution.
    process::exit(return_code);
}

fn validate_config() -> i32 {
    println!("Running special action: Validate configuration");

    let mut return_code = 0;
    for value in env::args_os().skip(1) {
        let path = PathBuf::from(value);
        print!("Checking {}... ", path.display());

        match Config::load(path) {
            Ok(_) => println!("success"),
            Err(error) => {
                println!("error");
                eprintln!("{error}");
                return_code += 1;
            }
        }
    }

    println!("All passed files checked, exiting");
    return_code
}
