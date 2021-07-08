/*
 * render/null.rs
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

//! A trivial renderer.
//!
//! This implementation of `Render` will consume any input syntax tree
//! and produce a unit value as output.

use super::prelude::*;

#[derive(Debug)]
pub struct NullRender;

impl Render for NullRender {
    type Output = ();

    #[inline]
    fn render(&self, _log: &Logger, _page_info: &PageInfo, _tree: &SyntaxTree) {}
}

#[test]
fn null() {
    let log = crate::build_logger();
    let page_info = PageInfo::dummy();
    let result = SyntaxTree::from_element_result(vec![], vec![], vec![]);
    let (tree, _) = result.into();
    let output = NullRender.render(&log, &page_info, &tree);

    assert_eq!(output, (), "Null render didn't produce the unit value");
}
