//! Parser-function and preprocessor helpers for delayed ListPages rows.

use super::{
    COUNTPAGES_MODULE_REGEX, CompatTextFragments, ListPagesGeneratedSlot,
    ListPagesModuleMatch, ListPagesRuntimeTextRange, LiteralRegionIndex, Range,
    RenderService, SharedRenderCostBudget, TextOrigin, find_list_pages_module_matches,
    find_list_pages_module_matches_with_budget,
};

pub(super) fn protect_row_variable_parser_functions(
    source: &mut String,
) -> Option<CompatTextFragments> {
    let lowercase = source.to_ascii_lowercase();
    let mut ranges = Vec::new();
    let mut cursor = 0;
    while let Some(relative_start) = lowercase[cursor..].find("[[#") {
        let start = cursor + relative_start;
        let Some(relative_end) = lowercase[start + 3..].find("]]") else {
            break;
        };
        let end = start + 3 + relative_end + 2;
        if source[start..end].contains("%%") {
            ranges.push(start..end);
        }
        cursor = end;
    }
    if ranges.is_empty() {
        return None;
    }

    let mut fragments = CompatTextFragments::new(source);
    for range in ranges.into_iter().rev() {
        let marker = fragments.push(&source[range.clone()]);
        source.replace_range(range, &marker);
    }
    Some(fragments)
}

pub(super) fn list_pages_template_requires_runtime_title(source: &str) -> bool {
    source.lines().any(|line| {
        let line = line.trim_start().to_ascii_lowercase();
        [
            "[[row",
            "[[cell",
            "[[table",
            "[[code",
            "[[html",
            "[[collapsible",
            "[[embedvideo",
        ]
        .iter()
        .any(|marker| line.starts_with(marker))
    }) || source.lines().any(|line| line.trim() == "@@@@")
}

pub(super) fn resolve_list_pages_expr_parser_functions(
    body: &mut String,
    generated_slots: &mut Vec<ListPagesGeneratedSlot>,
    runtime_text_ranges: &mut Vec<ListPagesRuntimeTextRange>,
) {
    if !body.contains("[[#") {
        return;
    }

    // FTML's delayed parser owns the recovery of a generated link inside a
    // parser-function branch. Resolving that branch textually here would turn
    // Wikidot's evidenced malformed triple-link result into a normal active
    // link after slot binding. Leave the complete linked branch intact; the
    // delayed parser can bind its generated slot and preserve that boundary.
    if linked_parser_function_ranges(body).iter().any(|range| {
        generated_slots.iter().any(|slot| {
            range.start <= slot.source_range.start && slot.source_range.end <= range.end
        })
    }) {
        return;
    }

    #[derive(Clone, Copy)]
    enum ProtectedKind {
        Generated(usize),
        RuntimeText(TextOrigin),
    }

    struct ProtectedOccurrence {
        source: String,
        marker: String,
        kind: ProtectedKind,
    }

    let original_generated_slots = generated_slots.clone();
    let mut source_occurrences = original_generated_slots
        .iter()
        .enumerate()
        .map(|(index, slot)| (slot.source_range.clone(), ProtectedKind::Generated(index)))
        .chain(runtime_text_ranges.iter().map(|range| {
            (
                range.source_range.clone(),
                ProtectedKind::RuntimeText(range.origin),
            )
        }))
        .collect::<Vec<_>>();
    source_occurrences.sort_by_key(|(range, _)| range.start);
    if source_occurrences
        .iter()
        .any(|(range, _)| range.end > body.len())
        || source_occurrences
            .windows(2)
            .any(|pair| pair[0].0.end > pair[1].0.start)
    {
        return;
    }

    let mut fragments = CompatTextFragments::new(body);
    let mut protected = String::with_capacity(body.len());
    let mut protected_occurrences = Vec::with_capacity(source_occurrences.len());
    let mut cursor = 0usize;
    for (source_range, kind) in source_occurrences {
        protected.push_str(&body[cursor..source_range.start]);
        let source = body[source_range.clone()].to_owned();
        let marker = fragments.push(&source);
        protected.push_str(&marker);
        protected_occurrences.push(ProtectedOccurrence {
            source,
            marker,
            kind,
        });
        cursor = source_range.end;
    }
    protected.push_str(&body[cursor..]);

    let generated_comment_gates =
        protect_generated_parser_function_comment_gates(&mut protected);
    let mut resolved = RenderService::resolve_wikidot_parser_functions(&protected);
    if let Some((opening, closing, standalone_closing)) = generated_comment_gates {
        resolved = resolved.replace(&standalone_closing, "[!-- --]");
        resolved =
            prune_generated_parser_function_comment_gates(resolved, &opening, &closing);
    }
    let mut present_occurrences = Vec::new();
    for occurrence in protected_occurrences {
        let mut positions = resolved.match_indices(&occurrence.marker);
        let Some((start, _)) = positions.next() else {
            continue;
        };
        if positions.next().is_some() {
            return;
        }
        present_occurrences.push((start, occurrence));
    }
    present_occurrences.sort_by_key(|(start, _)| *start);

    let mut restored = String::with_capacity(resolved.len());
    let mut resolved_cursor = 0usize;
    let mut remapped_generated_slots = Vec::new();
    let mut remapped_runtime_ranges = Vec::new();
    for (marker_start, occurrence) in present_occurrences {
        if marker_start < resolved_cursor {
            return;
        }
        restored.push_str(&resolved[resolved_cursor..marker_start]);
        let source_start = restored.len();
        restored.push_str(&occurrence.source);
        let source_end = restored.len();
        match occurrence.kind {
            ProtectedKind::Generated(index) => {
                let mut slot = original_generated_slots[index].clone();
                slot.source_range = source_start..source_end;
                remapped_generated_slots.push(slot);
            }
            ProtectedKind::RuntimeText(origin) => {
                remapped_runtime_ranges.push(ListPagesRuntimeTextRange {
                    source_range: source_start..source_end,
                    origin,
                });
            }
        }
        resolved_cursor = marker_start + occurrence.marker.len();
    }
    restored.push_str(&resolved[resolved_cursor..]);

    *generated_slots = remapped_generated_slots;
    *runtime_text_ranges = remapped_runtime_ranges;
    *body = restored;
}

