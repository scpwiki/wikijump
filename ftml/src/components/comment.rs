/*
 * components/comment.rs
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
    pub static ref COMMENT: Regex = build_regex(
        r"(\n)?\[!\-\-(.*?)\-\-\]",
        "si",
    );
}

#[derive(Debug)]
pub struct Comment;

impl Comment {
    pub fn new(text: &str) -> Option<Self> {
        COMMENT.captures(text).map(|_| Comment)
    }
}

impl Component for Bold {
    fn start(&self, f: &mut fmt::Formatter, classes: &str) -> fmt::Result {
        Ok(())
    }

    fn end(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}
