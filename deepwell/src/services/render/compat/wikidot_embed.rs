/*
 * services/render/wikidot_embed.rs
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

use super::super::service::RenderService;
use regex::Regex;
use std::sync::LazyLock;

const WIKIDOT_EMBED_IFRAME_SENTINEL_PREFIX: &str = "WIKIJUMPWIKIDOTEMBEDIFRAME";
const WIKIDOT_LOCAL_INTERWIKI_BASE: &str = "/-/wikidot-interwiki";

static WIKIDOT_RAW_EMBED_IFRAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)\[\[embed\]\]\s*(?P<iframe><iframe\b[^>]*></iframe>)\s*\[\[/embed\]\]"#,
    )
    .unwrap()
});
static WIKIDOT_EMBED_BLOCK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)\[\[(?P<block>embed|embedaudio|embedvideo)\]\](?P<payload>.*?)\[\[/(?P<close>embed|embedaudio|embedvideo)\]\]"#,
    )
    .unwrap()
});
const WIKIDOT_EMBED_NO_MATCH_HTML: &str =
    r#"<div class="error-block">Sorry, no match for the embedded content.</div>"#;
static WIKIDOT_RENDERED_ANCHOR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)<a href="[^"]+">(.*?)</a>"#).unwrap());
static WIKIDOT_STYLEFRAME_IFRAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"^<iframe src="(?P<src>//interwiki\.(?:scpwiki\.com|scp-jp\.org)/styleFrame\.html\?[^"]+)" style="display: none"></iframe>$"#,
    )
    .unwrap()
});
static WIKIDOT_INTERWIKI_FRAME_IFRAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"^<iframe src="(?P<src>//interwiki\.(?:scpwiki\.com|scp-jp\.org)/interwikiFrame\.html\?[^"]+)" allowtransparency="true" class="html-block-iframe scpnet-interwiki-frame"></iframe>$"#,
    )
    .unwrap()
});

impl RenderService {
    pub(in crate::services::render) fn protect_wikidot_embed_iframes(
        wikitext: &mut String,
    ) -> Vec<String> {
        let mut iframes = Vec::new();
        let protected = WIKIDOT_EMBED_BLOCK_REGEX
            .replace_all(wikitext, |captures: &regex::Captures<'_>| {
                let whole = captures.get(0).map_or("", |matched| matched.as_str());
                let opened = captures.name("block").map(|matched| matched.as_str());
                let closed = captures.name("close").map(|matched| matched.as_str());
                if opened != closed {
                    return whole.to_owned();
                }
                let rendered = Self::allowed_wikidot_embed_iframe_block(whole)
                    .unwrap_or_else(|| WIKIDOT_EMBED_NO_MATCH_HTML.to_owned());

                let marker =
                    format!("{WIKIDOT_EMBED_IFRAME_SENTINEL_PREFIX}{}X", iframes.len());
                iframes.push(rendered);
                marker
            })
            .into_owned();
        *wikitext = protected;
        iframes
    }

    fn allowed_wikidot_embed_iframe_block(block: &str) -> Option<String> {
        let captures = WIKIDOT_RAW_EMBED_IFRAME_REGEX.captures(block)?;
        let iframe = captures.name("iframe")?.as_str().trim();
        Self::allowed_wikidot_embed_iframe(iframe)
    }

    pub(in crate::services::render) fn restore_protected_wikidot_embed_iframes(
        mut html: String,
        iframes: &[String],
    ) -> String {
        for (index, iframe) in iframes.iter().enumerate() {
            let marker = format!("{WIKIDOT_EMBED_IFRAME_SENTINEL_PREFIX}{index}X");
            html = html.replace(&marker, iframe);
        }
        html
    }

    pub(in crate::services::render) fn allowed_wikidot_embed_iframe(
        iframe: &str,
    ) -> Option<String> {
        if let Some(captures) = WIKIDOT_STYLEFRAME_IFRAME_REGEX.captures(iframe) {
            return Some(Self::rewrite_wikidot_interwiki_iframe_src(
                iframe,
                &captures["src"],
                "styleFrame.html",
            ));
        }

        if let Some(captures) = WIKIDOT_INTERWIKI_FRAME_IFRAME_REGEX.captures(iframe) {
            return Some(Self::rewrite_wikidot_interwiki_iframe_src(
                iframe,
                &captures["src"],
                "interwikiFrame.html",
            ));
        }

        None
    }

    fn rewrite_wikidot_interwiki_iframe_src(
        iframe: &str,
        original_src: &str,
        local_file_name: &str,
    ) -> String {
        let query = original_src.split_once('?').map_or("", |(_, query)| query);
        let local_src =
            format!("{WIKIDOT_LOCAL_INTERWIKI_BASE}/{local_file_name}?{query}");

        iframe.replace(original_src, &local_src)
    }

    pub(in crate::services::render) fn decode_rendered_embed_block(
        block: &str,
    ) -> String {
        let without_anchors = WIKIDOT_RENDERED_ANCHOR_REGEX.replace_all(block, "$1");
        let text = without_anchors
            .replace("<br>", "")
            .replace("<br/>", "")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#34;", "\"")
            .replace("&#39;", "'")
            .replace("&amp;", "&");

        text.trim().to_owned()
    }
}
