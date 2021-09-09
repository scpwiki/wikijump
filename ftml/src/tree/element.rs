/*
 * tree/element.rs
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

use super::clone::{
    elements_to_owned, list_items_to_owned, option_string_to_owned, string_to_owned,
};
use super::{
    Alignment, AnchorTarget, AttributeMap, ClearFloat, Container, FloatAlignment,
    ImageSource, LinkLabel, LinkLocation, ListItem, ListType, Module, Table, TableItem,
};
use ref_map::*;
use std::borrow::Cow;
use std::num::NonZeroU32;
use std::slice;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", tag = "element", content = "data")]
pub enum Element<'t> {
    /// Generic element that contains other elements within it.
    ///
    /// Examples would include divs, italics, paragraphs, etc.
    Container(Container<'t>),

    /// A Wikidot module being invoked, along with its arguments.
    ///
    /// These modules require some kind of processing by backend software,
    /// so are represented in module forum rather than as elements to be
    /// directly rendered.
    Module(Module<'t>),

    /// An element only containing text.
    ///
    /// Should be formatted like typical body text.
    Text(Cow<'t, str>),

    /// Raw text.
    ///
    /// This should be formatted exactly as listed.
    /// For instance, spaces being rendered to HTML should
    /// produce a `&nbsp;`.
    Raw(Cow<'t, str>),

    /// An element indicating an email.
    ///
    /// Whether this should become a clickable href link or just text
    /// is up to the render implementation.
    Email(Cow<'t, str>),

    /// An element representing an HTML table.
    Table(Table<'t>),

    /// A particular portion of a table.
    ///
    /// This will not occur in final trees, but is a special
    /// `Element` returned during parsing.
    TableItem(TableItem<'t>),

    /// An element representing an arbitrary anchor.
    ///
    /// This is distinct from link in that it maps to HTML `<a>`,
    /// and does not necessarily mean a link to some other URL.
    Anchor {
        target: Option<AnchorTarget>,
        attributes: AttributeMap<'t>,
        elements: Vec<Element<'t>>,
    },

    /// An element linking to a different page.
    ///
    /// The "label" field is an optional field denoting what the link should
    /// display.
    ///
    /// The "link" field is either a page reference (relative URL) or full URL.
    Link {
        link: LinkLocation<'t>,
        label: LinkLabel<'t>,
        target: Option<AnchorTarget>,
    },

    /// An element representing an image and its associated metadata.
    ///
    /// The "source" field is the link to the image itself.
    ///
    /// The "link" field is what the `<a>` points to, when the user clicks on the image.
    Image {
        source: ImageSource<'t>,
        link: Option<LinkLocation<'t>>,
        alignment: Option<FloatAlignment>,
        attributes: AttributeMap<'t>,
    },

    /// An ordered or unordered list.
    List {
        #[serde(rename = "type")]
        ltype: ListType,
        attributes: AttributeMap<'t>,
        items: Vec<ListItem<'t>>,
    },

    /// A particular item of a list.
    ///
    /// This will not occur in final trees, but is a special
    /// `Element` returned during parsing.
    ListItem(Box<ListItem<'t>>),

    /// A radio button.
    ///
    /// The "name" field translates to HTML, but is standard for grouping them.
    /// The "checked" field determines if the radio button starts checked or not.
    RadioButton {
        name: Cow<'t, str>,
        checked: bool,
        attributes: AttributeMap<'t>,
    },

    /// A checkbox.
    ///
    /// The "checked" field determines if the radio button starts checked or not.
    CheckBox {
        checked: bool,
        attributes: AttributeMap<'t>,
    },

    /// A collapsible, containing content hidden to be opened on click.
    ///
    /// This is an interactable element provided by Wikidot which allows hiding
    /// all of the internal elements until it is opened by clicking, which can
    /// then be re-hidden by clicking again.
    #[serde(rename_all = "kebab-case")]
    Collapsible {
        elements: Vec<Element<'t>>,
        attributes: AttributeMap<'t>,
        start_open: bool,
        show_text: Option<Cow<'t, str>>,
        hide_text: Option<Cow<'t, str>>,
        show_top: bool,
        show_bottom: bool,
    },

    /// A table of contents block.
    ///
    /// This contains links to sub-headings on the page.
    TableOfContents {
        attributes: AttributeMap<'t>,
        align: Option<Alignment>,
    },

    /// A footnote reference.
    ///
    /// This specifies that a `[[footnote]]` was here, and that a clickable
    /// link to the footnote block should be added.
    ///
    /// The index is not saved because it is part of the rendering context.
    /// It is indirectly preserved as the index of the `footnotes` list in the syntax tree.
    Footnote,

    /// A footnote block, containing all the footnotes from throughout the page.
    ///
    /// If a `[[footnoteblock]]` is not added somewhere in the content of the page,
    /// then it is automatically appended to the end of the syntax tree.
    FootnoteBlock {
        title: Option<Cow<'t, str>>,
        hide: bool,
    },

    /// A user block, linking to their information and possibly showing their avatar.
    #[serde(rename_all = "kebab-case")]
    User {
        name: Cow<'t, str>,
        show_avatar: bool,
    },

    /// Element containing colored text.
    ///
    /// The CSS designation of the color is specified, followed by the elements contained within.
    Color {
        color: Cow<'t, str>,
        elements: Vec<Element<'t>>,
    },

    /// Element containing a code block.
    Code {
        contents: Cow<'t, str>,
        language: Option<Cow<'t, str>>,
    },

    /// Element containing a sandboxed HTML block.
    Html { contents: Cow<'t, str> },

    /// Element containing an iframe component.
    Iframe {
        attributes: AttributeMap<'t>,
        url: Cow<'t, str>,
    },

    /// A newline or line break.
    ///
    /// This calls for a newline in the final output, such as `<br>` in HTML.
    LineBreak,

    /// A collection of line breaks adjacent to each other.
    LineBreaks(NonZeroU32),

    /// A "clear float" div.
    ClearFloat(ClearFloat),

    /// A horizontal rule.
    HorizontalRule,
}

impl Element<'_> {
    /// Determines if the element is "unintentional whitespace".
    ///
    /// Specifically, it returns true if the element is:
    /// * `Element::LineBreak`
    /// * `Element::Text` where the contents all have the Unicode property `White_Space`.
    ///
    /// This does not count `Element::LineBreaks` because it is produced intentionally
    /// via `[[lines]]` rather than extra whitespace in between syntactical elements.
    pub fn is_whitespace(&self) -> bool {
        match self {
            Element::LineBreak => true,
            Element::Text(string) if string.chars().all(|c| c.is_whitespace()) => true,
            _ => false,
        }
    }

    /// Returns the Rust name of this Element variant.
    pub fn name(&self) -> &'static str {
        match self {
            Element::Container(container) => container.ctype().name(),
            Element::Module(module) => module.name(),
            Element::Text(_) => "Text",
            Element::Raw(_) => "Raw",
            Element::Email(_) => "Email",
            Element::Table(_) => "Table",
            Element::TableItem(_) => "TableItem",
            Element::Anchor { .. } => "Anchor",
            Element::Link { .. } => "Link",
            Element::List { .. } => "List",
            Element::ListItem(_) => "ListItem",
            Element::Image { .. } => "Image",
            Element::RadioButton { .. } => "RadioButton",
            Element::CheckBox { .. } => "CheckBox",
            Element::Collapsible { .. } => "Collapsible",
            Element::TableOfContents { .. } => "TableOfContents",
            Element::Footnote => "Footnote",
            Element::FootnoteBlock { .. } => "FootnoteBlock",
            Element::User { .. } => "User",
            Element::Color { .. } => "Color",
            Element::Code { .. } => "Code",
            Element::Html { .. } => "HTML",
            Element::Iframe { .. } => "Iframe",
            Element::LineBreak => "LineBreak",
            Element::LineBreaks { .. } => "LineBreaks",
            Element::ClearFloat(_) => "ClearFloat",
            Element::HorizontalRule => "HorizontalRule",
        }
    }

    /// Determines if this element type is able to be embedded in a paragraph.
    ///
    /// It does *not* look into the interiors of the element, it only does a
    /// surface-level check.
    ///
    /// This is to avoid making the call very expensive, but for a complete
    /// understanding of the paragraph requirements, see the `Elements` return.
    ///
    /// See https://developer.mozilla.org/en-US/docs/Web/Guide/HTML/Content_categories#phrasing_content
    pub fn paragraph_safe(&self) -> bool {
        match self {
            Element::Container(container) => container.ctype().paragraph_safe(),
            Element::Module(_) => false,
            Element::Text(_) | Element::Raw(_) | Element::Email(_) => true,
            Element::Table(_) => false,
            Element::TableItem(_) => false,
            Element::Anchor { .. } | Element::Link { .. } => true,
            Element::List { .. } => false,
            Element::ListItem(_) => false,
            Element::Image { .. } => true,
            Element::RadioButton { .. } | Element::CheckBox { .. } => true,
            Element::Collapsible { .. } => false,
            Element::TableOfContents { .. } => false,
            Element::Footnote => true,
            Element::FootnoteBlock { .. } => false,
            Element::User { .. } => true,
            Element::Color { .. } => true,
            Element::Code { .. } => true,
            Element::Html { .. } | Element::Iframe { .. } => false,
            Element::LineBreak | Element::LineBreaks { .. } => true,
            Element::ClearFloat(_) => false,
            Element::HorizontalRule => false,
        }
    }

    /// Deep-clones the object, making it an owned version.
    ///
    /// Note that `.to_owned()` on `Cow` just copies the pointer,
    /// it doesn't make an `Cow::Owned(_)` version like its name
    /// suggests.
    pub fn to_owned(&self) -> Element<'static> {
        match self {
            Element::Container(container) => Element::Container(container.to_owned()),
            Element::Module(module) => Element::Module(module.to_owned()),
            Element::Text(text) => Element::Text(string_to_owned(text)),
            Element::Raw(text) => Element::Raw(string_to_owned(text)),
            Element::Email(email) => Element::Email(string_to_owned(email)),
            Element::Table(table) => Element::Table(table.to_owned()),
            Element::TableItem(item) => Element::TableItem(item.to_owned()),
            Element::Anchor {
                target,
                attributes,
                elements,
            } => Element::Anchor {
                target: *target,
                attributes: attributes.to_owned(),
                elements: elements_to_owned(elements),
            },
            Element::Link {
                link,
                label,
                target,
            } => Element::Link {
                link: link.to_owned(),
                label: label.to_owned(),
                target: *target,
            },
            Element::List {
                ltype,
                attributes,
                items,
            } => Element::List {
                ltype: *ltype,
                attributes: attributes.to_owned(),
                items: list_items_to_owned(items),
            },
            Element::ListItem(boxed_list_item) => {
                let list_item: &ListItem = &*boxed_list_item;

                Element::ListItem(Box::new(list_item.to_owned()))
            }
            Element::Image {
                source,
                link,
                alignment,
                attributes,
            } => Element::Image {
                source: source.to_owned(),
                link: link.ref_map(|link| link.to_owned()),
                alignment: *alignment,
                attributes: attributes.to_owned(),
            },
            Element::RadioButton {
                name,
                checked,
                attributes,
            } => Element::RadioButton {
                name: string_to_owned(name),
                checked: *checked,
                attributes: attributes.to_owned(),
            },
            Element::CheckBox {
                checked,
                attributes,
            } => Element::CheckBox {
                checked: *checked,
                attributes: attributes.to_owned(),
            },
            Element::Collapsible {
                elements,
                attributes,
                start_open,
                show_text,
                hide_text,
                show_top,
                show_bottom,
            } => Element::Collapsible {
                elements: elements_to_owned(elements),
                attributes: attributes.to_owned(),
                start_open: *start_open,
                show_text: option_string_to_owned(show_text),
                hide_text: option_string_to_owned(hide_text),
                show_top: *show_top,
                show_bottom: *show_bottom,
            },
            Element::TableOfContents { align, attributes } => Element::TableOfContents {
                align: *align,
                attributes: attributes.to_owned(),
            },
            Element::Footnote => Element::Footnote,
            Element::FootnoteBlock { title, hide } => Element::FootnoteBlock {
                title: option_string_to_owned(title),
                hide: *hide,
            },
            Element::User { name, show_avatar } => Element::User {
                name: string_to_owned(name),
                show_avatar: *show_avatar,
            },
            Element::Color { color, elements } => Element::Color {
                color: string_to_owned(color),
                elements: elements_to_owned(elements),
            },
            Element::Code { contents, language } => Element::Code {
                contents: string_to_owned(contents),
                language: option_string_to_owned(language),
            },
            Element::Html { contents } => Element::Html {
                contents: string_to_owned(contents),
            },
            Element::Iframe { url, attributes } => Element::Iframe {
                url: string_to_owned(url),
                attributes: attributes.to_owned(),
            },
            Element::LineBreak => Element::LineBreak,
            Element::LineBreaks(amount) => Element::LineBreaks(*amount),
            Element::ClearFloat(clear_float) => Element::ClearFloat(*clear_float),
            Element::HorizontalRule => Element::HorizontalRule,
        }
    }
}

#[cfg(feature = "log")]
impl slog::Value for Element<'_> {
    fn serialize(
        &self,
        _: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        serializer.emit_str(key, self.name())
    }
}

/// Wrapper for the result of producing element(s).
///
/// This has an enum instead of a simple `Vec<Element>`
/// since the most common output is a single element,
/// and it makes little sense to heap allocate for every
/// single return if we can easily avoid it.
///
/// It also contains a field marking whether all of the
/// contents are paragraph-safe or not, used by `ParagraphStack`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Elements<'t> {
    Multiple(Vec<Element<'t>>),
    Single(Element<'t>),
    None,
}

impl Elements<'_> {
    #[inline]
    pub fn is_empty(&self) -> bool {
        match self {
            Elements::Multiple(elements) => elements.is_empty(),
            Elements::Single(_) => false,
            Elements::None => true,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Elements::Multiple(elements) => elements.len(),
            Elements::Single(_) => 1,
            Elements::None => 0,
        }
    }

    pub fn paragraph_safe(&self) -> bool {
        match self {
            Elements::Multiple(elements) => {
                elements.iter().all(|element| element.paragraph_safe())
            }
            Elements::Single(element) => element.paragraph_safe(),
            Elements::None => true,
        }
    }
}

impl<'t> AsRef<[Element<'t>]> for Elements<'t> {
    fn as_ref(&self) -> &[Element<'t>] {
        match self {
            Elements::Multiple(elements) => elements,
            Elements::Single(element) => slice::from_ref(element),
            Elements::None => &[],
        }
    }
}

impl<'t> From<Element<'t>> for Elements<'t> {
    #[inline]
    fn from(element: Element<'t>) -> Elements<'t> {
        Elements::Single(element)
    }
}

impl<'t> From<Option<Element<'t>>> for Elements<'t> {
    #[inline]
    fn from(element: Option<Element<'t>>) -> Elements<'t> {
        match element {
            Some(element) => Elements::Single(element),
            None => Elements::None,
        }
    }
}

impl<'t> From<Vec<Element<'t>>> for Elements<'t> {
    #[inline]
    fn from(elements: Vec<Element<'t>>) -> Elements<'t> {
        Elements::Multiple(elements)
    }
}

impl<'t> IntoIterator for Elements<'t> {
    type Item = Element<'t>;
    type IntoIter = ElementsIterator<'t>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Elements::None => ElementsIterator::None,
            Elements::Single(element) => ElementsIterator::Single(Some(element)),
            Elements::Multiple(mut elements) => {
                // So we can just pop for each step
                elements.reverse();
                ElementsIterator::Multiple(elements)
            }
        }
    }
}

/// Iterator implementation for `Elements`.
#[derive(Debug)]
pub enum ElementsIterator<'t> {
    Multiple(Vec<Element<'t>>),
    Single(Option<Element<'t>>),
    None,
}

impl<'t> Iterator for ElementsIterator<'t> {
    type Item = Element<'t>;

    #[inline]
    fn next(&mut self) -> Option<Element<'t>> {
        match self {
            ElementsIterator::Multiple(ref mut elements) => elements.pop(),
            ElementsIterator::Single(ref mut element) => element.take(),
            ElementsIterator::None => None,
        }
    }
}

#[test]
fn elements_iter() {
    macro_rules! check {
        ($elements:expr, $expected:expr $(,)?) => {{
            let actual: Vec<Element> = $elements.into_iter().collect();
            let expected = $expected;

            assert_eq!(
                actual, expected,
                "Actual element iteration doesn't match expected"
            );
        }};
    }

    check!(Elements::None, vec![]);
    check!(Elements::Single(text!("a")), vec![text!("a")]);
    check!(
        Elements::Multiple(vec![]), //
        vec![],
    );
    check!(
        Elements::Multiple(vec![text!("a")]), //
        vec![text!("a")],
    );
    check!(
        Elements::Multiple(vec![text!("a"), text!("b")]),
        vec![text!("a"), text!("b")],
    );
    check!(
        Elements::Multiple(vec![text!("a"), text!("b"), text!("c")]),
        vec![text!("a"), text!("b"), text!("c")],
    );
}
