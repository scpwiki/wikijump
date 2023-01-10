/*
 * url.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2023 Wikijump Team
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
use wikidot_normalize::normalize;

#[cfg(feature = "html")]
use crate::tree::LinkLocation;

pub const URL_SCHEMES: [&str; 19] = [
    "blob:",
    "chrome-extension://",
    "chrome://",
    "content://",
    "dns:",
    "feed:",
    "file://",
    "ftp://",
    "git://",
    "gopher://",
    "http://",
    "https://",
    "irc6://",
    "irc://",
    "ircs://",
    "mailto:",
    "resource://",
    "rtmp://",
    "sftp://",
];

pub fn is_url(url: &str) -> bool {
    // If it's a URL
    for scheme in &URL_SCHEMES {
        if url.starts_with(scheme) {
            return true;
        }
    }

    false
}

/// Returns true if the scheme for this URL is `javascript:` or `data:`.
///
/// Works case-insensitively (for ASCII).
pub fn dangerous_scheme(url: &str) -> bool {
    url.split_once(':')
        .map(|(scheme, _)| {
            scheme.eq_ignore_ascii_case("javascript")
                || scheme.eq_ignore_ascii_case("data")
        })
        .unwrap_or(false)
}

#[cfg(feature = "html")]
pub fn normalize_link<'a>(
    link: &'a LinkLocation<'a>,
    helper: &dyn BuildSiteUrl,
) -> Cow<'a, str> {
    match link {
        LinkLocation::Url(url) => normalize_href(url),
        LinkLocation::Page(page_ref) => {
            let (site, page) = page_ref.fields();

            match site {
                Some(site) => Cow::Owned(helper.build_url(site, page)),
                None => normalize_href(page),
            }
        }
    }
}

pub fn normalize_href(url: &str) -> Cow<str> {
    if is_url(url) || url.starts_with('#') || url == "javascript:;" {
        Cow::Borrowed(url)
    } else if dangerous_scheme(url) {
        warn!("Attempt to pass in dangerous URL: {url}");
        Cow::Borrowed("#invalid-url")
    } else {
        let mut url = str!(url);
        normalize(&mut url);
        url.insert(0, '/');
        Cow::Owned(url)
    }
}

pub trait BuildSiteUrl {
    fn build_url(&self, site: &str, path: &str) -> String;
}

#[test]
fn detect_dangerous_schemes() {
    macro_rules! check {
        ($input:expr, $result:expr $(,)?) => {
            assert_eq!(
                dangerous_scheme($input),
                $result,
                "For input {:?}, dangerous scheme detection failed",
                $input,
            )
        };
    }

    check!("http://example.com/", false);
    check!("https://example.com/", false);
    check!("irc://irc.scpwiki.com", false);
    check!("javascript:alert(1)", true);
    check!("JAVASCRIPT:alert(1)", true);
    check!("JaVaScRiPt:alert(document.cookie)", true);
    check!("data:text/plain;base64,SGVsbG8sIFdvcmxkIQ==", true);
    check!("data:text/javascript,alert(1)", true);
    check!("data:text/html,<script>alert('XSS');</script>", true);
    check!("DATA:text/html,<script>alert('XSS');</script>", true);
}
