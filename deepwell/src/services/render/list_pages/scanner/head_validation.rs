//! Module-head and argument-boundary validation for the ListPages scanner.

use super::{
    TextTokenCursor, WikidotArgumentValueKind, WikidotTagArgumentScan,
    classify_wikidot_right_bracket_and_advance_text_tokens,
    double_quote_ends_scanner_argument, is_module_argument_spacing,
    list_pages_bare_comparison_key_is_evidenced, list_pages_head_comment_end,
    list_pages_quote_ends_before_comment, normalize_module_head,
    project_list_pages_typography_in_place, quote_is_escaped,
    runtime_list_pages_key_is_supported, skip_horizontal_whitespace,
    skip_module_argument_spacing, wikidot_list_pages_arguments, wikidot_trimmed_name,
};
#[cfg(test)]
use super::{find_list_pages_module_matches, list_pages_runtime_head_can_execute};

pub(super) fn source_block_open_is_backslash_escaped(source: &str, start: usize) -> bool {
    let bytes = source.as_bytes();
    let mut cursor = start;
    let mut backslashes = 0usize;
    while cursor > 0 && bytes[cursor - 1] == b'\\' {
        cursor -= 1;
        backslashes += 1;
    }
    backslashes % 2 == 1
}

#[derive(Clone, Copy)]
pub(super) struct GenericHeadValidation {
    pub(super) valid: bool,
    pub(super) inspected: usize,
}

pub(super) fn validate_generic_head_arguments(
    bytes: &[u8],
    head_end: usize,
    argument_scan: WikidotTagArgumentScan,
    text_tokens: &TextTokenCursor,
) -> GenericHeadValidation {
    let mut text_tokens = text_tokens.clone();
    let mut cursor = argument_scan.name_end();
    let baseline_work = head_end.saturating_sub(cursor);
    let mut lookahead_work = 0usize;
    macro_rules! finish_validation {
        ($valid:expr) => {
            return GenericHeadValidation {
                valid: $valid,
                inspected: baseline_work.saturating_add(lookahead_work),
            }
        };
    }

    while cursor < head_end && argument_scan.in_positional_value(cursor) {
        cursor += 1;
    }

    loop {
        skip_module_argument_spacing(bytes, &mut cursor);
        if cursor == head_end {
            finish_validation!(true);
        }
        if cursor > head_end {
            finish_validation!(false);
        }

        let key_start = cursor;
        while cursor < head_end
            && (bytes[cursor].is_ascii_alphanumeric()
                || matches!(bytes[cursor], b'_' | b'-'))
        {
            cursor += 1;
        }
        if cursor == key_start {
            finish_validation!(false);
        }
        skip_horizontal_whitespace(bytes, &mut cursor);
        if cursor >= head_end || bytes[cursor] != b'=' {
            finish_validation!(false);
        }
        let equals = cursor;
        cursor += 1;
        skip_horizontal_whitespace(bytes, &mut cursor);
        if cursor >= head_end {
            finish_validation!(false);
        }

        match bytes[cursor] {
            b'"' => {
                cursor += 1;
                let mut closed = false;
                let mut trailing_backslashes = 0usize;
                while cursor < head_end {
                    if bytes[cursor] == b'"'
                        && !quote_is_escaped(bytes, cursor, &text_tokens)
                        && !text_tokens.advance_and_contains(cursor)
                    {
                        let (ends, inspected) = pinned_double_quote_ends_generic_argument(
                            bytes,
                            cursor,
                            head_end,
                            &text_tokens,
                        );
                        lookahead_work = lookahead_work.saturating_add(inspected);
                        if ends {
                            cursor += 1;
                            closed = true;
                            break;
                        }
                    }
                    if matches!(bytes[cursor], b'\n' | b'\r') {
                        if trailing_backslashes > 0 {
                            trailing_backslashes -= 1;
                            cursor = physical_line_resume(bytes, cursor);
                            continue;
                        }
                        finish_validation!(false);
                    }
                    if bytes[cursor] == b'\\' {
                        trailing_backslashes += 1;
                    } else {
                        trailing_backslashes = 0;
                    }
                    cursor += 1;
                }
                if !closed {
                    finish_validation!(false);
                }
            }
            _ if argument_scan.classify(bytes, equals, cursor)
                == WikidotArgumentValueKind::BareImageLink =>
            {
                let value_start = cursor;
                while cursor < head_end && !is_wikidot_head_spacing(bytes[cursor]) {
                    cursor += 1;
                }
                if cursor == value_start {
                    finish_validation!(false);
                }
            }
            _ => finish_validation!(false),
        }

        if cursor < head_end && !is_wikidot_head_spacing(bytes[cursor]) {
            finish_validation!(false);
        }
    }
}

