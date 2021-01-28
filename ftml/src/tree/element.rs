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

use super::AttributeMap;
use super::{Container, Module, StyledContainer};
use crate::enums::{AnchorTarget, LinkLabel};
use std::borrow::Cow;
use std::num::NonZeroU32;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", tag = "element", content = "data")]
pub enum Element<'t> {
    /// Generic element that contains other elements within it.
    ///
    /// Examples would include italics, paragraphs, etc.
    Container(Container<'t>),

    /// Generic element that contains other elements, with added styling.
    ///
    /// Examples would include divs, spans, etc.
    StyledContainer(StyledContainer<'t>),

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

    /// An element representing an arbitrary anchor.
    ///
    /// This is distinct from link in that it maps to HTML `<a>`,
    /// and does not necessarily mean a link to some other URL.
    Anchor {
        elements: Vec<Element<'t>>,
        attributes: AttributeMap<'t>,
        target: AnchorTarget,
    },

    /// An element linking to a different page.
    ///
    /// The "label" field is an optional field denoting what the link should
    /// display. If `None`, use the link's value itself, that is, `label.unwrap_or(url)`.
    ///
    /// The "url" field is either a page name (relative URL) or full URL.
    Link {
        url: Cow<'t, str>,
        label: LinkLabel<'t>,
        target: AnchorTarget,
    },

    /// A radio button.
    ///
    /// The "name" field translates to HTML, but is standard for grouping them.
    RadioButton {
        name: Cow<'t, str>,
        checked: bool,
        attributes: AttributeMap<'t>,
    },

    /// A checkbox.
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
        url: Cow<'t, str>,
        attributes: AttributeMap<'t>,
    },

    /// A newline or line break.
    ///
    /// This calls for a newline in the final output, such as `<br>` in HTML.
    LineBreak,

    /// A collection of line breaks adjacent to each other.
    LineBreaks(NonZeroU32),

    /// A horizontal rule.
    HorizontalRule,

    /// A null element.
    ///
    /// The element equivalent of a no-op instruction. No action should be taken,
    /// and it should be skipped over.
    Null,
}

impl Element<'_> {
    pub fn name(&self) -> &'static str {
        match self {
            Element::Container(container) => container.ctype().name(),
            Element::StyledContainer(container) => container.ctype().name(),
            Element::Module(module) => module.name(),
            Element::Text(_) => "Text",
            Element::Raw(_) => "Raw",
            Element::Email(_) => "Email",
            Element::Anchor { .. } => "Anchor",
            Element::Link { .. } => "Link",
            Element::RadioButton { .. } => "RadioButton",
            Element::CheckBox { .. } => "CheckBox",
            Element::Collapsible { .. } => "Collapsible",
            Element::Color { .. } => "Color",
            Element::Code { .. } => "Code",
            Element::Html { .. } => "HTML",
            Element::Iframe { .. } => "Iframe",
            Element::LineBreak => "LineBreak",
            Element::LineBreaks { .. } => "LineBreaks",
            Element::HorizontalRule => "HorizontalRule",
            Element::Null => "Null",
        }
    }
}

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
