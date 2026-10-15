//! Compatibility-local provenance for text hidden from FTML.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
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

    fn push_owned(&mut self, text: String) -> (String, usize) {
        let index = self.fragments.len();
        self.fragments.push(text);
        (format!("{}{index}X", self.namespace), index)
    }

    pub(in crate::services::render) fn push(&mut self, text: &str) -> String {
        self.push_owned(text.to_owned()).0
    }

    pub(in crate::services::render) fn push_escaped_html_text(
        &mut self,
        text: &str,
    ) -> String {
        self.push_owned(escape_html_text(text)).0
    }

    pub(in crate::services::render) fn push_tracked_escaped_html_text(
        &mut self,
        text: &str,
    ) -> (String, usize) {
        self.push_owned(escape_html_text(text))
    }

    pub(in crate::services::render) fn logical_len_for_tracked_fragments(
        &self,
        text: &str,
        tracked: &BTreeSet<usize>,
    ) -> Option<usize> {
        if tracked.is_empty() || !text.contains(&self.namespace) {
            return Some(text.len());
        }

        let mut logical_len = text.len();
        let mut cursor = 0;
        while let Some(offset) = text[cursor..].find(&self.namespace) {
            let start = cursor + offset;
            if let Some((index, marker_len)) = self.marker_at(&text[start..]) {
                if tracked.contains(&index) {
                    logical_len = logical_len
                        .checked_sub(marker_len)?
                        .checked_add(self.fragments[index].len())?;
                }
                cursor = start + marker_len;
            } else {
                cursor = start + self.namespace.len();
            }
        }
        Some(logical_len)
    }

    pub(in crate::services::render) fn is_empty(&self) -> bool {
        self.fragments.is_empty()
    }

    pub(in crate::services::render) fn contains_marker(&self, text: &str) -> bool {
        !self.fragments.is_empty()
            && text
                .to_ascii_lowercase()
                .contains(&self.namespace.to_ascii_lowercase())
    }

    pub(in crate::services::render) fn restore(&self, text: &str) -> String {
        if self.fragments.is_empty() {
            return text.to_owned();
        }

        let restored = self.restore_namespace(text, &self.namespace, b'X', |fragment| {
            fragment.to_owned()
        });
        let lowercase = self.namespace.to_ascii_lowercase();
        self.restore_namespace(&restored, &lowercase, b'x', |fragment| {
            let mut slug = fragment.to_owned();
            wikidot_normalize::normalize(&mut slug);
            slug
        })
    }

    /// Restores markers inside a page slug, which FTML has already normalized.
    ///
    /// Slug normalization lowercases the marker, so the lowercase namespace is
    /// replaced with the fragment's own Wikidot page-name normalization. This is
    /// the result Wikidot produces for an unexpanded variable inside a link target.
    pub(in crate::services::render) fn restore_page_slug(&self, slug: &str) -> String {
        let mut restored = self.restore(slug);
        wikidot_normalize::normalize(&mut restored);
        restored
    }

    fn restore_namespace(
        &self,
        text: &str,
        namespace: &str,
        terminator: u8,
        mut replacement: impl FnMut(&str) -> String,
    ) -> String {
        if !text.contains(namespace) {
            return text.to_owned();
        }

        let mut output = String::with_capacity(text.len());
        let mut cursor = 0;
        while let Some(offset) = text[cursor..].find(namespace) {
            let start = cursor + offset;
            output.push_str(&text[cursor..start]);
            if let Some((index, len)) =
                self.marker_at_namespace(&text[start..], namespace, terminator)
            {
                output.push_str(&replacement(&self.fragments[index]));
                cursor = start + len;
            } else {
                output.push_str(namespace);
                cursor = start + namespace.len();
            }
        }
        output.push_str(&text[cursor..]);
        output
    }

    /// Like `restore`, but also matches markers that were lowercased, as in
    /// page link targets FTML has already slug-normalized.
    pub(in crate::services::render) fn restore_ignoring_case(
        &self,
        text: &str,
    ) -> String {
        if self.fragments.is_empty() {
            return text.to_owned();
        }
        let namespace = self.namespace.to_ascii_lowercase();
        let lowered = text.to_ascii_lowercase();
        if !lowered.contains(&namespace) {
            return text.to_owned();
        }

        let mut output = String::with_capacity(text.len());
        let mut cursor = 0;
        while let Some(offset) = lowered[cursor..].find(&namespace) {
            let start = cursor + offset;
            output.push_str(&text[cursor..start]);
            let suffix = &lowered[start + namespace.len()..];
            let digits = suffix.bytes().take_while(u8::is_ascii_digit).count();
            let index = (digits > 0 && suffix.as_bytes().get(digits) == Some(&b'x'))
                .then(|| suffix[..digits].parse::<usize>().ok())
                .flatten()
                .filter(|index| *index < self.fragments.len());
            match index {
                Some(index) => {
                    output.push_str(&self.fragments[index]);
                    cursor = start + namespace.len() + digits + 1;
                }
                None => {
                    output.push_str(&text[start..start + namespace.len()]);
                    cursor = start + namespace.len();
                }
            }
        }
        output.push_str(&text[cursor..]);
        output
    }

    fn marker_at(&self, text: &str) -> Option<(usize, usize)> {
        self.marker_at_namespace(text, &self.namespace, b'X')
    }

    fn marker_at_namespace(
        &self,
        text: &str,
        namespace: &str,
        terminator: u8,
    ) -> Option<(usize, usize)> {
        let suffix = text.strip_prefix(namespace)?;
        let digits = suffix.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 || suffix.as_bytes().get(digits) != Some(&terminator) {
            return None;
        }
        let index = suffix[..digits].parse::<usize>().ok()?;
        (index < self.fragments.len()).then_some((index, namespace.len() + digits + 1))
    }
}

