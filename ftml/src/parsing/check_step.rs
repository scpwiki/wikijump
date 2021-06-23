/*
 * parsing/check_step.rs
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

use super::{ParseWarning, Parser, Token};

/// Helper function to assert that the current token matches, then step.
///
/// # Panics
/// Since an assert is used, this function will panic
/// if the extracted token does not match the one specified.
#[inline]
pub fn check_step(parser: &mut Parser, token: Token) -> Result<(), ParseWarning> {
    assert_eq!(
        parser.current().token,
        token,
        "Opening token isn't {}",
        token.name(),
    );

    parser.step()?;

    Ok(())
}

pub fn check_step_multiple(parser: &mut Parser, tokens: &[Token]) -> Result<(), ParseWarning> {
    assert!(
        tokens.contains(&parser.current().token),
        "Opening token isn't one of {:?}",
        tokens,
    );

    parser.step()?;

    Ok(())
}

#[test]
#[should_panic]
fn check_step_fail() {
    let log = crate::build_logger();
    let tokenization = crate::tokenize(&log, "**Apple** banana");
    let mut parser = Parser::new(&log, &tokenization);

    let _ = check_step(&mut parser, Token::Italics);
}

#[test]
#[should_panic]
fn check_step_multiple_fail() {
    let log = crate::build_logger();
    let tokenization = crate::tokenize(&log, "//Cherry//");
    let mut parser = Parser::new(&log, &tokenization);

    let _ = check_step_multiple(&mut parser, &[Token::Bold, Token::Underline]);
}
