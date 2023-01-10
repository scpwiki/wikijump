/*
 * render/html/test.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2023 Wikijump Team
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

use super::prelude::*;
use super::HtmlRender;
use crate::tree::BibliographyList;

#[test]
fn html() {
    let page_info = PageInfo::dummy();
    let settings = WikitextSettings::from_mode(WikitextMode::Page);
    let result = SyntaxTree::from_element_result(
        vec![],
        vec![],
        vec![],
        vec![],
        BibliographyList::new(),
        0,
    );
    let (tree, _) = result.into();
    let _output = HtmlRender.render(&tree, &page_info, &settings);
}
