/*
 * render/debug.rs
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

//! A simple renderer that outputs the `SyntaxTree` using Rust's debug formatter.

use super::prelude::*;

#[derive(Debug)]
pub struct DebugRender;

impl Render for DebugRender {
    type Output = String;

    #[inline]
    fn render(
        &self,
        tree: &SyntaxTree,
        page_info: &PageInfo,
        settings: &WikitextSettings,
    ) -> String {
        debug!("Running debug logger on syntax tree");
        format!("{settings:#?}\n{page_info:#?}\n{tree:#?}")
    }
}

#[test]
fn debug_render() {
    use crate::layout::Layout;
    use crate::tree::BibliographyList;

    let page_info = PageInfo::dummy();
    let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
    let result = SyntaxTree::from_element_result(
        vec![],
        vec![],
        (vec![], vec![]),
        vec![],
        (vec![], true),
        BibliographyList::new(),
        0,
    );
    let (tree, _) = result.into();
    let output = DebugRender.render(&tree, &page_info, &settings);
    assert!(!output.is_empty(), "DebugRender produced empty output");
}