pub(in crate::services::render) fn escape_html_text(value: &str) -> String {
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

    #[test]
    fn tracked_logical_length_counts_surviving_restored_fragments_once_per_marker() {
        let mut fragments = CompatTextFragments::new("");
        let (tracked, tracked_index) =
            fragments.push_tracked_escaped_html_text("[[<unsafe>&]]");
        let untracked = fragments.push("ordinary");
        let source = format!("{tracked}|{tracked}|{untracked}");
        let tracked_indices = BTreeSet::from([tracked_index]);
        let restored_tracked_len = escape_html_text("[[<unsafe>&]]").len();

        assert_eq!(
            fragments.logical_len_for_tracked_fragments(&source, &tracked_indices,),
            Some(restored_tracked_len * 2 + 2 + untracked.len()),
        );
        assert_eq!(
            fragments.logical_len_for_tracked_fragments(
                "marker was removed",
                &tracked_indices,
            ),
            Some("marker was removed".len()),
        );
    }

    #[test]
    fn restores_lowercased_markers_as_wikidot_page_slugs() {
        let mut fragments = CompatTextFragments::new("authored source");
        let url = fragments.push("{$previous-url}");
        let title = fragments.push("{$previous-title}");
        let lowercase_url = url.to_ascii_lowercase();

        // FTML lowercases a marker when it forms a page link target. The restored
        // target must match Wikidot's page-name normalization of the variable.
        assert_eq!(
            fragments.restore(&format!("<a href=\"/{lowercase_url}\">{title}</a>")),
            "<a href=\"/previous-url\">{$previous-title}</a>",
        );
        assert_eq!(fragments.restore_page_slug(&lowercase_url), "previous-url");
        assert_eq!(fragments.restore(&url), "{$previous-url}");
    }

    #[test]
    fn keeps_lowercase_marker_text_without_a_registered_fragment() {
        let fragments = CompatTextFragments::new("authored source");
        let lowercase = format!("{}0x", COMPAT_TEXT_MARKER_PREFIX.to_ascii_lowercase());
        assert_eq!(fragments.restore(&lowercase), lowercase);
    }
}
