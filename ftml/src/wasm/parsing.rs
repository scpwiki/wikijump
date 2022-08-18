/*
 * wasm/parsing.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2022 Wikijump Team
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
use super::prelude::*;
use super::settings::WikitextSettings;
use super::tokenizer::Tokenization;
use crate::parsing::{
    ParseException as RustParseException, ParseOutcome as RustParseOutcome,
};
use crate::tree::SyntaxTree as RustSyntaxTree;
use crate::utf16::Utf16IndexMap;
use crate::Tokenization as RustTokenization;
use std::sync::Arc;

// Typescript declarations

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &str = r#"

export interface IElement {
    element: string;
    data?: any;
}

export interface ISyntaxTree {
    elements: IElement[];
    styles: string[];
}

export interface IParseException {
    token: string;
    rule: string;
    span: {
        start: number;
        end: number;
    };
    kind: string;
}

"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ISyntaxTree")]
    pub type ISyntaxTree;

    #[wasm_bindgen(typescript_type = "IParseException[]")]
    pub type IParseExceptionArray;
}

// Wrapper structures

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ParseOutcome {
    inner: Arc<RustParseOutcome<RustSyntaxTree<'static>>>,
}

#[wasm_bindgen]
impl ParseOutcome {
    #[wasm_bindgen]
    pub fn copy(&self) -> ParseOutcome {
        ParseOutcome {
            inner: Arc::clone(&self.inner),
        }
    }

    #[wasm_bindgen]
    pub fn syntax_tree(&self) -> SyntaxTree {
        let tree = self.inner.value().clone();

        SyntaxTree {
            inner: Arc::new(tree),
        }
    }

    #[wasm_bindgen(typescript_type = "IParseException")]
    pub fn exceptions(&self) -> Result<IParseExceptionArray, JsValue> {
        rust_to_js!(self.inner.exceptions())
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct SyntaxTree {
    inner: Arc<RustSyntaxTree<'static>>,
}

#[wasm_bindgen]
impl SyntaxTree {
    #[inline]
    pub(crate) fn get(&self) -> &RustSyntaxTree<'static> {
        &self.inner
    }

    #[wasm_bindgen]
    pub fn copy(&self) -> SyntaxTree {
        SyntaxTree {
            inner: Arc::clone(&self.inner),
        }
    }

    #[wasm_bindgen(typescript_type = "ISyntaxTree")]
    pub fn data(&self) -> Result<ISyntaxTree, JsValue> {
        rust_to_js!(*self.inner)
    }
}

// Exported functions

#[wasm_bindgen]
pub fn parse(
    tokens: Tokenization,
    page_info: PageInfo,
    settings: WikitextSettings,
) -> Result<ParseOutcome, JsValue> {
    // Borrow and perform parsing
    let tokenization = tokens.get();
    let page_info = page_info.get();
    let settings = settings.get();
    let (syntax_tree, exceptions) =
        crate::parse(tokenization, page_info, settings).into();

    // Deep-clone AST to make it owned, so it can be
    // safely passed to JS, where it will live for an unknown time.
    let syntax_tree = syntax_tree.to_owned();

    // Convert exceptions to use UTF-16 indices
    let exceptions = convert_exceptions_utf16(tokenization, exceptions);

    // Create inner wrapper
    let inner = Arc::new(RustParseOutcome::new(syntax_tree, exceptions));

    Ok(ParseOutcome { inner })
}

// Utility functions

fn convert_exceptions_utf16(
    tokenization: &RustTokenization,
    exceptions: Vec<RustParseException>,
) -> Vec<RustParseException> {
    // As an optimization, we can avoid the (relatively expensive) Utf16IndexMap creation
    // if we know there are no exceptions to map indices of.
    if exceptions.is_empty() {
        return exceptions;
    }

    let full_text = tokenization.full_text().inner();
    let utf16_map = Utf16IndexMap::new(full_text);

    exceptions
        .into_iter()
        .map(|excpt| excpt.to_utf16_indices(&utf16_map))
        .collect()
}
