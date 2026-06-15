/*
 * settings/interwiki.rs
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

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::LazyLock;

/// An [`InterwikiSettings`] instance that has no prefixes.
pub static EMPTY_INTERWIKI: LazyLock<InterwikiSettings> =
    LazyLock::new(|| InterwikiSettings {
        prefixes: hashmap! {},
    });

#[allow(rustdoc::bare_urls)]
/// An [`InterwikiSettings`] instance that has the default prefixes.
///
/// These prefixes are:
/// - `wikipedia:path` => `https://wikipedia.org/wiki/path`
/// - `wp:path` => `https://wikipedia.org/wiki/path`
/// - `commons:path` => `https://commons.wikimedia.org/wiki/path`
/// - `google:path` => `https://google.com/search?q=path`
/// - `duckduckgo:path` => `https://duckduckgo.com/?q=path`
/// - `ddg:path` => `https://duckduckgo.com/?q=path`
/// - `dictionary:path` => `https://dictionary.com/browse/path`
/// - `thesaurus:path` => `https://thesaurus.com/browse/path`
pub static DEFAULT_INTERWIKI: LazyLock<InterwikiSettings> =
    LazyLock::new(|| InterwikiSettings {
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
    });

/// Settings that determine how to turn [`interwiki links`](http://org.wikidot.com/doc:wiki-syntax#toc21)
/// into full URLs.
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct InterwikiSettings {
    #[serde(flatten)]
    /// A map from each interwiki prefix to the interwiki URL. A '$$' in the URL indicates where the path specified in
    /// the Wikijump interwiki block should go.
    pub prefixes: HashMap<Cow<'static, str>, Cow<'static, str>>,
}

impl InterwikiSettings {
    /// Creates a new instance with no prefixes.
    #[inline]
    pub fn new() -> Self {
        InterwikiSettings::default()
    }

    /// Creates a full URL from an interwiki link.
    /// # Example
    /// ```
    /// # use ftml::settings::*;
    /// assert_eq!(DEFAULT_INTERWIKI.build("wikipedia:Mallard").unwrap(), "https://wikipedia.org/wiki/Mallard");
    /// ```
    ///
    /// Returns None if:
    /// - The link starts with a colon
    /// - There is no colon in the link
    /// - There is nothing after the colon
    /// - The interwiki prefix is not found
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

    macro_rules! test {
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

    test!("my-link", None);
    test!(
        "wikipedia:Mallard",
        Some("https://wikipedia.org/wiki/Mallard"),
    );
    test!(
        "wikipedia:SCP_Foundation",
        Some("https://wikipedia.org/wiki/SCP_Foundation"),
    );
    test!(
        "wikipedia:Special:RecentChanges",
        Some("https://wikipedia.org/wiki/Special:RecentChanges"),
    );
    test!(
        "wp:SCP_Foundation",
        Some("https://wikipedia.org/wiki/SCP_Foundation"),
    );
    test!(
        "wp:it:SCP_Foundation",
        Some("https://wikipedia.org/wiki/it:SCP_Foundation"),
    );
    test!(
        "commons:File:SCP-682.jpg",
        Some("https://commons.wikimedia.org/wiki/File:SCP-682.jpg"),
    );
    test!(
        "commons:Category:SCP_Foundation",
        Some("https://commons.wikimedia.org/wiki/Category:SCP_Foundation"),
    );
    test!(
        "google:what's+my+ip",
        Some("https://google.com/search?q=what's+my+ip"),
    );
    test!(
        "duckduckgo:what's+my+ip",
        Some("https://duckduckgo.com/?q=what's+my+ip"),
    );
    test!(
        "ddg:what's+my+ip",
        Some("https://duckduckgo.com/?q=what's+my+ip"),
    );
    test!("dictionary:oak", Some("https://dictionary.com/browse/oak"));
    test!("thesaurus:oak", Some("https://thesaurus.com/browse/oak"));
    test!("banana:fruit-salad", None);
    test!(":empty", None);
    test!("no-link:", None);
}
