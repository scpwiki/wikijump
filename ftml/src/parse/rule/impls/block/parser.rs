/*
 * parse/rule/impls/block/parser.rs
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

use super::arguments::Arguments;
use super::rule::{RULE_BLOCK, RULE_BLOCK_SPECIAL};
use super::BlockRule;
use crate::parse::collect::collect_merge;
use crate::parse::condition::ParseCondition;
use crate::parse::{
    parse_string, ExtractedToken, ParseError, ParseErrorKind, Parser, Token,
};
use crate::text::FullText;

#[derive(Debug)]
pub struct BlockParser<'p, 'r, 't> {
    log: slog::Logger,
    parser: &'p mut Parser<'r, 't>,
    special: bool,
}

impl<'p, 'r, 't> BlockParser<'p, 'r, 't>
where
    'r: 't,
{
    #[inline]
    pub fn new(
        log: &slog::Logger,
        parser: &'p mut Parser<'r, 't>,
        special: bool,
    ) -> Self {
        info!(
            log, "Creating block parser";
            "special" => special,
            "remaining-len" => parser.remaining().len(),
        );

        let log = slog::Logger::clone(log);
        let rule = if special {
            RULE_BLOCK_SPECIAL
        } else {
            RULE_BLOCK
        };

        parser.set_rule(rule);

        BlockParser {
            log,
            parser,
            special,
        }
    }

    // Parsing methods
    fn get_token(
        &mut self,
        token: Token,
        kind: ParseErrorKind,
    ) -> Result<&'t str, ParseError> {
        debug!(
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

    #[inline]
    pub fn get_identifier(
        &mut self,
        kind: ParseErrorKind,
    ) -> Result<&'t str, ParseError> {
        debug!(self.log, "Looking for identifier");

        self.get_token(Token::Identifier, kind)
    }

    pub fn get_line_break(&mut self) -> Result<(), ParseError> {
        debug!(self.log, "Looking for line break");

        self.get_token(Token::LineBreak, ParseErrorKind::BlockExpectedLineBreak)?;
        Ok(())
    }

    pub fn get_optional_space(&mut self) -> Result<(), ParseError> {
        debug!(self.log, "Looking for optional space");

        if self.current().token == Token::Whitespace {
            self.step()?;
        }

        Ok(())
    }

    pub fn get_end_block(&mut self) -> Result<&'t str, ParseError> {
        debug!(self.log, "Looking for end block");

        self.get_token(Token::LeftBlockEnd, ParseErrorKind::BlockExpectedEnd)?;
        self.get_optional_space()?;

        let name = self.get_identifier(ParseErrorKind::BlockMissingName)?;
        self.get_optional_space()?;
        self.get_token(Token::RightBlock, ParseErrorKind::BlockExpectedEnd)?;

        Ok(name)
    }

    /// Run the closure with a temporary, "cloned" version of this structure.
    ///
    /// This permits free manipualation of the pointer state with no
    /// effects on the actual parse state.
    #[inline]
    pub fn clone_run<F, T>(&self, f: F) -> T
    where
        F: FnOnce(BlockParser<'_, 'r, 't>) -> T,
    {
        let mut parser = self.parser.clone();
        let clone = BlockParser::new(&self.log, &mut parser, self.special);
        f(clone)
    }

    /// Keep consuming tokens until they match a certain pattern.
    ///
    /// This function iterates until a contiguous slice of tokens
    /// are found that match the kind specified in the slice.
    ///
    /// Following parse success, the token pointer remains at the
    /// location of the first token that matches.
    ///
    /// Returns immediately if the slice is empty.
    pub fn proceed_until(&mut self, tokens: &[Token]) -> Result<(), ParseError> {
        let (&first, tokens) = match tokens.split_first() {
            Some(split) => split,
            None => return Ok(()),
        };

        loop {
            // Iterate until we hit a first token match
            while self.current().token != first {
                self.step()?;
            }

            // Duplicate parser state to allow look-ahead, check if the rest matches
            let result =
                self.clone_run(|mut bparser| bparser.proceed_until_internal(tokens))?;

            // If it was a match, return
            if result {
                return Ok(());
            }

            // If it failed, step forward and try this again
            self.step()?;
        }
    }

    /// Internal helper function for `proceed_until`. Do not use directly.
    ///
    /// Sees if this particular pointer state matches the token list or not.
    /// Returns `true` if so, `false` otherwise.
    fn proceed_until_internal(&mut self, tokens: &[Token]) -> Result<bool, ParseError> {
        for token in tokens.iter().copied() {
            self.step()?;

            if self.current().token != token {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Try the code in the closure, resetting state on failure.
    ///
    /// * If `Ok(_)` is returned, pointer status is preserved, and `Some(_)` is returned.
    /// * If `Err(_)` is returned, pointer status is reverted, and `None` is returned.
    pub fn try_parse<F, T>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(BlockParser<'_, 'r, 't>) -> Result<T, ParseError>,
    {
        let log_error = |error: ParseError| {
            debug!(
                self.log,
                "Got error while attempting to parse in block: {:?}",
                error;
                "error-kind" => error.kind(),
            );
        };

        self.clone_run(f).map_err(log_error).ok()
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
                Token::RightBlock => return Ok(map),
                _ => return Err(self.make_error(ParseErrorKind::BlockMalformedArguments)),
            };

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

    pub fn get_argument_value(&mut self) -> Result<&'t str, ParseError> {
        debug!(self.log, "Looking for a value argument, then ']]'");

        collect_merge(
            &self.log,
            self.parser,
            self.parser.rule(),
            &[ParseCondition::current(Token::RightBlock)],
            &[
                ParseCondition::current(Token::ParagraphBreak),
                ParseCondition::current(Token::LineBreak),
            ],
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
