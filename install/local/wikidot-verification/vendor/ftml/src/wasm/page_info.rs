/*
 * wasm/page_info.rs
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

use super::prelude::*;
use crate::data::PageInfo as RustPageInfo;
use ref_map::*;
use std::sync::Arc;

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

    #[wasm_bindgen(constructor)]
    pub fn new(info: JsValue) -> Result<PageInfo, JsValue> {
        Ok(PageInfo {
            inner: Arc::new(js_to_rust!(info)?),
        })
    }

    // Getters

    #[wasm_bindgen(getter)]
    pub fn page(&self) -> String {
        self.inner.page.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn category(&self) -> Option<String> {
        self.inner.category.ref_map(ToString::to_string)
    }

    #[wasm_bindgen(getter)]
    pub fn site(&self) -> String {
        self.inner.site.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn title(&self) -> String {
        self.inner.title.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn alt_title(&self) -> Option<String> {
        self.inner.alt_title.ref_map(ToString::to_string)
    }

    #[wasm_bindgen(getter)]
    pub fn score(&self) -> f64 {
        self.inner.score.to_f64()
    }

    #[wasm_bindgen(getter)]
    pub fn tags(&self) -> Result<JsValue, JsValue> {
        rust_to_js!(self.inner.tags)
    }

    #[wasm_bindgen(getter)]
    pub fn language(&self) -> String {
        self.inner.language.to_string()
    }
}
