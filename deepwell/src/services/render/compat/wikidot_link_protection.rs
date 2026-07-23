/*
 * services/render/wikidot_link_protection.rs
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

use super::issued_markers::restore_issued_html_text_markers;
use super::literal_regions::LiteralRegionIndex;
use super::service::*;
use ftml::settings::WikitextSettings;
use std::borrow::Cow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ProtectedWikidotWikipediaLink {
    pub(super) link: WikidotWikipediaLink,
    pub(super) marker: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct WikidotWikipediaLink {
    pub(super) anchor: String,
    pub(super) href: String,
}

impl RenderService {
    pub(super) fn protect_wikidot_wikipedia_links(
        wikitext: &mut String,
        settings: &WikitextSettings,
    ) -> Vec<ProtectedWikidotWikipediaLink> {
        if !settings.enable_page_syntax {
            return Vec::new();
        }

        let source = wikitext.clone();
        let mut links = Vec::new();
        let mut output = String::with_capacity(source.len());
        let mut last = 0;
        let marker_nonce = Uuid::new_v4().as_simple().to_string();
        let literal_regions = LiteralRegionIndex::new_wikidot_syntax(&source);

        for captures in WIKIDOT_WIKIPEDIA_LINK_REGEX.captures_iter(&source) {
            let Some(link_match) = captures.get(0) else {
                continue;
            };

            output.push_str(&source[last..link_match.start()]);
            last = link_match.end();

            let Some(target) = captures.name("target").map(|matched| matched.as_str())
            else {
                output.push_str(link_match.as_str());
                continue;
            };

            if literal_regions.contains(link_match.start()) {
                output.push_str(link_match.as_str());
                continue;
            }

            let label = captures.name("label").map(|matched| matched.as_str());
            let link = build_wikidot_wikipedia_link(target, label);
            let marker = format!(
                "{WIKIDOT_WIKIPEDIA_LINK_SENTINEL_PREFIX}{marker_nonce}{}X",
                links.len(),
            );
            links.push(ProtectedWikidotWikipediaLink {
                link,
                marker: marker.clone(),
            });
            output.push_str(&marker);
        }

        if links.is_empty() {
            return links;
        }

        output.push_str(&source[last..]);
        *wikitext = output;
        links
    }

    pub(super) fn protect_wikidot_compat_links(
        wikitext: &mut String,
        settings: &WikitextSettings,
    ) -> Vec<ProtectedWikidotCompatLink> {
        if !settings.enable_page_syntax {
            return Vec::new();
        }

        let mut links = Vec::new();
        Self::protect_wikidot_anchor_markers(wikitext, &mut links);
        Self::protect_wikidot_current_page_links(wikitext, &mut links);
        Self::protect_wikidot_star_local_links(wikitext, &mut links);
        links
    }

    pub(super) fn protect_wikidot_anchor_markers(
        wikitext: &mut String,
        links: &mut Vec<ProtectedWikidotCompatLink>,
    ) {
        let source = wikitext.clone();
        let mut output = String::with_capacity(source.len());
        let mut last = 0;

        for captures in WIKIDOT_ANCHOR_MARKER_REGEX.captures_iter(&source) {
            let Some(marker_match) = captures.get(0) else {
                continue;
            };

            output.push_str(&source[last..marker_match.start()]);
            last = marker_match.end();

            if Self::is_inside_wikidot_literal_region(&source, marker_match.start()) {
                output.push_str(marker_match.as_str());
                continue;
            }

            let Some(name) = captures.name("name").map(|matched| matched.as_str().trim())
            else {
                output.push_str(marker_match.as_str());
                continue;
            };
            if name.is_empty() {
                output.push_str(marker_match.as_str());
                continue;
            }

            let marker = wikidot_compat_link_marker();
            links.push(ProtectedWikidotCompatLink {
                anchor: wikidot_named_anchor(name),
                marker: marker.clone(),
            });
            output.push_str(&marker);
        }

        if last == 0 {
            return;
        }

        output.push_str(&source[last..]);
        *wikitext = output;
    }

    pub(super) fn protect_wikidot_current_page_links(
        wikitext: &mut String,
        links: &mut Vec<ProtectedWikidotCompatLink>,
    ) {
        let source = wikitext.clone();
        let mut output = String::with_capacity(source.len());
        let mut last = 0;

        for captures in WIKIDOT_CURRENT_PAGE_LINK_REGEX.captures_iter(&source) {
            let Some(link_match) = captures.get(0) else {
                continue;
            };

            output.push_str(&source[last..link_match.start()]);
            last = link_match.end();

            if source[..link_match.start()].ends_with('[')
                || source[link_match.end()..].starts_with(']')
            {
                output.push_str(link_match.as_str());
                continue;
            }

            if Self::is_inside_wikidot_literal_region(&source, link_match.start()) {
                output.push_str(link_match.as_str());
                continue;
            }

            let Some(label) = captures
                .name("label")
                .map(|matched| matched.as_str().trim())
            else {
                output.push_str(link_match.as_str());
                continue;
            };
            if label.is_empty() {
                output.push_str(link_match.as_str());
                continue;
            }

            let marker = wikidot_compat_link_marker();
            links.push(ProtectedWikidotCompatLink {
                anchor: wikidot_current_page_anchor(label),
                marker: marker.clone(),
            });
            output.push_str(&marker);
        }

        if last == 0 {
            return;
        }

        output.push_str(&source[last..]);
        *wikitext = output;
    }

    pub(super) fn protect_wikidot_star_local_links(
        wikitext: &mut String,
        links: &mut Vec<ProtectedWikidotCompatLink>,
    ) {
        let source = wikitext.clone();
        let mut output = String::with_capacity(source.len());
        let mut last = 0;

        for captures in WIKIDOT_STAR_LOCAL_LINK_REGEX.captures_iter(&source) {
            let Some(link_match) = captures.get(0) else {
                continue;
            };

            output.push_str(&source[last..link_match.start()]);
            last = link_match.end();

            if Self::is_inside_wikidot_literal_region(&source, link_match.start()) {
                output.push_str(link_match.as_str());
                continue;
            }

            let Some(target) = captures.name("target").map(|matched| matched.as_str())
            else {
                output.push_str(link_match.as_str());
                continue;
            };
            let Some(label) = captures
                .name("label")
                .map(|matched| matched.as_str().trim())
            else {
                output.push_str(link_match.as_str());
                continue;
            };
            if label.is_empty() {
                output.push_str(link_match.as_str());
                continue;
            }

            let marker = wikidot_compat_link_marker();
            links.push(ProtectedWikidotCompatLink {
                anchor: wikidot_star_local_anchor(target, label),
                marker: marker.clone(),
            });
            output.push_str(&marker);
        }

        if last == 0 {
            return;
        }

        output.push_str(&source[last..]);
        *wikitext = output;
    }

    pub(super) fn restore_protected_wikidot_compat_links(
        html: String,
        links: &[ProtectedWikidotCompatLink],
    ) -> String {
        restore_issued_html_text_markers(
            html,
            WIKIDOT_COMPAT_LINK_SENTINEL_PREFIX,
            links
                .iter()
                .map(|link| (link.marker.as_str(), link.anchor.as_str())),
        )
    }

    pub(super) fn restore_protected_wikidot_wikipedia_links(
        html: String,
        links: &[ProtectedWikidotWikipediaLink],
    ) -> String {
        if links.is_empty() || !html.contains(WIKIDOT_WIKIPEDIA_LINK_SENTINEL_PREFIX) {
            return html;
        }

        let mut output = String::with_capacity(html.len());
        let mut last = 0;
        let mut cursor = 0;
        let mut in_tag = false;
        let mut tag_quote = None;
        let bytes = html.as_bytes();
        let prefix = WIKIDOT_WIKIPEDIA_LINK_SENTINEL_PREFIX.as_bytes();

        while cursor < bytes.len() {
            match bytes[cursor] {
                b'<' if !in_tag => in_tag = true,
                quote @ (b'\'' | b'"') if in_tag => match tag_quote {
                    Some(open_quote) if open_quote == quote => tag_quote = None,
                    None => tag_quote = Some(quote),
                    _ => {}
                },
                b'>' if in_tag && tag_quote.is_none() => in_tag = false,
                _ if !in_tag && bytes[cursor..].starts_with(prefix) => {
                    let nonce_start = cursor + prefix.len();
                    let nonce_end =
                        nonce_start + WIKIDOT_WIKIPEDIA_LINK_SENTINEL_NONCE_LEN;
                    if nonce_end >= bytes.len()
                        || !bytes[nonce_start..nonce_end]
                            .iter()
                            .all(u8::is_ascii_hexdigit)
                    {
                        cursor += 1;
                        continue;
                    }

                    let index_start = nonce_end;
                    let mut marker_end = index_start;
                    while marker_end < bytes.len() && bytes[marker_end].is_ascii_digit() {
                        marker_end += 1;
                    }

                    if marker_end > index_start
                        && bytes.get(marker_end) == Some(&b'X')
                        && let Ok(index) = html[index_start..marker_end].parse::<usize>()
                        && let Some(link) = links.get(index)
                        && link.marker == html[cursor..=marker_end]
                    {
                        output.push_str(&html[last..cursor]);
                        output.push_str(&link.link.anchor);
                        cursor = marker_end + 1;
                        last = cursor;
                        continue;
                    }
                }
                _ => {}
            }
            cursor += 1;
        }

        if last == 0 {
            return html;
        }

        output.push_str(&html[last..]);
        output
    }

    pub(super) fn record_protected_wikidot_wikipedia_backlinks(
        backlinks: &mut ftml::data::Backlinks<'_>,
        links: &[ProtectedWikidotWikipediaLink],
    ) {
        backlinks
            .external_links
            .extend(links.iter().map(|link| Cow::Owned(link.link.href.clone())));
    }

    pub(super) fn record_wikidot_wikipedia_backlinks(
        backlinks: &mut ftml::data::Backlinks<'_>,
        links: &[WikidotWikipediaLink],
    ) {
        backlinks
            .external_links
            .extend(links.iter().map(|link| Cow::Owned(link.href.clone())));
    }
}

pub(super) fn wikidot_compat_link_marker() -> String {
    format!(
        "{WIKIDOT_COMPAT_LINK_SENTINEL_PREFIX}{}X",
        Uuid::new_v4().as_simple(),
    )
}

pub(super) fn wikidot_named_anchor(name: &str) -> String {
    format!(
        r#"<a name="{name}"></a>"#,
        name = escape_list_pages_html_attr(name),
    )
}

pub(super) fn wikidot_current_page_anchor(label: &str) -> String {
    format!(
        r#"<a href="javascript:;">{label}</a>"#,
        label = escape_list_pages_html_text(label),
    )
}

pub(super) fn wikidot_star_local_anchor(target: &str, label: &str) -> String {
    let target = target.trim();
    let href = if target.starts_with('/') {
        target.to_owned()
    } else {
        format!("/{target}")
    };

    format!(
        r#"<a href="{href}" target="_blank">{label}</a>"#,
        href = escape_list_pages_html_attr(&href),
        label = escape_list_pages_html_text(label),
    )
}

pub(super) fn build_wikidot_wikipedia_link(
    target: &str,
    label: Option<&str>,
) -> WikidotWikipediaLink {
    let (language, page) = wikidot_wikipedia_target(target);
    let href = wikidot_wikipedia_href(language, page);
    let label = label
        .filter(|value| !value.is_empty())
        .map(Cow::Borrowed)
        .unwrap_or_else(|| Cow::Owned(page.replace('_', " ")));
    let anchor = format!(
        r#"<a href="{href}" onclick="window.open(this.href, '_blank'); return false;">{label}</a>"#,
        href = escape_list_pages_html_attr(&href),
        label = escape_list_pages_html_text(&label),
    );
    WikidotWikipediaLink { anchor, href }
}

pub(super) fn wikidot_wikipedia_href(language: &str, page: &str) -> String {
    format!("http://{language}.wikipedia.org/wiki/{page}")
}

pub(super) fn wikidot_wikipedia_target(target: &str) -> (&str, &str) {
    if let Some((language, page)) = target.split_once(':')
        && !page.is_empty()
        && (2..=3).contains(&language.len())
        && language
            .chars()
            .all(|character| character.is_ascii_alphabetic())
    {
        return (language, page);
    }

    ("en", target)
}
