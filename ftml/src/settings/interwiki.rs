/*
 * settings/interwiki.rs
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

use std::borrow::Cow;
use std::collections::HashMap;

lazy_static! {
    pub static ref EMPTY_INTERWIKI: InterwikiSettings = {
        InterwikiSettings {
            prefixes: hashmap! {},
        }
    };
    pub static ref DEFAULT_INTERWIKI: InterwikiSettings = {
        InterwikiSettings {
            prefixes: hashmap! {
                cow!("wikipedia") => cow!("https://wikipedia.org/wiki/$$"),
                cow!("wp") => cow!("https://wikipedia.org/wiki/$$"),
                cow!("commons") => cow!("https://commons.wikimedia.org/wiki/$$"),
                cow!("google") => cow!("https://google.com/search?q=$$"),
                cow!("duckduckgo") => cow!("https://duckduckgo.com/?q=$$"),
                cow!("ddg") => cow!("https://duckduckgo.com/?q=$$"),
                cow!("dictionary") => cow!("https://dictionary.com/browse/$$"),
                cow!("thesaurus") => cow!("https://thesaurus.com/browse/$$"),
            },
        }
    };
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct InterwikiSettings {
    #[serde(flatten)]
    pub prefixes: HashMap<Cow<'static, str>, Cow<'static, str>>,
}

impl InterwikiSettings {
    #[inline]
    pub fn new() -> Self {
        InterwikiSettings::default()
    }

    pub fn build(&self, link: &str) -> Option<String> {
        match link.find(':') {
            // Starting with a colon is not interwiki, skip.
            // Or, if no colon, no interwiki.
            Some(0) | None => None,

            // Split at first colon, any further are treated as part of the link contents.
            Some(idx) => {
                let (prefix, rest) = link.split_at(idx);
                let path = &rest[1..]; // Safe because we're splitting on ':', an ASCII character.

                // Special handling, if it's empty then fail
                if path.is_empty() {
                    return None;
                }

                // If there's an interwiki prefix, apply the template.
                self.prefixes.get(prefix).map(|template| {
                    // Substitute all $$s in the URL templates.
                    let mut url = template.replace("$$", path);

                    // Substitute all spaces into url-encoded form.
                    while let Some(idx) = url.find(' ') {
                        url.replace_range(idx..idx + 1, "%20");
                    }

                    url
                })
            }
        }
    }
}

#[test]
fn interwiki_prefixes() {
    use ref_map::*;

    macro_rules! check {
        ($link:expr, $expected:expr $(,)?) => {{
            let actual = DEFAULT_INTERWIKI.build($link);
            let expected = $expected;

            assert_eq!(
                actual.ref_map(|s| s.as_str()),
                expected,
                "Actual interwiki result doesn't match expected",
            );
        }};
    }

    check!("my-link", None);
    check!(
        "wikipedia:Mallard",
        Some("https://wikipedia.org/wiki/Mallard"),
    );
    check!(
        "wikipedia:SCP_Foundation",
        Some("https://wikipedia.org/wiki/SCP_Foundation"),
    );
    check!(
        "wikipedia:Special:RecentChanges",
        Some("https://wikipedia.org/wiki/Special:RecentChanges"),
    );
    check!(
        "wp:SCP_Foundation",
        Some("https://wikipedia.org/wiki/SCP_Foundation"),
    );
    check!(
        "wp:it:SCP_Foundation",
        Some("https://wikipedia.org/wiki/it:SCP_Foundation"),
    );
    check!(
        "commons:File:SCP-682.jpg",
        Some("https://commons.wikimedia.org/wiki/File:SCP-682.jpg"),
    );
    check!(
        "commons:Category:SCP_Foundation",
        Some("https://commons.wikimedia.org/wiki/Category:SCP_Foundation"),
    );
    check!(
        "google:what's+my+ip",
        Some("https://google.com/search?q=what's+my+ip"),
    );
    check!(
        "duckduckgo:what's+my+ip",
        Some("https://duckduckgo.com/?q=what's+my+ip"),
    );
    check!(
        "ddg:what's+my+ip",
        Some("https://duckduckgo.com/?q=what's+my+ip"),
    );
    check!("dictionary:oak", Some("https://dictionary.com/browse/oak"));
    check!("thesaurus:oak", Some("https://thesaurus.com/browse/oak"));
    check!("banana:fruit-salad", None);
    check!(":empty", None);
}
