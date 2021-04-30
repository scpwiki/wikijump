/*
 * wasm/utf16.rs
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
use crate::Utf16IndexMap as RustUtf16IndexMap;
use ouroboros::self_referencing;
use std::sync::Arc;

// Typescript declarations

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "[number, number]")]
    pub type IIndexSpan;
}

// Wrapper structures

#[self_referencing]
#[derive(Debug)]
struct Utf16IndexMapInner {
    text: String,

    #[borrows(text)]
    #[covariant]
    inner: RustUtf16IndexMap<'this>,
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct Utf16IndexMap {
    inner: Arc<Utf16IndexMapInner>,
}

#[wasm_bindgen]
impl Utf16IndexMap {
    #[inline]
    pub(crate) fn get(&self) -> &RustUtf16IndexMap {
        self.inner.borrow_inner()
    }

    #[wasm_bindgen(constructor)]
    pub fn new(text: String) -> Utf16IndexMap {
        let inner = Utf16IndexMapInnerBuilder {
            text,
            inner_builder: |text: &str| RustUtf16IndexMap::new(text),
        };

        Utf16IndexMap {
            inner: Arc::new(inner.build()),
        }
    }

    #[wasm_bindgen]
    pub fn copy(&self) -> Utf16IndexMap {
        Utf16IndexMap {
            inner: Arc::clone(&self.inner),
        }
    }

    fn check_index(&self, index: usize) -> Result<(), JsValue> {
        let text = self.inner.borrow_text();

        // Since we don't want the process to panic,
        // we do the check ourselves and throw a JS exception.
        if index > text.len() {
            let message = format!(
                "UTF-8 byte index out of range: {} (byte length {}",
                index,
                text.len(),
            );

            Err(JsValue::from_str(&message))
        } else {
            Ok(())
        }
    }

    #[wasm_bindgen]
    pub fn get_index(&self, index: usize) -> Result<usize, JsValue> {
        self.check_index(index)?;

        let new_index = self.get().get_index(index);
        Ok(new_index)
    }

    #[wasm_bindgen]
    pub fn get_span(&self, start: usize, end: usize) -> Result<IIndexSpan, JsValue> {
        self.check_index(start)?;
        self.check_index(end)?;

        let new_start = self.get().get_index(start);
        let new_end = self.get().get_index_end(end);
        rust_to_js!(vec![new_start, new_end])
    }
}
