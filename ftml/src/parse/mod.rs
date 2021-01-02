/*
 * parse/mod.rs
 *
 * ftml - Library to parse Wikidot text
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

#[macro_use]
mod macros;

mod collect;
mod condition;
mod consume;
mod error;
mod outcome;
mod paragraph;
mod parser;
mod result;
mod rule;
mod stack;
mod string;
mod token;

mod prelude {
    pub use super::{
        ExtractedToken, ParseError, ParseErrorKind, ParseException, ParseResult,
        ParseSuccess, Token,
    };
    pub use crate::text::FullText;
    pub use crate::tree::Element;
}

use self::paragraph::gather_paragraphs;
use self::parser::Parser;
use self::rule::impls::RULE_PAGE;
use crate::tokenize::Tokenization;
use crate::tree::SyntaxTree;
use std::borrow::Cow;

pub use self::error::{ParseError, ParseErrorKind, ParseException};
pub use self::outcome::ParseOutcome;
pub use self::result::{ParseResult, ParseSuccess};
pub use self::string::parse_string;
pub use self::token::{ExtractedToken, Token};

/// Parse through the given tokens and produce an AST.
///
/// This takes a list of `ExtractedToken` items produced by `tokenize()`.
pub fn parse<'r, 't>(
    log: &slog::Logger,
    tokenization: &'r Tokenization<'t>,
) -> ParseOutcome<SyntaxTree<'t>>
where
    'r: 't,
{
    let mut parser = Parser::new(log, tokenization);

    // Logging setup
    let log = &log.new(slog_o!(
        "filename" => slog_filename!(),
        "lineno" => slog_lineno!(),
        "function" => "parse",
        "tokens-len" => tokenization.tokens().len(),
    ));

    // At the top level, we gather elements into paragraphs
    info!(log, "Running parser on tokens");
    let result = gather_paragraphs(log, &mut parser, RULE_PAGE, &[], &[]);

    debug!(log, "Finished paragraph gathering, matching on consumption");
    match result {
        Ok(ParseSuccess {
            item: elements,
            exceptions,
            ..
        }) => {
            let (errors, styles) = extract_exceptions(exceptions);

            info!(
                log,
                "Finished parsing, producing final syntax tree";
                "errors-len" => errors.len(),
                "styles-len" => styles.len(),
            );

            SyntaxTree::from_element_result(elements, errors, styles)
        }
        Err(error) => {
            // This path is only reachable if invalid_tokens is non-empty.
            // As this is the highest-level, we do not have any premature ending tokens,
            // but rather keep going until the end of the input.
            //
            // Thus this path should not be reached.

            panic!(
                "Got parse error from highest-level paragraph gather: {:#?}",
                error,
            );
        }
    }
}

fn extract_exceptions(
    exceptions: Vec<ParseException>,
) -> (Vec<ParseError>, Vec<Cow<str>>) {
    let mut errors = Vec::new();
    let mut styles = Vec::new();

    for exception in exceptions {
        match exception {
            ParseException::Error(error) => errors.push(error),
            ParseException::Style(style) => styles.push(style),
        }
    }

    (errors, styles)
}
