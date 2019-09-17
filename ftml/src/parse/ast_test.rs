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

use super::{parse, Paragraph, SyntaxTree, Word};

macro_rules! valid {
    ($input:expr, $expected:expr) => {
        match parse($input) {
            Ok(ast) => assert_eq!(ast, $expected, "Outputed AST doesn't match expected"),
            Err(err) => panic!("Received error when parsing test input: {}", err),
        }
    };
}

macro_rules! invalid {
    ($input:expr) => {
        match parse($input) {
            Ok(ast) => panic!("Invalid test input parsed, produced AST: {:#?}", ast),
            Err(_) => (),
        }
    };
}

#[test]
fn test_valid() {
    valid!("", SyntaxTree::from_paragraphs(vec![]));
    valid!(
        "**bold**",
        SyntaxTree::from_paragraphs(vec![Paragraph::Words {
            centered: false,
            words: vec![Word::Bold {
                words: vec![Word::Text { contents: "bold" }]
            }]
        }])
    );
    valid!(
        "//italics//",
        SyntaxTree::from_paragraphs(vec![Paragraph::Words {
            centered: false,
            words: vec![Word::Italics {
                words: vec![Word::Text {
                    contents: "italics"
                }]
            }]
        }])
    );
    valid!(
        "__underline__",
        SyntaxTree::from_paragraphs(vec![Paragraph::Words {
            centered: false,
            words: vec![Word::Underline {
                words: vec![Word::Text {
                    contents: "underline"
                }]
            }]
        }])
    );
    valid!(
        "--strikethrough--",
        SyntaxTree::from_paragraphs(vec![Paragraph::Words {
            centered: false,
            words: vec![Word::Strikethrough {
                words: vec![Word::Text {
                    contents: "strikethrough"
                }]
            }]
        }])
    );
    valid!(
        "##rust|colored text here!##",
        SyntaxTree::from_paragraphs(vec![Paragraph::Words {
            centered: false,
            words: vec![Word::Color {
                color: "rust",
                words: vec![Word::Text {
                    contents: "colored text here!"
                }],
            }]
        }])
    );
    valid!(
        "^^superscript^^",
        SyntaxTree::from_paragraphs(vec![Paragraph::Words {
            centered: false,
            words: vec![Word::Superscript {
                words: vec![Word::Text {
                    contents: "superscript"
                }]
            }]
        }])
    );
    valid!(
        ",,subscript,,",
        SyntaxTree::from_paragraphs(vec![Paragraph::Words {
            centered: false,
            words: vec![Word::Subscript {
                words: vec![Word::Text {
                    contents: "subscript"
                }]
            }]
        }])
    );
    valid!(
        "{{monospace}}",
        SyntaxTree::from_paragraphs(vec![Paragraph::Words {
            centered: false,
            words: vec![Word::Monospace {
                words: vec![Word::Text {
                    contents: "monospace"
                }]
            }]
        }])
    );

    valid!(
        "@@ apple @@ @@banana@@",
        SyntaxTree::from_paragraphs(vec![Paragraph::Words {
            centered: false,
            words: vec![
                Word::Raw {
                    contents: " apple "
                },
                Word::Text { contents: " " },
                Word::Raw { contents: "banana" }
            ],
        }])
    );
}

#[test]
fn test_invalid() {
    invalid!("[[");
    invalid!("]]");
    invalid!("@@");
    invalid!("**");
    invalid!("__");
    invalid!("//");
    invalid!("{{");
    invalid!("}}");
    invalid!(",,");
    invalid!("^^");
    invalid!("##");
    invalid!("[[[");
    invalid!("]]]");

    invalid!("[!-- alpha --] [[ eref ");

    invalid!("@@ raw value");
    invalid!("@@ @@ @@");
    invalid!("@@ raw \n multiline @@");
    invalid!("apple @@ banana");
    invalid!("__**test** cherry {{ durian ^^up^^ __");
    invalid!("[[ unknown block ]]");
    invalid!("[[ ]]");
    invalid!("kiwi [[date 0]");
    invalid!("kiwi [[ date 0 ] ]");

    invalid!("[[span id=\"a\"]] [[span id=\"b\"]] incomplete span [[/span]]");
    invalid!("[[ * user rounder house ]]");
    invalid!("[[ user rounder house ]]");
    invalid!("[[module CustomMod bad_argument=\"value ]]");
    invalid!("[[module CustomMod bad_argument=value ]]");
    invalid!("[[module CustomMod]] [[/module]]");
    invalid!("[[image filename_with_a_space in_it.jpeg]]");
    invalid!("[[==image filename.png]]");
    invalid!("[[f=image filename.png]]");

    invalid!("// Incomplete italics");
    invalid!("** Incomplete bold");
    invalid!("__ Incomplete underline");
    invalid!("^^ Incomplete superscript");
    invalid!(",, Incomplete subscript");
    invalid!("---- Empty strikethrough"); // Conflicts with horiz separator
    invalid!("##NOT&A&COLOR|test##");

    invalid!("[[footnote]]");
    invalid!("[[>]]\nPINEAPPLE");
    invalid!("[[<]]\nPINEAPPLE");
    invalid!("[[=]]\nPINEAPPLE");
    invalid!("[[==]]\nPINEAPPLE");
    invalid!("[[>]]\nCHERRY\n[[/<]]");
    invalid!("[[<]]\nCHERRY\n[[/>]]");
    invalid!("[[=]]\nCHERRY\n[[/==]]");
    invalid!("[[==]]\nCHERRY\n[[/=]]");

    invalid!("[[date 1000 invalid_arg=\"test\"]]");
    invalid!("[[span invalid_arg=\"test\"]]apple[[/span]]");
    invalid!("[[user rounderhouse invalid_arg=\"test\"]]");
    invalid!("[[image director-sharp.jpeg invalid_arg=\"test\"]]");
    invalid!("[[code invalid_arg=\"test\"]]\napple\n[[/code]]");
    invalid!("[[div invalid_arg=\"test\"]][[/div]]");
    invalid!("an [[div]] inline div [[/div]]");

    invalid!("[[F>image filename.png]]");
    invalid!("[[F<image filename.png]]");
    invalid!("[[tablist]]");
    invalid!("[[tablist]] [[tab A]] [[/tablist]]");
    invalid!("[[tabview]] [[/tab]] [[/tabview]]");
    invalid!("[[gallery]] contents [[/gallery]]");

    invalid!("+++++++ h7 heading doesn't exist");
    invalid!("[[quote]] inline [[/quote]]");
    // Split and disjoint groups are explicitly unsupported
    invalid!("[[div]]\n[[quote]]\ncontents\n[[/div]]\n[[/quote]]");
    invalid!("[[div]]\n> contents\n> [[/div]]");
    invalid!("[[js]]");
    invalid!("[[javascript]]");

    invalid!("[[# false anchor]");
    invalid!("[# false empty link]]");
    invalid!("[[a herf=\"https://example.com/\"]]link[[/a]]");
    invalid!("[[# anchor-name-with-|-bad-ident]]");
    invalid!("[[=]]\n[[collapsible]]\n[[/=]]\n[[/collapsible]]\n");
    invalid!("hello [[html]] world\n");
    invalid!("hello [[iframe https://example.com]] world\n");
    invalid!("[[html]]");
    invalid!("[[iframe]]");
    invalid!("[[iframe https://example.com]]\ncontents\n[[/iframe]]\n");
    invalid!("[[css]]");
    invalid!("[[style]]");
}