pub(super) fn pinned_double_quote_ends_generic_argument(
    bytes: &[u8],
    quote: usize,
    head_end: usize,
    text_tokens: &TextTokenCursor,
) -> (bool, usize) {
    let mut text_tokens = text_tokens.clone();
    let start = quote + 1;
    let mut cursor = start;
    if cursor >= head_end {
        return (true, cursor.saturating_sub(start));
    }
    if matches!(bytes[cursor], b'\n' | b'\r') {
        return (true, 1);
    }
    if bytes[cursor] == b']'
        && classify_wikidot_right_bracket_and_advance_text_tokens(
            bytes,
            cursor,
            bytes.len(),
            &mut text_tokens,
        )
        .0
    {
        return (true, 1);
    }
    if !matches!(bytes[cursor], b' ' | b'\t') {
        return (false, 1);
    }

    let mut saw_key = false;
    loop {
        while cursor < head_end && matches!(bytes[cursor], b' ' | b'\t') {
            cursor += 1;
        }
        if cursor >= head_end {
            return (true, cursor.saturating_sub(start));
        }
        if matches!(bytes[cursor], b'\n' | b'\r') {
            return (false, cursor + 1 - start);
        }
        if bytes[cursor] == b']'
            && classify_wikidot_right_bracket_and_advance_text_tokens(
                bytes,
                cursor,
                bytes.len(),
                &mut text_tokens,
            )
            .0
        {
            return (true, cursor + 1 - start);
        }
        if bytes[cursor] == b'=' && !text_tokens.advance_and_contains(cursor) {
            return (saw_key, cursor + 1 - start);
        }

        let key_start = cursor;
        while cursor < head_end
            && (bytes[cursor].is_ascii_alphanumeric()
                || matches!(bytes[cursor], b'_' | b'-'))
        {
            cursor += 1;
        }
        if cursor == key_start || text_tokens.advance_and_contains(key_start) {
            return (false, cursor.saturating_add(1).saturating_sub(start));
        }
        saw_key = true;
    }
}

pub(super) fn is_horizontal_whitespace(byte: &u8) -> bool {
    matches!(byte, b' ' | b'\t')
}

pub(super) fn is_wikidot_head_spacing(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r')
}

pub(super) fn trimmed_utf8_span(
    source: &str,
    start: usize,
    end: usize,
) -> (usize, usize) {
    let raw = &source[start..end];
    let trimmed_start = start + raw.len() - raw.trim_start().len();
    let trimmed_end = start + raw.trim_end().len();
    (trimmed_start.min(trimmed_end), trimmed_end)
}

