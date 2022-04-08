/*
 * settings/mod.rs
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

/// Settings to tweak behavior in the ftml parser and renderer.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct WikitextSettings {
    /// What mode we're running in.
    pub mode: WikitextMode,

    /// Whether page-contextual syntax is permitted.
    ///
    /// This currently refers to:
    /// * Include
    /// * Module
    /// * Table of Contents
    /// * Button
    pub enable_page_syntax: bool,

    /// Whether IDs should have true values, or be excluded or randomly generated.
    ///
    /// In the latter case, IDs can be used for navigation, for instance
    /// the table of contents, but setting this to `true` is needed in any
    /// context where more than one instance of rendered wikitext could be emitted.
    pub use_true_ids: bool,

    /// Whether local paths are permitted.
    ///
    /// This applies to:
    /// * Files
    /// * Images
    pub allow_local_paths: bool,

    /// What interwiki prefixes are supported.
    ///
    /// All instances of `$$` in the destination URL are replaced with the link provided
    /// in the interwiki link. For instance, `[wikipedia:SCP_Foundation SCP Wiki]`, then
    /// `$$` will be replaced with `SCP_Foundation`.
    ///
    /// # Notes
    ///
    /// * These are matched case-sensitively.
    /// * Prefixes may not contain colons, they are matched up to the first colon, and
    ///   any beyond that are considered part of the link.
    /// * By convention, prefixes should be all-lowercase.
    pub interwiki: InterwikiSettings,
}

impl WikitextSettings {
    pub fn from_mode(mode: WikitextMode) -> Self {
        let interwiki = DEFAULT_INTERWIKI.clone();

        match mode {
            WikitextMode::Page => WikitextSettings {
                mode,
                enable_page_syntax: true,
                use_true_ids: true,
                allow_local_paths: true,
                interwiki,
            },
            WikitextMode::Draft => WikitextSettings {
                mode,
                enable_page_syntax: true,
                use_true_ids: false,
                allow_local_paths: true,
                interwiki,
            },
            WikitextMode::ForumPost | WikitextMode::DirectMessage => WikitextSettings {
                mode,
                enable_page_syntax: false,
                use_true_ids: false,
                allow_local_paths: false,
                interwiki,
            },
            WikitextMode::List => WikitextSettings {
                mode,
                enable_page_syntax: true,
                use_true_ids: false,
                allow_local_paths: true,
                interwiki,
            },
        }
    }
}

/// What mode parsing and rendering is done in.
///
/// Each variant has slightly different behavior associated
/// with them, beyond the typical flags for the rest of `WikitextSettings`.
///
/// The exact details of each are still being decided as this is implemented.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WikitextMode {
    /// Processing for the contents of a page on a site.
    Page,

    /// Processing for a draft of a page.
    Draft,

    /// Processing for the contents of a forum post, of which there may be many.
    ForumPost,

    /// Processing for the contents of a direct message, sent to a user.
    DirectMessage,

    /// Processing for modules or other contexts such as `ListPages`.
    List,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct InterwikiSettings {
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
                let (prefix, path) = link.split_at(idx);

                // If there's an interwiki prefix, apply the template.
                self.prefixes.get(prefix).map(|template| {
                    let mut url = str!(template);

                    // Substitute all $$s in the URL templates.
                    while let Some(idx) = template.find("$$") {
                        let range = idx..idx + 2;
                        url.replace_range(range, path);
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
