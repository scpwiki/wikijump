/*
 * render/html.rs
 *
 * wikidot-html - Convert Wikidot code to HTML
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

#![allow(unused_variable)]

use crate::{Result, SyntaxTree};
use crate::parse::{Line, LineInner, Word};
use std::fmt::{self, Display, Write};
use super::Render;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HtmlRender;

impl Render for HtmlRender {
    type Output = String;

    fn render(tree: &SyntaxTree) -> Result<String> {
        let mut buffer = String::new();

        for line in tree.lines() {
            write!(&mut buffer, "{}", LineWrap(line))?;
        }

        Ok(buffer)
    }
}

#[derive(Debug)]
struct LineWrap<'r, 'l>(&'r Line<'l>);

impl<'r, 'l> Display for LineWrap<'r, 'l> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::LineInner::*;

        match self.0.inner() {
            Align { alignment, contents } => unimplemented!(),
            Center { contents } => unimplemented!(),
            ClearFloat { direction } => unimplemented!(),
            CodeBlock { language, contents } => unimplemented!(),
            Div { id, class, style, contents } => unimplemented!(),
            Heading { level, contents } => unimplemented!(),
            HorizontalLine => unimplemented!(),
            Html { contents } => unimplemented!(),
            Iframe { url, args } => unimplemented!(),
            IfTags { required, prohibited, contents } => unimplemented!(),
            List { style, items } => unimplemented!(),
            Math { label, id, latex_env, expr } => unimplemented!(),
            Table { rows } => unimplemented!(),
            TableOfContents { } => unimplemented!(),
            QuoteBlock { contents } => unimplemented!(),
            Words { centered, contents } => {
                if *centered {
                    write!(buffer, html! {
                        div(style="text-align: center") {
                            // TODO contents
                        }
                    }).unwrap();
                } else {
                    // TODO regular
                }
            },
        }

        // TODO add self.0.newlines

        Ok(())
    }
}
