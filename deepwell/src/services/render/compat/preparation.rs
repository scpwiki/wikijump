/*
 * services/render/compat/preparation.rs
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

use super::super::list_pages::scanner::first_list_pages_module_opening_candidate;
use super::super::literal_regions::{LiteralRegionIndex, WikidotNativeQuoteIndex};
use super::CompatHtmlFragments;
use super::text_fragments::CompatTextFragments;
use ftml::settings::WikitextSettings;
use regex::Regex;
use std::ops::Range;
use std::sync::LazyLock;

static CSS_MODULE_OPEN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)\[\[module\s+css[^\]]*\]\]").unwrap());
static CSS_MODULE_OPEN_HEAD_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)^\[\[module\s+css(?P<head>[^\]]*)\]\]$").unwrap());
static MODULE_CLOSE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)\[\[/module\]\]").unwrap());
static AUTHORED_WIKIDOT_COMPAT_MARKER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)data-wikijump-compat-(?P<kind>listpages|list|members|backlinks|new-page|clone|date|css-module)",
    )
    .unwrap()
});
static AUTHORED_WIKIDOT_COMPAT_OPEN_TAG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)<table class="wiki-content-table" data-wikijump-compat-listpages="1">|<ul data-wikijump-compat-list="1">|<div id="ml-[0-9]+" data-wikijump-compat-members="1"[^>]*>|<div class="backlinks-module-box" data-wikijump-compat-backlinks="1"[^>]*>|<form class="new-page-box" data-wikijump-compat-new-page="1"[^>]*>|<a class="button" data-wikijump-compat-clone="1"[^>]*>|<span class="odate time_-?[0-9]+ format_[A-Za-z0-9%_.-]+" data-wikijump-compat-date="1" style="cursor: help; display: inline;">|<span data-wikijump-compat-listpages-preview="1" style="white-space: pre-wrap;">|<style data-wikijump-compat-css-module="1">"#,
    )
    .unwrap()
});

pub(in crate::services::render) fn protect_css_modules_before_first_list_pages(
    wikitext: &mut String,
    settings: &WikitextSettings,
) -> Option<CompatTextFragments> {
    if !settings.enable_page_syntax {
        return None;
    }
    let boundary = first_list_pages_module_opening_candidate(wikitext)?;
    if boundary == 0 || !CSS_MODULE_OPEN_REGEX.is_match(&wikitext[..boundary]) {
        return None;
    }

    let mut fragments = CompatTextFragments::new(wikitext);
    let suffix = wikitext.split_off(boundary);
    let source = wikitext.as_str();
    let literal_regions = LiteralRegionIndex::new(source);
    let syntax_literal_regions = LiteralRegionIndex::new_wikidot_syntax(source);
    let native_quote_lines = WikidotNativeQuoteIndex::new(source);
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut protected_any = false;

    while let Some(open) = CSS_MODULE_OPEN_REGEX.find_at(source, cursor) {
        if literal_regions.contains(open.start())
            || native_quote_lines.contains(open.start())
        {
            output.push_str(&source[cursor..open.end()]);
            cursor = open.end();
            continue;
        }
        let mut close_cursor = open.end();
        let close = loop {
            let Some(candidate) = MODULE_CLOSE_REGEX.find_at(source, close_cursor) else {
                output.push_str(&source[cursor..]);
                *wikitext = output;
                wikitext.push_str(&suffix);
                return protected_any.then_some(fragments);
            };
            if !syntax_literal_regions.contains(candidate.start()) {
                break candidate;
            }
            close_cursor = candidate.end();
        };
        output.push_str(&source[cursor..open.start()]);
        output.push_str(&fragments.push(&source[open.start()..close.end()]));
        cursor = close.end();
        protected_any = true;
    }
    output.push_str(&source[cursor..]);
    *wikitext = output;
    wikitext.push_str(&suffix);
    protected_any.then_some(fragments)
}

pub(in crate::services::render) fn extract_css_modules(
    wikitext: &mut String,
    settings: &WikitextSettings,
    compat_html: &mut CompatHtmlFragments,
) -> Vec<String> {
    if !settings.enable_page_syntax {
        return Vec::new();
    }

    let source = wikitext.as_str();
    let literal_regions = LiteralRegionIndex::new(source);
    let syntax_literal_regions = LiteralRegionIndex::new_wikidot_syntax(source);
    let native_quote_lines = WikidotNativeQuoteIndex::new(source);
    let mut output = String::with_capacity(source.len());
    let mut styles = Vec::new();
    let mut cursor = 0;

    while let Some(open) = CSS_MODULE_OPEN_REGEX.find_at(source, cursor) {
        if literal_regions.contains(open.start())
            || native_quote_lines.contains(open.start())
        {
            output.push_str(&source[cursor..open.end()]);
            cursor = open.end();
            continue;
        }
        let mut close_cursor = open.end();
        let close = loop {
            let Some(candidate) = MODULE_CLOSE_REGEX.find_at(source, close_cursor) else {
                output.push_str(&source[cursor..]);
                *wikitext = output;
                return styles;
            };
            if !syntax_literal_regions.contains(candidate.start()) {
                break candidate;
            }
            close_cursor = candidate.end();
        };
        let body = source[open.end()..close.start()].trim_matches('\n');
        let flags = css_module_flags(open.as_str());
        output.push_str(&source[cursor..open.start()]);
        if flags.show {
            output.push_str(
                &compat_html.push_block_html(render_css_module_code_block(body)),
            );
        }
        if !flags.disable {
            styles.push(escape_css_module_body(body));
        }
        cursor = close.end();
    }
    output.push_str(&source[cursor..]);
    *wikitext = output;
    styles
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CssModuleFlags {
    show: bool,
    disable: bool,
}

fn css_module_flags(open: &str) -> CssModuleFlags {
    let Some(captures) = CSS_MODULE_OPEN_HEAD_REGEX.captures(open) else {
        return CssModuleFlags::default();
    };
    let head = captures.name("head").map_or("", |head| head.as_str());
    let mut flags = CssModuleFlags::default();
    let mut cursor = 0;
    while cursor < head.len() {
        skip_css_module_whitespace(head, &mut cursor);
        if cursor >= head.len() {
            break;
        }

        if let Some((key, value, next)) = css_module_exact_argument_at(head, cursor) {
            let enabled = matches!(value, "true" | "yes");
            match key {
                "show" => flags.show = enabled,
                "disable" => flags.disable = enabled,
                _ => {}
            }
            cursor = next;
        } else {
            skip_css_module_non_whitespace(head, &mut cursor);
        }
    }
    flags
}

fn css_module_exact_argument_at(
    head: &str,
    cursor: usize,
) -> Option<(&str, &str, usize)> {
    for (key, prefix) in [("show", r#"show=""#), ("disable", r#"disable=""#)] {
        let value_start = cursor.checked_add(prefix.len())?;
        if !head[cursor..].starts_with(prefix) {
            continue;
        }
        let relative_end = head[value_start..].find('"')?;
        let value_end = value_start + relative_end;
        let next = value_end + '"'.len_utf8();
        if next < head.len()
            && !head[next..].chars().next().is_some_and(char::is_whitespace)
        {
            continue;
        }
        return Some((key, &head[value_start..value_end], next));
    }
    None
}

fn skip_css_module_whitespace(head: &str, cursor: &mut usize) {
    while *cursor < head.len()
        && head[*cursor..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
    {
        *cursor += head[*cursor..]
            .chars()
            .next()
            .expect("cursor should point at a character")
            .len_utf8();
    }
}

fn skip_css_module_non_whitespace(head: &str, cursor: &mut usize) {
    while *cursor < head.len()
        && head[*cursor..]
            .chars()
            .next()
            .is_some_and(|character| !character.is_whitespace())
    {
        *cursor += head[*cursor..]
            .chars()
            .next()
            .expect("cursor should point at a character")
            .len_utf8();
    }
}

fn render_css_module_code_block(body: &str) -> String {
    let mut output =
        String::from(r#"<div class="code" data-wj-language="css"><pre><code>"#);
    output.push_str(&escape_css_module_code_html(body));
    output.push_str("</code></pre></div>");
    output
}

fn escape_css_module_code_html(body: &str) -> String {
    body.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_css_module_body(body: &str) -> String {
    body.replace('<', r"\3C ")
}

pub(in crate::services::render) fn neutralize_authored_markers(wikitext: &mut String) {
    if !AUTHORED_WIKIDOT_COMPAT_MARKER_REGEX.is_match(wikitext) {
        return;
    }
    let source = wikitext.clone();
    let literal_regions = LiteralRegionIndex::new(&source);
    let mut replacements: Vec<(Range<usize>, String)> = Vec::new();

    for candidate in AUTHORED_WIKIDOT_COMPAT_OPEN_TAG_REGEX.find_iter(&source) {
        if literal_regions.contains(candidate.start()) {
            continue;
        }
        for captures in
            AUTHORED_WIKIDOT_COMPAT_MARKER_REGEX.captures_iter(candidate.as_str())
        {
            let full_match = captures.get(0).expect("compat marker capture exists");
            let start = candidate.start() + full_match.start();
            replacements.push((
                start..candidate.start() + full_match.end(),
                format!(
                    "data-wikijump-authored-compat-{}",
                    captures["kind"].to_ascii_lowercase(),
                ),
            ));
        }
    }

    for (range, replacement) in replacements.into_iter().rev() {
        wikitext.replace_range(range, &replacement);
    }
}
