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
    AttributeMap, Container, ContainerType, Element, ImageSource, ListItem, ListType,
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
        use_include_compatibility: false,
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
            let (tree, warnings) = result.into();

            let actual = tree.elements;
            let expected = append_footnote_block($elements);

            assert!(warnings.is_empty(), "Warnings produced during parsing!");
            assert_eq!(actual, expected, "Actual elements didn't match expected");
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
    check!(
        r#"[[a id="u-u-apple"]]X[[/a]]"#,
        vec![Element::Container(Container::new(
            ContainerType::Paragraph,
            vec![Element::Anchor {
                target: None,
                attributes: AttributeMap::from(btreemap! {
                    cow!("id") => cow!("u-u-apple"),
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

    // Lists [[ul]] / [[ol]]
    check!(
        r#"[[ul id="apple"]] [[li id="u-banana"]]X[[/li]] [[/ul]]"#,
        vec![Element::List {
            ltype: ListType::Bullet,
            attributes: AttributeMap::from(btreemap! {
                cow!("id") => cow!("u-apple"),
            }),
            items: vec![ListItem::Elements {
                attributes: AttributeMap::from(btreemap! {
                    cow!("id") => cow!("u-banana"),
                }),
                elements: vec![text!("X")],
            }],
        }],
    );
    check!(
        r#"[[ul id="u-apple"]] [[li id="banana"]]X[[/li]] [[/ul]]"#,
        vec![Element::List {
            ltype: ListType::Bullet,
            attributes: AttributeMap::from(btreemap! {
                cow!("id") => cow!("u-apple"),
            }),
            items: vec![ListItem::Elements {
                attributes: AttributeMap::from(btreemap! {
                    cow!("id") => cow!("u-banana"),
                }),
                elements: vec![text!("X")],
            }],
        }],
    );

    check!(
        r#"[[ol id="apple"]] [[li id="u-banana"]]X[[/li]] [[/ol]]"#,
        vec![Element::List {
            ltype: ListType::Numbered,
            attributes: AttributeMap::from(btreemap! {
                cow!("id") => cow!("u-apple"),
            }),
            items: vec![ListItem::Elements {
                attributes: AttributeMap::from(btreemap! {
                    cow!("id") => cow!("u-banana"),
                }),
                elements: vec![text!("X")],
            }],
        }],
    );
    check!(
        r#"[[ol id="u-apple"]] [[li id="banana"]]X[[/li]] [[/ol]]"#,
        vec![Element::List {
            ltype: ListType::Numbered,
            attributes: AttributeMap::from(btreemap! {
                cow!("id") => cow!("u-apple"),
            }),
            items: vec![ListItem::Elements {
                attributes: AttributeMap::from(btreemap! {
                    cow!("id") => cow!("u-banana"),
                }),
                elements: vec![text!("X")],
            }],
        }],
    );

    // Radio buttons and checkboxes
    check!(
        r#"[[radio vegetables class="apple" id="banana"]] Celery
[[radio vegetables class="u-cherry" id="u-durian"]] Lettuce"#,
        vec![Element::Container(Container::new(
            ContainerType::Paragraph,
            vec![
                Element::RadioButton {
                    name: cow!("vegetables"),
                    checked: false,
                    attributes: AttributeMap::from(btreemap! {
                        cow!("class") => cow!("apple"),
                        cow!("id") => cow!("u-banana"),
                    }),
                },
                text!("Celery"),
                Element::LineBreak,
                Element::RadioButton {
                    name: cow!("vegetables"),
                    checked: false,
                    attributes: AttributeMap::from(btreemap! {
                        cow!("class") => cow!("u-cherry"),
                        cow!("id") => cow!("u-durian"),
                    }),
                },
                text!("Lettuce"),
            ],
            AttributeMap::new(),
        ))],
    );
    check!(
        r#"[[checkbox class="apple" id="banana"]] Celery
[[checkbox class="u-cherry" id="u-durian"]] Lettuce"#,
        vec![Element::Container(Container::new(
            ContainerType::Paragraph,
            vec![
                Element::CheckBox {
                    checked: false,
                    attributes: AttributeMap::from(btreemap! {
                        cow!("class") => cow!("apple"),
                        cow!("id") => cow!("u-banana"),
                    }),
                },
                text!("Celery"),
                Element::LineBreak,
                Element::CheckBox {
                    checked: false,
                    attributes: AttributeMap::from(btreemap! {
                        cow!("class") => cow!("u-cherry"),
                        cow!("id") => cow!("u-durian"),
                    }),
                },
                text!("Lettuce"),
            ],
            AttributeMap::new(),
        ))],
    );

    // Collapsibles [[collapsible]]
    check!(
        r#"[[collapsible class="apple" id="banana"]]X[[/collapsible]]"#,
        vec![Element::Collapsible {
            elements: vec![Element::Container(Container::new(
                ContainerType::Paragraph,
                vec![text!("X")],
                AttributeMap::new(),
            ))],
            attributes: AttributeMap::from(btreemap! {
                cow!("class") => cow!("apple"),
                cow!("id") => cow!("u-banana"),
            }),
            start_open: false,
            show_text: None,
            hide_text: None,
            show_top: true,
            show_bottom: false,
        }],
    );
    check!(
        r#"[[collapsible class="u-apple" id="u-banana"]]X[[/collapsible]]"#,
        vec![Element::Collapsible {
            elements: vec![Element::Container(Container::new(
                ContainerType::Paragraph,
                vec![text!("X")],
                AttributeMap::new(),
            ))],
            attributes: AttributeMap::from(btreemap! {
                cow!("class") => cow!("u-apple"),
                cow!("id") => cow!("u-banana"),
            }),
            start_open: false,
            show_text: None,
            hide_text: None,
            show_top: true,
            show_bottom: false,
        }],
    );

    // Table of contents [[toc]]
    check!(
        r#"[[toc id="apple"]]"#,
        vec![Element::TableOfContents {
            attributes: AttributeMap::from(btreemap! {
                cow!("id") => cow!("u-apple"),
            }),
            align: None,
        }],
    );
    check!(
        r#"[[toc id="u-apple"]]"#,
        vec![Element::TableOfContents {
            attributes: AttributeMap::from(btreemap! {
                cow!("id") => cow!("u-apple"),
            }),
            align: None,
        }],
    );

    // Iframes [[iframe]]
    check!(
        r#"[[iframe https://example.com/ id="apple"]]"#,
        vec![Element::Iframe {
            attributes: AttributeMap::from(btreemap! {
                cow!("id") => cow!("u-apple"),
            }),
            url: cow!("https://example.com/"),
        }],
    );
    check!(
        r#"[[iframe https://example.com/ id="u-apple"]]"#,
        vec![Element::Iframe {
            attributes: AttributeMap::from(btreemap! {
                cow!("id") => cow!("u-apple"),
            }),
            url: cow!("https://example.com/"),
        }],
    );
}
