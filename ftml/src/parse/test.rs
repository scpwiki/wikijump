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

use crate::parse::{ParseError, ParseErrorKind, Token};
use crate::tree::{Container, ContainerType, Element, SyntaxTree};

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

            println!("Actual elements: {:#?}", elements);
            println!("Actual errors: {:#?}", errors);

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

    macro_rules! container {
        ($type:tt, $elements:expr) => {
            Element::Container(Container::new(ContainerType::$type, $elements))
        };
    }

    test!("", vec![], vec![]);

    test!(" ", vec![Element::Text(" ")], vec![]);

    test!("abc", vec![Element::Text("abc")], vec![]);

    test!("\n", vec![Element::LineBreak], vec![]);

    test!(
        "**bold** text",
        vec![
            container!(Bold, vec![Element::Text("bold")]),
            Element::Text(" "),
            Element::Text("text"),
        ],
        vec![],
    );

    test!(
        "**fail bold",
        vec![
            Element::Text("**"),
            Element::Text("fail"),
            Element::Text(" "),
            Element::Text("bold"),
        ],
        vec![ParseError::new_raw(
            Token::Bold,
            "fallback",
            0..2,
            ParseErrorKind::NoRulesMatch,
        )],
    );

    test!(
        "//italics// text",
        vec![
            container!(Italics, vec![Element::Text("italics")]),
            Element::Text(" "),
            Element::Text("text"),
        ],
        vec![],
    );

    test!(
        "//fail italics",
        vec![
            Element::Text("//"),
            Element::Text("fail"),
            Element::Text(" "),
            Element::Text("italics"),
        ],
        vec![ParseError::new_raw(
            Token::Italics,
            "fallback",
            0..2,
            ParseErrorKind::NoRulesMatch,
        )],
    );

    test!(
        "single [!-- stuff here --] comment",
        vec![
            Element::Text("single"),
            Element::Text(" "),
            Element::Text(" "),
            Element::Text("comment"),
        ],
        vec![],
    );

    test!(
        "multiline\n[!-- stuff \n here --]\n comment",
        vec![
            Element::Text("multiline"),
            Element::LineBreak,
            Element::LineBreak,
            Element::Text(" "),
            Element::Text("comment"),
        ],
        vec![],
    );

    test!(
        "fail [!-- comment",
        vec![
            Element::Text("fail"),
            Element::Text(" "),
            Element::Text("[!--"),
            Element::Text(" "),
            Element::Text("comment"),
        ],
        vec![ParseError::new_raw(
            Token::LeftComment,
            "fallback",
            5..9,
            ParseErrorKind::NoRulesMatch,
        )],
    );

    test!(
        "fail --] comment",
        vec![
            Element::Text("fail"),
            Element::Text(" "),
            Element::Text("--]"),
            Element::Text(" "),
            Element::Text("comment"),
        ],
        vec![ParseError::new_raw(
            Token::RightComment,
            "fallback",
            5..8,
            ParseErrorKind::NoRulesMatch,
        )],
    );

    test!("@@@@", vec![Element::Raw(vec![])], vec![]);

    test!("@@@@@", vec![Element::Raw(vec!["@"])], vec![]);

    test!("@@@@@@", vec![Element::Raw(vec!["@@"])], vec![]);

    test!(
        "test @@@@ string",
        vec![
            Element::Text("test"),
            Element::Text(" "),
            Element::Raw(vec![]),
            Element::Text(" "),
            Element::Text("string"),
        ],
        vec![],
    );

    test!(
        "test @@@@@@ string",
        vec![
            Element::Text("test"),
            Element::Text(" "),
            Element::Raw(vec!["@@"]),
            Element::Text(" "),
            Element::Text("string"),
        ],
        vec![],
    );

    test!(
        "@@raw @< >@ content@@",
        vec![Element::Raw(vec![
            "raw", " ", "@<", " ", ">@", " ", "content",
        ])],
        vec![],
    );

    test!(
        "not @@**@@ bold",
        vec![
            Element::Text("not"),
            Element::Text(" "),
            Element::Raw(vec!["**"],),
            Element::Text(" "),
            Element::Text("bold"),
        ],
        vec![],
    );

    test!(
        "@<raw @@ content>@",
        vec![Element::Raw(vec!["raw", " ", "@@", " ", "content"])],
        vec![],
    );

    test!(
        "interrupted @@\n@@",
        vec![
            Element::Text("interrupted"),
            Element::Text(" "),
            Element::Text("@@"),
            Element::LineBreak,
            Element::Text("@@"),
        ],
        vec![
            // From interrupted raw
            ParseError::new_raw(
                Token::Raw, //
                "fallback",
                12..14,
                ParseErrorKind::NoRulesMatch,
            ),
            // Trying the ending raw as an opener
            ParseError::new_raw(
                Token::Raw, //
                "fallback",
                15..17,
                ParseErrorKind::NoRulesMatch,
            ),
        ],
    );

    test!(
        "interrupted @<\n>@",
        vec![
            Element::Text("interrupted"),
            Element::Text(" "),
            Element::Text("@<"),
            Element::LineBreak,
            Element::Text(">@"),
        ],
        vec![
            // From interrupted raw
            ParseError::new_raw(
                Token::LeftRaw,
                "fallback",
                12..14,
                ParseErrorKind::NoRulesMatch,
            ),
            // Trying the ending raw as an opener
            ParseError::new_raw(Token::Raw, "fallback", 15..17, ParseErrorKind::NoRulesMatch,),
        ],
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