pub(in crate::services::render) fn protect_generated_parser_function_comment_gates(
    source: &mut String,
) -> Option<(String, String, String)> {
    let mut fragments = CompatTextFragments::new(source);
    let opening = fragments.push("");
    let closing = fragments.push("");
    let standalone_closing = fragments.push("");
    let mut replacements = Vec::new();
    let mut line_start = 0usize;

    for line in source.split_inclusive('\n') {
        let line_body = line.strip_suffix('\n').unwrap_or(line);
        let leading = line_body.len() - line_body.trim_start_matches([' ', '\t']).len();
        let trimmed = &line_body[leading..];
        let lowercase = trimmed.to_ascii_lowercase();
        if !lowercase.starts_with("[[#ifexpr ") && !lowercase.starts_with("[[#if ") {
            line_start += line.len();
            continue;
        }
        let Some(pipe) = trimmed.rfind('|') else {
            line_start += line.len();
            continue;
        };
        let branch_start = pipe + 1 + trimmed[pipe + 1..].len()
            - trimmed[pipe + 1..].trim_start_matches([' ', '\t']).len();
        let branch = &trimmed[branch_start..];
        let (token, marker) = if branch.starts_with("[!--")
            && branch["[!--".len()..].trim_start_matches([' ', '\t']) == "]]"
        {
            ("[!--", opening.as_str())
        } else if branch.starts_with("--]")
            && branch["--]".len()..].trim_start_matches([' ', '\t']) == "]]"
        {
            ("--]", closing.as_str())
        } else {
            line_start += line.len();
            continue;
        };
        let start = line_start + leading + branch_start;
        replacements.push((start..start + token.len(), line_start + leading, marker));
        line_start += line.len();
    }

    if replacements.is_empty() {
        return None;
    }
    // Gate tokens inside parser-function result branches look like real
    // comments to the ordinary literal index and can make every later closer
    // appear comment-owned. Mask only the candidate tokens in an
    // offset-preserving projection, then use that index to reject function
    // lines that genuinely live in code, raw, HTML, or authored comments.
    let mut literal_projection = source.clone();
    for (range, _, _) in replacements.iter().rev() {
        literal_projection.replace_range(range.clone(), &" ".repeat(range.len()));
    }
    // Neutralize the matching standalone close while constructing the literal
    // index; otherwise the generated opener makes that close look comment-
    // owned and prevents us from preserving it through preprocessing.
    let mut projection_line_start = 0usize;
    for line in source.split_inclusive('\n') {
        let line_body = line.strip_suffix('\n').unwrap_or(line);
        let leading = line_body.len() - line_body.trim_start_matches([' ', '\t']).len();
        if line_body[leading..].trim() == "[!-- --]" {
            let range = projection_line_start + leading
                ..projection_line_start + leading + "[!-- --]".len();
            literal_projection.replace_range(range, &" ".repeat("[!-- --]".len()));
        }
        projection_line_start += line.len();
    }
    let literal_regions = LiteralRegionIndex::new(&literal_projection);
    replacements
        .retain(|(_, function_start, _)| !literal_regions.contains(*function_start));
    if replacements.is_empty() {
        return None;
    }

    // The matching generated close is a standalone `[!-- --]` line rather
    // than another parser-function opener.  Preserve those lines too, while
    // keeping authored comments in code/raw/HTML owned by their literal
    // context.
    let generated_gate_present = !replacements.is_empty();
    let mut line_start = 0usize;
    for line in source.split_inclusive('\n') {
        let line_body = line.strip_suffix('\n').unwrap_or(line);
        let leading = line_body.len() - line_body.trim_start_matches([' ', '\t']).len();
        let trimmed = &line_body[leading..];
        if trimmed == "[!-- --]"
            && generated_gate_present
            && !literal_regions.contains(line_start + leading)
        {
            replacements.push((
                line_start + leading..line_start + leading + "[!-- --]".len(),
                line_start + leading,
                standalone_closing.as_str(),
            ));
        }
        line_start += line.len();
    }
    replacements.sort_by_key(|(range, _, _)| range.start);
    for (range, _, marker) in replacements.into_iter().rev() {
        source.replace_range(range, marker);
    }
    Some((opening, closing, standalone_closing))
}

