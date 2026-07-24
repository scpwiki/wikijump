/*
 * main.rs
 *
 * wikijump-locales-validator - Validate Wikijump's Fluent localization files
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

#[macro_use]
extern crate str_macro;

#[macro_use]
extern crate unic_langid;

mod check;
mod messages;

use check::ValidationOutcome;
use std::process::ExitCode;

fn main() -> ExitCode {
    match check::run("../fluent") {
        Ok(ValidationOutcome::Valid) => {
            println!();
            println!("Everything looks in order.");
            ExitCode::SUCCESS
        }
        Ok(ValidationOutcome::Invalid) => {
            eprintln!();
            eprintln!("Some validation issues found! See above.");
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("Unable to validate Fluent files: {error}");
            ExitCode::FAILURE
        }
    }
}
