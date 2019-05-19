/*
 * filter/include.rs
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

use crate::{Error, Result};
use regex::{Regex, RegexBuilder};
use std::borrow::Cow;
use std::collections::HashMap;

lazy_static! {
    static ref INCLUDE: Regex = {
        RegexBuilder::new(r"(?x)
            \[\[
                \s*include\s+
                (?P<resource>[^ \]]+)
                (?P<args>.*?)
            \]\]")
            .multi_line(true)
            .dot_matches_new_line(true)
            .case_insensitive(true)
            .build()
            .unwrap()
    };

    static ref INCLUDE_ARG: Regex = {
        RegexBuilder::new(r"\s+(?P<key>\w+)\s*=\s*(?P<value>[^\|]+)\s*")
            .multi_line(true)
            .build()
            .unwrap()
    };
}

// Helper function
pub fn substitute(text: &mut String, includer: &Includer) -> Result<()> {
    while let Some(capture) = INCLUDE.captures(text) {
        let mut args = HashMap::new();

        let name = capture
            .name("resource")
            .expect("Named capture group not found")
            .as_str();
        let raw_args = capture
            .name("args")
            .expect("Named capture group not found")
            .as_str();

        if !raw_args.trim().is_empty() {
            for raw_arg in raw_args.split("|") {
                match INCLUDE_ARG.captures(raw_arg) {
                    Some(capture) => {
                        let key = capture
                            .name("key")
                            .expect("Named capture group not found")
                            .as_str();
                        let value = capture
                            .name("value")
                            .expect("Named capture group not found")
                            .as_str();

                        args.insert(key, value);
                    }
                    None => {
                        return Err(Error::Msg(format!(
                            "Include arguments for '{}' are improperly formatted",
                            name
                        )))
                    }
                }
            }
        }

        let mtch = capture
            .get(0)
            .expect("Regular expression lacks a full match");
        let range = mtch.start()..mtch.end();

        let resource = includer.get_resource(name, args)?;
        text.replace_range(range, resource.as_ref());
    }

    Ok(())
}

// Trait definition
pub trait Includer {
    fn get_resource(&self, name: &str, args: HashMap<&str, &str>) -> Result<Cow<str>>;
}

// Implementations
#[derive(Debug, Clone)]
pub struct NullIncluder;

impl Includer for NullIncluder {
    fn get_resource(&self, _name: &str, _args: HashMap<&str, &str>) -> Result<Cow<str>> {
        Ok(Cow::Borrowed(""))
    }
}

#[derive(Debug, Clone)]
pub struct NotFoundIncluder;

impl Includer for NotFoundIncluder {
    fn get_resource(&self, name: &str, _args: HashMap<&str, &str>) -> Result<Cow<str>> {
        Ok(Cow::Owned(format!("[[div style=\"line-height: 141%; color: #b00; padding: 1em; margin: 1em; border: 1px solid #faa;\"]]\nIncluded page \"{}\" does not exist\n[[/div]]", name)))
    }
}