fn prune_generated_parser_function_comment_gates(
    source: String,
    opening: &str,
    closing: &str,
) -> String {
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0usize;
    loop {
        let next_opening = source[cursor..].find(opening).map(|at| cursor + at);
        let next_closing = source[cursor..].find(closing).map(|at| cursor + at);
        match (next_opening, next_closing) {
            (None, None) => {
                output.push_str(&source[cursor..]);
                return output;
            }
            (Some(open), Some(close)) if open < close => {
                output.push_str(&source[cursor..open]);
                cursor = close + closing.len();
            }
            // A malformed, crossing, or unmatched generated boundary must
            // retain ordinary comment syntax rather than deleting an
            // ambiguous span.
            _ => {
                return source.replace(opening, "[!--").replace(closing, "--]");
            }
        }
    }
}

pub(super) fn substitute_literal_advanced_table_opener_typography(source: &mut String) {
    let literal_regions = LiteralRegionIndex::new(source);
    let mut table_depth = 0usize;
    let mut quoted_values = Vec::new();
    let mut line_start = 0usize;

    while line_start < source.len() {
        let line_end = source[line_start..]
            .find('\n')
            .map_or(source.len(), |offset| line_start + offset);
        let line = &source[line_start..line_end];
        let trimmed_start = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();
        let literal = literal_regions.contains(line_start + trimmed_start);

        if !literal && complete_wikidot_block_line(&lower, "/table") {
            table_depth = table_depth.saturating_sub(1);
        } else if !literal && complete_wikidot_block_line(&lower, "table") {
            table_depth = table_depth.saturating_add(1);
        } else if !literal
            && table_depth == 0
            && (complete_wikidot_block_line(&lower, "cell")
                || complete_wikidot_block_line(&lower, "hcell"))
        {
            let absolute_start = line_start + trimmed_start;
            let bytes = trimmed.as_bytes();
            let mut cursor = 0usize;
            let mut line_ranges = Vec::new();
            let mut balanced = true;
            while cursor < bytes.len() {
                let quote = bytes[cursor];
                if quote != b'"' && quote != b'\'' {
                    cursor += 1;
                    continue;
                }
                let value_start = cursor + 1;
                let Some(relative_end) =
                    bytes[value_start..].iter().position(|byte| *byte == quote)
                else {
                    balanced = false;
                    break;
                };
                let value_end = value_start + relative_end;
                line_ranges
                    .push(absolute_start + value_start..absolute_start + value_end);
                cursor = value_end + 1;
            }
            if balanced {
                quoted_values.extend(line_ranges);
            }
        }

        if line_end == source.len() {
            break;
        }
        line_start = line_end + 1;
    }

    for range in quoted_values.into_iter().rev() {
        let mut value = source[range.clone()].to_owned();
        ftml::preproc::typography::substitute_wikidot(&mut value);
        source.replace_range(range, &value);
    }
}

fn complete_wikidot_block_line(line: &str, name: &str) -> bool {
    let Some(body) = line
        .strip_prefix("[[")
        .and_then(|line| line.strip_suffix("]]"))
    else {
        return false;
    };
    let Some(suffix) = body.strip_prefix(name) else {
        return false;
    };
    suffix.is_empty() || suffix.chars().next().is_some_and(char::is_whitespace)
}

fn linked_parser_function_ranges(source: &str) -> Vec<Range<usize>> {
    let lowercase = source.to_ascii_lowercase();
    let mut ranges = Vec::new();
    let mut cursor = 0;
    while let Some(relative_start) = lowercase[cursor..].find("[[#") {
        let start = cursor + relative_start;
        let Some(relative_end) = lowercase[start + 3..].find("]]") else {
            break;
        };
        let end = start + 3 + relative_end + 2;
        let candidate = &lowercase[start..end];
        if [
            "%%title_linked%%",
            "%%linked_title%%",
            "%%tags_linked%%",
            "%%tagslinked%%",
        ]
        .iter()
        .any(|marker| candidate.contains(marker))
        {
            ranges.push(start..end);
        }
        cursor = end;
    }
    ranges
}

