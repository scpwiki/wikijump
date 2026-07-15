//! Attachment ownership carried across Wikidot include expansion.

use std::collections::HashMap;
use std::ops::Range;
use std::sync::LazyLock;

use ftml::tree::VariableMap;
use regex::Regex;
use uuid::Uuid;

use super::literal_regions::LiteralRegionIndex;
use super::percent_encoding::percent_encode_path_segment;

static IMAGE_OPEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)\[\[(?:[=<>]|f[<>])?image(?P<separator>[ \t]+)"#).unwrap()
});
static INCLUDE_OPEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)\[\[include(?:[ \t\r\n]+|\[!--.*?--\])+"#).unwrap()
});
static VARIABLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^\{\$(?P<name>[A-Za-z0-9_-]+)\}$"#).unwrap());
static ATTACHMENT_MARKER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"__wj_attachment_[0-9a-f]{32}__"#).unwrap());
const MAX_ATTACHMENT_DIRECTIVE_NESTING: usize = 64;

#[derive(Clone, Debug)]
struct DirectiveHead {
    whole: Range<usize>,
    body: Range<usize>,
}

#[derive(Clone, Debug)]
struct AttachmentValue<'a> {
    raw: &'a str,
    semantic: &'a str,
    range: Range<usize>,
}

#[derive(Clone, Debug)]
pub(super) struct WikidotIncludeArgument<'a> {
    pub(super) raw_key: &'a str,
    pub(super) value: &'a str,
    value_range: Range<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AttachmentOwner {
    pub(super) site_slug: String,
    pub(super) page_slug: String,
}

pub(super) type AttachmentVariableOwners = HashMap<String, AttachmentOwner>;

#[derive(Debug, Default)]
pub(super) struct AttachmentProvenanceRegistry {
    entries: HashMap<String, AttachmentProvenanceEntry>,
}

#[derive(Debug)]
struct AttachmentProvenanceEntry {
    value: String,
    restore_value: String,
    owner: AttachmentOwner,
}

impl AttachmentProvenanceRegistry {
    fn issue(
        &mut self,
        value: &str,
        restore_value: String,
        owner: &AttachmentOwner,
    ) -> String {
        let marker = format!("__wj_attachment_{}__", Uuid::new_v4().simple());
        self.entries.insert(
            marker.clone(),
            AttachmentProvenanceEntry {
                value: value.to_owned(),
                restore_value,
                owner: owner.clone(),
            },
        );
        marker
    }

    pub(super) fn decode(&self, value: &str) -> Option<(&String, &AttachmentOwner)> {
        self.entries
            .get(value.trim())
            .map(|entry| (&entry.value, &entry.owner))
    }

    pub(super) fn restore_unresolved(&mut self, source: &mut String) {
        if self.entries.is_empty() {
            return;
        }
        if ATTACHMENT_MARKER.is_match(source) {
            *source = ATTACHMENT_MARKER
                .replace_all(source, |matched: &regex::Captures<'_>| {
                    let marker = matched.get(0).expect("whole marker match").as_str();
                    self.entries.get(marker).map_or_else(
                        || marker.to_owned(),
                        |entry| entry.restore_value.clone(),
                    )
                })
                .into_owned();
        }
        self.entries.clear();
    }
}

pub(super) fn protect_forwarded_attachment_variables(
    source: &mut String,
    variables: &VariableMap<'_>,
    owners: &AttachmentVariableOwners,
    registry: &mut AttachmentProvenanceRegistry,
) {
    let original = source.clone();
    let literals = LiteralRegionIndex::new_wikidot_syntax(&original);
    let mut replacements = Vec::new();
    for head in directive_heads(&original, &INCLUDE_OPEN, true) {
        if literals.contains(head.whole.start)
            || nested_in_current_line_head(&original, head.whole.start)
        {
            continue;
        }
        let body = &original[head.body.clone()];
        let argument_delimiter = include_page_ref_end(body);
        let argument_start = if body.as_bytes().get(argument_delimiter) == Some(&b'|') {
            argument_delimiter
        } else {
            skip_include_space(body, argument_delimiter)
        };
        for argument in include_argument_values(body, argument_start) {
            let Some(variable) = VARIABLE.captures(argument.semantic) else {
                continue;
            };
            let name = variable.name("name").unwrap().as_str();
            let (Some(variable_value), Some(owner)) =
                (variables.get(name), owners.get(name))
            else {
                continue;
            };
            let ordinary_value = preserve_argument_quotes(
                argument.raw,
                trim_forwarded_variable_value(variable_value),
            );
            let restore_value = format!(
                "{}{}",
                ordinary_value,
                &body[argument.range.start + argument.raw.len()..argument.range.end],
            );
            replacements.push((
                head.body.start + argument.range.start
                    ..head.body.start + argument.range.end,
                registry.issue(&ordinary_value, restore_value, owner),
            ));
        }
    }
    apply_replacements(source, replacements);
}

