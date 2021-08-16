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
use crate::render::html::{HtmlOutput as RustHtmlOutput, HtmlRender};
use crate::render::text::TextRender;
use crate::render::Render;
use crate::PageInfo as RustPageInfo;
use ref_map::*;
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

export interface IPageInfo {
    page: string;
    category: string | null;
    site: string;
    title: string;
    alt_title: string | null;
    rating: number;
    tags: string[];
    language: string;
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

    #[wasm_bindgen(typescript_type = "IPageInfo")]
    pub type IPageInfo;

    #[wasm_bindgen(typescript_type = "string[]")]
    pub type ITags;
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

    // Getters

    #[wasm_bindgen(method, getter)]
    pub fn page(&self) -> String {
        self.inner.page.to_string()
    }

    #[wasm_bindgen(method, getter)]
    pub fn category(&self) -> Option<String> {
        self.inner.category.ref_map(ToString::to_string)
    }

    #[wasm_bindgen(method, getter)]
    pub fn site(&self) -> String {
        self.inner.site.to_string()
    }

    #[wasm_bindgen(method, getter)]
    pub fn title(&self) -> String {
        self.inner.title.to_string()
    }

    #[wasm_bindgen(method, getter)]
    pub fn alt_title(&self) -> Option<String> {
        self.inner.alt_title.ref_map(ToString::to_string)
    }

    #[wasm_bindgen(method, getter)]
    pub fn rating(&self) -> f32 {
        self.inner.rating
    }

    #[wasm_bindgen(method, getter, typescript_type = "ITags")]
    pub fn tags(&self) -> Result<ITags, JsValue> {
        rust_to_js!(self.inner.tags)
    }

    #[wasm_bindgen(method, getter)]
    pub fn language(&self) -> String {
        self.inner.language.to_string()
    }
}

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
pub fn render_html(page_info: PageInfo, syntax_tree: SyntaxTree) -> HtmlOutput {
    let log = &*LOGGER;
    let page_info = page_info.get();
    let tree = syntax_tree.get();
    let html = HtmlRender.render(&log, page_info, tree);

    HtmlOutput {
        inner: Arc::new(html),
    }
}

#[wasm_bindgen]
pub fn render_text(page_info: PageInfo, syntax_tree: SyntaxTree) -> String {
    let log = &*LOGGER;
    let page_info = page_info.get();
    let tree = syntax_tree.get();
    let text = TextRender.render(&log, page_info, tree);

    text
}
