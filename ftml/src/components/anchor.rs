/*
 * components/anchor.rs
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
    static ref ANCHOR: Regex = build_regex(
        r"(?:\[\[# )([-_A-Za-z0-9.%]+?)(?:\]\])",
        "i",
    );
}

#[derive(Debug)]
pub struct Anchor<'a> {
    name: &'a str,
}

impl<'a> Anchor<'a> {
    pub fn new(text: &'a str) -> Option<Self> {
        ANCHOR.captures(text).map(|m| Anchor { name: &m[1] })
    }
}

impl<'a> Component for Anchor<'a> {
    fn start(&self, f: &mut fmt::Formatter, classes: &str) -> fmt::Result {
        write!(f, "<a name=\"{}\" class=\"{}\"></a>", self.name, classes)
    }
}
