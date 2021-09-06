/*
 * render/debug.rs
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

//! A simple renderer that outputs the `SyntaxTree` using Rust's debug formatter.

use super::prelude::*;

#[derive(Debug)]
pub struct DebugRender;

impl Render for DebugRender {
    type Output = String;

    #[inline]
    fn render(&self, log: &Logger, page_info: &PageInfo, tree: &SyntaxTree) -> String {
        info!(log, "Running debug logger on syntax tree");

        format!("{:#?}\n{:#?}", page_info, tree)
    }
}

#[test]
fn debug() {
    // Expected outputs
    const OUTPUT: &str = r#"PageInfo {
    page: "some-page",
    category: None,
    site: "sandbox",
    title: "A page for the age",
    alt_title: None,
    rating: 69.0,
    tags: [
        "tale",
        "_cc",
    ],
    language: "default",
}
SyntaxTree {
    elements: [
        Text(
            "apple",
        ),
        Text(
            " ",
        ),
        Container(
            Container {
                ctype: Bold,
                attributes: {},
                elements: [
                    Text(
                        "banana",
                    ),
                ],
            },
        ),
    ],
    styles: [
        "span.hidden-text { display: none; }",
    ],
    table_of_contents: [],
    footnotes: [],
}"#;

    let log = crate::build_logger();
    let page_info = PageInfo::dummy();

    // Syntax tree construction
    let elements = vec![
        text!("apple"),
        text!(" "),
        Element::Container(Container::new(
            ContainerType::Bold,
            vec![text!("banana")],
            AttributeMap::new(),
        )),
    ];
    let warnings = vec![];
    let styles = vec![cow!("span.hidden-text { display: none; }")];
    let table_of_contents = vec![];
    let footnotes = vec![];

    let result = SyntaxTree::from_element_result(
        elements,
        warnings,
        styles,
        table_of_contents,
        footnotes,
    );
    let (tree, _) = result.into();

    // Perform rendering
    let output = DebugRender.render(&log, &page_info, &tree);
    assert_eq!(
        output, OUTPUT,
        "Pretty JSON syntax tree output doesn't match",
    );
}
