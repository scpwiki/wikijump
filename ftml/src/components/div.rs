/*
 * components/div.rs
 *
 * wikidot-html - Library to convert Wikidot syntax into HTML
 * Copyright (c) 2019 Ammon Smith for Project Foundation
 *
 * wikidot-html is available free of charge under the terms of the MIT
 * License. You are free to redistribute and/or modify it under those
 * terms. It is distributed in the hopes that it will be useful, but
 * WITHOUT ANY WARRANTY. See the LICENSE file for more details.
 *
 */

use super::prelude::*;

lazy_static! {
    pub static ref DIV: Regex = build_regex(
        r"(?:\n)?\[\[div(\s.*?)?\]\] *\n((?:(?R)|.)*?)\[\[\/div\]\] *",
        "msi",
    );
}

#[derive(Debug)]
pub struct Div<'a> {
    style: &'a str,
    content: &'a str,
}

impl<'a> Div<'a> {
    pub fn new(text: &'a str) -> Option<Self> {
        DIV.captures(text).map(|m| Div { style: m[1], content: m[2] })
    }
}

impl<'a> Component for Div<'a> {
    fn start(&self, f: &mut fmt::Formatter, classes: &str) -> fmt::Result {
        assert_eq!(classes, "");

        write!(f, "<div {}>\n{}\n</div>", self.style, self.content)
    }
}