pub(super) fn qualify_relative_image_variable_attachments(
    source: &mut String,
    variables: &VariableMap<'_>,
    owners: &AttachmentVariableOwners,
) {
    if !IMAGE_OPEN.is_match(source) {
        return;
    }
    let original = source.clone();
    let literals = LiteralRegionIndex::new_wikidot_syntax(&original);
    let mut replacements = Vec::new();
    for head in directive_heads(&original, &IMAGE_OPEN, false) {
        if literals.contains(head.whole.start)
            || nested_in_current_line_head(&original, head.whole.start)
        {
            continue;
        }
        let body = &original[head.body.clone()];
        let (target, links) = image_attachment_values(body);
        if let Some(target) = target {
            qualify_variable(
                head.body.start,
                &target,
                variables,
                owners,
                &mut replacements,
            );
        }
        for link in links {
            qualify_variable(
                head.body.start,
                &link,
                variables,
                owners,
                &mut replacements,
            );
        }
    }
    apply_replacements(source, replacements);
}

fn qualify_variable(
    base: usize,
    target: &AttachmentValue<'_>,
    variables: &VariableMap<'_>,
    owners: &AttachmentVariableOwners,
    replacements: &mut Vec<(Range<usize>, String)>,
) {
    let Some(variable) = VARIABLE.captures(target.semantic) else {
        return;
    };
    let name = variable.name("name").unwrap().as_str();
    let (Some(value), Some(owner)) = (variables.get(name), owners.get(name)) else {
        return;
    };
    let Some(value) = semantic_attachment_value(value) else {
        return;
    };
    if relative(value) {
        replacements.push((
            base + target.range.start..base + target.range.end,
            owned_url(owner, value),
        ));
    }
}

pub(super) fn semantic_attachment_value(value: &str) -> Option<&str> {
    let value = value.trim();
    let first = value.as_bytes().first().copied();
    let last = value.as_bytes().last().copied();
    let starts_quote = first.is_some_and(|byte| matches!(byte, b'"' | b'\''));
    let ends_quote = last.is_some_and(|byte| matches!(byte, b'"' | b'\''));

    match (starts_quote, ends_quote) {
        (false, false) => Some(value),
        (true, true)
            if value.len() >= 2 && first == last && !terminal_quote_is_escaped(value) =>
        {
            Some(&value[1..value.len() - 1])
        }
        _ => None,
    }
}

fn directive_heads(
    source: &str,
    opening: &Regex,
    allow_multiline: bool,
) -> Vec<DirectiveHead> {
    let mut heads = Vec::new();
    let mut search_start = 0;
    while let Some(found) = opening.find(&source[search_start..]) {
        let whole_start = search_start + found.start();
        let body_start = search_start + found.end();
        let recovery_start = allow_multiline
            .then(|| opening.find(&source[body_start..]))
            .flatten()
            .map(|next| body_start + next.start())
            .filter(|next| {
                source[..*next]
                    .rsplit_once('\n')
                    .is_some_and(|(_, prefix)| prefix.trim().is_empty())
            });
        let scan_limit = if allow_multiline {
            recovery_start.unwrap_or(source.len())
        } else {
            source[body_start..]
                .find('\n')
                .map_or(source.len(), |line_end| body_start + line_end)
        };
        let Some(whole_end) = find_wikidot_directive_end(source, body_start, scan_limit)
        else {
            search_start = recovery_start.unwrap_or_else(|| {
                source[body_start..]
                    .find('\n')
                    .map_or(source.len(), |line_end| body_start + line_end + 1)
            });
            continue;
        };
        heads.push(DirectiveHead {
            whole: whole_start..whole_end,
            body: body_start..whole_end - 2,
        });
        search_start = whole_end;
    }
    heads
}

pub(super) fn find_wikidot_directive_end(
    source: &str,
    mut offset: usize,
    scan_limit: usize,
) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut quote = None;
    let mut depth = 1usize;
    while offset < scan_limit {
        if let Some(expected) = quote {
            if bytes[offset] == expected && !quote_is_escaped_at(source, offset) {
                quote = None;
            }
            offset = advance_one_char(source, offset);
            continue;
        }

        if matches!(bytes[offset], b'"' | b'\'') {
            quote = Some(bytes[offset]);
            offset += 1;
        } else if offset + 4 <= scan_limit
            && source[offset..scan_limit].starts_with("[!--")
        {
            let close = source[offset + 4..scan_limit].find("--]")?;
            offset += 4 + close + 3;
        } else if offset + 3 <= scan_limit
            && source[offset..scan_limit].starts_with("[[[")
        {
            let close = source[offset + 3..scan_limit].find("]]]")?;
            offset += 3 + close + 3;
        } else if offset + 2 <= scan_limit && source[offset..scan_limit].starts_with("[[")
        {
            depth = depth.checked_add(1)?;
            if depth > MAX_ATTACHMENT_DIRECTIVE_NESTING {
                return None;
            }
            offset += 2;
        } else if source[offset..].starts_with("]]") {
            if offset + 2 > scan_limit {
                return None;
            }
            depth -= 1;
            offset += 2;
            if depth == 0 {
                return Some(offset);
            }
        } else if bytes[offset] == b'[' {
            let close = source[offset + 1..scan_limit].find(']')?;
            offset += 1 + close + 1;
        } else {
            offset = advance_one_char(source, offset);
        }
    }
    None
}

