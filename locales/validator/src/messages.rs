/*
 * messages.rs
 *
 * wikijump-locales-validator - Validate Wikijump's Fluent localization files
 * Copyright (C) 2021 Wikijump Team
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

use fluent_syntax::ast;
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

/// The "primary" locale, to compare other locales against.
///
/// This is defined as one which is always complete, containing
/// every message key used by the application.
///
/// Thus, we can compare all other locales to it, ensuring they
/// are equal or subsets, raising errors on any new message keys,
/// as they are either typos or removed keys.
const PRIMARY_LOCALE: LanguageIdentifier = langid!("en");

#[derive(Debug, Default, Clone)]
pub struct Catalog {
    locales: HashMap<LanguageIdentifier, Messages>,
}

impl Catalog {
    pub fn add_message(&mut self, locale: LanguageIdentifier, message: &ast::Message<&str>) {
        let mut messages = self.locales.entry(locale).or_default();
        let base_key = message.id.name;

        if let Some(ast::Pattern { elements }) = &message.value {
            let key = str!(base_key);
            let usages = MessageUsages::from_elements(elements);
            messages.add(key, usages);
        }

        for ast::Attribute { id, value } in &message.attributes {
            let key = format!("{}.{}", base_key, id.name);
            let usages = MessageUsages::from_elements(&value.elements);
            messages.add(key, usages);
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Messages {
    messages: HashMap<String, MessageUsages>,
}

impl Messages {
    pub fn add(&mut self, key: String, usages: MessageUsages) {
        if self.messages.contains_key(&key) {
            // We do check/panic instead of insert()
            // because the key is gone once we insert,
            // so we can't use it in our message without cloning.
            panic!("Duplicate message key: {}", key);
        }

        self.messages.insert(key, usages);
    }
}

#[derive(Debug, Default, Clone)]
pub struct MessageUsages {
    functions: Vec<String>,
    messages: Vec<String>,
    terms: Vec<String>,
    variables: Vec<String>,
}

impl MessageUsages {
    pub fn from_elements(elements: &[ast::PatternElement<&str>]) -> Self {
        let mut usages = Self::default();
        usages.add_elements(elements);
        usages
    }

    pub fn add_elements(&mut self, elements: &[ast::PatternElement<&str>]) {
        use ast::PatternElement::*;

        for element in elements {
            match element {
                TextElement { .. } => (),
                Placeable { expression } => {
                    self.add_expression(expression);
                }
            }
        }
    }

    pub fn add_expression(&mut self, expression: &ast::Expression<&str>) {
        use ast::Expression::*;

        match expression {
            Select {
                selector: _,
                variants,
            } => {
                for variant in variants {
                    self.add_elements(&variant.value.elements);
                }
            }
            Inline(inline_expr) => {
                self.add_inline_expression(inline_expr);
            }
        }
    }

    pub fn add_inline_expression(&mut self, inline_expr: &ast::InlineExpression<&str>) {
        use ast::InlineExpression::*;

        match inline_expr {
            StringLiteral { .. } | NumberLiteral { .. } => (),
            FunctionReference { id, .. } => self.functions.push(str!(id.name)),
            MessageReference { id, .. } => self.messages.push(str!(id.name)),
            TermReference { id, .. } => self.terms.push(str!(id.name)),
            VariableReference { id, .. } => self.variables.push(str!(id.name)),
            Placeable { expression } => self.add_expression(expression),
        }
    }
}