pub(super) fn module_subname_end(
    bytes: &[u8],
    mut cursor: usize,
    text_tokens: &mut TextTokenCursor,
) -> usize {
    while cursor < bytes.len() && !is_wikidot_head_spacing(bytes[cursor]) {
        if bytes[cursor] == b']' {
            let (right_block, token_len) =
                classify_wikidot_right_bracket_and_advance_text_tokens(
                    bytes,
                    cursor,
                    bytes.len(),
                    text_tokens,
                );
            if right_block {
                break;
            }
            cursor += token_len;
        } else {
            cursor += 1;
        }
    }
    cursor
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ModuleHeadValidation {
    DefiniteInvalid,
    ValidRuntimeBoundaryDivergence,
    ValidRuntimeUnsafe,
    RuntimeSafe,
}

pub(super) fn validate_module_head(
    source: &str,
    list_pages_compatibility: bool,
) -> ModuleHeadValidation {
    let normalized = normalize_module_head(source);
    let mut runtime_safe = normalized == source;
    let mut projected = normalized.into_bytes();
    if project_list_pages_typography_in_place(&mut projected) {
        runtime_safe = false;
    }
    let source = String::from_utf8(projected)
        .expect("module head typography projection preserves UTF-8");
    let bytes = source.as_bytes();
    let mut text_tokens = TextTokenCursor::new(&source);
    let mut cursor = 0usize;
    let mut runtime_boundary_divergence = false;

    loop {
        skip_module_argument_spacing(bytes, &mut cursor);
        if cursor == bytes.len() {
            return if runtime_boundary_divergence {
                ModuleHeadValidation::ValidRuntimeBoundaryDivergence
            } else if list_pages_compatibility && !runtime_safe {
                ModuleHeadValidation::ValidRuntimeUnsafe
            } else {
                ModuleHeadValidation::RuntimeSafe
            };
        }

        let key_start = cursor;
        let mut syntax_crossing_token_end = None;
        while cursor < bytes.len() {
            if bytes[cursor] == b'='
                || (bytes[cursor] == b'!' && bytes.get(cursor + 1) == Some(&b'='))
                || (list_pages_compatibility && matches!(bytes[cursor], b'<' | b'>'))
                || is_module_argument_spacing(bytes[cursor])
            {
                break;
            }
            if let Some(end) = text_tokens.advance_to_range_end_at(cursor) {
                runtime_safe = false;
                if bytes[cursor..end].contains(&b'=') {
                    syntax_crossing_token_end = Some(end);
                } else {
                    cursor = end;
                    continue;
                }
            }
            if bytes[cursor..].starts_with(b"{$") {
                let Some(relative_end) = source[cursor + 2..].find('}') else {
                    return ModuleHeadValidation::DefiniteInvalid;
                };
                let end = cursor + 2 + relative_end + 1;
                if cursor + 2 == end - 1
                    || !bytes[cursor + 2..end - 1]
                        .iter()
                        .all(u8::is_ascii_alphanumeric)
                {
                    return ModuleHeadValidation::DefiniteInvalid;
                }
                cursor = end;
                runtime_safe = false;
                continue;
            }
            if bytes[cursor..].starts_with(b"[!--") {
                cursor += 4;
                runtime_safe = false;
                continue;
            }
            if bytes[cursor..].starts_with(b"--]") {
                cursor += 3;
                runtime_safe = false;
                continue;
            }
            if bytes[cursor] == b'-' {
                while bytes.get(cursor) == Some(&b'-') {
                    cursor += 1;
                }
                continue;
            }
            if !bytes[cursor].is_ascii() {
                return ModuleHeadValidation::DefiniteInvalid;
            }
            if !bytes[cursor].is_ascii_alphanumeric()
                && !matches!(bytes[cursor], b'_' | b'-')
            {
                return ModuleHeadValidation::DefiniteInvalid;
            }
            cursor += 1;
        }
        if cursor == key_start {
            return ModuleHeadValidation::DefiniteInvalid;
        }
        let runtime_key = &source[key_start..cursor];
        let runtime_key_supported = runtime_list_pages_key_is_supported(runtime_key);
        if list_pages_compatibility && !runtime_key_supported {
            runtime_safe = false;
        }
        skip_horizontal_whitespace(bytes, &mut cursor);

        let mut comparison_operator = false;
        if list_pages_compatibility
            && runtime_key_supported
            && matches!(bytes.get(cursor), Some(b'!' | b'<' | b'>'))
            && !(runtime_key.starts_with('_')
                && bytes.get(cursor..cursor + 2) == Some(&b"!="[..]))
        {
            comparison_operator = true;
            let first = bytes[cursor];
            cursor += 1;
            if bytes.get(cursor) == Some(&b'=')
                || first == b'<' && bytes.get(cursor) == Some(&b'>')
            {
                cursor += 1;
            }
        } else {
            if bytes.get(cursor) == Some(&b'!') {
                if !list_pages_compatibility
                    || !runtime_key_supported
                    || bytes.get(key_start) != Some(&b'_')
                {
                    return ModuleHeadValidation::DefiniteInvalid;
                }
                cursor += 1;
            }
            if bytes.get(cursor) != Some(&b'=') {
                return ModuleHeadValidation::DefiniteInvalid;
            }
            cursor += 1;
        }
        skip_horizontal_whitespace(bytes, &mut cursor);
        if cursor == bytes.len() || is_module_argument_spacing(bytes[cursor]) {
            return ModuleHeadValidation::DefiniteInvalid;
        }

        let quote = bytes[cursor];
        if comparison_operator
            && !matches!(quote, b'\'' | b'"')
            && !list_pages_bare_comparison_key_is_evidenced(runtime_key)
        {
            return ModuleHeadValidation::DefiniteInvalid;
        }
        let list_pages_url_value = list_pages_compatibility
            && quote == b'"'
            && list_pages_url_value_quote_starts_at(bytes, cursor);
        let quote_owned = matches!(quote, b'\'' | b'"')
            && !list_pages_url_value
            && (syntax_crossing_token_end.is_some_and(|end| cursor < end)
                || text_tokens.advance_and_contains(cursor));
        if matches!(quote, b'\'' | b'"') {
            let quote_crosses_syntax_token =
                quote_owned && syntax_crossing_token_end.is_some_and(|end| cursor < end);
            runtime_safe &= !quote_owned;
            if quote == b'\'' && (!list_pages_compatibility || !runtime_key_supported) {
                return ModuleHeadValidation::DefiniteInvalid;
            }
            cursor += 1;
            let mut closed = false;
            while cursor < bytes.len() {
                if quote == b'"' && bytes[cursor] == b'\\' {
                    runtime_safe = false;
                }
                if bytes[cursor] == quote {
                    let list_pages_url_quote_end = list_pages_url_value
                        && list_pages_url_value_quote_ends_at(bytes, cursor, key_start);
                    let list_pages_comment_quote_end = list_pages_compatibility
                        && quote == b'"'
                        && list_pages_quote_ends_before_comment(bytes, cursor);
                    if !list_pages_url_quote_end
                        && !list_pages_comment_quote_end
                        && (syntax_crossing_token_end.is_some_and(|end| cursor < end)
                            || text_tokens.advance_and_contains(cursor))
                    {
                        runtime_safe = false;
                        cursor += 1;
                        continue;
                    }
                    if quote_is_escaped(bytes, cursor, &text_tokens)
                        || (quote == b'"'
                            && !double_quote_ends_scanner_argument(
                                bytes,
                                cursor,
                                &text_tokens,
                            )
                            && !list_pages_url_quote_end
                            && !list_pages_comment_quote_end)
                    {
                        runtime_safe = false;
                        cursor += 1;
                        continue;
                    }
                    cursor += 1;
                    closed = true;
                    runtime_boundary_divergence |= quote_crosses_syntax_token;
                    break;
                }
                cursor += 1;
            }
            if !closed {
                return ModuleHeadValidation::DefiniteInvalid;
            }
            if list_pages_compatibility
                && bytes
                    .get(cursor..)
                    .is_some_and(|tail| tail.starts_with(b"[!--"))
            {
                let Some(comment_end) = list_pages_head_comment_end(bytes, cursor) else {
                    return ModuleHeadValidation::DefiniteInvalid;
                };
                cursor = comment_end;
                runtime_safe = false;
            }
        } else if list_pages_compatibility && runtime_key_supported {
            runtime_safe &= !quote_owned;
            let value_start = cursor;
            while cursor < bytes.len() {
                if is_module_argument_spacing(bytes[cursor]) {
                    break;
                }
                if bytes[cursor] == b']' {
                    return ModuleHeadValidation::DefiniteInvalid;
                }
                if matches!(bytes[cursor], b'\'' | b'"') {
                    runtime_safe = false;
                }
                let character = source[cursor..]
                    .chars()
                    .next()
                    .expect("cursor is before the module head end");
                if character.is_whitespace() {
                    return ModuleHeadValidation::DefiniteInvalid;
                }
                cursor += character.len_utf8();
            }
            if cursor == value_start {
                return ModuleHeadValidation::DefiniteInvalid;
            }
        } else {
            return ModuleHeadValidation::DefiniteInvalid;
        }

        if cursor < bytes.len() && !is_module_argument_spacing(bytes[cursor]) {
            return ModuleHeadValidation::DefiniteInvalid;
        }
    }
}

pub(super) fn list_pages_url_value_quote_starts_at(bytes: &[u8], quote: usize) -> bool {
    let value = bytes.get(quote + 1..).unwrap_or_default();
    value
        .get(..4)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"@url"))
        && matches!(value.get(4), Some(b'"' | b'|'))
}

