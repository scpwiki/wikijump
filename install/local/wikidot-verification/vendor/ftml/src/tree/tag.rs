/*
 * tree/tag.rs
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlTag {
    Tag(&'static str),
    TagAndClass {
        tag: &'static str,
        class: &'static str,
    },
    TagAndStyle {
        tag: &'static str,
        style: &'static str,
    },
    TagAndId {
        tag: &'static str,
        id: String,
    },
}

impl HtmlTag {
    #[inline]
    pub fn new(tag: &'static str) -> HtmlTag {
        HtmlTag::Tag(tag)
    }

    #[inline]
    pub fn with_class(tag: &'static str, class: &'static str) -> HtmlTag {
        HtmlTag::TagAndClass { tag, class }
    }

    #[inline]
    pub fn with_style(tag: &'static str, style: &'static str) -> HtmlTag {
        HtmlTag::TagAndStyle { tag, style }
    }

    #[inline]
    pub fn with_id(tag: &'static str, id: String) -> HtmlTag {
        HtmlTag::TagAndId { tag, id }
    }

    #[inline]
    pub fn tag(&self) -> &'static str {
        match self {
            HtmlTag::Tag(tag) => tag,
            HtmlTag::TagAndClass { tag, .. } => tag,
            HtmlTag::TagAndStyle { tag, .. } => tag,
            HtmlTag::TagAndId { tag, .. } => tag,
        }
    }
}
