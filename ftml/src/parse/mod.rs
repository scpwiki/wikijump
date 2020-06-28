/*
 * parse/mod.rs
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

mod stack;
mod state;
mod token;

use self::stack::Stack;
use self::state::State;
use self::token::Token;
use crate::tree::SyntaxTree;
use slog::Logger;

pub fn parse<'a>(log: &Logger, text: &'a str) -> SyntaxTree<'a> {
    let log = log.new(slog_o!("function" => "parse", "text" => str!(text)));

    info!(log, "Running parser on text");

    let extracted = Token::extract_all(&log, text);
    let mut stack = Stack::new();
    let mut state = State::Normal;

    for extract in extracted {
        state.consume(&mut stack, extract);
    }

    // TODO
    stack.into_syntax_tree()
}
