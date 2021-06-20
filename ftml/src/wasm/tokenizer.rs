/*
 * wasm/tokenizer.rs
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
use crate::Tokenization as RustTokenization;
use self_cell::self_cell;
use std::sync::Arc;

// Typescript declarations

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &str = r#"

export interface IToken {
    token: string;
    slice: string;
    span: {
        start: number;
        end: number;
    };
}

"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IToken[]")]
    pub type ITokenArray;
}

// Wrapper structures

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
    pub(crate) fn get(&self) -> &RustTokenization {
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

    #[wasm_bindgen(typescript_type = "ITokenArray")]
    pub fn tokens(&self) -> Result<ITokenArray, JsValue> {
        self.inner
            .with_dependent(|_, inner| rust_to_js!(inner.tokens()))
    }
}

// Exported functions

#[wasm_bindgen]
pub fn tokenize(text: String) -> Tokenization {
    let log = &*LOGGER;
    let inner =
        TokenizationInner::new(text, |text: &String| crate::tokenize(&log, text));

    Tokenization {
        inner: Arc::new(inner),
    }
}
