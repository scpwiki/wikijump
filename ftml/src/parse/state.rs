/*
 * parse/state.rs
 *
 * ftml - Library to parse Wikidot code
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

use super::stack::Stack;
use super::token::{ExtractedToken, Token};
use crate::tree::Element;
use strum_macros::IntoStaticStr;

#[derive(IntoStaticStr, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum State {
    /// Regular text state, not inside anything.
    Normal,

    /// We've read a `[[`, expecting a tag of some sort.
    Tag,

    /// We've read a `[[*`, expecting a special tag of some sort.
    TagSpecial,
}

impl State {
    pub fn consume<'r, 'a>(
        &mut self,
        log: &slog::Logger,
        stack: &'r mut Stack<'a>,
        extract: ExtractedToken<'a>,
    ) {
        debug!(
            log,
            "Running state consume for '{:?}' (token: {:?})",
            *self, extract;
            "state" => *self,
            "token" => extract.token,
            "slice" => extract.slice,
            "span-start" => extract.span.start,
            "span-end" => extract.span.end,
        );

        // Modify stack based on new token
        let new_state = match *self {
            State::Normal => consume_normal(stack, extract),
            State::Tag => consume_tag(stack, extract),
            State::TagSpecial => consume_tag_special(stack, extract),
        };

        // Set new state
        *self = new_state;
    }

    #[inline]
    fn name(self) -> &'static str {
        self.into()
    }
}

fn consume_normal<'a>(stack: &mut Stack<'a>, extract: ExtractedToken<'a>) -> State {
    let ExtractedToken { token, slice, span } = extract;

    match token {
        Token::Identifier | Token::Text | Token::Whitespace => {
            stack.append(Element::Text(slice));
            State::Normal
        }
        Token::LeftTag => State::Tag,
        Token::LeftTagSpecial => State::TagSpecial,
        _ => todo!(),
    }
}

fn consume_tag<'a>(_stack: &mut Stack<'a>, extract: ExtractedToken<'a>) -> State {
    let ExtractedToken { token, slice, span } = extract;

    todo!()
}

fn consume_tag_special<'a>(_stack: &mut Stack<'a>, extract: ExtractedToken<'a>) -> State {
    let ExtractedToken { token, slice, span } = extract;

    todo!()
}

impl slog::Value for State {
    fn serialize(
        &self,
        _: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        serializer.emit_str(key, self.name())
    }
}
