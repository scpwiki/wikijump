/*
 * test/id_prefix.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2022 Wikijump Team
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
use crate::settings::{WikitextMode, WikitextSettings, EMPTY_INTERWIKI};
use crate::tree::{
    AttributeMap, Container, ContainerType, Element, ImageSource, SyntaxTree,
};
use std::borrow::Cow;

#[test]
fn isolate_user_ids() {
    macro_rules! cow {
        ($text:expr) => {
            Cow::Borrowed($text)
        };
    }

    macro_rules! text {
        ($text:expr) => {
            Element::Text(cow!($text))
        };
    }

    let page_info = PageInfo {
        page: cow!("isolated-user-id-test"),
        category: None,
        site: cow!("test"),
        title: cow!("test"),
        alt_title: None,
        rating: 0.0,
        tags: vec![],
        language: cow!("default"),
    };

    let settings = WikitextSettings {
        mode: WikitextMode::Page,
        enable_page_syntax: true,
        use_true_ids: true,
        isolate_user_ids: true,
        allow_local_paths: true,
        interwiki: EMPTY_INTERWIKI.clone(),
    };

    fn append_footnote_block(mut elements: Vec<Element>) -> Vec<Element> {
        elements.push(Element::FootnoteBlock {
            title: None,
            hide: false,
        });
        elements
    }

    macro_rules! check {
        ($wikitext:expr, $elements:expr $(,)?) => {{
            let mut text = str!($wikitext);

            crate::preprocess(&mut text);
            let tokens = crate::tokenize(&text);
            let result = crate::parse(&tokens, &page_info, &settings);
            let (actual_tree, warnings) = result.into();

            let expected_tree = SyntaxTree {
                elements: append_footnote_block($elements),
                styles: vec![],
                table_of_contents: vec![],
                footnotes: vec![],
            };

            assert!(warnings.is_empty(), "Warnings produced during parsing!");
            assert_eq!(
                actual_tree, expected_tree,
                "Actual syntax tree didn't match expected",
            );
        }};
    }

    // Trivial
    check!("", vec![]);

    // Anchor block [[a]]
    check!(
        r#"[[a id="apple"]]X[[/a]]"#,
        vec![Element::Container(Container::new(
            ContainerType::Paragraph,
            vec![Element::Anchor {
                target: None,
                attributes: AttributeMap::from(btreemap! {
                    cow!("id") => cow!("u-apple"),
                }),
                elements: vec![text!("X")],
            }],
            AttributeMap::new(),
        ))],
    );
    check!(
        r#"[[a id="u-apple"]]X[[/a]]"#,
        vec![Element::Container(Container::new(
            ContainerType::Paragraph,
            vec![Element::Anchor {
                target: None,
                attributes: AttributeMap::from(btreemap! {
                    cow!("id") => cow!("u-apple"),
                }),
                elements: vec![text!("X")],
            }],
            AttributeMap::new(),
        ))],
    );

    // Images [[image]]
    check!(
        r#"[[image example.png class="apple" id="banana"]]"#,
        vec![Element::Container(Container::new(
            ContainerType::Paragraph,
            vec![Element::Image {
                source: ImageSource::File1 {
                    file: cow!("example.png"),
                },
                link: None,
                alignment: None,
                attributes: AttributeMap::from(btreemap! {
                    cow!("class") => cow!("apple"),
                    cow!("id") => cow!("u-banana"),
                }),
            }],
            AttributeMap::new(),
        ))],
    );
    check!(
        r#"[[image example.png class="u-apple" id="u-banana"]]"#,
        vec![Element::Container(Container::new(
            ContainerType::Paragraph,
            vec![Element::Image {
                source: ImageSource::File1 {
                    file: cow!("example.png"),
                },
                link: None,
                alignment: None,
                attributes: AttributeMap::from(btreemap! {
                    cow!("class") => cow!("u-apple"),
                    cow!("id") => cow!("u-banana"),
                }),
            }],
            AttributeMap::new(),
        ))],
    );
}
