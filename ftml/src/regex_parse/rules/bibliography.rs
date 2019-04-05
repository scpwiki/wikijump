/*
 * parse/rules/bibliography.rs
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

//! Processing rules for bibliographies.
//! Includes BibItem, Bibliography, and BibCite (in that order).
//! Currently mocked.

use crate::{ParseState, Result};

pub fn rule_bibliography(_state: &mut ParseState) -> Result<()> {
    println!("MOCK: rule.bibliography");
    Ok(())
}

#[test]
fn test_bibliography() {
    // TODO
}
