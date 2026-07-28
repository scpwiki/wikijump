/*
 * services/view/redirect.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Runtime resolution of preserved Wikidot Redirect modules.
//!
//! FTML intentionally leaves this module in source form. Deepwell recognizes the
//! narrow, provenance-backed module shape here because producing an HTTP redirect
//! is site-runtime behavior. Unsupported shapes remain preserved in normal output.

use crate::services::render::LiteralRegionIndex;
use regex::Regex;
use std::sync::LazyLock;

const MAX_REDIRECT_DESTINATION_BYTES: usize = 8 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct WikidotRedirectModule {
    destination: String,
    module_source: String,
    location: String,
}

static REDIRECT_MODULE_PREFIX_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\[\[module[ \t]+redirect(?:[ \t]|\]\])")
        .expect("Redirect module prefix regular expression should compile")
});
static REDIRECT_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)^\[\[module[ \t]+redirect(?P<head>(?:"[^"]*"|'[^']*'|[^\]])*)\]\]$"#,
    )
    .expect("Redirect module regular expression should compile")
});
static REDIRECT_ARGUMENT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)(?P<key>[a-z][a-z0-9_-]*)[ \t]*=[ \t]*(?:"(?P<double>[^"]*)"|'(?P<single>[^']*)')"#,
    )
    .expect("Redirect argument regular expression should compile")
});

pub(super) fn wikidot_redirect_location(
    source: &str,
    current_slug: &str,
    no_redirect: bool,
) -> Option<String> {
    if no_redirect {
        return None;
    }

    wikidot_redirect_module(source, current_slug).map(|module| module.location)
}

pub(super) fn wikidot_redirect_noredirect_body_html(
    source: &str,
    current_slug: &str,
    compiled_body_html: String,
) -> String {
    let Some(module) = wikidot_redirect_module(source, current_slug) else {
        return compiled_body_html;
    };

    replace_preserved_redirect_module_html(&compiled_body_html, &module)
        .unwrap_or(compiled_body_html)
}

fn wikidot_redirect_module(
    source: &str,
    current_slug: &str,
) -> Option<WikidotRedirectModule> {
    let literal_regions = LiteralRegionIndex::new_wikidot_module_recognition(source);
    let mut parsed_module = None;
    let mut line_offset = 0;

    for line_with_ending in source.split_inclusive('\n') {
        let line = line_with_ending
            .strip_suffix('\n')
            .unwrap_or(line_with_ending);
        let trimmed =
            line.trim_matches(|character| matches!(character, ' ' | '\t' | '\r'));
        let Some(prefix) = REDIRECT_MODULE_PREFIX_REGEX.find(trimmed) else {
            line_offset += line_with_ending.len();
            continue;
        };
        let candidate_offset =
            line_offset + (line.len() - line.trim_start_matches([' ', '\t']).len());
        line_offset += line_with_ending.len();

        if literal_regions.contains(candidate_offset + prefix.start()) {
            continue;
        }

        let captures = REDIRECT_MODULE_REGEX.captures(trimmed)?;
        let head = captures.name("head")?.as_str();
        let destination = parse_destination_argument(head)?;
        let location = redirect_location(&destination, current_slug)?;
        let module = WikidotRedirectModule {
            destination,
            module_source: trimmed.to_owned(),
            location,
        };
        if parsed_module.replace(module).is_some() {
            return None;
        }
    }

    parsed_module
}

fn parse_destination_argument(head: &str) -> Option<String> {
    let mut destination = None;
    let mut cursor = 0;

    for captures in REDIRECT_ARGUMENT_REGEX.captures_iter(head) {
        let matched = captures.get(0)?;
        if !head[cursor..matched.start()].trim().is_empty() {
            return None;
        }
        cursor = matched.end();

        if !captures["key"].eq_ignore_ascii_case("destination") || destination.is_some() {
            return None;
        }
        let value = captures
            .name("double")
            .or_else(|| captures.name("single"))?
            .as_str();
        destination = Some(value.to_owned());
    }

    if !head[cursor..].trim().is_empty() {
        return None;
    }
    destination
}

fn redirect_location(destination: &str, current_slug: &str) -> Option<String> {
    if destination.is_empty()
        || destination.len() > MAX_REDIRECT_DESTINATION_BYTES
        || destination.trim() != destination
        || destination
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        || destination.contains('\\')
    {
        return None;
    }

    if destination.starts_with("//") {
        return None;
    }

    if destination
        .get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("http://"))
        || destination
            .get(..8)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("https://"))
    {
        let parsed = reqwest::Url::parse(destination).ok()?;
        if !matches!(parsed.scheme(), "http" | "https")
            || parsed.host_str().is_none()
            || !parsed.username().is_empty()
            || parsed.password().is_some()
        {
            return None;
        }
        return Some(destination.to_owned());
    }

    if destination.contains("://") {
        return None;
    }

    let path_end = destination.find(['?', '#']).unwrap_or(destination.len());
    let path = &destination[..path_end];
    let local_path = path.strip_prefix('/').unwrap_or(path);
    if local_path.is_empty()
        || local_path == "."
        || local_path == ".."
        || local_path.split('/').any(|segment| segment == "..")
        || local_path.eq_ignore_ascii_case(current_slug)
    {
        return None;
    }

    if let Some((scheme, _)) = local_path.split_once(':')
        && matches!(
            scheme.to_ascii_lowercase().as_str(),
            "data" | "file" | "javascript" | "vbscript"
        )
    {
        return None;
    }

    Some(if destination.starts_with('/') {
        destination.to_owned()
    } else {
        format!("/{destination}")
    })
}

fn replace_preserved_redirect_module_html(
    compiled_body_html: &str,
    module: &WikidotRedirectModule,
) -> Option<String> {
    let escaped_module_source = escape_wikidot_html_text(&module.module_source);
    let paragraph = format!("<p>{escaped_module_source}</p>");
    let notice = wikidot_redirect_notice_html(&module.destination);

    if compiled_body_html.matches(&paragraph).count() == 1 {
        return Some(compiled_body_html.replacen(&paragraph, &notice, 1));
    }

    if compiled_body_html.matches(&escaped_module_source).count() == 1 {
        return Some(compiled_body_html.replacen(&escaped_module_source, &notice, 1));
    }

    None
}

fn wikidot_redirect_notice_html(destination: &str) -> String {
    format!(
        concat!(
            "<div class=\"error-block\">\n",
            "\tThis is the Redirect module that redirects the browser directly to the ",
            "&quot;{}&quot; page.\n",
            "</div>"
        ),
        escape_wikidot_html_text(destination),
    )
}

fn escape_wikidot_html_text(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }

    escaped
}

#[cfg(test)]
mod tests {
    use super::{
        WikidotRedirectModule, replace_preserved_redirect_module_html,
        wikidot_redirect_location, wikidot_redirect_noredirect_body_html,
    };
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct RedirectEvidenceCase {
        fullname: String,
        destination: String,
        location: String,
    }

    #[test]
    fn resolves_all_forty_provenance_backed_redirect_destinations() {
        let cases: Vec<RedirectEvidenceCase> = serde_json::from_str(include_str!(
            "../../../tests/fixtures/wikidot_redirects.json"
        ))
        .expect("redirect fixture should parse");
        assert_eq!(cases.len(), 40);

        for case in cases {
            let source = format!(
                "ordinary content\n[[module Redirect destination=\"{}\"]]\nmore content",
                case.destination,
            );
            assert_eq!(
                wikidot_redirect_location(&source, &case.fullname, false).as_deref(),
                Some(case.location.as_str()),
                "{}",
                case.fullname,
            );
        }
    }

    #[test]
    fn preserves_destination_query_and_fragment() {
        assert_eq!(
            wikidot_redirect_location(
                "[[module Redirect destination=\"target?view=full#history\"]]",
                "source",
                false,
            )
            .as_deref(),
            Some("/target?view=full#history"),
        );
        assert_eq!(
            wikidot_redirect_location(
                "[[module Redirect destination=\"https://example.com/target?q=1#part\"]]",
                "source",
                false,
            )
            .as_deref(),
            Some("https://example.com/target?q=1#part"),
        );
    }

    #[test]
    fn invalid_or_unsupported_destinations_remain_literal() {
        for destination in [
            "",
            "//example.com/path",
            "ftp://example.com/path",
            "javascript:alert(1)",
            "data:text/html,redirect",
            "../outside",
            "target path",
            "https://user:password@example.com/path",
        ] {
            let source = format!("[[module Redirect destination=\"{destination}\"]]");
            assert_eq!(wikidot_redirect_location(&source, "source", false), None);
        }
    }

    #[test]
    fn ordinary_and_ambiguous_pages_do_not_redirect() {
        assert_eq!(
            wikidot_redirect_location("ordinary page", "source", false),
            None,
        );
        assert_eq!(
            wikidot_redirect_location(
                "[[module Redirect destination=target]]",
                "source",
                false,
            ),
            None,
        );
        assert_eq!(
            wikidot_redirect_location(
                "[[module Redirect destination=\"one\"]]\n[[module Redirect destination=\"two\"]]",
                "source",
                false,
            ),
            None,
        );
        assert_eq!(
            wikidot_redirect_location(
                "prefix [[module Redirect destination=\"target\"]]",
                "source",
                false,
            ),
            None,
        );
    }

    #[test]
    fn literal_regions_do_not_create_redirects() {
        for source in [
            "[!--\n[[module Redirect destination=\"target\"]]\n--]",
            "[[code]]\n[[module Redirect destination=\"target\"]]\n[[/code]]",
            "@@[[module Redirect destination=\"target\"]]@@",
            "[[html]]\n[[module Redirect destination=\"target\"]]\n[[/html]]",
        ] {
            assert_eq!(wikidot_redirect_location(source, "source", false), None);
        }
    }

    #[test]
    fn self_redirects_fail_closed() {
        for destination in ["source", "/source", "source?again=1", "/source#again"] {
            let source = format!("[[module Redirect destination=\"{destination}\"]]");
            assert_eq!(wikidot_redirect_location(&source, "source", false), None);
        }
    }

    #[test]
    fn noredirect_option_preserves_the_page_instead_of_redirecting() {
        assert_eq!(
            wikidot_redirect_location(
                "[[module Redirect destination=\"target\"]]",
                "source",
                true,
            ),
            None,
        );
    }

    #[test]
    fn noredirect_view_replaces_preserved_module_with_wikidot_notice() {
        assert_eq!(
            wikidot_redirect_noredirect_body_html(
                "[[module Redirect destination=\"target\"]]",
                "source",
                r#"<p>[[module Redirect destination=&quot;target&quot;]]</p>"#.to_owned(),
            ),
            concat!(
                "<div class=\"error-block\">\n",
                "\tThis is the Redirect module that redirects the browser directly to the ",
                "&quot;target&quot; page.\n",
                "</div>",
            ),
        );
    }

    #[test]
    fn noredirect_notice_escapes_destination_text() {
        let module = WikidotRedirectModule {
            destination: "unsafe<&'>".to_owned(),
            module_source: "[[module Redirect destination=\"unsafe<&'>\"]]".to_owned(),
            location: "/unsafe<&'>".to_owned(),
        };

        let notice = replace_preserved_redirect_module_html(
            r#"<p>[[module Redirect destination=&quot;unsafe&lt;&amp;&#39;&gt;&quot;]]</p>"#,
            &module,
        )
        .expect("preserved redirect module should be replaced");
        assert!(notice.contains("unsafe&lt;&amp;&#39;&gt;"));
    }
}
