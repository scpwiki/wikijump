/*
 * license.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
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

//! Constant data for licenses usable by Wikijump sites.

use crate::error::prelude::*;
use crate::locales::Localizations;
use fluent::FluentArgs;
use serde::Serialize;
use std::str::FromStr;
use unic_langid::LanguageIdentifier;

pub use crate::types::License;

pub const WIKIDOT_OTHER_LICENSE: &str = "other";
pub const WIKIDOT_COPYRIGHT_LICENSE: &str = "copyright";
pub const WIKIDOT_CUSTOM_LICENSE_MAX_CHARS: usize = 300;

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum WikidotLicense {
    Standard(License),
    Other(String),
    Copyright,
}

impl WikidotLicense {
    pub fn from_storage(license: &str, license_other: Option<&str>) -> Result<Self> {
        match license {
            WIKIDOT_OTHER_LICENSE => {
                let source = license_other.ok_or_raise(|| {
                    Error::new(
                        "custom Wikidot license is missing its description",
                        ErrorType::License,
                    )
                })?;
                Ok(Self::Other(str!(source)))
            }
            WIKIDOT_COPYRIGHT_LICENSE => Ok(Self::Copyright),
            value => Ok(License::from_str(value).map(Self::Standard).map_err(|_| {
                Error::new(
                    format!("unknown Wikidot license mode {value:?}"),
                    ErrorType::License,
                )
            })?),
        }
    }

    pub fn into_storage(self) -> (String, Option<String>) {
        match self {
            Self::Standard(license) => (license.to_string(), None),
            Self::Other(html) => (str!(WIKIDOT_OTHER_LICENSE), Some(html)),
            Self::Copyright => (str!(WIKIDOT_COPYRIGHT_LICENSE), None),
        }
    }

    pub fn render_other(&self, year: i32) -> Option<String> {
        match self {
            Self::Other(html) => Some(html.replace("%%year%%", &year.to_string())),
            _ => None,
        }
    }
}

pub fn validate_wikidot_license_override(
    license: Option<&str>,
    license_other: Option<&str>,
) -> Result<(Option<String>, Option<String>)> {
    let Some(license) = license else {
        ensure!(
            license_other.is_none(),
            Error::new(
                "an inherited license cannot carry a custom description",
                ErrorType::License,
            ),
        );
        return Ok((None, None));
    };

    let value = match license {
        WIKIDOT_OTHER_LICENSE => WikidotLicense::Other(sanitize_wikidot_custom_license(
            license_other.ok_or_raise(|| {
                Error::new(
                    "custom Wikidot license is missing its description",
                    ErrorType::License,
                )
            })?,
        )?),
        value => WikidotLicense::from_storage(value, None)?,
    };
    Ok(value.into_storage()).map(|(license, other)| (Some(license), other))
}

pub fn sanitize_wikidot_custom_license(source: &str) -> Result<String> {
    ensure!(
        source.encode_utf16().count() <= WIKIDOT_CUSTOM_LICENSE_MAX_CHARS,
        Error::new(
            format!(
                "custom Wikidot license exceeds {WIKIDOT_CUSTOM_LICENSE_MAX_CHARS} characters"
            ),
            ErrorType::License,
        ),
    );

    let mut output = String::with_capacity(source.len());
    let mut stack = Vec::new();
    let mut rest = source;
    while let Some(position) = rest.find('<') {
        escape_license_text(&rest[..position], &mut output);
        rest = &rest[position..];

        if let Some(next) = copy_simple_license_tag(rest, &mut output, &mut stack)? {
            rest = next;
            continue;
        }
        if let Some(next) = copy_anchor_tag(rest, &mut output, &mut stack)? {
            rest = next;
            continue;
        }
        if let Some(next) = copy_image_tag(rest, &mut output)? {
            rest = next;
            continue;
        }

        bail!(Error::new(
            "custom Wikidot license contains unsupported markup",
            ErrorType::License,
        ));
    }
    escape_license_text(rest, &mut output);
    ensure!(
        stack.is_empty(),
        Error::new(
            "custom Wikidot license contains unbalanced markup",
            ErrorType::License,
        ),
    );
    Ok(output)
}

fn copy_simple_license_tag<'a>(
    input: &'a str,
    output: &mut String,
    stack: &mut Vec<&'static str>,
) -> Result<Option<&'a str>> {
    for (tag, open, close) in
        [("strong", "<strong>", "</strong>"), ("em", "<em>", "</em>")]
    {
        if let Some(rest) = input.strip_prefix(open) {
            output.push_str(open);
            stack.push(tag);
            return Ok(Some(rest));
        }
        if let Some(rest) = input.strip_prefix(close) {
            ensure!(
                stack.pop() == Some(tag),
                Error::new(
                    "custom Wikidot license contains unbalanced markup",
                    ErrorType::License,
                ),
            );
            output.push_str(close);
            return Ok(Some(rest));
        }
    }
    for tag in ["<br>", "<br/>", "<br />"] {
        if let Some(rest) = input.strip_prefix(tag) {
            output.push_str("<br />");
            return Ok(Some(rest));
        }
    }
    Ok(None)
}

fn copy_anchor_tag<'a>(
    input: &'a str,
    output: &mut String,
    stack: &mut Vec<&'static str>,
) -> Result<Option<&'a str>> {
    if let Some(rest) = input.strip_prefix("</a>") {
        ensure!(
            stack.pop() == Some("a"),
            Error::new(
                "custom Wikidot license contains unbalanced markup",
                ErrorType::License,
            ),
        );
        output.push_str("</a>");
        return Ok(Some(rest));
    }
    let Some(rest) = input.strip_prefix("<a href=\"") else {
        return Ok(None);
    };
    let Some(end) = rest.find("\">") else {
        bail!(Error::new(
            "custom Wikidot license contains an invalid link",
            ErrorType::License,
        ));
    };
    let href = &rest[..end];
    ensure_safe_license_url(href, false)?;
    output.push_str("<a href=\"");
    escape_license_attribute(href, output);
    output.push_str("\">");
    stack.push("a");
    Ok(Some(&rest[end + 2..]))
}

fn copy_image_tag<'a>(input: &'a str, output: &mut String) -> Result<Option<&'a str>> {
    let Some(rest) = input.strip_prefix("<img src=\"") else {
        return Ok(None);
    };
    let Some(src_end) = rest.find("\" alt=\"") else {
        bail!(Error::new(
            "custom Wikidot license contains an invalid image",
            ErrorType::License,
        ));
    };
    let src = &rest[..src_end];
    let alt_and_end = &rest[src_end + 7..];
    let Some(alt_end) = alt_and_end.find('"') else {
        bail!(Error::new(
            "custom Wikidot license contains an invalid image",
            ErrorType::License,
        ));
    };
    let alt = &alt_and_end[..alt_end];
    let suffix = &alt_and_end[alt_end + 1..];
    let suffix = suffix
        .strip_prefix(" />")
        .or_else(|| suffix.strip_prefix("/>"))
        .or_else(|| suffix.strip_prefix('>'))
        .ok_or_raise(|| {
            Error::new(
                "custom Wikidot license contains an invalid image",
                ErrorType::License,
            )
        })?;
    ensure_safe_license_url(src, true)?;
    output.push_str("<img src=\"");
    escape_license_attribute(src, output);
    output.push_str("\" alt=\"");
    escape_license_attribute(alt, output);
    output.push_str("\" />");
    Ok(Some(suffix))
}

fn ensure_safe_license_url(value: &str, image: bool) -> Result<()> {
    let lower = value.trim().to_ascii_lowercase();
    let has_control = value.chars().any(char::is_control);
    let allowed = !value.is_empty()
        && !has_control
        && (value.starts_with('/')
            || value.starts_with('#')
            || value.starts_with('?')
            || lower.starts_with("http://")
            || lower.starts_with("https://")
            || (!image && lower.starts_with("mailto:"))
            || !lower.contains(':'));
    ensure!(
        allowed,
        Error::new(
            "custom Wikidot license contains an unsafe URL",
            ErrorType::License,
        ),
    );
    Ok(())
}

fn escape_license_text(value: &str, output: &mut String) {
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '>' => output.push_str("&gt;"),
            character => output.push(character),
        }
    }
}

fn escape_license_attribute(value: &str, output: &mut String) {
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '"' => output.push_str("&quot;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            character => output.push(character),
        }
    }
}

#[cfg(test)]
mod wikidot_license_tests {
    use super::*;

    #[test]
    fn custom_license_canonicalizes_the_live_wikidot_allowlist() {
        let source = "Codex & %%year%% <strong>Strong</strong> <em>Em</em> <a href=\"/page\">Local</a> <img src=\"/icon.png\" alt=\"Icon\"/> <br/>";
        let html = sanitize_wikidot_custom_license(source).unwrap();
        assert_eq!(
            html,
            "Codex &amp; %%year%% <strong>Strong</strong> <em>Em</em> <a href=\"/page\">Local</a> <img src=\"/icon.png\" alt=\"Icon\" /> <br />",
        );
        assert_eq!(
            WikidotLicense::from_storage("other", Some(&html)).unwrap(),
            WikidotLicense::Other(html.clone()),
        );
        assert_eq!(
            WikidotLicense::Other(html).render_other(2026).unwrap(),
            "Codex &amp; 2026 <strong>Strong</strong> <em>Em</em> <a href=\"/page\">Local</a> <img src=\"/icon.png\" alt=\"Icon\" /> <br />",
        );
    }

    #[test]
    fn custom_license_rejects_unsupported_or_unsafe_markup() {
        for source in [
            "<span>not allowed</span>",
            "<strong class=\"x\">not allowed</strong>",
            "<a href=\"javascript:alert(1)\">unsafe</a>",
            "<img src=\"data:text/html,x\" alt=\"unsafe\" />",
            "<strong>unbalanced",
        ] {
            assert!(sanitize_wikidot_custom_license(source).is_err(), "{source}");
        }
    }

    #[test]
    fn category_license_modes_have_unambiguous_storage() {
        assert_eq!(
            validate_wikidot_license_override(Some("cc-by-sa-3.0"), None).unwrap(),
            (Some(str!("cc-by-sa-3.0")), None),
        );
        assert_eq!(
            validate_wikidot_license_override(Some("copyright"), None).unwrap(),
            (Some(str!("copyright")), None),
        );
        assert_eq!(
            validate_wikidot_license_override(Some("other"), Some("Text")).unwrap(),
            (Some(str!("other")), Some(str!("Text"))),
        );
        assert_eq!(
            validate_wikidot_license_override(None, None).unwrap(),
            (None, None),
        );
        assert!(validate_wikidot_license_override(None, Some("orphaned")).is_err());
    }
}

impl License {
    pub fn url(self) -> &'static str {
        match self {
            License::CcBySa40 => "https://creativecommons.org/licenses/by-sa/4.0/",
            License::CcBy40 => "https://creativecommons.org/licenses/by/4.0/",
            License::CcByNd40 => "https://creativecommons.org/licenses/by-nd/4.0/",
            License::CcByNc40 => "https://creativecommons.org/licenses/by-nc/4.0/",
            License::CcByNcSa40 => "https://creativecommons.org/licenses/by-nc-sa/4.0/",
            License::CcByNcNd40 => "https://creativecommons.org/licenses/by-nc-nd/4.0/",
            License::CcBySa30 => "https://creativecommons.org/licenses/by-sa/3.0/",
            License::CcBy30 => "https://creativecommons.org/licenses/by/3.0/",
            License::CcByNd30 => "https://creativecommons.org/licenses/by-nd/3.0/",
            License::CcByNc30 => "https://creativecommons.org/licenses/by-nc/3.0/",
            License::CcByNcSa30 => "https://creativecommons.org/licenses/by-nc-sa/3.0/",
            License::CcByNcNd30 => "https://creativecommons.org/licenses/by-nc-nd/3.0/",
            License::CcBySa25 => "https://creativecommons.org/licenses/by-sa/2.5/",
            License::CcBy25 => "https://creativecommons.org/licenses/by/2.5/",
            License::CcByNd25 => "https://creativecommons.org/licenses/by-nd/2.5/",
            License::CcByNc25 => "https://creativecommons.org/licenses/by-nc/2.5/",
            License::CcByNcSa25 => "https://creativecommons.org/licenses/by-nc-sa/2.5/",
            License::CcByNcNd25 => "https://creativecommons.org/licenses/by-nc-nd/2.5/",
            License::GnuFdl13 => "https://www.gnu.org/licenses/fdl-1.3.html",
            License::GnuFdl12 => "https://www.gnu.org/licenses/old-licenses/fdl-1.2.html",
            License::GnuFdl11 => "https://www.gnu.org/licenses/old-licenses/fdl-1.1.html",
            License::Cc0 => "https://creativecommons.org/public-domain/cc0/",
        }
    }

    fn fluent_key(self) -> &'static str {
        match self {
            // Creative Commons 4.0
            License::CcBySa40 => "license.cc-by-sa-4-0",
            License::CcBy40 => "license.cc-by-4-0",
            License::CcByNd40 => "license.cc-by-nd-4-0",
            License::CcByNc40 => "license.cc-by-nc-4-0",
            License::CcByNcSa40 => "license.cc-by-nc-sa-4-0",
            License::CcByNcNd40 => "license.cc-by-nc-nd-4-0",

            // Creative Commons 3.0
            License::CcBySa30 => "license.cc-by-sa-3-0",
            License::CcBy30 => "license.cc-by-3-0",
            License::CcByNd30 => "license.cc-by-nd-3-0",
            License::CcByNc30 => "license.cc-by-nc-3-0",
            License::CcByNcSa30 => "license.cc-by-nc-sa-3-0",
            License::CcByNcNd30 => "license.cc-by-nc-nd-3-0",

            // Creative Commons 2.5
            License::CcBySa25 => "license.cc-by-sa-2-5",
            License::CcBy25 => "license.cc-by-2-5",
            License::CcByNd25 => "license.cc-by-nd-2-5",
            License::CcByNc25 => "license.cc-by-nc-2-5",
            License::CcByNcSa25 => "license.cc-by-nc-sa-2-5",
            License::CcByNcNd25 => "license.cc-by-nc-nd-2-5",

            // GNU Free Documentation License
            License::GnuFdl13 => "license.gnu-fdl-1-3",
            License::GnuFdl12 => "license.gnu-fdl-1-2",
            License::GnuFdl11 => "license.gnu-fdl-1-1",

            // Public Domain
            License::Cc0 => "license.cc0",
        }
    }

    pub fn translate(
        self,
        localization: &Localizations,
        locales: &[LanguageIdentifier],
    ) -> Result<String> {
        assert!(!locales.is_empty(), "No languages specified");
        let args = FluentArgs::new();
        let name = localization
            .translate(locales, self.fluent_key(), &args)
            .or_raise(|| {
                Error::new("failed to translate license name", ErrorType::License)
            })?;

        Ok(name.to_string())
    }
}

#[test]
fn license_urls_are_stable() {
    assert_eq!(
        License::CcBySa40.url(),
        "https://creativecommons.org/licenses/by-sa/4.0/",
    );
    assert_eq!(
        License::CcBy40.url(),
        "https://creativecommons.org/licenses/by/4.0/",
    );
    assert_eq!(
        License::CcByNd40.url(),
        "https://creativecommons.org/licenses/by-nd/4.0/",
    );
    assert_eq!(
        License::CcByNc40.url(),
        "https://creativecommons.org/licenses/by-nc/4.0/",
    );
    assert_eq!(
        License::CcByNcSa40.url(),
        "https://creativecommons.org/licenses/by-nc-sa/4.0/",
    );
    assert_eq!(
        License::CcByNcNd40.url(),
        "https://creativecommons.org/licenses/by-nc-nd/4.0/",
    );
    assert_eq!(
        License::CcBySa30.url(),
        "https://creativecommons.org/licenses/by-sa/3.0/",
    );
    assert_eq!(
        License::CcBy30.url(),
        "https://creativecommons.org/licenses/by/3.0/",
    );
    assert_eq!(
        License::CcByNd30.url(),
        "https://creativecommons.org/licenses/by-nd/3.0/",
    );
    assert_eq!(
        License::CcByNc30.url(),
        "https://creativecommons.org/licenses/by-nc/3.0/",
    );
    assert_eq!(
        License::CcByNcSa30.url(),
        "https://creativecommons.org/licenses/by-nc-sa/3.0/",
    );
    assert_eq!(
        License::CcByNcNd30.url(),
        "https://creativecommons.org/licenses/by-nc-nd/3.0/",
    );
    assert_eq!(
        License::CcBySa25.url(),
        "https://creativecommons.org/licenses/by-sa/2.5/",
    );
    assert_eq!(
        License::CcBy25.url(),
        "https://creativecommons.org/licenses/by/2.5/",
    );
    assert_eq!(
        License::CcByNd25.url(),
        "https://creativecommons.org/licenses/by-nd/2.5/",
    );
    assert_eq!(
        License::CcByNc25.url(),
        "https://creativecommons.org/licenses/by-nc/2.5/",
    );
    assert_eq!(
        License::CcByNcSa25.url(),
        "https://creativecommons.org/licenses/by-nc-sa/2.5/",
    );
    assert_eq!(
        License::CcByNcNd25.url(),
        "https://creativecommons.org/licenses/by-nc-nd/2.5/",
    );
    assert_eq!(
        License::GnuFdl13.url(),
        "https://www.gnu.org/licenses/fdl-1.3.html",
    );
    assert_eq!(
        License::GnuFdl12.url(),
        "https://www.gnu.org/licenses/old-licenses/fdl-1.2.html",
    );
    assert_eq!(
        License::GnuFdl11.url(),
        "https://www.gnu.org/licenses/old-licenses/fdl-1.1.html",
    );
    assert_eq!(
        License::Cc0.url(),
        "https://creativecommons.org/public-domain/cc0/",
    );
}

#[test]
fn license_fluent_keys_are_stable() {
    assert_eq!(License::CcBySa40.fluent_key(), "license.cc-by-sa-4-0");
    assert_eq!(License::CcBy40.fluent_key(), "license.cc-by-4-0");
    assert_eq!(License::CcByNd40.fluent_key(), "license.cc-by-nd-4-0");
    assert_eq!(License::CcByNc40.fluent_key(), "license.cc-by-nc-4-0");
    assert_eq!(License::CcByNcSa40.fluent_key(), "license.cc-by-nc-sa-4-0",);
    assert_eq!(License::CcByNcNd40.fluent_key(), "license.cc-by-nc-nd-4-0",);
    assert_eq!(License::CcBySa30.fluent_key(), "license.cc-by-sa-3-0");
    assert_eq!(License::CcBy30.fluent_key(), "license.cc-by-3-0");
    assert_eq!(License::CcByNd30.fluent_key(), "license.cc-by-nd-3-0");
    assert_eq!(License::CcByNc30.fluent_key(), "license.cc-by-nc-3-0");
    assert_eq!(License::CcByNcSa30.fluent_key(), "license.cc-by-nc-sa-3-0",);
    assert_eq!(License::CcByNcNd30.fluent_key(), "license.cc-by-nc-nd-3-0",);
    assert_eq!(License::CcBySa25.fluent_key(), "license.cc-by-sa-2-5");
    assert_eq!(License::CcBy25.fluent_key(), "license.cc-by-2-5");
    assert_eq!(License::CcByNd25.fluent_key(), "license.cc-by-nd-2-5");
    assert_eq!(License::CcByNc25.fluent_key(), "license.cc-by-nc-2-5");
    assert_eq!(License::CcByNcSa25.fluent_key(), "license.cc-by-nc-sa-2-5",);
    assert_eq!(License::CcByNcNd25.fluent_key(), "license.cc-by-nc-nd-2-5",);
    assert_eq!(License::GnuFdl13.fluent_key(), "license.gnu-fdl-1-3");
    assert_eq!(License::GnuFdl12.fluent_key(), "license.gnu-fdl-1-2");
    assert_eq!(License::GnuFdl11.fluent_key(), "license.gnu-fdl-1-1");
    assert_eq!(License::Cc0.fluent_key(), "license.cc0");
}
