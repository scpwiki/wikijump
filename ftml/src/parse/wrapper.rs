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

#[derive(Debug)]
pub enum ParserWrapper<'p, 'r, 't>
where
    'r: 't,
{
    Parser(&'p mut Parser<'r, 't>),
    BlockParser(&'p mut BlockParser<'p, 'r, 't>),
}

impl<'p, 'r, 't> ParserWrapper<'p, 'r, 't>
where
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

impl<'p, 'r, 't> From<&'p mut Parser<'r, 't>> for ParserWrapper<'p, 'r, 't>
where
    'r: 't,
{
    #[inline]
    fn from(parser: &'p mut Parser<'r, 't>) -> Self {
        ParserWrapper::Parser(parser)
    }
}

impl<'p, 'r, 't> From<&'p mut BlockParser<'p, 'r, 't>> for ParserWrapper<'p, 'r, 't>
where
    'r: 't,
{
    #[inline]
    fn from(bparser: &'p mut BlockParser<'p, 'r, 't>) -> Self {
        ParserWrapper::BlockParser(bparser)
    }
}
