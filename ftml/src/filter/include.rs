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
use std::ops::Range;

#[derive(Debug, Clone, Parser)]
#[grammar = "filter/include.pest"]
pub struct IncludeParser;

#[derive(Debug, Clone)]
struct IncludeRef {
    range: Range<usize>,
    resource: Cow<'static, str>,
}

// Helper function
pub fn substitute(text: &mut String, includer: &Includer) -> Result<()> {
    let pairs = match IncludeParser::parse(Rule::page, text) {
        Ok(mut pairs) => get_inner_pairs!(pairs),
        Err(err) => {
            return Err(Error::Msg(format!(
                "Include transform parsing error: {}",
                err
            )));
        }
    };

    let mut includes = Vec::new();
    let mut args = HashMap::new();

    // Iterate through [[include]]s
    for pair in pairs {
        if pair.as_rule() != Rule::include {
            continue;
        }

        let range = {
            let span = pair.as_span();
            span.start()..span.end()
        };

        let mut pairs = pair.into_inner();
        let name = {
            let pair = pairs.next().expect("Include pairs iterator was empty");

            debug_assert_eq!(pair.as_rule(), Rule::resource);

            pair.as_str()
        };

        // Parse arguments
        args.clear();
        for pair in pairs {
            debug_assert_eq!(pair.as_rule(), Rule::argument);

            let mut pairs = pair.into_inner();

            let key = pairs
                .next()
                .expect("Argument pairs iterator was empty")
                .as_str()
                .trim();

            let value = pairs
                .next()
                .expect("Argument pairs iterator had only one element")
                .as_str()
                .trim();

            args.insert(key, value);
        }

        // Fetch included resource
        // TODO limit recursion
        let resource = includer.get_resource(name, &args)?;

        includes.push(IncludeRef { range, resource });
    }

    // Go through in reverse order to not mess up indices
    //
    // This is playing a bit fast and loose with references since
    // we don't actually have a borrow of the string slice.

    includes.reverse();
    for include in includes {
        let IncludeRef { range, resource } = include;

        text.replace_range(range, resource.as_ref());
    }

    // TODO

    Ok(())
}

// Trait definition
pub trait Includer {
    fn get_resource(&self, name: &str, args: &HashMap<&str, &str>) -> Result<Cow<'static, str>>;
}

// Implementations
#[derive(Debug, Clone)]
pub struct NullIncluder;

impl Includer for NullIncluder {
    fn get_resource(&self, _name: &str, _args: &HashMap<&str, &str>) -> Result<Cow<'static, str>> {
        Ok(Cow::Borrowed(""))
    }
}

#[derive(Debug, Clone)]
pub struct TestIncluder;

impl Includer for TestIncluder {
    fn get_resource(&self, name: &str, args: &HashMap<&str, &str>) -> Result<Cow<'static, str>> {
        Ok(Cow::Owned(format!(
            "<INCLUDE '{}' #{}>",
            name,
            args.len(),
        )))
    }
}

#[derive(Debug, Clone)]
pub struct NotFoundIncluder;

impl Includer for NotFoundIncluder {
    fn get_resource(&self, name: &str, _args: &HashMap<&str, &str>) -> Result<Cow<'static, str>> {
        Ok(Cow::Owned(format!(
            "[[div style=\"line-height: 141%; color: #b00; padding: 1em; margin: 1em; border: 1px solid #faa;\"]]\nIncluded page \"{}\" does not exist\n[[/div]]", name,
        )))
    }
}

#[cfg(test)]
const TEST_CASES: [(&str, &str); 11] = [
    ("", ""),
    ("[[include component:thingy]]", "<INCLUDE 'component:thingy' #0>"),
    (
        "[[include component:image-block\n  name=test.png |\n  caption=SCP-XX\n]]",
        "<INCLUDE 'component:image-block' #2>",
    ),
    (
        "apple [[include some-page key=value | key2=value2]] banana",
        "apple <INCLUDE 'some-page' #2> banana",
    ),
    (
        "A\n[[include first-page\n  name=test |\n  caption=thing |\n]]\nB\n[[include second-page]]\nC",
        "A\n<INCLUDE 'first-page' #2>\nB\n<INCLUDE 'second-page' #0>\nC",
    ),
    (
        "A\n[[include B]]\nC\n[[include D]]\nE\n[[include F]]\nG\n[[include H]]\nI\n[[include J]]\nK",
        "A\n<INCLUDE 'B' #0>\nC\n<INCLUDE 'D' #0>\nE\n<INCLUDE 'F' #0>\nG\n<INCLUDE 'H' #0>\nI\n<INCLUDE 'J' #0>\nK",
    ),
    (
        "[[ INCLUDE component:thing \n\n | name = ARG yes amazing thing\n with newline | ]]",
        "<INCLUDE 'component:thing' #1>",
    ),
    (
        "A\n[[include no-sep arg = value]]\nB",
        "A\n<INCLUDE 'no-sep' #1>\nB",
    ),
    (
        "A\n[[include pre-sep | arg = value]]\nB",
        "A\n<INCLUDE 'pre-sep' #1>\nB",
    ),
    (
        "A\n[[include post-sep arg = value | ]]\nB",
        "A\n<INCLUDE 'post-sep' #1>\nB",
    ),
    (
        "A\n[[include both-sep | arg = value | ]]\nB",
        "A\n<INCLUDE 'both-sep' #1>\nB",
    ),
];

#[test]
fn test_substitute() {
    use super::test::test_substitution;

    test_substitution("include", |s| substitute(s, &TestIncluder), &TEST_CASES);
}
