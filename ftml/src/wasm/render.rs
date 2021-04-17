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

use super::error::error_to_js;
use super::parsing::SyntaxTree;
use super::prelude::*;
use crate::data::PageInfo as RustPageInfo;
use crate::render::html::{HtmlOutput as RustHtmlOutput, HtmlRender};
use crate::render::Render;
use std::sync::Arc;

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

export interface IPageInfo {
    slug: string;
    category: string | null;
    title: string;
    alt_title: string | null;
    rating: number;
    tags: string[];
    locale: string;
}

"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IHtmlMeta[]")]
    pub type IHtmlMetaArray;

    #[wasm_bindgen(typescript_type = "IPageInfo")]
    pub type IPageInfo;
}

// Wrapper structures

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct PageInfo {
    inner: Arc<RustPageInfo<'static>>,
}

#[wasm_bindgen]
impl PageInfo {
    #[inline]
    pub(crate) fn get(&self) -> &RustPageInfo<'static> {
        &self.inner
    }

    #[wasm_bindgen]
    pub fn copy(&self) -> PageInfo {
        PageInfo {
            inner: Arc::clone(&self.inner),
        }
    }

    #[wasm_bindgen(constructor, typescript_type = "IPageInfo")]
    pub fn new(object: IPageInfo) -> Result<PageInfo, JsValue> {
        let rust_page_info = object.into_serde().map_err(error_to_js)?;

        Ok(PageInfo {
            inner: Arc::new(rust_page_info),
        })
    }
}

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
pub fn render_html(page_info: PageInfo, syntax_tree: SyntaxTree) -> HtmlOutput {
    let log = &*LOGGER;
    let page_info = page_info.get();
    let tree = syntax_tree.get();
    let html = HtmlRender.render(&log, page_info, tree);
    HtmlOutput(html)
}
