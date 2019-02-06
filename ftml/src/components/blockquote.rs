/*
 * components/blockquote.rs
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
    pub static ref BLOCKQUOTE: Regex = build_regex(
        r"\n(?:(?:\>).*\n)(\?!(?:\>))",
        "Us",
    );
}

#[derive(Debug)]
pub struct BlockQuote;

impl BlockQuote {
    pub fn new(text: &'a str) -> Option<Self> {
        BLOCKQUOTE.captures(text).map(|_| BlockQuote)
    }
}

impl Component for BlockQuote {
    fn start(&self, f: &mut fmt::Formatter, classes: &str) -> fmt::Result {
        write!(f, "<blockquote class=\"{}\">", classes)
    }

    fn end(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "</blockquote>\n")
    }
}
