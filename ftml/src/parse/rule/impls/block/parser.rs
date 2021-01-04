/*
 * parse/rule/impls/block/parser.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2021 Ammon Smith
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

// TODO remove when more block rules added
#![allow(dead_code)]

use super::arguments::Arguments;
use super::BlockRule;
use crate::parse::collect::{collect_text, collect_text_keep};
use crate::parse::condition::ParseCondition;
use crate::parse::{
    parse_string, ExtractedToken, ParseError, ParseErrorKind, Parser, Token,
};
use crate::text::FullText;
use crate::tree::Element;

#[derive(Debug)]
pub struct BlockParser<'p, 'r, 't> {
    log: slog::Logger,
    parser: &'p mut Parser<'r, 't>,
}

impl<'p, 'r, 't> BlockParser<'p, 'r, 't>
where
    'r: 'p + 't,
{
    #[inline]
    pub fn new(log: &slog::Logger, parser: &'p mut Parser<'r, 't>) -> Self {
        info!(
            log, "Creating block parser";
            "remaining-len" => parser.remaining().len(),
        );

        let log = slog::Logger::clone(log);

        BlockParser { log, parser }
    }

    // Getters
    #[inline]
    pub fn get(&self) -> &Parser<'r, 't> {
        &self.parser
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut Parser<'r, 't> {
        &mut self.parser
    }

    // State evaluation
    #[inline]
    pub fn evaluate_fn<F>(&self, f: F) -> bool
    where
        F: FnOnce(&mut BlockParser<'_, 'r, 't>) -> Result<bool, ParseError>,
    {
        debug!(&self.log, "Evaluating closure for parser condition");

        let mut parser = self.parser.clone();
        let mut bparser = BlockParser::new(&self.log, &mut parser);
        f(&mut bparser).unwrap_or(false)
    }

    pub fn save_evaluate_fn<F>(&mut self, f: F) -> Option<&'r ExtractedToken<'t>>
    where
        F: FnOnce(&mut BlockParser<'_, 'r, 't>) -> Result<bool, ParseError>,
    {
        debug!(
            &self.log,
            "Evaluating closure for parser condition, saving progress on success",
        );

        let mut parser = self.parser.clone();
        let mut bparser = BlockParser::new(&self.log, &mut parser);
        if f(&mut bparser).unwrap_or(false) {
            let last = self.parser.current();
            self.parser.update(&parser);
            Some(last)
        } else {
            None
        }
    }

    // Parsing methods
    fn get_token(
        &mut self,
        token: Token,
        kind: ParseErrorKind,
    ) -> Result<&'t str, ParseError> {
        trace!(
            &self.log,
            "Looking for token {:?} (error {:?})",
            token,
            kind;
            "token" => token,
            "error-kind" => kind,
        );

        let current = self.current();
        if current.token == token {
            let text = current.slice;
            self.step()?;
            Ok(text)
        } else {
            Err(self.make_error(kind))
        }
    }

    fn get_optional_token(&mut self, token: Token) -> Result<(), ParseError> {
        trace!(
            &self.log,
            "Looking for optional token {:?}",
            token;
            "token" => token,
        );

        if self.current().token == token {
            self.step()?;
        }

        Ok(())
    }

    pub fn get_line_break(&mut self) -> Result<(), ParseError> {
        debug!(self.log, "Looking for line break");

        self.get_token(Token::LineBreak, ParseErrorKind::BlockExpectedLineBreak)?;
        Ok(())
    }

    #[inline]
    pub fn get_optional_space(&mut self) -> Result<(), ParseError> {
        debug!(self.log, "Looking for optional space");
        self.get_optional_token(Token::Whitespace)
    }

    pub fn get_block_name(&mut self) -> Result<(&'t str, bool), ParseError> {
        debug!(self.log, "Looking for identifier");

        self.get_optional_token(Token::LeftBlock)?;
        self.get_optional_space()?;

        // Collect block name and determine whether the head is done
        collect_text_keep(
            &self.log,
            self.parser,
            self.parser.rule(),
            &[
                ParseCondition::current(Token::Whitespace),
                ParseCondition::current(Token::RightBlock),
            ],
            &[
                ParseCondition::current(Token::ParagraphBreak),
                ParseCondition::current(Token::LineBreak),
            ],
            Some(ParseErrorKind::BlockMissingName),
        )
        .map(|(name, last)| {
            let name = name.trim();
            let in_block = match last.token {
                Token::Whitespace => true,
                Token::RightBlock => false,

                // collect_text_keep() already checked the token
                _ => unreachable!(),
            };

            (name, in_block)
        })
    }

    pub fn get_end_block(&mut self) -> Result<&'t str, ParseError> {
        debug!(self.log, "Looking for end block");

        self.get_token(Token::LeftBlockEnd, ParseErrorKind::BlockExpectedEnd)?;
        self.get_optional_space()?;

        let (name, in_block) = self.get_block_name()?;
        if in_block {
            self.get_optional_space()?;
            self.get_token(Token::RightBlock, ParseErrorKind::BlockExpectedEnd)?;
        }

        Ok(name)
    }

    // Body parsing

    /// Generic helper function that performs the primary block collection.
    ///
    /// Extended by the other, more specific functions.
    fn get_body_generic<T, F1, F2>(
        &mut self,
        valid_end_block_names: &[&str],
        newline_separator: bool,
        mut process: F1,
        finish: F2,
    ) -> Result<T, ParseError>
    where
        F1: FnMut() -> Result<(), ParseError>,
        F2: FnOnce(&'r ExtractedToken<'t>) -> Result<T, ParseError>,
    {
        trace!(&self.log, "Running generic in block body parser");

        debug_assert_eq!(
            valid_end_block_names.is_empty(),
            false,
            "List of valid end block names is empty, no success is possible",
        );

        // If this flag is set, then the block must be on its own line
        if newline_separator {
            self.get_line_break()?;
        }

        // Keep iterating until we find the end.
        // Preserve parse progress if we've hit the end block.
        let mut first = true;

        loop {
            let at_end_block = self.save_evaluate_fn(|parser| {
                // Check that the end block is on a new line, if required
                if newline_separator {
                    // Only check after the first, to permit empty blocks
                    if !first {
                        parser.get_line_break()?;
                    }
                }

                // Check if it's an end block
                //
                // This will ignore any errors produced,
                // since it's just more text
                let name = parser.get_end_block()?;

                // Check if it's valid
                for end_block_name in valid_end_block_names {
                    if name.eq_ignore_ascii_case(end_block_name) {
                        return Ok(true);
                    }
                }

                Ok(false)
            });

            // If there's a match, return the last body token
            if let Some(last_token) = at_end_block {
                return finish(last_token);
            }

            // Run the passed-in closure
            process()?;

            // Step and continue
            self.step()?;
            first = false;
        }
    }

    /// Collect a block's body to its end, as string slice.
    ///
    /// This requires that the has already been parsed using
    /// one of the "get argument" methods.
    ///
    /// The `newline_separator` argument designates whether this
    /// block assumes multiline construction (e.g. `[[div]]`, `[[code]]`)
    /// or not (e.g. `[[span]]`).
    pub fn get_body_text(
        &mut self,
        valid_end_block_names: &[&str],
        newline_separator: bool,
    ) -> Result<&'t str, ParseError> {
        debug!(
            &self.log,
            "Getting block body as text";
            "valid-end-block-names" => format!("{:?}", valid_end_block_names),
            "newline-separator" => newline_separator,
        );

        // State variables for collecting span
        let start = self.current();
        let end = self.get_body_generic(
            valid_end_block_names,
            newline_separator,
            || Ok(()),
            |last| Ok(last),
        )?;

        Ok(self.full_text().slice_partial(&self.log, start, end))
    }

    pub fn get_body_elements(
        &mut self,
        valid_end_block_names: &[&str],
        newline_separator: bool,
        as_paragraphs: bool,
    ) -> Result<Vec<Element<'t>>, ParseError> {
        debug!(
            &self.log,
            "Getting block body as elements";
            "valid-end-block_names" => format!("{:?}", valid_end_block_names),
            "newline-separator" => newline_separator,
            "as-paragraphs" => as_paragraphs,
        );

        let x = self.get_body_generic(
            valid_end_block_names,
            newline_separator,
            || todo!(),
            |last| todo!(),
        )?;

        todo!()
    }

    // Block argument parsing
    pub fn get_argument_map(&mut self) -> Result<Arguments<'t>, ParseError> {
        debug!(self.log, "Looking for key value arguments, then ']]'");

        let mut map = Arguments::new();
        loop {
            self.get_optional_space()?;

            // Try to get the argument key
            // Determines if we stop or keep parsing
            let current = self.current();
            let key = match current.token {
                Token::Identifier => current.slice,
                Token::RightBlock => {
                    self.step()?;
                    return Ok(map);
                }
                _ => return Err(self.make_error(ParseErrorKind::BlockMalformedArguments)),
            };
            self.step()?;

            // Equal sign
            self.get_optional_space()?;
            self.get_token(Token::Equals, ParseErrorKind::BlockMalformedArguments)?;

            // Get the argument value
            self.get_optional_space()?;
            let value_raw =
                self.get_token(Token::String, ParseErrorKind::BlockMalformedArguments)?;

            // Parse the string
            let value = parse_string(value_raw);

            // Add to argument map
            map.insert(key, value);
        }
    }

    pub fn get_argument_value(
        &mut self,
        error_kind: Option<ParseErrorKind>,
    ) -> Result<&'t str, ParseError> {
        debug!(self.log, "Looking for a value argument, then ']]'");

        collect_text(
            &self.log,
            self.parser,
            self.parser.rule(),
            &[ParseCondition::current(Token::RightBlock)],
            &[
                ParseCondition::current(Token::ParagraphBreak),
                ParseCondition::current(Token::LineBreak),
            ],
            error_kind,
        )
    }

    pub fn get_argument_none(&mut self) -> Result<(), ParseError> {
        debug!(self.log, "No arguments, looking for ']]'");

        self.get_optional_space()?;
        self.get_token(Token::RightBlock, ParseErrorKind::BlockMissingCloseBrackets)?;
        Ok(())
    }

    // Utilities
    #[inline]
    pub fn set_block(&mut self, block_rule: &BlockRule) {
        info!(
            self.log,
            "Running block rule {} for these tokens",
            block_rule.name;
        );

        self.parser.set_rule(block_rule.rule());
    }

    // Mirrored methods from underlying Parser
    #[inline]
    pub fn current(&self) -> &'r ExtractedToken<'t> {
        self.parser.current()
    }

    #[inline]
    pub fn remaining(&self) -> &'r [ExtractedToken<'t>] {
        self.parser.remaining()
    }

    #[inline]
    pub fn full_text(&self) -> FullText<'t> {
        self.parser.full_text()
    }

    #[inline]
    pub fn step(&mut self) -> Result<&'r ExtractedToken<'t>, ParseError> {
        self.parser.step()
    }

    #[inline]
    pub fn make_error(&self, kind: ParseErrorKind) -> ParseError {
        self.parser.make_error(kind)
    }
}