pub(super) fn list_pages_url_value_quote_ends_at(
    bytes: &[u8],
    quote: usize,
    lower_bound: usize,
) -> bool {
    let Some(opening) = bytes[lower_bound..quote]
        .iter()
        .rposition(|byte| *byte == b'"')
        .map(|offset| lower_bound + offset)
    else {
        return false;
    };
    list_pages_url_value_quote_starts_at(bytes, opening)
        && !bytes[opening + 1..quote].contains(&b'"')
}

pub(super) fn nested_list_pages_head_token_is_module(bytes: &[u8], start: usize) -> bool {
    let mut cursor = start + 2;
    skip_horizontal_whitespace(bytes, &mut cursor);
    if bytes.get(cursor) == Some(&b'/') {
        cursor += 1;
        skip_horizontal_whitespace(bytes, &mut cursor);
    }
    let (Some(name), _) = wikidot_trimmed_name(bytes, cursor) else {
        return false;
    };
    let name = name.strip_suffix(b"_").unwrap_or(name);
    name.eq_ignore_ascii_case(b"module") || name.eq_ignore_ascii_case(b"module654")
}

pub(super) fn nested_list_pages_head_token_end(
    source: &str,
    start: usize,
) -> Option<usize> {
    source
        .get(start + 2..)?
        .find("]]")
        .map(|relative| start + 2 + relative + 2)
}

