/*
 * wasm/render.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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

use super::parsing::SyntaxTree;
use super::prelude::*;
use crate::render::html::{HtmlOutput as RustHtmlOutput, HtmlRender};
use crate::render::Render;

// Typescript declarations

// TODO

// Wrapper structures

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct HtmlOutput(RustHtmlOutput);

// Exported functions

#[wasm_bindgen]
pub fn render_html(syntax_tree: SyntaxTree, should_log: bool) -> HtmlOutput {
    let log = get_logger(should_log);
    let tree = syntax_tree.borrow();
    let html = HtmlRender.render(&log, tree);
    HtmlOutput(html)
}
