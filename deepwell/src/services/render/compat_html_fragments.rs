//! Render-local provenance for HTML produced by trusted runtime producers.

use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use uuid::Uuid;

pub(super) const COMPAT_HTML_MARKER_PREFIX: &str = "WIKIJUMPWIKIDOTCOMPATHTML";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct CompatHtmlFragments {
    namespace: String,
    fragments: Vec<CompatFragment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum CompatFragment {
    Html(String),
    Plain { plain: String, html: String },
}

impl CompatHtmlFragments {
    pub(super) fn new(untrusted_source: &str) -> Self {
        let namespace = loop {
            let candidate =
                format!("{COMPAT_HTML_MARKER_PREFIX}{}I", Uuid::new_v4().as_simple(),);
            if !untrusted_source.contains(&candidate) {
                break candidate;
            }
        };
        Self {
            namespace,
            fragments: Vec::new(),
        }
    }

    pub(super) fn push_html(&mut self, html: String) -> String {
        self.push_fragment(CompatFragment::Html(html))
    }

    pub(super) fn push_plain(&mut self, plain: &str) -> String {
        self.push_fragment(CompatFragment::Plain {
            plain: plain.to_owned(),
            html: escape_in_any_html_context(plain),
        })
    }

    fn push_fragment(&mut self, fragment: CompatFragment) -> String {
        let index = self.fragments.len();
        self.fragments.push(fragment);
        format!("{}{index}X", self.namespace)
    }

    pub(super) fn restore(&self, text: &str) -> String {
        self.restore_with(text, |fragment| match fragment {
            CompatFragment::Html(html) => Some(html.as_str()),
            CompatFragment::Plain { html, .. } => Some(html.as_str()),
        })
    }

    pub(super) fn restore_plain(&self, text: &str) -> String {
        self.restore_with(text, |fragment| match fragment {
            CompatFragment::Plain { plain, .. } => Some(plain.as_str()),
            CompatFragment::Html(_) => None,
        })
    }

    fn restore_with<'a>(
        &'a self,
        text: &str,
        value: impl Fn(&'a CompatFragment) -> Option<&'a str>,
    ) -> String {
        if self.fragments.is_empty() || !text.contains(&self.namespace) {
            return text.to_owned();
        }
        let mut output = String::with_capacity(text.len());
        let mut cursor = 0;
        while let Some(offset) = text[cursor..].find(&self.namespace) {
            let start = cursor + offset;
            output.push_str(&text[cursor..start]);
            if let Some((index, len)) = self.marker_at(&text[start..]) {
                if let Some(fragment) = value(&self.fragments[index]) {
                    output.push_str(fragment);
                    cursor = start + len;
                } else {
                    output.push_str(&text[start..start + len]);
                    cursor = start + len;
                }
            } else {
                output.push_str(&self.namespace);
                cursor = start + self.namespace.len();
            }
        }
        output.push_str(&text[cursor..]);
        output
    }

    fn marker_at(&self, text: &str) -> Option<(usize, usize)> {
        let suffix = text.strip_prefix(&self.namespace)?;
        let digits = suffix.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 || suffix.as_bytes().get(digits) != Some(&b'X') {
            return None;
        }
        let index = suffix[..digits].parse::<usize>().ok()?;
        (index < self.fragments.len())
            .then_some((index, self.namespace.len() + digits + 1))
    }
}

fn escape_in_any_html_context(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
            escaped.push(character);
        } else {
            write!(&mut escaped, "&#x{:X};", character as u32)
                .expect("writing to a String cannot fail");
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restores_only_registered_in_range_fragments_without_recursion() {
        let mut fragments = CompatHtmlFragments::new("authored source");
        let marker_shaped = format!("{COMPAT_HTML_MARKER_PREFIX}deadbeefI0X");
        let first = fragments.push_html(format!("<b>{marker_shaped}</b>"));
        let second = fragments.push_html("<i>second</i>".to_owned());
        let foreign =
            format!("{COMPAT_HTML_MARKER_PREFIX}ffffffffffffffffffffffffffffffffI0X");
        assert_eq!(
            fragments.restore(&format!("{first}|{second}|{foreign}")),
            format!("<b>{marker_shaped}</b>|<i>second</i>|{foreign}"),
        );
    }

    #[test]
    fn malformed_and_out_of_range_markers_remain_literal() {
        let mut fragments = CompatHtmlFragments::new("");
        let valid = fragments.push_html("<b>trusted</b>".to_owned());
        let malformed = format!("{}nopeX", fragments.namespace);
        let out_of_range = format!("{}9X", fragments.namespace);
        assert_eq!(fragments.restore(&valid), "<b>trusted</b>");
        assert_eq!(fragments.restore(&malformed), malformed);
        assert_eq!(fragments.restore(&out_of_range), out_of_range);
    }

    #[test]
    fn restores_plain_fragments_by_destination_without_recursion() {
        let mut fragments = CompatHtmlFragments::new("");
        let forged = format!("{COMPAT_HTML_MARKER_PREFIX}deadbeefI0X");
        let marker =
            fragments.push_plain(&format!(r#"tag ] <img onerror='x'> {forged}"#));

        assert_eq!(
            fragments.restore(&marker),
            format!(
                "tag&#x20;&#x5D;&#x20;&#x3C;img&#x20;onerror&#x3D;&#x27;x&#x27;&#x3E;&#x20;{forged}"
            ),
        );
        assert_eq!(
            fragments.restore_plain(&marker),
            format!(r#"tag ] <img onerror='x'> {forged}"#),
        );
    }
}