fn include_argument_values(
    body: &str,
    argument_start: usize,
) -> Vec<AttachmentValue<'_>> {
    let mut values = Vec::new();
    let mut segment_start = argument_start;
    loop {
        let Some(segment_end) = next_top_level_pipe(body, segment_start) else {
            return Vec::new();
        };
        let segment_end = segment_end.unwrap_or(body.len());
        let segment = &body[segment_start..segment_end];
        if let Some(value) = exact_include_argument_value(segment) {
            values.push(AttachmentValue {
                raw: value.raw,
                semantic: value.semantic,
                range: segment_start + value.range.start..segment_start + value.range.end,
            });
        }

        if segment_end == body.len() {
            break;
        }
        segment_start = segment_end + 1;
    }
    values
}

fn include_page_ref_end(body: &str) -> usize {
    let mut offset = 0;
    while offset < body.len() {
        if body.as_bytes()[offset].is_ascii_whitespace()
            || body.as_bytes()[offset] == b'|'
            || body[offset..].starts_with("[!--")
        {
            break;
        }
        offset = advance_one_char(body, offset);
    }
    offset
}

fn exact_include_argument_value(segment: &str) -> Option<AttachmentValue<'_>> {
    let argument = parse_wikidot_include_argument(segment)?;
    Some(AttachmentValue {
        raw: argument.value,
        semantic: semantic_attachment_value(argument.value)?,
        range: argument.value_range.start..segment.len(),
    })
}

pub(super) fn parse_wikidot_include_argument(
    segment: &str,
) -> Option<WikidotIncludeArgument<'_>> {
    let mut offset = skip_include_space(segment, 0);
    let key_start = offset;
    while offset < segment.len()
        && (segment.as_bytes()[offset].is_ascii_alphanumeric()
            || matches!(segment.as_bytes()[offset], b'_' | b'-'))
    {
        offset += 1;
    }
    if key_start == offset {
        return None;
    }
    let raw_key = &segment[key_start..offset];
    offset = skip_include_space(segment, offset);
    if segment.as_bytes().get(offset) != Some(&b'=') {
        return None;
    }
    offset = skip_include_space(segment, offset + 1);
    let value_start = offset;
    let value_end = trim_include_space_end(segment, value_start)?;
    Some(WikidotIncludeArgument {
        raw_key,
        value: &segment[value_start..value_end],
        value_range: value_start..value_end,
    })
}

pub(super) fn wikidot_include_segment_is_space(segment: &str) -> bool {
    skip_include_space(segment, 0) == segment.len()
}

fn skip_include_space(source: &str, mut offset: usize) -> usize {
    loop {
        offset = skip_ascii_whitespace(source, offset);
        if !source[offset..].starts_with("[!--") {
            return offset;
        }
        let Some(close) = source[offset + 4..].find("--]") else {
            return offset;
        };
        offset += 4 + close + 3;
    }
}

fn trim_include_space_end(source: &str, minimum: usize) -> Option<usize> {
    let mut offset = minimum;
    let mut semantic_end = minimum;
    while offset < source.len() {
        if source.as_bytes()[offset].is_ascii_whitespace() {
            offset += 1;
        } else if source[offset..].starts_with("[!--") {
            let close = source[offset + 4..].find("--]")?;
            offset += 4 + close + 3;
        } else {
            offset = advance_one_char(source, offset);
            semantic_end = offset;
        }
    }
    Some(semantic_end)
}

pub(super) fn split_wikidot_include_argument_segments(source: &str) -> Option<Vec<&str>> {
    let mut segments = Vec::new();
    let mut segment_start = 0;
    while let Some(segment_end) = next_top_level_pipe(source, segment_start)? {
        segments.push(&source[segment_start..segment_end]);
        segment_start = segment_end + 1;
    }
    segments.push(&source[segment_start..]);
    Some(segments)
}

fn next_top_level_pipe(source: &str, mut offset: usize) -> Option<Option<usize>> {
    let bytes = source.as_bytes();
    let mut quote = None;
    while offset < bytes.len() {
        if let Some(expected) = quote {
            if bytes[offset] == expected && !quote_is_escaped_at(source, offset) {
                quote = None;
            }
            offset = advance_one_char(source, offset);
            continue;
        }
        if matches!(bytes[offset], b'"' | b'\'') {
            quote = Some(bytes[offset]);
            offset += 1;
        } else if source[offset..].starts_with("[!--") {
            let close = source[offset + 4..].find("--]")?;
            offset += 4 + close + 3;
        } else if source[offset..].starts_with("[[[") {
            let close = source[offset + 3..].find("]]]")?;
            offset += 3 + close + 3;
        } else if source[offset..].starts_with("[[") {
            offset = find_wikidot_directive_end(source, offset + 2, source.len())?;
        } else if bytes[offset] == b'[' {
            let close = source[offset + 1..].find(']')?;
            offset += 1 + close + 1;
        } else if bytes[offset] == b'|' {
            return Some(Some(offset));
        } else {
            offset = advance_one_char(source, offset);
        }
    }
    Some(None)
}

