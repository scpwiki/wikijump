/*
 * parse/token.rs
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

use super::data::ImageAlignment;

#[derive(Logos, Debug, PartialEq, Eq)]
pub enum Token {
    //
    // Symbols
    //
    #[token("[")]
    LeftBracket,

    #[token("]")]
    RightBracket,

    #[token("|")]
    Pipe,

    #[token("[[")]
    LeftTag,

    #[token("[[*")]
    LeftTagSpecial,

    #[token("]]")]
    RightTag,

    #[token("[[#")]
    LeftAnchor,

    #[token("--")]
    DoubleDash,

    #[token("---")]
    TripleDash,

    #[token("\n", priority = 2)]
    Newline,

    #[regex(r"\s+")]
    Whitespace,

    //
    // Text components
    //
    #[regex(r"\w+", priority = 2)]
    Identifier,

    #[regex(r"[A-Za-z0-9_+\-\.]+@[A-Za-z0-9\-]+\.[A-Za-z0-9\.]+")]
    Email,

    #[regex(r"(https?|ftp)://[^ \n\|\[\]]+")]
    Url,

    #[regex(r"(f?[<>])|=", ImageAlignment::read, priority = 3)]
    ImageAlignment(ImageAlignment),

    //
    // Formatting
    //
    #[token("**", priority = 3)]
    Bold,

    #[token("//", priority = 3)]
    Italics,

    #[token("__", priority = 3)]
    Underline,

    #[token("^^", priority = 3)]
    Superscript,

    #[token(",,", priority = 3)]
    Subscript,

    #[token("{{")]
    LeftMonospace,

    #[token("}}")]
    RightMonospace,

    #[token("##")]
    Color,

    #[token("@@")]
    Raw,

    #[token("@<")]
    LeftRaw,

    #[token(">@")]
    RightRaw,

    //
    // Links
    //
    #[token("[[[")]
    LeftLink,

    #[token("]]]")]
    RightLink,

    //
    // Tables
    //
    #[token("||")]
    TableColumn,

    #[token("||~")]
    TableColumnTitle,

    //
    // Alignment
    //
    #[token("[[>]]")]
    RightAlignOpen,

    #[token("[[/>]]")]
    RightAlignClose,

    #[token("[[<]]")]
    LeftAlignOpen,

    #[token("[[/<]]")]
    LeftAlignClose,

    #[token("[[=]]")]
    CenterAlignOpen,

    #[token("[[/=]]")]
    CenterAlignClose,

    #[token("[[==]]")]
    JustifyAlignOpen,

    #[token("[[/==]]")]
    JustifyAlignClose,

    //
    // Miscellaneous / "error" case
    //
    #[error]
    Text,
}

// for testing the function during development
#[test]
fn tokens() {
    use logos::Logos;

    fn print_tokens(text: &str) {
        let mut lex = Token::lexer(text);
        let mut tokens = Vec::new();

        while let Some(token) = lex.next() {
            let range = lex.span();
            let text = lex.slice();
            tokens.push((token, range, text));
        }

        println!("Input: {:?}\nOutput: {:#?}", text, tokens);
    }

    print_tokens("**Hello** world!");
    print_tokens("[[>]]\n[[module Rate]]\n[[/>]]");
}
