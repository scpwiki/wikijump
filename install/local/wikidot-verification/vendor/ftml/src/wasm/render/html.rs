/*
 * wasm/render/html.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2026 Wikijump Team
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

//! Isolated module for WASM HTML rendering.
//!
//! This submodule is separate to easily gate it within `#[cfg(feature = "html")]`,
//! and so imports essentially the same fields as its parent.

use super::super::page_info::PageInfo;
use super::super::parsing::SyntaxTree;
use super::super::prelude::*;
use super::super::settings::WikitextSettings;
use crate::render::Render;
use crate::render::html::{HtmlOutput as RustHtmlOutput, HtmlRender};
use std::sync::Arc;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct HtmlOutput {
    inner: Arc<RustHtmlOutput>,
}

#[wasm_bindgen]
impl HtmlOutput {
    #[wasm_bindgen]
    pub fn copy(&self) -> HtmlOutput {
        HtmlOutput {
            inner: Arc::clone(&self.inner),
        }
    }

    #[wasm_bindgen]
    pub fn body(&self) -> String {
        self.inner.body.clone()
    }

    #[wasm_bindgen]
    pub fn html_meta(&self) -> Result<JsValue, JsValue> {
        rust_to_js!(self.inner.meta)
    }

    #[wasm_bindgen]
    pub fn backlinks(&self) -> Result<JsValue, JsValue> {
        rust_to_js!(self.inner.backlinks)
    }
}

// Function exports

#[wasm_bindgen]
pub fn render_html(
    syntax_tree: SyntaxTree,
    page_info: PageInfo,
    settings: WikitextSettings,
) -> HtmlOutput {
    let tree = syntax_tree.get();
    let page_info = page_info.get();
    let settings = settings.get();
    let html = HtmlRender.render(tree, page_info, settings);

    HtmlOutput {
        inner: Arc::new(html),
    }
}
