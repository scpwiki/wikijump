/*
 * parsing/parser_wrap.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2026 Wikijump Team
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
    original: AcceptsPartial,
}

impl<'p, 'r, 't> ParserWrap<'p, 'r, 't> {
    #[inline]
    pub fn new(parser: &'p mut Parser<'r, 't>, flag: AcceptsPartial) -> Self {
        let original = parser.accepts_partial();
        parser.set_accepts_partial(flag);

        ParserWrap { parser, original }
    }
}

impl<'r, 't> Deref for ParserWrap<'_, 'r, 't> {
    type Target = Parser<'r, 't>;

    fn deref(&self) -> &Parser<'r, 't> {
        self.parser
    }
}

impl<'r, 't> DerefMut for ParserWrap<'_, 'r, 't> {
    fn deref_mut(&mut self) -> &mut Parser<'r, 't> {
        self.parser
    }
}

impl Drop for ParserWrap<'_, '_, '_> {
    fn drop(&mut self) {
        self.parser.set_accepts_partial(self.original);
    }
}

#[test]
fn wrap() {
    use crate::data::PageInfo;
    use crate::layout::Layout;
    use crate::settings::{WikitextMode, WikitextSettings};

    let page_info = PageInfo::dummy();
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let tokens = crate::tokenize("Test input");
    let mut parser = Parser::new(&tokens, &page_info, &settings);

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
