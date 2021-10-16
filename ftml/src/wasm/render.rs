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

use super::page_info::PageInfo;
use super::parsing::SyntaxTree;
use super::prelude::*;
use super::settings::WikitextSettings;
use crate::render::html::{HtmlOutput as RustHtmlOutput, HtmlRender};
use crate::render::text::TextRender;
use crate::render::Render;
use std::sync::Arc;

// Typescript declarations

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &str = r#"

export interface IHtmlOutput {
    body: string;
    style: string;
    meta: IHtmlMeta[];
}

export interface IHtmlMeta {
    tag_type: string;
    name: string;
    value: string;
}

export interface IBacklinks {
    included_pages: string[];
    internal_links: string[];
    external_links: string[];
}

"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "string[]")]
    pub type IStyleArray;

    #[wasm_bindgen(typescript_type = "IHtmlMeta[]")]
    pub type IHtmlMetaArray;

    #[wasm_bindgen(typescript_type = "IBacklinks")]
    pub type IBacklinks;
}

// Wrapper structures

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

    #[wasm_bindgen(typescript_type = "IStyleArray")]
    pub fn styles(&self) -> Result<IStyleArray, JsValue> {
        rust_to_js!(self.inner.styles)
    }

    #[wasm_bindgen(typescript_type = "IHtmlMetaArray")]
    pub fn html_meta(&self) -> Result<IHtmlMetaArray, JsValue> {
        rust_to_js!(self.inner.meta)
    }

    #[wasm_bindgen(typescript_type = "IBacklinks")]
    pub fn backlinks(&self) -> Result<IBacklinks, JsValue> {
        rust_to_js!(self.inner.backlinks)
    }
}

// Exported functions

#[wasm_bindgen]
pub fn render_html(
    syntax_tree: SyntaxTree,
    page_info: PageInfo,
    settings: WikitextSettings,
) -> HtmlOutput {
    let log = &*LOGGER;
    let tree = syntax_tree.get();
    let page_info = page_info.get();
    let settings = settings.get();
    let html = HtmlRender.render(&log, tree, page_info, settings);

    HtmlOutput {
        inner: Arc::new(html),
    }
}

#[wasm_bindgen]
pub fn render_text(
    syntax_tree: SyntaxTree,
    page_info: PageInfo,
    settings: WikitextSettings,
) -> String {
    let log = &*LOGGER;
    let tree = syntax_tree.get();
    let page_info = page_info.get();
    let settings = settings.get();
    let text = TextRender.render(&log, tree, page_info, settings);

    text
}
