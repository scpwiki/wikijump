/*
 * parsing/rule/impls/block/blocks/embed.rs
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

use super::prelude::*;
use std::collections::HashMap;
use unicase::UniCase;

pub const BLOCK_EMBED: BlockRule = BlockRule {
    name: "block-embed",
    accepts_names: &["embed"],
    accepts_star: false,
    accepts_score: false,
    accepts_newlines: true,
    parse_fn,
};

fn parse_fn<'r, 't>(
    log: &Logger,
    parser: &mut Parser<'r, 't>,
    name: &'t str,
    flag_star: bool,
    flag_score: bool,
    in_head: bool,
) -> ParseResult<'r, 't, Elements<'t>> {
    info!(
        log,
        "Parsing embed block";
        "in-head" => in_head,
        "name" => name,
    );

    assert!(!flag_star, "Embed doesn't allow star flag");
    assert!(!flag_score, "Embed doesn't allow star flag");
    assert_block_name(&BLOCK_EMBED, name);

    todo!()
}

// Embed sources

lazy_static! {
    static ref EMBED_SOURCES: HashMap<UniCase<&'static str>, EmbedSource<'static>> = {
        hashmap! {
            UniCase::ascii("youtube") => EmbedSource {
                required: &["video"],
                optional: &[
                    ("width", "1280"),
                    ("height", "720"),
                    ("title", "YouTube video player"),
                ],
                template: r#"<iframe width="%%width%%" height="%%height%%" title="%%title%%" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>"#,
            }
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EmbedSource<'a> {
    required: &'a [&'a str],
    optional: &'a [(&'a str, &'a str)],
    template: &'a str,
}

impl<'a> EmbedSource<'a> {
    pub fn build(&self, arguments: &mut Arguments<'a>) -> Option<String> {
        let mut template = String::from(self.template);
        let mut buffer = String::new();

        // Substitute required variables
        for &name in self.required {
            let value = match arguments.get(name) {
                Some(value) => value,
                None => return None,
            };

            Self::substitute(&mut template, &mut buffer, name, &value);
        }

        // Substitute optional variables
        for &(name, default) in self.optional {
            let arg = arguments.get(name);
            let value = match arg {
                Some(ref value) => value.as_ref(),
                None => default,
            };

            Self::substitute(&mut template, &mut buffer, name, value);
        }

        // Return result
        Some(template)
    }

    fn substitute(template: &mut String, buffer: &mut String, name: &str, value: &str) {
        buffer.clear();
        str_write!(buffer, "%%{}%%", name);

        while let Some(start) = template.find(buffer.as_str()) {
            let end = start + buffer.len();

            template.replace_range(start..end, value);
        }
    }
}

#[test]
fn embed_source() {
    const SOURCE: EmbedSource = EmbedSource {
        required: &["apple"],
        optional: &[("banana", "10"), ("cherry", "20")],
        template: "Fruit: %%apple%% (%%banana%%, %%cherry%%)",
    };

    macro_rules! check {
        ($arguments:expr $(,)?) => {
            check!($arguments => None)
        };

        ($arguments:expr, $expected:expr $(,)?) => {
            check!($arguments => Some(str!($expected)))
        };

        ($arguments:expr => $expected:expr) => {{
            let arguments_raw: &[(&str, &str)] = &$arguments;

            let mut arguments = Arguments::new();

            for (key, value) in arguments_raw {
                arguments.insert(key, cow!(value));
            }

            let actual = SOURCE.build(&mut arguments);
            assert_eq!(actual, $expected, "Actual embedded value doesn't match expected");
        }};
    }

    check!([]);
    check!([("banana", "foo")]);

    check!(
        [("apple", "foo")], //
        "Fruit: foo (10, 20)",
    );

    check!(
        [("apple", "bar"), ("cherry", "xyz")], //
        "Fruit: bar (10, xyz)",
    );

    check!(
        [("cherry", "3"), ("banana", "2"), ("apple", "1")], //
        "Fruit: 1 (2, 3)",
    );
}
