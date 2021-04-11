/*
 * wasm/parsing.rs
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
use super::prelude::*;
use super::tokenizer::Tokenization;
use crate::parsing::ParseOutcome as RustParseOutcome;
use crate::tree::SyntaxTree as RustSyntaxTree;

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

export interface IParseWarning {
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

    #[wasm_bindgen(typescript_type = "IParseWarning[]")]
    pub type IParseWarningArray;
}

// Wrapper structures

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ParseOutcome(RustParseOutcome<RustSyntaxTree<'static>>);

#[wasm_bindgen]
impl ParseOutcome {
    #[wasm_bindgen]
    pub fn syntax_tree(&self) -> SyntaxTree {
        SyntaxTree(self.0.value().clone())
    }

    #[wasm_bindgen(typescript_type = "IParseWarning")]
    pub fn warnings(&self) -> Result<IParseWarningArray, JsValue> {
        rust_to_js!(self.0.warnings())
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct SyntaxTree(RustSyntaxTree<'static>);

#[wasm_bindgen]
impl SyntaxTree {
    #[inline]
    pub(crate) fn borrow(&self) -> &RustSyntaxTree<'static> {
        &self.0
    }

    #[wasm_bindgen(typescript_type = "ISyntaxTree")]
    pub fn get(&self) -> Result<ISyntaxTree, JsValue> {
        rust_to_js!(self.0)
    }
}

// Exported functions

#[wasm_bindgen]
pub fn parse(tokens: Tokenization) -> Result<ParseOutcome, JsValue> {
    let log = &*LOGGER;

    // Borrow and perform parsing
    let tokenization = tokens.borrow();
    let (syntax_tree, warnings) = crate::parse(log, tokenization).into();

    // HACK: instead of implementing an exhaustive
    // to_owned() clone for all of the sub-objects,
    // we're just going to serialize/deserialize.
    let syntax_tree = {
        let syntax_tree_js = JsValue::from_serde(&syntax_tree).map_err(error_to_js)?;
        let syntax_tree = JsValue::into_serde(&syntax_tree_js).map_err(error_to_js)?;
        syntax_tree
    };

    Ok(ParseOutcome(RustParseOutcome::new(syntax_tree, warnings)))
}
