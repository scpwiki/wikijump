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
use pest::Parser;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, Clone, Parser)]
#[grammar = "filter/include.pest"]
pub struct IncludeParser;

// Helper function
pub fn substitute(text: &mut String, includer: &Includer) -> Result<()> {
    let pairs = match IncludeParser::parse(Rule::include, text) {
        Ok(mut pairs) => get_inner_pairs!(pairs),
        Err(err) => {
            return Err(Error::Msg(format!(
                "Include transform parsing error: {}",
                err
            )));
        }
    };

println!("> {:#?}", &pairs);

    // TODO

    /*
    let mtch = capture
        .get(0)
        .expect("Regular expression lacks a full match");
    let range = mtch.start()..mtch.end();

    let resource = includer.get_resource(name, args)?;
    text.replace_range(range, resource.as_ref());
    */

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
pub struct TextIncluder<'a>(pub &'a str);

impl<'a> Includer for TextIncluder<'a> {
    fn get_resource(&self, _name: &str, _args: HashMap<&str, &str>) -> Result<Cow<str>> {
        Ok(Cow::Borrowed(self.0))
    }
}

#[derive(Debug, Clone)]
pub struct NotFoundIncluder;

impl Includer for NotFoundIncluder {
    fn get_resource(&self, name: &str, _args: HashMap<&str, &str>) -> Result<Cow<str>> {
        Ok(Cow::Owned(format!("[[div style=\"line-height: 141%; color: #b00; padding: 1em; margin: 1em; border: 1px solid #faa;\"]]\nIncluded page \"{}\" does not exist\n[[/div]]", name)))
    }
}

#[cfg(test)]
const TEST_CASES: [(&str, &str); 1] = [
    ("", ""),
];

#[test]
fn test_substitute() {
    use super::test::test_substitution;

    test_substitution("include", |s| substitute(s, &TextIncluder("<INCLUDE>")), &TEST_CASES);
}
