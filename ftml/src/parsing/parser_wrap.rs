/*
 * parsing/parser_wrap.rs
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

use super::Parser;
use crate::tree::AcceptsPartial;
use std::ops::{Deref, DerefMut};

/// A wrapper around `Parser` which sets / clears the partial flag.
///
/// On creation, it sets an `AcceptsPartial` flag, and on destruction
/// it clears it.
#[derive(Debug)]
pub struct ParserWrap<'p, 'r, 't> {
    parser: &'p mut Parser<'r, 't>,
}

impl<'p, 'r, 't> ParserWrap<'p, 'r, 't> {
    #[inline]
    pub fn new(parser: &'p mut Parser<'r, 't>, flag: AcceptsPartial) -> Self {
        parser.set_accepts_partial(flag);

        ParserWrap { parser }
    }
}

impl<'p, 'r, 't> Deref for ParserWrap<'p, 'r, 't> {
    type Target = Parser<'r, 't>;

    fn deref(&self) -> &Parser<'r, 't> {
        self.parser
    }
}

impl<'p, 'r, 't> DerefMut for ParserWrap<'p, 'r, 't> {
    fn deref_mut(&mut self) -> &mut Parser<'r, 't> {
        self.parser
    }
}

impl Drop for ParserWrap<'_, '_, '_> {
    fn drop(&mut self) {
        self.parser.set_accepts_partial(AcceptsPartial::None);
    }
}

#[test]
fn wrap() {
    let log = &crate::build_logger();
    let page_info = crate::data::PageInfo::dummy();
    let tokens = crate::tokenize(log, "Test input");
    let mut parser = Parser::new(log, &page_info, &tokens);

    assert_eq!(
        parser.accepts_partial(),
        AcceptsPartial::None,
        "Initial partial flag is not none",
    );

    {
        let mut wrap = ParserWrap::new(&mut parser, AcceptsPartial::ListItem);

        assert_eq!(
            wrap.accepts_partial(),
            AcceptsPartial::ListItem,
            "Partial flag wasn't set",
        );

        wrap.step().expect("Unable to step");
    }

    assert_eq!(
        parser.accepts_partial(),
        AcceptsPartial::None,
        "Partial flag wasn't cleared after drop",
    );
}
