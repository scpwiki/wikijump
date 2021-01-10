/*
 * parse/wrapper.rs
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

use super::{BlockParser, Parser};

/// Structure for `Parser`-wrapping objects.
///
/// This allows both `Parser` and any superset objects to be passed in
/// and utilized, with all
///
/// # Lifetimes
/// `'r` and `'t` represent the typical lifetimes used throughout the code,
/// for the original source text (`'t`) and the original list of extracted tokens (`'r`).
///
/// The `'p` lifetime represents how long the `Parser` object itself lasts.
/// This is superseded by `'o`, which is the length of time the particular
/// `Parser`-wrapping object lasts. For the base case of `&mut Parser`, `'o == 'p`.
#[derive(Debug)]
pub enum ParserWrapper<'o, 'p, 'r, 't>
where
    'p: 'o,
    'r: 't,
{
    Parser(&'o mut Parser<'r, 't>),
    BlockParser(&'o mut BlockParser<'p, 'r, 't>),
}

impl<'o, 'p, 'r, 't> ParserWrapper<'o, 'p, 'r, 't>
where
    'p: 'o,
    'r: 't,
{
    pub fn as_ref(&self) -> &Parser<'r, 't> {
        match self {
            ParserWrapper::Parser(parser) => parser,
            ParserWrapper::BlockParser(ref bparser) => bparser.get(),
        }
    }

    #[inline]
    pub fn as_mut(&mut self) -> &mut Parser<'r, 't> {
        match self {
            ParserWrapper::Parser(parser) => parser,
            ParserWrapper::BlockParser(ref mut bparser) => bparser.get_mut(),
        }
    }

    #[inline]
    pub fn as_block_parser(&mut self) -> &mut BlockParser<'p, 'r, 't> {
        match self {
            ParserWrapper::BlockParser(ref mut bparser) => bparser,
            _ => panic!("Incorect variant, not BlockParser: {:?}", self),
        }
    }
}

impl<'o, 'p, 'r, 't> From<&'o mut Parser<'r, 't>> for ParserWrapper<'o, 'p, 'r, 't>
where
    'p: 'o,
    'r: 't,
{
    #[inline]
    fn from(parser: &'o mut Parser<'r, 't>) -> Self {
        ParserWrapper::Parser(parser)
    }
}

impl<'o, 'p, 'r, 't> From<&'o mut BlockParser<'p, 'r, 't>>
    for ParserWrapper<'o, 'p, 'r, 't>
where
    'p: 'o,
    'r: 't,
{
    #[inline]
    fn from(bparser: &'o mut BlockParser<'p, 'r, 't>) -> Self {
        ParserWrapper::BlockParser(bparser)
    }
}
