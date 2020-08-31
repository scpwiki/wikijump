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

mod consume;
mod rule;
mod stack;
mod token;

use self::consume::consume;
use self::rule::RuleResult;
use self::token::Token;
use crate::tree::SyntaxTree;
use slog::Logger;

pub fn parse<'a>(log: &Logger, text: &'a str) -> SyntaxTree<'a> {
    let log = &log.new(slog_o!("function" => "parse", "text" => str!(text)));

    info!(log, "Running parser on text");

    let tokens = Token::extract_all(log, text);
    let mut tokens = tokens.as_slice();
    let mut elements = Vec::new();

    while !tokens.is_empty() {
        // Consume tokens to get next element
        let RuleResult { offset, element } = {
            let (extracted, next) = tokens
                .split_first() //
                .expect("Tokens list is empty");

            consume(log, extracted, next)
        };

        // We need to consume at least one token, otherwise this loops forever.
        // Returning a 0 is definitely a bug.
        assert_ne!(offset, 0, "Returned token offset was zero");

        // Update state
        tokens = &tokens[offset..];
        elements.push(element);
    }

    debug!(log, "Finished running parser, returning gathered elements");

    // TODO
    SyntaxTree { elements }
}

#[test]
fn ast() {
    let logger = crate::build_logger();
    let text = "some test string";
    let tree = parse(&logger, text);
    println!("Tree: {:#?}", tree);
}
