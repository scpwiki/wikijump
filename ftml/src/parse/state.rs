/*
 * parse/state.rs
 *
 * wikidot-html - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith for Project Foundation
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

use crate::Token;
use regex::Regex;
use std::ops::RangeBounds;

#[derive(Debug, Clone)]
pub struct ParseState {
    text: String,
    tokens: Vec<Token>,
}

impl ParseState {
    pub fn new(text: String) -> Self {
        let mut this = ParseState {
            text,
            tokens: Vec::new(),
        };

        this.replace_all("\0", "");
        this
    }

    pub fn push_token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    #[inline]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[inline]
    pub fn token(&self, index: usize) -> Option<&Token> {
        self.tokens.get(index)
    }

    #[inline]
    pub fn tokens(&self) -> &[Token] {
        self.tokens.as_slice()
    }

    #[inline]
    pub fn insert(&mut self, idx: usize, ch: char) {
        self.text.insert(idx, ch);
    }

    #[inline]
    pub fn push(&mut self, ch: char) {
        self.text.push(ch);
    }

    #[inline]
    pub fn push_str(&mut self, s: &str) {
        self.text.push_str(s);
    }

    #[inline]
    pub fn replace_range<R: RangeBounds<usize>>(&mut self, range: R, replace_with: &str) {
        self.text.replace_range(range, replace_with);
    }

    pub fn replace_all(&mut self, pattern: &str, replace_with: &str) {
        while let Some(idx) = self.text.find(pattern) {
            self.text
                .replace_range(idx..idx + pattern.len(), replace_with);
        }
    }

    pub fn replace_all_regex(&mut self, regex: &Regex, replace_with: &str) {
        while let Some(mtch) = regex.find(&self.text) {
            self.text
                .replace_range(mtch.start()..mtch.end(), replace_with);
        }
    }

    pub fn replace_once(&mut self, pattern: &str, replace_with: &str) {
        if let Some(idx) = self.text.find(pattern) {
            self.text
                .replace_range(idx..idx + pattern.len(), replace_with);
        }
    }

    pub fn replace_once_regex(&mut self, regex: &Regex, replace_with: &str) {
        if let Some(mtch) = regex.find(&self.text) {
            self.text
                .replace_range(mtch.start()..mtch.end(), replace_with);
        }
    }
}
