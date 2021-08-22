/*
 * test/large.rs
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

use crate::data::PageInfo;
use crate::parsing::{ParseWarningKind, Token};
use crate::tree::{Element, SyntaxTree};
use std::borrow::Cow;

/// Test the parser's recursion limit.
///
/// Manually implemented test, since this test would be
/// tremendously huge on disk as a JSON file, and
/// also goes past serde_json's recursion limit, lol.
#[test]
fn recursion_depth() {
    let log = crate::build_logger();
    let page_info = PageInfo::dummy();

    // Build wikitext input
    let mut input = String::new();

    for _ in 0..101 {
        input.push_str("[[div]]\n");
    }

    for _ in 0..101 {
        input.push_str("[[/div]]\n");
    }

    // Run parser steps
    crate::preprocess(&log, &mut input);
    let tokens = crate::tokenize(&log, &input);
    let (tree, warnings) = crate::parse(&log, &page_info, &tokens).into();

    // Check outputted warnings
    let warning = warnings.get(0).expect("No warnings produced");
    assert_eq!(warning.token(), Token::LeftBlock);
    assert_eq!(warning.rule(), "block-div");
    assert_eq!(warning.span(), 800..802);
    assert_eq!(warning.kind(), ParseWarningKind::RecursionDepthExceeded);

    // Check syntax tree
    //
    // It outputs the entire input string as text

    let SyntaxTree { elements, .. } = tree;
    assert_eq!(elements.len(), 1);

    let element = elements.get(0).expect("No elements produced");
    let input_cow = Cow::Borrowed(input.as_ref());
    assert_eq!(element, &Element::Text(input_cow));
}

/// Test the parser's ability to process large bodies
#[test]
fn large_payload() {
    const ITERATIONS: usize = 50;

    let log = crate::build_logger();
    let page_info = PageInfo::dummy();

    // Build wikitext input
    let mut input = String::new();

    for _ in 0..ITERATIONS {
        // Lines intentionally broken in weird places
        input.push_str("
[[div]]
Lorem ipsum dolor sit amet, consectetur adipiscing elit.
Maecenas sed risus sed ex suscipit ultricies ac quis metus.
Mauris facilisis dui quam, in mollis velit ultrices vitae. Nam pretium accumsan arcu eu ultricies. Sed viverra eleifend elit at blandit. Aenean tempor vitae ipsum vitae lacinia.
Proin eu maximus nulla, id imperdiet libero. Duis convallis posuere arcu vitae sodales. Cras porta ac ligula non porttitor.
Proin et sodales arcu. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos. Mauris eget ante maximus, tincidunt enim nec, dignissim mi.
Quisque tincidunt convallis faucibus. Praesent vel semper dolor, vel tincidunt mi.

In hac habitasse platea dictumst. Vestibulum fermentum libero nec erat porttitor fermentum. Etiam at convallis odio, gravida commodo ipsum. Phasellus consequat nisl vitae ultricies pulvinar. Integer scelerisque eget nisl id fermentum. Pellentesque pretium, enim non molestie rhoncus, dolor diam porta mauris, eu cursus dolor est condimentum nisi. Phasellus tellus est, euismod non accumsan at, congue eget erat.

% ]] ! $ * -- @< _
[[/div]]
        ");
    }

    // Run parser steps
    crate::preprocess(&log, &mut input);
    let tokens = crate::tokenize(&log, &input);
    let (_tree, warnings) = crate::parse(&log, &page_info, &tokens).into();

    // Check output
    assert_eq!(warnings.len(), ITERATIONS * 3);
}
