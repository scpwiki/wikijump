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
use crate::parse::rule::collect::try_merge;
use crate::parse::{parse_string, ParseError, ParseErrorKind, Parser, Token};

#[derive(Debug)]
pub struct BlockParser<'p, 'l, 'r, 't> {
    log: &'l slog::Logger,
    parser: &'p mut Parser<'l, 'r, 't>,
    special: bool,
}

impl<'p, 'l, 'r, 't> BlockParser<'p, 'l, 'r, 't>
where
    'r: 't,
{
    #[inline]
    pub fn new(
        log: &'l slog::Logger,
        special: bool,
        parser: &'p mut Parser<'l, 'r, 't>,
    ) -> Self {
        info!(
            log, "Creating block parser";
            "special" => special,
            "remaining-len" => parser.remaining().len(),
        );

        parser.set_rule(if special {
            RULE_BLOCK_SPECIAL
        } else {
            RULE_BLOCK
        });

        BlockParser {
            log,
            special,
            parser,
        }
    }

    // Getters
    #[inline]
    pub fn parser(&self) -> &'p Parser<'l, 'r, 't> {
        self.parser
    }

    #[inline]
    pub fn parser_mut(&mut self) -> &'p mut Parser<'l, 'r, 't> {
        self.parser
    }

    // Parsing methods
    fn get_token(
        &mut self,
        token: Token,
        kind: ParseErrorKind,
    ) -> Result<&'t str, ParseError> {
        debug!(
            self.log,
            "Looking for token {:?} (error {:?})",
            token,
            kind;
            "token" => token,
            "error-kind" => kind,
        );

        if self.extracted.token == token {
            let text = self.extracted.slice;
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

        if self.extracted.token == Token::Whitespace {
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
            while self.extracted.token != first {
                self.step()?;
            }

            // Save current position, check if the rest match
            let (extracted, remaining) = (self.extracted, self.remaining);
            let result = self.proceed_until_internal(tokens)?;

            // We always restore pointer position.
            //
            // This reverts any forward changes during crawling,
            // and also resets the pointer if this is a match.
            self.extracted = extracted;
            self.remaining = remaining;

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

            if self.extracted.token != token {
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
        F: FnOnce(&mut Self) -> Result<T, ParseError>,
    {
        let (extracted, remaining) = (self.extracted, self.remaining);

        match f(self) {
            Ok(result) => Some(result),
            Err(error) => {
                debug!(
                    self.log,
                    "Got error while attempting to parse in block: {:?}",
                    error;
                    "error-kind" => error.kind(),
                );

                self.extracted = extracted;
                self.remaining = remaining;

                None
            }
        }
    }

    // Block argument parsing
    pub fn get_argument_map(&mut self) -> Result<Arguments<'t>, ParseError> {
        debug!(self.log, "Looking for key value arguments, then ']]'");

        let mut map = Arguments::new();
        loop {
            self.get_optional_space()?;

            // Try to get the argument key
            // Determines if we stop or keep parsing
            let key = match self.extracted.token {
                Token::Identifier => self.extracted.slice,
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

        let (value, remaining, _) = try_merge(
            self.log,
            (self.extracted, self.remaining, self.full_text),
            self.rule,
            &[Token::RightBlock],
            &[Token::ParagraphBreak, Token::LineBreak],
            &[],
        )?
        .into();

        self.update(remaining)?;

        Ok(value)
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

        self.rule = block_rule.rule();
    }

    #[cold]
    #[inline]
    pub fn make_error(&self, kind: ParseErrorKind) -> ParseError {
        ParseError::new(kind, self.rule, self.extracted)
    }
}
