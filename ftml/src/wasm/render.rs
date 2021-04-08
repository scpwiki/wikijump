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

use super::prelude::*;
use crate::render::{html::HtmlRender, Render};
use crate::tree::SyntaxTree;

// TODO render isn't implemented yet, this is mostly a stub

// Return type here is really Result<HtmlOutput, serde_wasm_bindgen::Error>,
// but converted into JsValue.

#[wasm_bindgen]
pub fn render_html(syntax_tree: JsValue) -> Result<JsValue, JsValue> {
    let tree: SyntaxTree = serde_wasm_bindgen::from_value(syntax_tree)?;
    let output = HtmlRender.render(&tree);
    let output_js = serde_wasm_bindgen::to_value(&output)?;
    Ok(output_js)
}
