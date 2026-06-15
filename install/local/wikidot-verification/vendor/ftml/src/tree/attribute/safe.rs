/*
 * tree/attribute/safe.rs
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
use std::collections::HashSet;
use std::sync::LazyLock;
use unicase::UniCase;

macro_rules! hashset_unicase {
    () => {
        hashset![]
    };

    ($($x:expr),+ $(,)?) => {
        hashset![
            $(
                UniCase::ascii($x)
            ),+
        ]
    };
}

/// List of safe attributes. All others will be filtered out.
///
/// See <https://scuttle.atlassian.net/wiki/spaces/WD/pages/1030782977/Allowed+Attributes+in+Wikitext>
pub static SAFE_ATTRIBUTES: LazyLock<HashSet<UniCase<&'static str>>> =
    LazyLock::new(|| {
        hashset_unicase![
            "accept",
            "align",
            "alt",
            "autocapitalize",
            "autoplay",
            "background",
            "bgcolor",
            "border",
            "buffered",
            "checked",
            "cite",
            "class",
            "cols",
            "colspan",
            "contenteditable",
            "controls",
            "coords",
            "datetime",
            "decoding",
            "default",
            "dir",
            "dirname",
            "disabled",
            "download",
            "draggable",
            "for",
            "form",
            "frameborder",
            "headers",
            "height",
            "hidden",
            "high",
            "href",
            "hreflang",
            "id",
            "inputmode",
            "ismap",
            "itemprop",
            "kind",
            "label",
            "lang",
            "list",
            "loop",
            "low",
            "max",
            "maxlength",
            "min",
            "minlength",
            "multiple",
            "muted",
            "name",
            "optimum",
            "pattern",
            "placeholder",
            "poster",
            "preload",
            "readonly",
            "required",
            "reversed",
            "role",
            "rows",
            "rowspan",
            "scope",
            "selected",
            "shape",
            "size",
            "sizes",
            "span",
            "spellcheck",
            "src",
            "srclang",
            "srcset",
            "start",
            "step",
            "style",
            "tabindex",
            "target",
            "title",
            "translate",
            "type",
            "usemap",
            "value",
            "width",
            "wrap",
        ]
    });

/// List of all HTML5 attributes with special boolean behavior.
///
/// That is, you set them to "true" by having the attribute present without
/// a value, and "false" by excluding them.
///
/// A notable example is "checked" for `<input>`:
/// * `<input type="checkbox" checked>` means the checkbox starts checked
/// * `<input type="checkbox">` means it starts unchecked
///
/// This list includes all such attributes, even if they are not part of
/// `SAFE_ATTRIBUTES`.
pub static BOOLEAN_ATTRIBUTES: LazyLock<HashSet<UniCase<&'static str>>> =
    LazyLock::new(|| {
        hashset_unicase![
            "allowfullscreen",
            "allowpaymentrequest",
            "async",
            "autofocus",
            "autoplay",
            "checked",
            "controls",
            "default",
            "disabled",
            "formnovalidate",
            "hidden",
            "ismap",
            "itemscope",
            "loop",
            "multiple",
            "muted",
            "nomodule",
            "novalidate",
            "open",
            "playsinline",
            "readonly",
            "required",
            "reversed",
            "selected",
            "truespeed",
        ]
    });

/// List of HTML attributes which need to be checked for XSS.
///
/// For instance, you could have `href="javascript:doSomething()"`,
/// which would escape normal sandboxing and run trusted javascript
/// from untrusted user data.
///
/// ## See also
/// * `detect_dangerous_schemes()`
/// * `normalize_href()`
pub static URL_ATTRIBUTES: LazyLock<HashSet<UniCase<&'static str>>> =
    LazyLock::new(|| hashset_unicase!["href", "src"]);

static ATTRIBUTE_SUFFIX_SAFE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[a-zA-z0-9\-]+").unwrap());

pub const SAFE_ATTRIBUTE_PREFIXES: [&str; 2] = ["aria-", "data-"];

pub fn is_safe_attribute(attribute: UniCase<&str>) -> bool {
    if SAFE_ATTRIBUTES.contains(&attribute) {
        return true;
    }

    for prefix in &SAFE_ATTRIBUTE_PREFIXES {
        if attribute.starts_with(prefix) && ATTRIBUTE_SUFFIX_SAFE.is_match(&attribute) {
            return true;
        }
    }

    false
}
