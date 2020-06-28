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

use std::ops::Range;
use strum_macros::IntoStaticStr;

#[derive(Debug, Clone, PartialEq)]
pub struct ExtractedToken<'a> {
    pub token: Token,
    pub slice: &'a str,
    pub span: Range<usize>,
}

#[derive(Logos, IntoStaticStr, Debug, Copy, Clone, PartialEq, Eq)]
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

    #[token("=")]
    Equals,

    #[token("--")]
    DoubleDash,

    #[token("---")]
    TripleDash,

    #[token("\n", priority = 2)]
    Newline,

    #[regex(r"\s+", priority = 1)]
    Whitespace,

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
    // Text components
    //
    #[regex("[A-Za-z0-9]+")]
    Identifier,

    #[regex(r"[A-Za-z0-9+\-\.]+@[A-Za-z0-9\-]+\.[A-Za-z0-9\.]+", priority = 1)]
    Email,

    #[regex(r"(https?|ftp)://[^ \n\|\[\]]+")]
    Url,

    //
    // Miscellaneous / "error" case
    //
    #[error]
    Text,
}

impl Token {
    pub fn extract_all<'a>(logger: &slog::Logger, text: &'a str) -> Vec<ExtractedToken<'a>> {
        use logos::Logos;

        debug!(logger, "Running lexer on input");

        let mut lex = Token::lexer(text);
        let mut tokens = Vec::new();

        while let Some(token) = lex.next() {
            let slice = lex.slice();
            let span = lex.span();

            trace!(
                logger,
                "Extracted token from lexer";
                "token" => token,
                "slice" => slice,
                "span-start" => span.start,
                "span-end" => span.end,
            );

            tokens.push(ExtractedToken { token, slice, span });
        }

        tokens
    }

    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }
}

impl slog::Value for Token {
    fn serialize(
        &self,
        _: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        serializer.emit_str(key, self.name())
    }
}
