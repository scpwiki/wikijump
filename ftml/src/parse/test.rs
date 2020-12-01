/*
 * parse/test.rs
 *
 * ftml - Library to parse Wikidot code
 * Copyright (C) 2019-2020 Ammon Smith
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

use crate::tree::SyntaxTree;

#[test]
fn ast() {
    let log = crate::build_logger();

    macro_rules! test {
        ($text:expr, $elements:expr, $errors:expr,) => {
            test!($text, $elements, $errors)
        };

        ($text:expr, $elements:expr, $errors:expr) => {{
            let text = $text;
            let expected_elements = $elements;
            let expected_errors = $errors;

            println!("Testing parsing! input: {:?}", text);
            println!("Expected elements: {:#?}", expected_elements);
            println!("Expected errors: {:#?}", expected_errors);

            info!(&log, "Testing AST parsing!"; "text" => text);

            let tokens = crate::tokenize(&log, text);
            let result = crate::parse(&log, &tokens);
            let (tree, errors) = result.into();
            let SyntaxTree { elements } = tree;

            assert_eq!(
                elements,
                expected_elements,
                "Resultant elements (left) did not match expected (right)",
            );

            assert_eq!(
                errors,
                expected_errors,
                "Resultant error list (left) did not match expected (right)",
            );
        }};
    }

    test!(
        "",
        vec![],
        vec![],
    );
}

#[test]
fn json() {
    let log = crate::build_logger();
    let text = "**apple //banana//** cherry";
    let tokens = crate::tokenize(&log, text);
    let result = crate::parse(&log, &tokens);
    println!("{:#?}", result.value());
    println!("Errors: {:#?}", result.errors());

    let json = serde_json::to_string_pretty(&result).unwrap();
    println!("JSON:\n{}", json);
}
