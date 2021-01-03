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
use crate::parse::collect::{collect_merge, collect_merge_keep};
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
    'r: 'p + 't,
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
        F: FnOnce(BlockParser<'_, 'r, 't>) -> Result<bool, ParseError>,
    {
        let mut parser = self.parser.clone();
        let bparser = BlockParser::new(&self.log, &mut parser, self.special);

        f(bparser).unwrap_or(false)
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

        // Skip '[[' if we're on it
        if self.current().token == Token::LeftBlock {
            self.step()?;
        }

        // Collect block name and determine whether the head is done
        collect_merge_keep(
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

                // collect_merge_keep() already checked the token
                _ => unreachable!(),
            };

            (name, in_block)
        })
    }

    pub fn get_end_block(&mut self) -> Result<&'t str, ParseError> {
        debug!(self.log, "Looking for end block");

        self.get_token(Token::LeftBlockEnd, ParseErrorKind::BlockExpectedEnd)?;
        self.get_optional_space()?;

        let (name, ended) = self.get_block_name()?;
        if !ended {
            self.get_optional_space()?;
            self.get_token(Token::RightBlock, ParseErrorKind::BlockExpectedEnd)?;
        }

        Ok(name)
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

    pub fn get_argument_value(
        &mut self,
        error_kind: Option<ParseErrorKind>,
    ) -> Result<&'t str, ParseError> {
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
