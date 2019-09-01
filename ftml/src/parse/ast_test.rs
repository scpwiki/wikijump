/*
 * parse/ast_test.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith
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

//! Tests for the parser.
//! This ensures that all input strings produce the expected output AST.

use super::{parse, SyntaxTree};

macro_rules! valid {
    ($input:expr, $expected:expr) => (
        match parse($input) {
            Ok(ast) => assert_eq!(ast, $expected, "Outputed AST doesn't match expected"),
            Err(err) => panic!("Received error when parsing test input: {}", err),
        }
    )
}

macro_rules! invalid {
    ($input:expr) => (
        match parse($input) {
            Ok(ast) => panic!("Invalid test input parsed, produced AST: {:#?}", ast),
            Err(_) => (),
        }
    )
}

#[test]
fn test_valid() {
    valid!("", SyntaxTree::from_paragraphs(vec![]));
}

#[test]
fn test_invalid() {
    invalid!("[[");
}
