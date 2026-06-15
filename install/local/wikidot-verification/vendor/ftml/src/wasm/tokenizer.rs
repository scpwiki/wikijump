/*
 * wasm/tokenizer.rs
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
use crate::Tokenization as RustTokenization;
use crate::parsing::ExtractedToken as RustExtractedToken;
use crate::utf16::Utf16IndexMap;
use self_cell::self_cell;
use std::sync::Arc;

self_cell!(
    struct TokenizationInner {
        owner: String,

        #[covariant]
        dependent: RustTokenization,
    }

    impl {Debug}
);

#[wasm_bindgen]
#[derive(Debug)]
pub struct Tokenization {
    inner: Arc<TokenizationInner>,
}

#[wasm_bindgen]
impl Tokenization {
    #[inline]
    pub(crate) fn get(&self) -> &RustTokenization<'_> {
        self.inner.borrow_dependent()
    }

    #[wasm_bindgen]
    pub fn copy(&self) -> Tokenization {
        Tokenization {
            inner: Arc::clone(&self.inner),
        }
    }

    #[wasm_bindgen]
    pub fn text(&self) -> String {
        self.inner.borrow_owner().clone()
    }

    #[wasm_bindgen]
    pub fn tokens(&self) -> Result<JsValue, JsValue> {
        self.inner
            .with_dependent(|_, inner| rust_to_js!(convert_tokens_utf16(inner)))
    }
}

// Exported functions

#[wasm_bindgen]
pub fn tokenize(text: String) -> Tokenization {
    let inner = TokenizationInner::new(text, |text: &String| crate::tokenize(text));

    Tokenization {
        inner: Arc::new(inner),
    }
}

// Utility functions

fn convert_tokens_utf16<'a>(
    tokenization: &'a RustTokenization,
) -> Vec<RustExtractedToken<'a>> {
    // Because the list of tokens is almost certainly not empty,
    // we don't perform the same check here that we do for errors.

    let full_text = tokenization.full_text().inner();
    let utf16_map = Utf16IndexMap::new(full_text);

    tokenization
        .tokens()
        .iter()
        .map(|token| token.to_utf16_indices(&utf16_map))
        .collect()
}
