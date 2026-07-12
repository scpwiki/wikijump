//! Render-local provenance for compatibility text hidden from FTML.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub(super) const COMPAT_TEXT_MARKER_PREFIX: &str = "WIKIJUMPWIKIDOTCOMPATTEXT";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct CompatTextFragments {
    namespace: String,
    fragments: Vec<String>,
}

impl CompatTextFragments {
    pub(super) fn new(untrusted_source: &str) -> Self {
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

    pub(super) fn push(&mut self, text: &str) -> String {
        let index = self.fragments.len();
        self.fragments.push(text.to_owned());
        format!("{}{index}X", self.namespace)
    }

    pub(super) fn restore(&self, text: &str) -> String {
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
}
