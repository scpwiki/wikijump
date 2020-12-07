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

use crate::enums::{AnchorTarget, LinkLabel};
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
        // For plain enum container types
        ($type:tt, $elements:expr) => {
            container!(ContainerType::$type; $elements)
        };

        // For container types with added data
        ($type:expr; $elements:expr) => {
            Element::Container(Container::new($type, $elements))
        };

        // Comma variants
        ($type:tt, $elements:expr,) => {
            container!($type, $elements)
        };

        ($type:expr; $elements:expr,) => {
            container!($type; $elements)
        };
    }

    test!(
        "//italics// text",
        vec![
            container!(Italics, vec![text!("italics")]),
            text!(" "),
            text!("text"),
        ],
        vec![],
    );

    test!(
        "//fail italics",
        vec![text!("//"), text!("fail"), text!(" "), text!("italics")],
        vec![ParseError::new_raw(
            Token::Italics,
            "fallback",
            0..2,
            ParseErrorKind::NoRulesMatch,
        )],
    );

    test!(
        "__underline__ text",
        vec![
            container!(Underline, vec![text!("underline")]),
            text!(" "),
            text!("text"),
        ],
        vec![],
    );

    test!(
        "__fail underline",
        vec![text!("__"), text!("fail"), text!(" "), text!("underline")],
        vec![ParseError::new_raw(
            Token::Underline,
            "fallback",
            0..2,
            ParseErrorKind::NoRulesMatch,
        )],
    );

    test!(
        "^^super^^ script",
        vec![
            container!(Superscript, vec![text!("super")]),
            text!(" "),
            text!("script"),
        ],
        vec![],
    );

    test!(
        "^^fail superscript",
        vec![text!("^^"), text!("fail"), text!(" "), text!("superscript")],
        vec![ParseError::new_raw(
            Token::Superscript,
            "fallback",
            0..2,
            ParseErrorKind::NoRulesMatch,
        )],
    );

    test!(
        ",,sub,, script",
        vec![
            container!(Subscript, vec![text!("sub")]),
            text!(" "),
            text!("script"),
        ],
        vec![],
    );

    test!(
        ",,fail subscript",
        vec![text!(",,"), text!("fail"), text!(" "), text!("subscript")],
        vec![ParseError::new_raw(
            Token::Subscript,
            "fallback",
            0..2,
            ParseErrorKind::NoRulesMatch,
        )],
    );

    test!(
        "{{mono}} space",
        vec![
            container!(Monospace, vec![text!("mono")]),
            text!(" "),
            text!("space"),
        ],
        vec![],
    );

    test!(
        "{{fail monospace",
        vec![text!("{{"), text!("fail"), text!(" "), text!("monospace")],
        vec![ParseError::new_raw(
            Token::LeftMonospace,
            "fallback",
            0..2,
            ParseErrorKind::NoRulesMatch,
        )],
    );

    test!(
        "}}fail monospace",
        vec![text!("}}"), text!("fail"), text!(" "), text!("monospace")],
        vec![ParseError::new_raw(
            Token::RightMonospace,
            "fallback",
            0..2,
            ParseErrorKind::NoRulesMatch,
        )],
    );

    test!(
        "--strike-- through",
        vec![
            container!(Strikethrough, vec![text!("strike")]),
            text!(" "),
            text!("through"),
        ],
        vec![],
    );

    test!(
        "--fallback em dash",
        vec![
            text!("\u{2014}"), // em dash
            text!("fallback"),
            text!(" "),
            text!("em"),
            text!(" "),
            text!("dash"),
        ],
        vec![],
    );

    test!(
        "single [!-- stuff here --] comment",
        vec![
            text!("single"),
            text!(" "),
            Element::Null,
            text!(" "),
            text!("comment"),
        ],
        vec![],
    );

    test!(
        "multiline\n[!-- stuff \n here --]\n comment",
        vec![
            text!("multiline"),
            Element::LineBreak,
            Element::Null,
            Element::LineBreak,
            text!(" "),
            text!("comment"),
        ],
        vec![],
    );

    test!(
        "fail [!-- comment",
        vec![
            text!("fail"),
            text!(" "),
            text!("[!--"),
            text!(" "),
            text!("comment"),
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
            text!("fail"),
            text!(" "),
            text!("--]"),
            text!(" "),
            text!("comment"),
        ],
        vec![ParseError::new_raw(
            Token::RightComment,
            "fallback",
            5..8,
            ParseErrorKind::NoRulesMatch,
        )],
    );

    test!("@@@@", vec![Element::Raw(cow!(""))], vec![]);

    test!("@@@@@", vec![Element::Raw(cow!("@"))], vec![]);

    test!("@@@@@@", vec![Element::Raw(cow!("@@"))], vec![]);

    test!(
        "test @@@@ string",
        vec![
            text!("test"),
            text!(" "),
            Element::Raw(cow!("")),
            text!(" "),
            text!("string"),
        ],
        vec![],
    );

    test!(
        "test @@@@@@ string",
        vec![
            text!("test"),
            text!(" "),
            Element::Raw(cow!("@@")),
            text!(" "),
            text!("string"),
        ],
        vec![],
    );

    test!("@<>@", vec![Element::Raw(cow!(""))], vec![]);

    test!(
        "@@raw @< >@ content@@",
        vec![Element::Raw(cow!("raw @< >@ content"))],
        vec![],
    );

    test!(
        "not @@**@@ bold",
        vec![
            text!("not"),
            text!(" "),
            Element::Raw(cow!("**")),
            text!(" "),
            text!("bold"),
        ],
        vec![],
    );

    test!(
        "@<raw @@ content>@",
        vec![Element::Raw(cow!("raw @@ content"))],
        vec![],
    );

    test!(
        "interrupted @@\n@@",
        vec![
            text!("interrupted"),
            text!(" "),
            text!("@@"),
            Element::LineBreak,
            text!("@@"),
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
            text!("interrupted"),
            text!(" "),
            text!("@<"),
            Element::LineBreak,
            text!(">@"),
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
            ParseError::new_raw(
                Token::RightRaw,
                "fallback",
                15..17,
                ParseErrorKind::NoRulesMatch,
            ),
        ],
    );

    test!(
        "##blue|text here##",
        vec![Element::Color {
            color: cow!("blue"),
            elements: vec![text!("text"), text!(" "), text!("here")],
        }],
        vec![],
    );

    test!(
        "###ccc|css color!##",
        vec![Element::Color {
            color: cow!("#ccc"),
            elements: vec![text!("css"), text!(" "), text!("color"), text!("!")],
        }],
        vec![],
    );

    test!(
        "##not color",
        vec![text!("##"), text!("not"), text!(" "), text!("color")],
        vec![ParseError::new_raw(
            Token::Color,
            "fallback",
            0..2,
            ParseErrorKind::NoRulesMatch,
        )],
    );

    test!(
        "##invalid\n|text##",
        vec![
            text!("##"),
            text!("invalid"),
            Element::LineBreak,
            text!("|"),
            text!("text"),
            text!("##"),
        ],
        vec![
            ParseError::new_raw(
                Token::Color, //
                "fallback",
                0..2,
                ParseErrorKind::NoRulesMatch,
            ),
            ParseError::new_raw(
                Token::Color,
                "fallback",
                15..17,
                ParseErrorKind::NoRulesMatch,
            ),
        ],
    );

    test!(
        "[https://example.com/ Some link!]",
        vec![Element::Link {
            url: cow!("https://example.com/"),
            label: LinkLabel::Text(cow!("Some link!")),
            anchor: AnchorTarget::Same,
        }],
        vec![],
    );

    test!(
        "[*http://scp-sandbox-3.wikidot.com/system:recent-changes Sandbox: Recent Changes ]",
        vec![Element::Link {
            url: cow!("http://scp-sandbox-3.wikidot.com/system:recent-changes"),
            label: LinkLabel::Text(cow!("Sandbox: Recent Changes")),
            anchor: AnchorTarget::NewTab,
        }],
        vec![],
    );

    test!(
        "[ not a link ]",
        vec![
            text!("["),
            text!(" "),
            text!("not"),
            text!(" "),
            text!("a"),
            text!(" "),
            text!("link"),
            text!(" "),
            text!("]"),
        ],
        // No errors, because bare "[" is considered text
        vec![],
    );

    test!(
        "[* not a link ]",
        vec![
            text!("[*"),
            text!(" "),
            text!("not"),
            text!(" "),
            text!("a"),
            text!(" "),
            text!("link"),
            text!(" "),
            text!("]"),
        ],
        vec![ParseError::new_raw(
            Token::LeftBracketSpecial,
            "fallback",
            0..2,
            ParseErrorKind::NoRulesMatch,
        )],
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
