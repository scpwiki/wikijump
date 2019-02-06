/*
 * components/color.rs
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
    pub static ref COLOR_TEXT: Regex = build_regex(
        r"##(.+?)\|(.+?)##",
        "",
    );
}

#[derive(Debug)]
pub struct ColorText<'a> {
    color: &'a str,
    content: &'a str,
}

impl<'a> ColorText<'a> {
    pub fn new(text: &'a str) -> Option<Self> {
        COLOR_TEXT.captures(text).map(|m| ColorText { color: m[1], content: m[2] })
    }
}

impl<'a> Component for Bold<'a> {
    fn start(&self, f: &mut fmt::Formatter, classes: &str) -> fmt::Result {
        write!(f, "<span style=\"color: {};\" class=\"{}\">", self.color, classes, self.content)
    }
}
