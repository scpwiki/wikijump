/*
 * parser/check_step.rs
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