pub(super) fn right_boundary_ends_physical_line(bytes: &[u8], mut cursor: usize) -> bool {
    while matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
        cursor += 1;
    }
    matches!(bytes.get(cursor), None | Some(b'\n' | b'\r'))
}

pub(super) fn final_unclosed_list_pages_head_boundary(
    source: &str,
    subname_end: usize,
    line_end: usize,
) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut opening_end = line_end;
    let mut right_brackets = 0usize;
    while opening_end > subname_end {
        match bytes[opening_end - 1] {
            b']' => {
                right_brackets += 1;
                opening_end -= 1;
            }
            b' ' | b'\t' => opening_end -= 1,
            _ => break,
        }
    }
    if right_brackets == 0 {
        return None;
    }

    let head = source[subname_end..opening_end].trim_start_matches([' ', '\t']);
    (!list_pages_head_double_quotes_are_balanced(head)
        && !wikidot_list_pages_arguments(head).is_empty())
    .then_some(opening_end)
}

pub(super) fn list_pages_definite_invalid_head_can_execute(head: &str) -> bool {
    if head.contains(['\u{000b}', '\u{000c}', '\u{00a0}', '\u{2007}'])
        || head.contains("[!--")
        || head.contains("--]")
        || head.contains('@') && !head.trim_end().ends_with("@@")
        || head.contains("\n=")
        || head.contains("\r=")
        || !list_pages_head_quotes_are_balanced(head, b'\'')
        || !list_pages_head_bracket_tokens_are_supported(head)
    {
        return false;
    }

    let arguments = wikidot_list_pages_arguments(head);
    if arguments.is_empty() {
        !head.contains(['=', '"', '\'', '['])
    } else {
        !list_pages_head_contains_nested_module_token(head)
    }
}

pub(super) fn list_pages_head_bracket_tokens_are_supported(head: &str) -> bool {
    let bytes = head.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'[' => {
                if bytes.get(cursor + 1) != Some(&b'[')
                    || nested_list_pages_head_token_is_module(bytes, cursor)
                {
                    return false;
                }
                let Some(relative_end) = bytes[cursor + 2..]
                    .windows(2)
                    .position(|pair| pair == b"]]")
                else {
                    return false;
                };
                cursor += 2 + relative_end + 2;
            }
            b']' => return false,
            _ => cursor += 1,
        }
    }
    true
}

pub(super) fn list_pages_head_contains_nested_module_token(head: &str) -> bool {
    let bytes = head.as_bytes();
    let mut cursor = 0usize;
    while cursor + 1 < bytes.len() {
        let Some(relative) = bytes[cursor..].windows(2).position(|pair| pair == b"[[")
        else {
            return false;
        };
        let start = cursor + relative;
        if nested_list_pages_head_token_is_module(bytes, start) {
            return true;
        }
        cursor = start + 2;
    }
    false
}

pub(super) fn list_pages_head_double_quotes_are_balanced(head: &str) -> bool {
    list_pages_head_quotes_are_balanced(head, b'"')
}

pub(super) fn list_pages_head_quotes_are_balanced(head: &str, quote: u8) -> bool {
    let bytes = head.as_bytes();
    let mut open = false;
    let mut backslashes = 0usize;
    for byte in bytes {
        if *byte == quote && backslashes.is_multiple_of(2) {
            open = !open;
        }
        backslashes = if *byte == b'\\' { backslashes + 1 } else { 0 };
    }
    !open
}

