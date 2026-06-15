/*
 * tree/align.rs
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

use regex::Regex;
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Alignment {
    Left,
    Right,
    Center,
    Justify,
}

impl Alignment {
    pub fn name(self) -> &'static str {
        match self {
            Alignment::Left => "left",
            Alignment::Right => "right",
            Alignment::Center => "center",
            Alignment::Justify => "justify",
        }
    }

    pub fn wd_html_style(self) -> &'static str {
        match self {
            Alignment::Left => "text-align: left;",
            Alignment::Right => "text-align: right;",
            Alignment::Center => "text-align: center;",
            Alignment::Justify => "text-align: justify;",
        }
    }

    pub fn wj_html_class(self) -> &'static str {
        match self {
            Alignment::Left => "wj-align-left",
            Alignment::Right => "wj-align-right",
            Alignment::Center => "wj-align-center",
            Alignment::Justify => "wj-align-justify",
        }
    }
}

impl TryFrom<&'_ str> for Alignment {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "<" => Ok(Alignment::Left),
            ">" => Ok(Alignment::Right),
            "=" => Ok(Alignment::Center),
            "==" => Ok(Alignment::Justify),
            _ => Err(()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct FloatAlignment {
    pub align: Alignment,
    pub float: bool,
}

impl FloatAlignment {
    pub fn parse(name: &str) -> Option<Self> {
        use std::sync::LazyLock;

        static IMAGE_ALIGNMENT_REGEX: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^[fF]?([<=>])").unwrap());

        IMAGE_ALIGNMENT_REGEX
            .find(name)
            .and_then(|mtch| FloatAlignment::try_from(mtch.as_str()).ok())
    }

    pub fn wd_html_class(self) -> &'static str {
        match (self.align, self.float) {
            (Alignment::Left, false) => "alignleft",
            (Alignment::Right, false) => "alignright",
            (Alignment::Center, false) => "aligncenter",
            (Alignment::Left, true) => "floatleft",
            (Alignment::Right, true) => "floatright",
            (Alignment::Center, true) => "floatcenter",
            (Alignment::Justify, _) => {
                // When this case is reached, it means that some element
                // permits justify alignment even though there should not
                // be any argument settings which enable this.
                //
                // For instance, see FloatAlignment::try_from(&str).
                //
                // There is no CSS class in Wikidot for this alignment, so
                // with both of these factors combined, we should panic.
                panic!("Attempted to return HTML class for Wikidot justify alignment");
            }
        }
    }

    pub fn wj_html_class(self) -> &'static str {
        match (self.align, self.float) {
            (align, false) => align.wj_html_class(),
            (Alignment::Left, true) => "wj-float-left",
            (Alignment::Center, true) => "wj-float-center",
            (Alignment::Right, true) => "wj-float-right",
            (Alignment::Justify, true) => "wj-float-justify",
        }
    }
}

impl TryFrom<&'_ str> for FloatAlignment {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (align, float) = match value {
            "=" => (Alignment::Center, false),
            "<" => (Alignment::Left, false),
            ">" => (Alignment::Right, false),
            "f<" | "F<" => (Alignment::Left, true),
            "f>" | "F>" => (Alignment::Right, true),
            _ => return Err(()),
        };

        Ok(FloatAlignment { align, float })
    }
}

#[test]
fn image_alignment() {
    macro_rules! test {
        ($input:expr) => {
            test!($input => None)
        };

        ($input:expr, $align:expr, $float:expr) => {
            test!($input => Some(FloatAlignment {
                align: $align,
                float: $float,
            }))
        };

        ($input:expr => $expected:expr) => {{
            let actual = FloatAlignment::parse($input);
            let expected = $expected;

            assert_eq!(
                actual, expected,
                "Actual image alignment result does not match expected",
            );
        }};
    }

    test!("");
    test!("image");

    test!("=image", Alignment::Center, false);
    test!(">image", Alignment::Right, false);
    test!("<image", Alignment::Left, false);
    test!("f>image", Alignment::Right, true);
    test!("f<image", Alignment::Left, true);

    test!("=IMAGE", Alignment::Center, false);
    test!(">IMAGE", Alignment::Right, false);
    test!("<IMAGE", Alignment::Left, false);
    test!("f>IMAGE", Alignment::Right, true);
    test!("f<IMAGE", Alignment::Left, true);

    test!("F>IMAGE", Alignment::Right, true);
    test!("F<IMAGE", Alignment::Left, true);
}
