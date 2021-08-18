/*
 * parsing/parser.rs
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

use super::condition::ParseCondition;
use super::prelude::*;
use super::rule::Rule;
use super::RULE_PAGE;
use crate::data::PageInfo;
use crate::log::prelude::*;
use crate::render::text::TextRender;
use crate::tokenizer::Tokenization;
use crate::tree::HeadingLevel;
use std::cell::RefCell;
use std::rc::Rc;
use std::{mem, ptr};

const MAX_RECURSION_DEPTH: usize = 100;

#[derive(Debug, Clone)]
pub struct Parser<'r, 't> {
    // Logger instance
    log: Logger,

    // Parse state
    current: &'r ExtractedToken<'t>,
    remaining: &'r [ExtractedToken<'t>],
    full_text: FullText<'t>,

    // Rule state
    rule: Rule,
    depth: usize,

    // Table of Contents
    //
    // Schema: Vec<(depth, _, name)>
    //
    // Note: These two are in Rc<_> items so that the Parser
    //       can be cloned. It is intended as a cheap pointer
    //       object, with the true contents here preserved
    //       across parser child instances.
    table_of_contents: Rc<RefCell<Vec<(usize, (), String)>>>,

    // Footnotes
    //
    // Schema: Vec<List of elements for a footnote>
    footnotes: Rc<RefCell<Vec<Vec<Element<'t>>>>>,

    // Flags
    in_list: bool,
    start_of_line: bool,
}

impl<'r, 't> Parser<'r, 't> {
    /// Constructor. Should only be created by `parse()`.
    ///
    /// All other instances should be `.clone()` or `.clone_with_rule()`d from
    /// the main instance used during parsing.
    pub(crate) fn new(log: &Logger, tokenization: &'r Tokenization<'t>) -> Self {
        let log = Logger::clone(log);
        let full_text = tokenization.full_text();
        let (current, remaining) = tokenization
            .tokens()
            .split_first()
            .expect("Parsed tokens list was empty (expected at least one element)");

        let table_of_contents = Rc::new(RefCell::new(Vec::new()));
        let footnotes = Rc::new(RefCell::new(Vec::new()));

        Parser {
            log,
            current,
            remaining,
            full_text,
            rule: RULE_PAGE,
            depth: 0,
            table_of_contents,
            footnotes,
            in_list: false,
            start_of_line: true,
        }
    }

    // Getters
    #[inline]
    pub fn log(&self) -> Logger {
        Logger::clone(&self.log)
    }

    #[inline]
    pub fn full_text(&self) -> FullText<'t> {
        self.full_text
    }

    #[inline]
    pub fn rule(&self) -> Rule {
        self.rule
    }

    #[inline]
    pub fn in_list(&self) -> bool {
        self.in_list
    }

    #[inline]
    pub fn start_of_line(&self) -> bool {
        self.start_of_line
    }

    // Setters
    #[inline]
    pub fn set_rule(&mut self, rule: Rule) {
        self.rule = rule;
    }

    pub fn clone_with_rule(&self, rule: Rule) -> Self {
        let mut clone = self.clone();
        clone.set_rule(rule);
        clone
    }

    pub fn depth_increment(&mut self) -> Result<(), ParseWarning> {
        debug!(self.log, "Incrementing recursion depth"; "depth" => self.depth);

        self.depth += 1;

        if self.depth > MAX_RECURSION_DEPTH {
            return Err(self.make_warn(ParseWarningKind::RecursionDepthExceeded));
        }

        Ok(())
    }

    #[inline]
    pub fn depth_decrement(&mut self) {
        debug!(self.log, "Decrementing recursion depth"; "depth" => self.depth);

        self.depth -= 1;
    }

    #[inline]
    pub fn set_list_flag(&mut self, value: bool) {
        self.in_list = value;
    }

    // Table of Contents
    pub fn push_table_of_contents_entry(
        &mut self,
        heading: HeadingLevel,
        name_elements: &[Element],
    ) {
        // TODO provide real PageInfo
        let info = PageInfo {
            page: cow!("table-of-contents"),
            category: None,
            site: cow!(""),
            title: cow!("Table of Contents"),
            alt_title: None,
            rating: 0.0,
            tags: vec![],
            language: cow!("unknown"),
        };

        // Headings are 1-indexed (e.g. H1), but depth lists are 0-indexed
        let level = usize::from(heading.value()) - 1;

        // Render name as text, so it lacks formatting
        let name = TextRender.render_partial(&self.log, &info, name_elements);

        self.table_of_contents.borrow_mut().push((level, (), name));
    }

    #[cold]
    pub fn remove_table_of_contents(&mut self) -> Vec<(usize, (), String)> {
        mem::take(&mut self.table_of_contents.borrow_mut())
    }

    // Footnotes
    pub fn push_footnote(&mut self, contents: Vec<Element<'t>>) {
        // TODO
        self.footnotes.borrow_mut().push(contents);
    }

    #[cold]
    pub fn remove_footnotes(&mut self) -> Vec<Vec<Element<'t>>> {
        mem::take(&mut self.footnotes.borrow_mut())
    }

    // State evaluation
    pub fn evaluate(&self, condition: ParseCondition) -> bool {
        debug!(
            &self.log,
            "Evaluating parser condition";
            "current-token" => self.current.token,
            "current-slice" => self.current.slice,
            "current-span" => SpanWrap::from(&self.current.span),
        );

        match condition {
            ParseCondition::CurrentToken(token) => self.current.token == token,
            ParseCondition::TokenPair(current, next) => {
                if self.current().token != current {
                    trace!(
                        &self.log,
                        "Current token in pair doesn't match, failing";
                        "expected" => current,
                        "actual" => self.current().token,
                    );

                    return false;
                }

                match self.look_ahead(0) {
                    Some(actual) => {
                        if actual.token != next {
                            trace!(
                                &self.log,
                                "Second token in pair doesn't match, failing";
                                "expected" => next,
                                "actual" => actual.token,
                            );

                            return false;
                        }
                    }
                    None => {
                        trace!(
                            &self.log,
                            "Second token in pair doesn't exist, failing";
                            "expected" => next,
                        );

                        return false;
                    }
                }

                true
            }
        }
    }

    #[inline]
    pub fn evaluate_any(&self, conditions: &[ParseCondition]) -> bool {
        trace!(
            &self.log,
            "Evaluating to see if any parser condition is true";
            "conditions-len" => conditions.len(),
        );

        conditions.iter().any(|&condition| self.evaluate(condition))
    }

    #[inline]
    pub fn evaluate_fn<F>(&self, f: F) -> bool
    where
        F: FnOnce(&mut Parser<'r, 't>) -> Result<bool, ParseWarning>,
    {
        debug!(&self.log, "Evaluating closure for parser condition");

        f(&mut self.clone()).unwrap_or(false)
    }

    pub fn save_evaluate_fn<F>(&mut self, f: F) -> Option<&'r ExtractedToken<'t>>
    where
        F: FnOnce(&mut Parser<'r, 't>) -> Result<bool, ParseWarning>,
    {
        debug!(
            &self.log,
            "Evaluating closure for parser condition, saving progress on success",
        );

        let mut parser = self.clone();
        if f(&mut parser).unwrap_or(false) {
            let last = self.current;
            self.update(&parser);
            Some(last)
        } else {
            None
        }
    }

    // Token pointer state and manipulation
    #[inline]
    pub fn current(&self) -> &'r ExtractedToken<'t> {
        self.current
    }

    #[inline]
    pub fn remaining(&self) -> &'r [ExtractedToken<'t>] {
        self.remaining
    }

    #[inline]
    pub fn update(&mut self, parser: &Parser<'r, 't>) {
        self.current = parser.current;
        self.remaining = parser.remaining;
    }

    #[inline]
    pub fn same_pointer(&self, old_remaining: &'r [ExtractedToken<'t>]) -> bool {
        ptr::eq(self.remaining, old_remaining)
    }

    /// Move the token pointer forward one step.
    #[inline]
    pub fn step(&mut self) -> Result<&'r ExtractedToken<'t>, ParseWarning> {
        debug!(self.log, "Stepping to the next token");

        // Set the start-of-line flag.
        //
        // I don't include Token::InputStart since we address that by
        // simply having the start_of_line flag set on parser creation.
        self.start_of_line = matches!(self.current.token, Token::LineBreak | Token::ParagraphBreak);

        // Step to the next token.
        match self.remaining.split_first() {
            Some((current, remaining)) => {
                self.current = current;
                self.remaining = remaining;
                Ok(current)
            }
            None => Err(self.make_warn(ParseWarningKind::EndOfInput)),
        }
    }

    /// Move the token pointer forward `count` steps.
    #[inline]
    pub fn step_n(&mut self, count: usize) -> Result<(), ParseWarning> {
        trace!(self.log, "Stepping n times"; "count" => count);

        for _ in 0..count {
            self.step()?;
        }

        Ok(())
    }

    /// Look for the token `offset + 1` beyond the current one.
    ///
    /// For instance, submitting `0` will yield the first item of `parser.remaining()`.
    #[inline]
    pub fn look_ahead(&self, offset: usize) -> Option<&'r ExtractedToken<'t>> {
        debug!(self.log, "Looking ahead to a token"; "offset" => offset);

        self.remaining.get(offset)
    }

    /// Like `look_ahead`, except returns a warning if the token isn't found.
    #[inline]
    pub fn look_ahead_warn(
        &self,
        offset: usize,
    ) -> Result<&'r ExtractedToken<'t>, ParseWarning> {
        self.look_ahead(offset)
            .ok_or_else(|| self.make_warn(ParseWarningKind::EndOfInput))
    }

    // Utilities
    #[cold]
    #[inline]
    pub fn make_warn(&self, kind: ParseWarningKind) -> ParseWarning {
        ParseWarning::new(kind, self.rule, self.current)
    }
}
