/*
 * layout.rs
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

use std::str::FromStr;

/// Describes the desired (HTML) DOM layout to be emitted.
///
/// This is used as a transition mechanism between our dependencies on the pecularities
/// of old, legacy Wikidot HTML generation and a newer better system we are calling the
/// "Wikijump" layout.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Layout {
    Wikidot,
    Wikijump,
}

impl Layout {
    #[inline]
    pub fn value(self) -> &'static str {
        match self {
            Layout::Wikidot => "wikidot",
            Layout::Wikijump => "wikijump",
        }
    }

    #[inline]
    pub fn legacy(self) -> bool {
        match self {
            Layout::Wikidot => true,
            Layout::Wikijump => false,
        }
    }

    #[inline]
    pub fn description(self) -> &'static str {
        match self {
            Layout::Wikidot => "Wikidot (legacy)",
            Layout::Wikijump => "Wikijump",
        }
    }
}

impl FromStr for Layout {
    type Err = LayoutParseError;

    fn from_str(s: &str) -> Result<Self, LayoutParseError> {
        if s.eq_ignore_ascii_case("wikidot") {
            Ok(Layout::Wikidot)
        } else if s.eq_ignore_ascii_case("wikijump") {
            Ok(Layout::Wikijump)
        } else {
            Err(LayoutParseError)
        }
    }
}

#[derive(Debug)]
pub struct LayoutParseError;

#[test]
fn test_parse() {
    macro_rules! test_ok {
        ($input:expr, $expected:ident $(,)?) => {{
            let actual: Layout = $input.parse().expect("Invalid layout string");
            let expected = Layout::$expected;

            assert_eq!(
                actual, expected,
                "Actual layout enum doesn't match expected",
            );
        }};
    }

    macro_rules! test_err {
        ($input:expr $(,)?) => {{
            let result: Result<Layout, LayoutParseError> = $input.parse();
            result.expect_err("Unexpected valid layout string");
        }};
    }

    test_ok!("wikidot", Wikidot);
    test_ok!("Wikidot", Wikidot);
    test_ok!("WIKIDOT", Wikidot);

    test_ok!("wikijump", Wikijump);
    test_ok!("Wikijump", Wikijump);
    test_ok!("WIKIJUMP", Wikijump);

    test_err!("invalid");
    test_err!("XXX");
    test_err!("foobar");
}

#[test]
fn test_values() {
    macro_rules! test {
        ($variant:ident, $value:expr, $legacy:expr, $description:expr $(,)?) => {{
            let layout = Layout::$variant;
            assert_eq!(layout.value(), $value);
            assert_eq!(layout.legacy(), $legacy);
            assert_eq!(layout.description(), $description);
        }};
    }

    test!(Wikidot, "wikidot", true, "Wikidot (legacy)");
    test!(Wikijump, "wikijump", false, "Wikijump");
}