fn preserve_argument_quotes(raw: &str, value: &str) -> String {
    if raw.len() >= 2
        && matches!(raw.as_bytes()[0], b'"' | b'\'')
        && raw.as_bytes()[0] == raw.as_bytes()[raw.len() - 1]
    {
        let quote = raw.as_bytes()[0] as char;
        format!("{quote}{value}{quote}")
    } else {
        value.to_owned()
    }
}

fn trim_forwarded_variable_value(value: &str) -> &str {
    value.trim_end_matches([' ', '\t', '\r', '\n'])
}

fn image_attachment_values(
    body: &str,
) -> (Option<AttachmentValue<'_>>, Vec<AttachmentValue<'_>>) {
    let target_start = skip_ascii_whitespace(body, 0);
    let Some((target, mut offset)) = parse_image_value(body, target_start) else {
        return (None, Vec::new());
    };
    let mut links = Vec::new();

    while offset < body.len() {
        offset = skip_ascii_whitespace(body, offset);
        if offset == body.len() {
            break;
        }
        let key_start = offset;
        while offset < body.len()
            && (body.as_bytes()[offset].is_ascii_alphanumeric()
                || matches!(body.as_bytes()[offset], b'_' | b'-'))
        {
            offset += 1;
        }
        if key_start == offset {
            offset += body[offset..].chars().next().unwrap().len_utf8();
            continue;
        }
        let key = &body[key_start..offset];
        offset = skip_ascii_whitespace(body, offset);
        if body.as_bytes().get(offset) != Some(&b'=') {
            while offset < body.len() && !body.as_bytes()[offset].is_ascii_whitespace() {
                offset += 1;
            }
            continue;
        }
        offset = skip_ascii_whitespace(body, offset + 1);
        let Some((value, next)) = parse_image_value(body, offset) else {
            break;
        };
        if key.eq_ignore_ascii_case("link") {
            links.push(value);
        }
        offset = next;
    }

    (Some(target), links)
}

fn parse_image_value(source: &str, start: usize) -> Option<(AttachmentValue<'_>, usize)> {
    if start >= source.len() {
        return None;
    }
    let bytes = source.as_bytes();
    let end = if matches!(bytes[start], b'"' | b'\'') {
        let quote = bytes[start];
        let mut offset = start + 1;
        let mut close = None;
        while offset < bytes.len() {
            if bytes[offset] == quote && !quote_is_escaped_at(source, offset) {
                close = Some(offset + 1);
                break;
            }
            offset += 1;
        }
        close?
    } else {
        let mut offset = start;
        while offset < bytes.len() && !bytes[offset].is_ascii_whitespace() {
            offset += 1;
        }
        offset
    };
    if end < source.len() && !source.as_bytes()[end].is_ascii_whitespace() {
        return None;
    }
    let raw = &source[start..end];
    let semantic = semantic_attachment_value(raw)?;
    Some((
        AttachmentValue {
            raw,
            semantic,
            range: start..end,
        },
        end,
    ))
}

fn skip_ascii_whitespace(source: &str, mut offset: usize) -> usize {
    while offset < source.len() && source.as_bytes()[offset].is_ascii_whitespace() {
        offset += 1;
    }
    offset
}

fn advance_one_char(source: &str, offset: usize) -> usize {
    offset
        + source[offset..]
            .chars()
            .next()
            .expect("offset is inside source")
            .len_utf8()
}

fn quote_is_escaped_at(source: &str, quote: usize) -> bool {
    source.as_bytes()[..quote]
        .iter()
        .rev()
        .take_while(|byte| **byte == b'\\')
        .count()
        % 2
        == 1
}

fn terminal_quote_is_escaped(value: &str) -> bool {
    value.as_bytes()[..value.len() - 1]
        .iter()
        .rev()
        .take_while(|byte| **byte == b'\\')
        .count()
        % 2
        == 1
}

pub(super) fn qualify_included_relative_image_attachments(
    source: &mut String,
    site: &str,
    page: &str,
) {
    if !IMAGE_OPEN.is_match(source) {
        return;
    }
    let original = source.clone();
    let literals = LiteralRegionIndex::new_wikidot_syntax(&original);
    let owner = AttachmentOwner {
        site_slug: site.to_owned(),
        page_slug: page.to_owned(),
    };
    let mut replacements = Vec::new();
    for head in directive_heads(&original, &IMAGE_OPEN, false) {
        if literals.contains(head.whole.start)
            || nested_in_current_line_head(&original, head.whole.start)
        {
            continue;
        }
        let body = &original[head.body.clone()];
        let (target, links) = image_attachment_values(body);
        if let Some(target) = target {
            qualify_literal(head.body.start, &target, &owner, &mut replacements);
        }
        for link in links {
            qualify_literal(head.body.start, &link, &owner, &mut replacements);
        }
    }
    apply_replacements(source, replacements);
}