pub(super) fn continuation_revealed_argument_boundary(
    bytes: &[u8],
    mut cursor: usize,
) -> Option<usize> {
    let mut pending_backslashes = 0usize;
    let mut consumed_line_end = false;
    loop {
        match bytes.get(cursor) {
            Some(b'\\') => {
                pending_backslashes += 1;
                cursor += 1;
            }
            Some(b'\n' | b'\r') if pending_backslashes > 0 => {
                pending_backslashes -= 1;
                cursor = physical_line_resume(bytes, cursor);
                consumed_line_end = true;
            }
            _ => break,
        }
    }
    (consumed_line_end && pending_backslashes == 0).then_some(cursor)
}

pub(super) fn scanner_argument_boundary_at(
    bytes: &[u8],
    mut cursor: usize,
    text_tokens: &TextTokenCursor,
) -> bool {
    let mut lookahead_tokens = text_tokens.clone();
    if cursor >= bytes.len()
        || matches!(bytes[cursor], b'\n' | b'\r')
        || (bytes[cursor] == b']'
            && classify_wikidot_right_bracket_and_advance_text_tokens(
                bytes,
                cursor,
                bytes.len(),
                &mut lookahead_tokens,
            )
            .0)
    {
        return true;
    }
    while matches!(bytes.get(cursor), Some(b' ' | b'\t' | b'\0')) {
        cursor += 1;
    }
    if cursor >= bytes.len() {
        return true;
    }
    if bytes.get(cursor) == Some(&b']')
        && classify_wikidot_right_bracket_and_advance_text_tokens(
            bytes,
            cursor,
            bytes.len(),
            &mut lookahead_tokens,
        )
        .0
    {
        return true;
    }
    let key_start = cursor;
    while bytes
        .get(cursor)
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        cursor += 1;
    }
    if cursor == key_start {
        return false;
    }
    if lookahead_tokens.advance_and_contains(key_start) {
        return false;
    }
    let key_end = cursor;
    while matches!(bytes.get(cursor), Some(b' ' | b'\t' | b'\0')) {
        cursor += 1;
    }
    let runtime_key = std::str::from_utf8(&bytes[key_start..key_end]).unwrap_or_default();
    if matches!(bytes.get(cursor), Some(b'!' | b'<' | b'>'))
        && !(runtime_key.starts_with('_')
            && bytes.get(cursor..cursor + 2) == Some(&b"!="[..]))
    {
        let first = bytes[cursor];
        cursor += 1;
        if bytes.get(cursor) == Some(&b'=')
            || first == b'<' && bytes.get(cursor) == Some(&b'>')
        {
            cursor += 1;
        }
        while matches!(bytes.get(cursor), Some(b' ' | b'\t' | b'\0')) {
            cursor += 1;
        }
        return matches!(bytes.get(cursor), Some(b'\'' | b'"'))
            || list_pages_bare_comparison_key_is_evidenced(runtime_key);
    }
    if bytes.get(cursor) == Some(&b'!') {
        cursor += 1;
    }
    bytes.get(cursor) == Some(&b'=') && !lookahead_tokens.advance_and_contains(cursor)
}

pub(super) fn physical_line_resume(bytes: &[u8], line_end: usize) -> usize {
    debug_assert!(matches!(bytes.get(line_end), Some(b'\n' | b'\r')));
    if bytes[line_end] == b'\r' && bytes.get(line_end + 1) == Some(&b'\n') {
        line_end + 2
    } else {
        line_end + 1
    }
}

pub(super) fn next_physical_line_resume(bytes: &[u8], mut cursor: usize) -> usize {
    while !matches!(bytes.get(cursor), None | Some(b'\n' | b'\r')) {
        cursor += 1;
    }
    if cursor == bytes.len() {
        cursor
    } else {
        physical_line_resume(bytes, cursor)
    }
}

pub(super) fn quote_follows_argument_equals(
    bytes: &[u8],
    quote: usize,
    lower_bound: usize,
) -> bool {
    let mut cursor = quote;
    while cursor > lower_bound && matches!(bytes[cursor - 1], b' ' | b'\t') {
        cursor -= 1;
    }
    cursor > lower_bound && bytes[cursor - 1] == b'='
}

#[cfg(test)]
#[path = "head_validation_tests.rs"]
mod tests;
