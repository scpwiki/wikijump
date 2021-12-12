/*
 * tree/align.rs
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

    pub fn html_class(self) -> &'static str {
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
        lazy_static! {
            static ref IMAGE_ALIGNMENT_REGEX: Regex =
                Regex::new(r"^[fF]?([<=>])").unwrap();
        }

        IMAGE_ALIGNMENT_REGEX
            .find(name)
            .and_then(|mtch| FloatAlignment::try_from(mtch.as_str()).ok())
    }

    pub fn html_class(self) -> &'static str {
        match (self.align, self.float) {
            (align, false) => align.html_class(),
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
    macro_rules! check {
        ($input:expr) => {
            check!($input => None)
        };

        ($input:expr, $align:expr, $float:expr) => {
            check!($input => Some(FloatAlignment {
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

    check!("");
    check!("image");

    check!("=image", Alignment::Center, false);
    check!(">image", Alignment::Right, false);
    check!("<image", Alignment::Left, false);
    check!("f>image", Alignment::Right, true);
    check!("f<image", Alignment::Left, true);

    check!("=IMAGE", Alignment::Center, false);
    check!(">IMAGE", Alignment::Right, false);
    check!("<IMAGE", Alignment::Left, false);
    check!("f>IMAGE", Alignment::Right, true);
    check!("f<IMAGE", Alignment::Left, true);

    check!("F>IMAGE", Alignment::Right, true);
    check!("F<IMAGE", Alignment::Left, true);
}
