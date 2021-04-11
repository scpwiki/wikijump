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
use ouroboros::self_referencing;
use wasm_bindgen::JsCast;

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

#[self_referencing]
#[derive(Debug)]
struct TokenizationInner {
    text: String,

    #[borrows(text)]
    #[covariant]
    inner: RustTokenization<'this>,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Tokenization(Arc<TokenizationInner>);

#[wasm_bindgen]
impl Tokenization {
    #[inline]
    pub(crate) fn borrow_inner(&self) -> &RustTokenization {
        self.0.borrow_inner()
    }

    #[wasm_bindgen(typescript_type = "ITokenArray")]
    pub fn tokens(&self) -> Result<ITokenArray, JsValue> {
        self.0.with_inner(|inner| {
            let tokens = inner.tokens();
            let js = JsValue::from_serde(&tokens).map_err(error_to_js)?;
            Ok(js.unchecked_into())
        })
    }
}

// Exported functions

#[wasm_bindgen]
pub fn tokenize(text: String, should_log: bool) -> Tokenization {
    let log = get_logger(should_log);

    let inner = TokenizationInnerBuilder {
        text,
        inner_builder: |text: &str| crate::tokenize(&log, text),
    };

    Tokenization(Arc::new(inner.build()))
}
