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
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
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

/// A list of all Fluent functions made available by DEEPWELL.
/// Any outside this list will be considered invalid.
const VALID_FLUENT_FUNCTIONS: [&str; 0] = [];

#[derive(Debug, Default, Clone)]
pub struct Catalog {
    locales: HashMap<LanguageIdentifier, Messages>,
    terms: HashSet<String>,
}

impl Catalog {
    pub fn add_message(&mut self, locale: LanguageIdentifier, message: &ast::Message<&str>) {
        let base_key = message.id.name;
        let messages = self.locales.entry(locale).or_default();

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

    pub fn add_term(&mut self, term: &ast::Term<&str>) {
        let base_key = term.id.name;

        // There is always a value, so no if let.
        self.terms.insert(str!(base_key));

        for ast::Attribute { id, .. } in &term.attributes {
            let key = format!("{}.{}", base_key, id.name);
            self.terms.insert(key);
        }
    }

    pub fn print_summary(&self) {
        println!();
        println!("Found locales:");

        for locale in self.locales.keys() {
            println!("* {}", locale);
        }

        println!();
        println!("Found terms:");

        for term in &self.terms {
            println!("* {}", term);
        }
    }

    pub fn check(&self, return_code: &mut u8) {
        macro_rules! fail {
            ($($arg:tt)*) => {{
                *return_code = 1;
                eprint!("!! ");
                eprintln!($($arg)*);
            }};
        }

        println!();
        println!(
            "Running checks, comparing to primary locale {}...",
            PRIMARY_LOCALE,
        );

        let primary = match self.locales.get(&PRIMARY_LOCALE) {
            Some(messages) => messages,
            None => {
                fail!("No messages found for primary locale");
                return;
            }
        };

        for (locale, messages) in self
            .locales
            .iter()
            .filter(|(locale, _)| *locale != &PRIMARY_LOCALE)
        {
            println!("+ Checking locale {}", locale);

            for (key, usages) in messages.iter() {
                // Ensure all paths match ones in the primary
                let primary_usages = match primary.get(key) {
                    Some(usages) => usages,
                    None => {
                        fail!("Message key not found in parent: {}", key);
                        continue;
                    }
                };

                // Check usage information
                for function in &usages.functions {
                    if !VALID_FLUENT_FUNCTIONS.contains(&function.as_str()) {
                        fail!("Invalid Fluent function {}", function);
                    }
                }

                for variable in &usages.variables {
                    if !primary_usages.variables.contains(&variable) {
                        fail!("Variable reference not found in parent: {}", variable);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Messages {
    inner: HashMap<String, MessageUsages>,
}

impl Messages {
    pub fn add(&mut self, key: String, usages: MessageUsages) {
        if self.inner.contains_key(&key) {
            // We do check/panic instead of insert()
            // because the key is gone once we insert,
            // so we can't use it in our message without cloning.
            panic!("Duplicate message key: {}", key);
        }

        self.inner.insert(key, usages);
    }
}

impl Deref for Messages {
    type Target = HashMap<String, MessageUsages>;

    fn deref(&self) -> &Self::Target {
        &self.inner
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
