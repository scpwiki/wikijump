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

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &str = r#"

export interface IHtmlOutput {
    html: string;
    style: string;
    meta: IHtmlMeta[];
}

export interface IHtmlMeta {
    tag_type: string;
    name: string;
    value: string;
}

"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IHtmlMeta[]")]
    pub type IHtmlMetaArray;
}

// Wrapper structures

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct HtmlOutput(RustHtmlOutput);

#[wasm_bindgen]
impl HtmlOutput {
    #[wasm_bindgen]
    pub fn html(&self) -> String {
        self.0.html.clone()
    }

    #[wasm_bindgen]
    pub fn style(&self) -> String {
        self.0.style.clone()
    }

    #[wasm_bindgen(typescript_type = "IHtmlMetaArray")]
    pub fn html_meta(&self) -> Result<IHtmlMetaArray, JsValue> {
        rust_to_js!(self.0.meta)
    }
}

// Exported functions

#[wasm_bindgen]
pub fn render_html(syntax_tree: SyntaxTree, should_log: bool) -> HtmlOutput {
    let log = get_logger(should_log);
    let tree = syntax_tree.borrow();
    let html = HtmlRender.render(&log, tree);
    HtmlOutput(html)
}