fn linked_parser_function_projection(source: &str) -> String {
    let mut projection = source.as_bytes().to_vec();
    for (start, _) in source.match_indices("[[#") {
        projection[start..start + 3].fill(b' ');
    }
    String::from_utf8(projection).expect("ASCII masking preserves UTF-8")
}

pub(in crate::services::render) fn find_list_pages_module_matches_with_delayed_links(
    source: &str,
) -> Vec<ListPagesModuleMatch<'_>> {
    find_list_pages_module_matches_with_delayed_links_inner(source, None)
}

pub(in crate::services::render) fn find_list_pages_module_matches_with_delayed_links_budgeted<
    'a,
>(
    source: &'a str,
    budget: &SharedRenderCostBudget,
) -> Vec<ListPagesModuleMatch<'a>> {
    find_list_pages_module_matches_with_delayed_links_inner(source, Some(budget))
}

fn find_list_pages_module_matches_with_delayed_links_inner<'a>(
    source: &'a str,
    budget: Option<&SharedRenderCostBudget>,
) -> Vec<ListPagesModuleMatch<'a>> {
    let projection = linked_parser_function_projection(source);
    let projection_start = projection.as_ptr() as usize;
    let modules = budget.map_or_else(
        || find_list_pages_module_matches(&projection),
        |budget| find_list_pages_module_matches_with_budget(&projection, budget),
    );
    modules
        .into_iter()
        .map(|module| {
            let head_start = module.head.as_ptr() as usize - projection_start;
            let head_end = head_start + module.head.len();
            ListPagesModuleMatch {
                start: module.start,
                body_start: module.body_start,
                end: module.end,
                head: &source[head_start..head_end],
                body: &source[module.body_start..module.body_start + module.body.len()],
                original: &source[module.start..module.end],
                runtime_safe: module.runtime_safe,
                preserve_original: module.preserve_original,
                preserve_as_module654: module.preserve_as_module654,
                consume_empty_tail: module.consume_empty_tail,
            }
        })
        .collect()
}

/// Resolve document-level parser functions while retaining complete ListPages
/// row templates for their evidenced post-substitution phase.
///
/// Module heads remain in the outer phase. Only structurally recognized bodies
/// are delayed; malformed or ambiguous module text keeps the ordinary
/// fail-closed preprocessor behavior.
pub(in crate::services::render) fn resolve_wikidot_parser_functions_outside_list_pages(
    source: &str,
) -> String {
    if !source.contains("[[#") {
        return source.to_owned();
    }

    let modules = find_list_pages_module_matches_with_delayed_links(source);
    let mut deferred_bodies = modules
        .iter()
        .filter_map(|module| {
            let range = module.body_start..module.body_start + module.body.len();
            source[range.clone()].contains("[[#").then_some(range)
        })
        .collect::<Vec<_>>();
    deferred_bodies.extend(COUNTPAGES_MODULE_REGEX.captures_iter(source).filter_map(
        |captures| {
            let body = captures.name("body")?;
            body.as_str()
                .contains("[[#")
                .then_some(body.start()..body.end())
        },
    ));
    if deferred_bodies.is_empty() {
        return ftml::preproc::resolve_wikidot_parser_functions(source);
    }

    deferred_bodies.sort_by_key(|range| range.start);
    let mut merged_bodies: Vec<Range<usize>> = Vec::with_capacity(deferred_bodies.len());
    for range in deferred_bodies {
        if let Some(previous) = merged_bodies.last_mut()
            && range.start <= previous.end
        {
            previous.end = previous.end.max(range.end);
        } else {
            merged_bodies.push(range);
        }
    }

    let mut fragments = CompatTextFragments::new(source);
    let mut protected = String::with_capacity(source.len());
    let mut cursor = 0;
    for range in merged_bodies {
        protected.push_str(&source[cursor..range.start]);
        protected.push_str(&fragments.push(&source[range.clone()]));
        cursor = range.end;
    }
    protected.push_str(&source[cursor..]);

    let generated_comment_gates =
        protect_generated_parser_function_comment_gates(&mut protected);
    let mut resolved = ftml::preproc::resolve_wikidot_parser_functions(&protected);
    if let Some((opening, closing, standalone_closing)) = generated_comment_gates {
        resolved = resolved
            .replace(&opening, "[!--")
            .replace(&closing, "--]")
            .replace(&standalone_closing, "[!-- --]");
    }
    fragments.restore(&resolved)
}
