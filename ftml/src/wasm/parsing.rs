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

use super::prelude::*;
use super::tokenizer::Tokenization;
use crate::parsing::ParseWarning as RustParseWarning;
use crate::tree::SyntaxTree as RustSyntaxTree;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ParseOutcome {
    syntax_tree: SyntaxTree,
    warnings: ParseWarnings,
}

#[wasm_bindgen]
impl ParseOutcome {
    #[wasm_bindgen]
    pub fn syntax_tree(&self) -> SyntaxTree {
        self.syntax_tree.clone()
    }

    #[wasm_bindgen]
    pub fn warnings(&self) -> ParseWarnings {
        self.warnings.clone()
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct SyntaxTree(Arc<RustSyntaxTree<'static>>);

impl SyntaxTree {
    #[inline]
    pub fn get(&self) -> &RustSyntaxTree<'static> {
        &*self.0
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ParseWarnings(Arc<Vec<RustParseWarning>>);

impl ParseWarnings {
    #[inline]
    pub fn get(&self) -> &[RustParseWarning] {
        &*self.0
    }
}

#[wasm_bindgen]
pub fn parse(tokens: Tokenization, should_log: bool) -> Result<ParseOutcome, JsValue> {
    let log = get_logger(should_log);

    // Borrow and perform parsing
    let tokenization = tokens.borrow_inner();
    let (syntax_tree, warnings) = crate::parse(log, tokenization).into();

    // HACK: instead of implementing an exhaustive
    // to_owned() clone for all of the sub-objects,
    // we're just going to serialize/deserialize.
    let syntax_tree = {
        let syntax_tree_js = serde_wasm_bindgen::to_value(&syntax_tree)?;
        let syntax_tree = serde_wasm_bindgen::from_value(syntax_tree_js)?;
        syntax_tree
    };

    // Build JS-compatible objects
    let syntax_tree = SyntaxTree(Arc::new(syntax_tree));
    let warnings = ParseWarnings(Arc::new(warnings.into_iter().collect()));

    let outcome = ParseOutcome {
        syntax_tree,
        warnings,
    };

    Ok(outcome)
}
