//! Compatibility-local provenance for text hidden from FTML.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub(in crate::services::render) const COMPAT_TEXT_MARKER_PREFIX: &str =
    "WIKIJUMPWIKIDOTCOMPATTEXT";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(in crate::services::render) struct CompatTextFragments {
    namespace: String,
    fragments: Vec<String>,
}

impl CompatTextFragments {
    pub(in crate::services::render) fn new(untrusted_source: &str) -> Self {
        let namespace = loop {
            let candidate =
                format!("{COMPAT_TEXT_MARKER_PREFIX}{}I", Uuid::new_v4().as_simple());
            if !untrusted_source.contains(&candidate) {
                break candidate;
            }
        };
        Self {
            namespace,
            fragments: Vec::new(),
        }
    }

    pub(in crate::services::render) fn push(&mut self, text: &str) -> String {
        let index = self.fragments.len();
        self.fragments.push(text.to_owned());
        format!("{}{index}X", self.namespace)
    }

    pub(in crate::services::render) fn push_escaped_html_text(
        &mut self,
        text: &str,
    ) -> String {
        self.push(&escape_html_text(text))
    }

    pub(in crate::services::render) fn restore(&self, text: &str) -> String {
        if self.fragments.is_empty() || !text.contains(&self.namespace) {
            return text.to_owned();
        }

        let mut output = String::with_capacity(text.len());
        let mut cursor = 0;
        while let Some(offset) = text[cursor..].find(&self.namespace) {
            let start = cursor + offset;
            output.push_str(&text[cursor..start]);
            if let Some((index, len)) = self.marker_at(&text[start..]) {
                output.push_str(&self.fragments[index]);
                cursor = start + len;
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

fn escape_html_text(value: &str) -> String {
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
    use super::*;

    #[test]
    fn restores_only_registered_fragments_without_recursion() {
        let mut fragments = CompatTextFragments::new("authored source");
        let marker_shaped = format!("{COMPAT_TEXT_MARKER_PREFIX}deadbeefI0X");
        let first = fragments.push(&format!("{{$first-{marker_shaped}}}"));
        let second = fragments.push("{$second}");
        let foreign =
            format!("{COMPAT_TEXT_MARKER_PREFIX}ffffffffffffffffffffffffffffffffI0X");

        assert_eq!(
            fragments.restore(&format!("{first}|{second}|{foreign}")),
            format!("{{$first-{marker_shaped}}}|{{$second}}|{foreign}"),
        );
    }

    #[test]
    fn malformed_and_out_of_range_markers_remain_literal() {
        let mut fragments = CompatTextFragments::new("");
        let valid = fragments.push("{$valid}");
        let malformed = format!("{}nopeX", fragments.namespace);
        let out_of_range = format!("{}9X", fragments.namespace);

        assert_eq!(fragments.restore(&valid), "{$valid}");
        assert_eq!(fragments.restore(&malformed), malformed);
        assert_eq!(fragments.restore(&out_of_range), out_of_range);
    }

    #[test]
    fn escaped_html_text_uses_the_same_unforgeable_registry() {
        let authored_marker =
            format!("{COMPAT_TEXT_MARKER_PREFIX}ffffffffffffffffffffffffffffffffI0X");
        let mut fragments = CompatTextFragments::new(&authored_marker);
        let dangerous = fragments.push_escaped_html_text("<script>&\"'");
        let second = fragments.push_escaped_html_text("[[module ListPages]]");

        assert_eq!(
            fragments.restore(&format!("{dangerous}|{second}|{authored_marker}")),
            format!(
                "&lt;script&gt;&amp;&quot;&#39;|[[module ListPages]]|{authored_marker}"
            ),
        );
    }
}
