/*
 * components/bold.rs
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
    pub static ref BOLD: Regex = build_regex(
        r"\*\*([^\*].*)\*\*",
        "U",
    );
}

#[derive(Debug)]
pub struct Bold<'a> {
    content: &'a str,
}

impl<'a> Bold<'a> {
    pub fn new(text: &'a str) -> Option<Self> {
        BOLD.captures(text).map(|m| Bold { content: m[1] })
    }
}

impl<'a> Component for Bold<'a> {
    fn start(&self, f: &mut fmt::Formatter, classes: &str) -> fmt::Result {
        write!(f, "<b class=\"{}\">{}</b>", style, self.content)
    }
}
