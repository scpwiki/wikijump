/*
 * parsing/mod.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Wikijump Team
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

mod boolean;
mod check_step;
mod collect;
mod condition;
mod consume;
mod depth;
mod exception;
mod outcome;
mod paragraph;
mod parser;
mod result;
mod rule;
mod string;
mod strip;
mod token;

mod prelude {
    pub use crate::log::prelude::*;
    pub use crate::parsing::{
        ExtractedToken, ParseException, ParseResult, ParseSuccess, ParseWarning,
        ParseWarningKind, Token,
    };
    pub use crate::text::FullText;
    pub use crate::tree::{Element, Elements, ElementsIterator};
}

use self::depth::{process_depths, DepthItem, DepthList};
use self::paragraph::{gather_paragraphs, NO_CLOSE_CONDITION};
use self::parser::Parser;
use self::rule::impls::RULE_PAGE;
use self::string::parse_string;
use self::strip::strip_newlines;
use crate::log::prelude::*;
use crate::tokenizer::Tokenization;
use crate::tree::SyntaxTree;
use std::borrow::Cow;

pub use self::boolean::{parse_boolean, NonBooleanValue};
pub use self::exception::{ParseException, ParseWarning, ParseWarningKind};
pub use self::outcome::ParseOutcome;
pub use self::result::{ParseResult, ParseSuccess};
pub use self::token::{ExtractedToken, Token};

/// Parse through the given tokens and produce an AST.
///
/// This takes a list of `ExtractedToken` items produced by `tokenize()`.
pub fn parse<'r, 't>(
    log: &Logger,
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
    let result = gather_paragraphs(log, &mut parser, RULE_PAGE, NO_CLOSE_CONDITION);

    debug!(log, "Finished paragraph gathering, matching on consumption");
    match result {
        Ok(ParseSuccess {
            item: elements,
            exceptions,
            ..
        }) => {
            let (warnings, styles) = extract_exceptions(exceptions);

            info!(
                log,
                "Finished parsing, producing final syntax tree";
                "warnings-len" => warnings.len(),
                "styles-len" => styles.len(),
            );

            SyntaxTree::from_element_result(elements, warnings, styles)
        }
        Err(warning) => {
            // This path is only reachable if a very bad error occurs.
            //
            // If this happens, then just return the input source as the output
            // and the warning.

            warn!(
                log,
                "Fatal error occurred at highest-level parsing: {:#?}", warning,
            );

            let elements = vec![text!(tokenization.full_text().inner())];
            let warnings = vec![warning];
            let styles = vec![];

            SyntaxTree::from_element_result(elements, warnings, styles)
        }
    }
}

fn extract_exceptions(
    exceptions: Vec<ParseException>,
) -> (Vec<ParseWarning>, Vec<Cow<str>>) {
    let mut warnings = Vec::new();
    let mut styles = Vec::new();

    for exception in exceptions {
        match exception {
            ParseException::Warning(warning) => warnings.push(warning),
            ParseException::Style(style) => styles.push(style),
        }
    }

    (warnings, styles)
}