fn qualify_literal(
    base: usize,
    target: &AttachmentValue<'_>,
    owner: &AttachmentOwner,
    replacements: &mut Vec<(Range<usize>, String)>,
) {
    if relative(target.semantic) {
        replacements.push((
            base + target.range.start..base + target.range.end,
            owned_url(owner, target.semantic),
        ));
    }
}

pub(super) fn owned_url(owner: &AttachmentOwner, value: &str) -> String {
    format!(
        "https://{}.wikidot.com/local--files/{}/{}",
        owner.site_slug,
        owner.page_slug,
        percent_encode_path_segment(value),
    )
}
fn apply_replacements(
    source: &mut String,
    mut replacements: Vec<(Range<usize>, String)>,
) {
    replacements.sort_by_key(|(range, _)| range.start);
    debug_assert!(
        replacements
            .windows(2)
            .all(|pair| pair[0].0.end <= pair[1].0.start),
        "attachment replacements must not overlap",
    );
    for (range, value) in replacements.into_iter().rev() {
        source.replace_range(range, &value);
    }
}
fn nested_in_current_line_head(source: &str, offset: usize) -> bool {
    let line_start = source[..offset].rfind('\n').map_or(0, |line| line + 1);
    let mut cursor = line_start;
    let mut depth = 0usize;
    while cursor < offset {
        if source[cursor..offset].starts_with("[!--") {
            let Some(close) = source[cursor + 4..offset].find("--]") else {
                return true;
            };
            cursor += 4 + close + 3;
        } else if source[cursor..offset].starts_with("[[") {
            depth = depth.saturating_add(1);
            cursor += 2;
        } else if source[cursor..offset].starts_with("]]") {
            depth = depth.saturating_sub(1);
            cursor += 2;
        } else {
            cursor = advance_one_char(source, cursor);
        }
    }
    depth > 0
}
pub(super) fn relative(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && !value.starts_with('#')
        && !value.starts_with("//")
        && !value.contains("{$")
        && !has_scheme(value)
}
fn has_scheme(value: &str) -> bool {
    let Some((scheme, _)) = value.split_once(':') else {
        return false;
    };
    let mut chars = scheme.chars();
    chars.next().is_some_and(|ch| ch.is_ascii_alphabetic())
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ftml::prelude::*;
    use ftml::render::{Render, html::HtmlRender};
    use ftml::tree::VariableMap;
    use std::borrow::Cow;

    #[test]
    fn target_and_link_are_independent_and_same_values_do_not_collide() {
        let variables = [
            (Cow::Borrowed("image"), Cow::Borrowed("same.png")),
            (Cow::Borrowed("href"), Cow::Borrowed("same.png")),
        ]
        .into_iter()
        .collect::<VariableMap<'_>>();
        let owners = [
            (
                "image".to_owned(),
                AttachmentOwner {
                    site_slug: "one".into(),
                    page_slug: "fragment:one".into(),
                },
            ),
            (
                "href".to_owned(),
                AttachmentOwner {
                    site_slug: "two".into(),
                    page_slug: "fragment:two".into(),
                },
            ),
        ]
        .into_iter()
        .collect();
        let mut source =
            "[[image {$image} link={$href}]]\n[[image fixed.png link={$href}]]"
                .to_owned();
        qualify_relative_image_variable_attachments(&mut source, &variables, &owners);
        assert!(
            source.contains("one.wikidot.com/local--files/fragment:one/same.png"),
            "{source}"
        );
        assert_eq!(
            source
                .matches("two.wikidot.com/local--files/fragment:two/same.png")
                .count(),
            2,
            "{source}"
        );
    }

    #[test]
    fn quoted_absolute_include_value_is_not_reowned_as_an_attachment() {
        let variables = [(
            Cow::Borrowed("link"),
            Cow::Borrowed(
                r#""https://scp-wiki.wdfiles.com/local--files/fragment:2117-1/2117.png""#,
            ),
        )]
        .into_iter()
        .collect::<VariableMap<'_>>();
        let owners = [(
            "link".to_owned(),
            AttachmentOwner {
                site_slug: "scp-wiki".into(),
                page_slug: "fragment:2117-1".into(),
            },
        )]
        .into_iter()
        .collect();
        let mut source = "[[image 2117.png link={$link}]]".to_owned();

        qualify_relative_image_variable_attachments(&mut source, &variables, &owners);

        assert_eq!(source, "[[image 2117.png link={$link}]]");
    }

    #[test]
    fn quoted_relative_include_value_uses_its_provenance_owner() {
        let variables = [(Cow::Borrowed("name"), Cow::Borrowed(r#""2117.png""#))]
            .into_iter()
            .collect::<VariableMap<'_>>();
        let owners = [(
            "name".to_owned(),
            AttachmentOwner {
                site_slug: "scp-wiki".into(),
                page_slug: "fragment:2117-1".into(),
            },
        )]
        .into_iter()
        .collect();
        let mut source = "[[image {$name}]]".to_owned();

        qualify_relative_image_variable_attachments(&mut source, &variables, &owners);

        assert_eq!(
            source,
            "[[image https://scp-wiki.wikidot.com/local--files/fragment:2117-1/2117.png]]",
        );
    }

    #[test]
    fn malformed_or_escaped_variable_quotes_fail_closed() {
        let owner = AttachmentOwner {
            site_slug: "scp-wiki".into(),
            page_slug: "fragment:quote-boundary".into(),
        };
        for value in [
            r#""one-sided.png"#,
            r#"one-sided.png""#,
            r#""mismatched.png'"#,
            r#"'mismatched.png""#,
            r#""escaped.png\""#,
            r#"'escaped.png\'"#,
        ] {
            let variables = [(Cow::Borrowed("name"), Cow::Borrowed(value))]
                .into_iter()
                .collect::<VariableMap<'_>>();
            let owners = [("name".to_owned(), owner.clone())].into_iter().collect();
            let mut source = "[[image {$name}]]".to_owned();

            qualify_relative_image_variable_attachments(&mut source, &variables, &owners);

            assert_eq!(source, "[[image {$name}]]", "value={value:?}");
        }
    }

    #[test]
    fn forwarded_occurrences_use_opaque_registry_entries() {
        let variables = [
            (Cow::Borrowed("left"), Cow::Borrowed("same.png")),
            (Cow::Borrowed("right"), Cow::Borrowed("same.png")),
        ]
        .into_iter()
        .collect::<VariableMap<'_>>();
        let owners = [
            (
                "left".to_owned(),
                AttachmentOwner {
                    site_slug: "one".into(),
                    page_slug: "one".into(),
                },
            ),
            (
                "right".to_owned(),
                AttachmentOwner {
                    site_slug: "two".into(),
                    page_slug: "two".into(),
                },
            ),
        ]
        .into_iter()
        .collect();
        let mut registry = AttachmentProvenanceRegistry::default();
        let mut source = "[[include child | a={$left} | b={$right}]]".to_owned();
        protect_forwarded_attachment_variables(
            &mut source,
            &variables,
            &owners,
            &mut registry,
        );
        let markers = ATTACHMENT_MARKER
            .find_iter(&source)
            .map(|matched| matched.as_str())
            .collect::<Vec<_>>();
        assert_eq!(markers.len(), 2, "{source}");
        assert_ne!(markers[0], markers[1]);
        assert_ne!(
            registry.decode(markers[0]).unwrap().1,
            registry.decode(markers[1]).unwrap().1
        );
    }

    #[test]
    fn exact_forwarding_preserves_quotes_and_composites_use_normal_substitution() {
        let variables = [(Cow::Borrowed("asset"), Cow::Borrowed("space file.png \r\n"))]
            .into_iter()
            .collect::<VariableMap<'_>>();
        let owner = AttachmentOwner {
            site_slug: "origin".into(),
            page_slug: "fragment:origin".into(),
        };
        let owners = [("asset".to_owned(), owner)].into_iter().collect();
        let mut registry = AttachmentProvenanceRegistry::default();
        let mut source = concat!(
            "[[include child | exact={$asset} | quoted=\"{$asset}\" | ",
            "composite=thumb-{$asset}]]",
        )
        .to_owned();

        protect_forwarded_attachment_variables(
            &mut source,
            &variables,
            &owners,
            &mut registry,
        );

        assert_eq!(registry.entries.len(), 2, "{source}");
        assert!(source.contains("composite=thumb-{$asset}"), "{source}");
        assert!(!source.contains("thumb-__wj_attachment_"), "{source}");
        source = source.replace(
            "{$asset}",
            trim_forwarded_variable_value(variables.get("asset").unwrap()),
        );
        registry.restore_unresolved(&mut source);
        assert_eq!(
            source,
            concat!(
                "[[include child | exact=space file.png | ",
                "quoted=\"space file.png\" | composite=thumb-space file.png]]",
            ),
        );
        assert!(!source.contains("__wj_attachment_"));
    }

    #[test]
    fn exact_forwarding_scans_quoted_closers_nested_links_and_recovers_after_malformed_head()
     {
        let variables = [(Cow::Borrowed("asset"), Cow::Borrowed("origin.png"))]
            .into_iter()
            .collect::<VariableMap<'_>>();
        let owners = [(
            "asset".to_owned(),
            AttachmentOwner {
                site_slug: "origin".into(),
                page_slug: "fragment:origin".into(),
            },
        )]
        .into_iter()
        .collect();
        let mut registry = AttachmentProvenanceRegistry::default();
        let mut source = concat!(
            "[[include broken | value=\"unterminated\n",
            "[[include child | caption=\"quoted ]] | closer\" | ",
            "nested=[[[page|日本語 label]]] | unicode=日本語 | exact={$asset}]]\n",
            "[[include child no_pipe={$asset}]]",
        )
        .to_owned();

        protect_forwarded_attachment_variables(
            &mut source,
            &variables,
            &owners,
            &mut registry,
        );

        assert_eq!(registry.entries.len(), 2, "{source}");
        assert!(
            source.contains("nested=[[[page|日本語 label]]]"),
            "{source}"
        );
        assert!(source.contains("unicode=日本語"), "{source}");
        registry.restore_unresolved(&mut source);
        assert!(source.contains("exact=origin.png]]"), "{source}");
        assert!(source.ends_with("no_pipe=origin.png]]"), "{source}");

        let mut malformed_stress = (0..2_048)
            .map(|index| format!("[[include broken-{index} | value=[[[unterminated"))
            .collect::<Vec<_>>()
            .join("\n");
        malformed_stress.push_str("\n[[include child no_pipe={$asset}]]");
        let mut stress_registry = AttachmentProvenanceRegistry::default();
        protect_forwarded_attachment_variables(
            &mut malformed_stress,
            &variables,
            &owners,
            &mut stress_registry,
        );
        assert_eq!(stress_registry.entries.len(), 1);
        stress_registry.restore_unresolved(&mut malformed_stress);
        assert!(malformed_stress.ends_with("no_pipe=origin.png]]"));

        let mut comment_space = concat!(
            "[[include\n[!-- opening gap --]\nchild ",
            "[!-- page gap [x] | still comment --] name [!-- key gap --] = ",
            "[!-- value gap --] {$asset} [!-- tail gap --] | ",
            "[!-- disabled={$asset}|still-disabled --] literal=x]]\n",
            "[[include[!-- separator comment --] child | name={$asset}]]",
        )
        .to_owned();
        let mut comment_registry = AttachmentProvenanceRegistry::default();
        protect_forwarded_attachment_variables(
            &mut comment_space,
            &variables,
            &owners,
            &mut comment_registry,
        );
        assert_eq!(comment_registry.entries.len(), 2, "{comment_space}");
        let first_marker = ATTACHMENT_MARKER
            .find(&comment_space)
            .expect("comment-space exact argument should issue a marker")
            .as_str();
        assert!(comment_registry.decode(first_marker).is_some());
        assert!(!comment_space.contains("tail gap"), "{comment_space}");
        assert!(
            comment_space.contains("disabled={$asset}|still-disabled"),
            "{comment_space}",
        );
        comment_registry.restore_unresolved(&mut comment_space);
        assert_eq!(comment_space.matches("origin.png").count(), 2);
        assert!(
            comment_space.contains("origin.png [!-- tail gap --] |"),
            "{comment_space}",
        );
        assert!(
            comment_space.contains("disabled={$asset}|still-disabled"),
            "{comment_space}",
        );
    }

    #[test]
    fn image_head_parsing_keeps_target_and_attributes_disjoint() {
        let mut source = concat!(
            "[[image \"my link=foo.png\" alt=\"quoted link=not.png\" ",
            "data-link=data.png link=\"full link=bar.png\" link=second.png]]\n",
            "[[image https://example.test/x?link=query.png]]\n",
            "[[>image 日本語.png link=全体画像.png]]",
        )
        .to_owned();

        qualify_included_relative_image_attachments(
            &mut source,
            "site",
            "fragment:owner",
        );

        for filename in [
            "my%20link%3Dfoo.png",
            "full%20link%3Dbar.png",
            "second.png",
            "%E6%97%A5%E6%9C%AC%E8%AA%9E.png",
            "%E5%85%A8%E4%BD%93%E7%94%BB%E5%83%8F.png",
        ] {
            assert_eq!(
                source
                    .matches(&format!("/local--files/fragment:owner/{filename}"))
                    .count(),
                1,
                "{source}",
            );
        }
        assert!(source.contains("alt=\"quoted link=not.png\""), "{source}");
        assert!(source.contains("data-link=data.png"), "{source}");
        assert!(
            source.contains("https://example.test/x?link=query.png"),
            "{source}",
        );
        assert!(!source.contains("/local--files/fragment:owner/query.png"));
    }

    #[test]
    fn every_ftml_image_modifier_uses_the_included_attachment_owner() {
        let mut source = [
            "[[image plain.png]]",
            "[[=image centered.png]]",
            "[[<image left.png]]",
            "[[>image right.png]]",
            "[[f<image float-left.png]]",
            "[[F>IMAGE float-right.png]]",
        ]
        .join("\n");

        qualify_included_relative_image_attachments(
            &mut source,
            "site",
            "fragment:owner",
        );

        assert_eq!(
            source
                .matches("site.wikidot.com/local--files/fragment:owner/")
                .count(),
            6,
            "{source}",
        );
        for modifier in [
            "[[=image ",
            "[[<image ",
            "[[>image ",
            "[[f<image ",
            "[[F>IMAGE ",
        ] {
            assert!(source.contains(modifier), "{source}");
        }
    }

    #[test]
    fn malformed_image_values_fail_closed_without_hiding_a_later_valid_head() {
        let malformed = [
            r#"[[image "one-sided.png]]"#,
            r#"[[image one-sided.png" link=stolen.png]]"#,
            r#"[[image 'mismatched.png" link=stolen.png]]"#,
            r#"[[image "escaped.png\"]]"#,
        ];
        let mut source = format!("{}\n[[image valid.png]]", malformed.join("\n"));

        qualify_included_relative_image_attachments(
            &mut source,
            "site",
            "fragment:owner",
        );

        for original in malformed {
            assert!(source.contains(original), "{source}");
        }
        assert_eq!(
            source.matches("/local--files/fragment:owner/").count(),
            1,
            "{source}"
        );
        assert!(
            source.contains("/local--files/fragment:owner/valid.png"),
            "{source}"
        );
        assert!(!source.contains("/local--files/fragment:owner/stolen.png"));

        let mut deeply_nested = format!(
            "[[image {}\n[[image recovered.png]]",
            "[[".repeat(MAX_ATTACHMENT_DIRECTIVE_NESTING + 1),
        );
        qualify_included_relative_image_attachments(
            &mut deeply_nested,
            "site",
            "fragment:owner",
        );
        assert!(
            deeply_nested.contains("/local--files/fragment:owner/recovered.png"),
            "{deeply_nested}",
        );
    }

    #[test]
    fn owned_url_encodes_each_path_segment_exactly_once() {
        let owner = AttachmentOwner {
            site_slug: "origin-site".into(),
            page_slug: "fragment:origin-key".into(),
        };
        let cases = [
            ("space name.png", "space%20name.png"),
            ("query?.png", "query%3F.png"),
            ("hash#.png", "hash%23.png"),
            ("percent%.png", "percent%25.png"),
            ("quote\"'.png", "quote%22%27.png"),
            ("bracket[].png", "bracket%5B%5D.png"),
            ("日本語.png", "%E6%97%A5%E6%9C%AC%E8%AA%9E.png"),
            ("already%20encoded.png", "already%2520encoded.png"),
        ];
        for (filename, encoded) in cases {
            assert_eq!(
                owned_url(&owner, filename),
                format!(
                    "https://origin-site.wikidot.com/local--files/fragment:origin-key/{encoded}"
                )
            );
        }
    }

    #[test]
    fn literal_regions_are_unchanged_and_unresolved_markers_restore_normal_values() {
        let mut source = concat!(
            "[!-- [[image comment.png link=comment-full.png]] --]\n",
            "[[code]]\n[[image code.png link=code-full.png]]\n[[/code]]\n",
            "[[span title=\"[[image attr.png link=attr-full.png]]\"]]x[[/span]]\n",
        )
        .to_owned();
        let original = source.clone();
        qualify_included_relative_image_attachments(&mut source, "site", "page");
        assert_eq!(source, original);

        let mut registry = AttachmentProvenanceRegistry::default();
        let marker = registry.issue(
            "hidden.png",
            "hidden.png".to_owned(),
            &AttachmentOwner {
                site_slug: "site".into(),
                page_slug: "page".into(),
            },
        );
        let mut unresolved = format!("before {marker} after");
        unresolved.push_str(" user __wj_attachment_00000000000000000000000000000000__");
        registry.restore_unresolved(&mut unresolved);
        assert_eq!(
            unresolved,
            concat!(
                "before hidden.png after user ",
                "__wj_attachment_00000000000000000000000000000000__",
            ),
        );
        assert!(registry.entries.is_empty());
    }

    #[test]
    fn final_rendered_src_and_href_preserve_encoded_filename_identity() {
        let filename = "space ?#%\"[]日本%20.png";
        let variables = [(Cow::Borrowed("file"), Cow::Borrowed(filename))]
            .into_iter()
            .collect::<VariableMap<'_>>();
        let owners = [(
            "file".to_owned(),
            AttachmentOwner {
                site_slug: "origin-site".into(),
                page_slug: "fragment:origin".into(),
            },
        )]
        .into_iter()
        .collect::<AttachmentVariableOwners>();
        let mut source = "[[image \"{$file}\" link=\"{$file}\"]]".to_owned();
        qualify_relative_image_variable_attachments(&mut source, &variables, &owners);

        ftml::preprocess(&mut source);
        let tokens = ftml::tokenize(&source);
        let page_info = ftml::data::PageInfo {
            page: Cow::Borrowed("consumer"),
            category: None,
            site: Cow::Borrowed("origin-site"),
            title: Cow::Borrowed("Consumer"),
            alt_title: None,
            score: ftml::data::ScoreValue::Integer(0),
            tags: Vec::new(),
            language: Cow::Borrowed("en"),
        };
        let settings = WikitextSettings::from_mode(WikitextMode::Page, Layout::Wikidot);
        let result = ftml::parse(&tokens, &page_info, &settings);
        let (tree, _) = result.into();
        let html = HtmlRender.render(&tree, &page_info, &settings).body;
        let expected = concat!(
            "https://origin-site.wikidot.com/local--files/fragment:origin/",
            "space%20%3F%23%25%22%5B%5D%E6%97%A5%E6%9C%AC%2520.png",
        );
        assert_eq!(html.matches(expected).count(), 2, "{html}");
    }
}
