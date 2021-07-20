/*
 * test/prop/element.rs
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

use crate::tree::attribute::SAFE_ATTRIBUTES;
use crate::tree::{
    Alignment, AnchorTarget, AttributeMap, Container, ContainerType, Element,
    HeadingLevel, ImageAlignment, ImageSource, LinkLabel, Module, SyntaxTree,
};
use proptest::option;
use proptest::prelude::*;
use std::borrow::Cow;
use std::num::NonZeroU32;

// Constants

lazy_static! {
    static ref SAFE_ATTRIBUTES_VEC: Vec<&'static str> =
        SAFE_ATTRIBUTES.iter().map(|s| s.as_ref()).collect();
}

const SIMPLE_EMAIL_REGEX: &str = r"\w+([-+.']\w+)*@\w+([-.]\w+)*\.\w+([-.]\w+)*";

// Helper macros

macro_rules! select {
    ($items:expr) => {
        proptest::sample::select(&$items[..])
    };
}

macro_rules! cow {
    ($strategy:expr) => {
        $strategy.prop_map(Cow::Owned)
    };
}

// Leaf elements

fn arb_attribute_map() -> impl Strategy<Value = AttributeMap<'static>> {
    proptest::collection::btree_map(
        // Key
        prop_oneof![
            // Safe attribute
            select!(SAFE_ATTRIBUTES_VEC).prop_map(|s| Cow::Owned(str!(s))),
            // Random attribute
            cow!(r"\w+"),
        ],
        // Value
        cow!(".*"),
        // Length
        0..12,
    )
    .prop_map(|map| AttributeMap::from(map))
}

#[inline]
fn arb_optional_str() -> impl Strategy<Value = Option<Cow<'static, str>>> {
    option::of(cow!(".*"))
}

fn arb_module() -> impl Strategy<Value = Element<'static>> {
    let join = (arb_optional_str(), arb_attribute_map()).prop_map(
        |(button_text, attributes)| Module::Join {
            button_text,
            attributes,
        },
    );

    let page_tree = (
        arb_optional_str(),
        any::<bool>(),
        any::<u32>().prop_map(NonZeroU32::new),
    )
        .prop_map(|(root, show_root, depth)| Module::PageTree {
            root,
            show_root,
            depth,
        });

    prop_oneof![
        Just(Module::Rate),
        arb_optional_str().prop_map(|page| Module::Backlinks { page }),
        any::<bool>().prop_map(|include_hidden| Module::Categories { include_hidden }),
        join,
        page_tree,
    ]
    .prop_map(Element::Module)
}

fn arb_target() -> impl Strategy<Value = Option<AnchorTarget>> {
    option::of(select!([
        AnchorTarget::NewTab,
        AnchorTarget::Parent,
        AnchorTarget::Top,
        AnchorTarget::Same,
    ]))
}

fn arb_link() -> impl Strategy<Value = Element<'static>> {
    let label = prop_oneof![
        cow!(".*").prop_map(LinkLabel::Text),
        option::of(cow!(".*")).prop_map(LinkLabel::Url),
        Just(LinkLabel::Page),
    ];

    (cow!(".+"), label, arb_target()).prop_map(|(url, label, target)| Element::Link {
        url,
        label,
        target,
    })
}

fn arb_image() -> impl Strategy<Value = Element<'static>> {
    let source = prop_oneof![
        cow!(".*").prop_map(ImageSource::Url),
        cow!(".*").prop_map(|file| ImageSource::File1 { file }),
        (cow!(".*"), cow!(".*"))
            .prop_map(|(page, file)| ImageSource::File2 { page, file }),
        (cow!(".*"), cow!(".*"), cow!(".*"))
            .prop_map(|(site, page, file)| ImageSource::File3 { site, page, file }),
    ];

    let alignment = select!([
        Alignment::Left,
        Alignment::Right,
        Alignment::Center,
        Alignment::Justify,
    ]);

    let image_alignment = option::of(
        (alignment, any::<bool>())
            .prop_map(|(align, float)| ImageAlignment { align, float }),
    );

    (
        source,
        arb_optional_str(),
        image_alignment,
        arb_attribute_map(),
    )
        .prop_map(|(source, link, alignment, attributes)| Element::Image {
            source,
            link,
            alignment,
            attributes,
        })
}

fn arb_code() -> impl Strategy<Value = Element<'static>> {
    (cow!(".*"), arb_optional_str())
        .prop_map(|(contents, language)| Element::Code { contents, language })
}

fn arb_checkbox() -> impl Strategy<Value = Element<'static>> {
    (any::<bool>(), arb_attribute_map()).prop_map(|(checked, attributes)| {
        Element::CheckBox {
            checked,
            attributes,
        }
    })
}

// Container elements

fn arb_container<S>(elements: S) -> impl Strategy<Value = Element<'static>>
where
    S: Strategy<Value = Vec<Element<'static>>>,
{
    let alignment = select!([
        Alignment::Left,
        Alignment::Right,
        Alignment::Center,
        Alignment::Justify,
    ]);

    let heading = select!([
        HeadingLevel::One,
        HeadingLevel::Two,
        HeadingLevel::Three,
        HeadingLevel::Four,
        HeadingLevel::Five,
        HeadingLevel::Six,
    ]);

    let container_type = prop_oneof![
        Just(ContainerType::Bold),
        Just(ContainerType::Italics),
        Just(ContainerType::Underline),
        Just(ContainerType::Superscript),
        Just(ContainerType::Subscript),
        Just(ContainerType::Strikethrough),
        Just(ContainerType::Monospace),
        Just(ContainerType::Span),
        Just(ContainerType::Div),
        Just(ContainerType::Mark),
        Just(ContainerType::Blockquote),
        Just(ContainerType::Insertion),
        Just(ContainerType::Deletion),
        Just(ContainerType::Hidden),
        Just(ContainerType::Invisible),
        Just(ContainerType::Size),
        Just(ContainerType::Paragraph),
        alignment.prop_map(|align| ContainerType::Align(align)),
        heading.prop_map(|level| ContainerType::Header(level)),
    ];

    (container_type, elements, arb_attribute_map()).prop_map(
        |(ctype, elements, attributes)| {
            Element::Container(Container::new(ctype, elements, attributes))
        },
    )
}

fn arb_collapsible<S>(elements: S) -> impl Strategy<Value = Element<'static>>
where
    S: Strategy<Value = Vec<Element<'static>>>,
{
    (
        elements,
        arb_attribute_map(),
        any::<bool>(),
        arb_optional_str(),
        arb_optional_str(),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(
            |(
                elements,
                attributes,
                start_open,
                show_text,
                hide_text,
                show_top,
                show_bottom,
            )| Element::Collapsible {
                elements,
                attributes,
                start_open,
                show_text,
                hide_text,
                show_top,
                show_bottom,
            },
        )
}

// The full syntax tree, recursive

fn arb_tree() -> impl Strategy<Value = SyntaxTree<'static>> {
    let leaf = prop_oneof![
        cow!(".*").prop_map(Element::Text),
        cow!(".*").prop_map(Element::Raw),
        cow!(SIMPLE_EMAIL_REGEX).prop_map(Element::Email),
        arb_module(),
        arb_link(),
        arb_image(),
        // TODO: Element::List
        // TODO: Element::RadioButton
        arb_checkbox(),
        // TODO: Element::User
        arb_code(),
        cow!(".*").prop_map(|contents| Element::Html { contents }),
        // TODO: Element::Iframe
        Just(Element::LineBreak),
        (1..50u32).prop_map(|count| Element::LineBreaks(NonZeroU32::new(count).unwrap())),
        Just(Element::HorizontalRule),
    ];

    let element = leaf.prop_recursive(
        50,  // Levels deep
        200, // Number of total nodes
        20,  // Up to X items per collection
        |inner| {
            // Inner strategy for recursive cases
            macro_rules! elements {
                () => {
                    proptest::collection::vec(inner.clone(), 1..100)
                };
            }

            prop_oneof![
                arb_container(elements!()),
                // TODO: Element::Anchor
                arb_collapsible(elements!()),
                // TODO: Element::IfCategory
                // TODO: Element::IfTags
                // TODO: Element::Color
            ]
        },
    );

    (
        proptest::collection::vec(element, 1..100),
        proptest::collection::vec(cow!(".*"), 0..128),
    )
        .prop_map(|(elements, styles)| SyntaxTree { elements, styles })
}

// TODO
