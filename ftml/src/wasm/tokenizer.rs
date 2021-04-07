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
use ouroboros::self_referencing;
use std::sync::Arc;

#[self_referencing]
#[derive(Debug)]
struct TokenizationInner {
    text: String,

    #[borrows(text)]
    #[covariant]
    inner: crate::Tokenization<'this>,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Tokenization(Arc<TokenizationInner>);

#[wasm_bindgen]
impl Tokenization {
    #[inline]
    pub(crate) fn borrow_inner(&self) -> &crate::Tokenization {
        self.0.borrow_inner()
    }

    #[wasm_bindgen]
    pub fn tokens(&self) -> Result<JsValue, JsValue> {
        self.0.with_inner(|inner| {
            let tokens = inner.tokens();
            let js = serde_wasm_bindgen::to_value(&tokens)?;
            Ok(js)
        })
    }
}

#[wasm_bindgen]
pub fn tokenize(text: String, should_log: bool) -> Tokenization {
    let log = get_logger(should_log);

    let inner = TokenizationInnerBuilder {
        text,
        inner_builder: |text: &str| crate::tokenize(&log, text),
    };

    Tokenization(Arc::new(inner.build()))
}
