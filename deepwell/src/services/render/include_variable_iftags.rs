//! Structural resolution for Wikidot include-variable `iftags` names.
//!
//! Some Wikidot sources spell `iftags` as `ift{$name}gs`: the include value
//! `a` turns it into a real gate, while an omitted value leaves a transparent
//! wrapper. Resolve that choice before ordinary include substitution so the
//! source structure, rather than a theme name or body substring, determines
//! the result.

use super::iftags::wikidot_tag_conditions_match;
use super::literal_regions::LiteralRegionIndex;
use ftml::data::PageInfo;
use ftml::tree::VariableMap;
use regex::Regex;
use std::ops::Range;
use std::sync::LazyLock;

static TOKEN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?ix)
        (?P<dynamic_open>\[\[ift\{\$(?P<open_name>[A-Za-z0-9_-]+)\}gs(?P<spec>[^\]\r\n]*)\]\])
        |(?P<dynamic_close>\[\[/ift\{\$(?P<close_name>[A-Za-z0-9_-]+)\}gs\]\])
        |(?P<ordinary_open>\[\[iftags(?P<ordinary_spec>[^\]\r\n]*)\]\])
        |(?P<ordinary_close>\[\[/iftags\]\])
        "#,
    )
    .expect("include-variable iftags token regex")
});

#[derive(Debug)]
enum GateKind {
    Dynamic { name: String, spec: String },
    Ordinary { spec: String },
}

#[derive(Debug)]
struct OpenGate {
    kind: GateKind,
    open: Range<usize>,
    children: Vec<ClosedGate>,
}

#[derive(Debug)]
struct ClosedGate {
    kind: GateKind,
    open: Range<usize>,
    close: Range<usize>,
    children: Vec<ClosedGate>,
}

impl ClosedGate {
    fn replacement_range(&self, source: &str) -> Range<usize> {
        let line_start = source[..self.open.start]
            .rfind('\n')
            .map_or(0, |index| index + 1);
        let prefix = &source[line_start..self.open.start];
        let start = if !prefix.is_empty()
            && prefix.chars().all(|character| {
                character == '>' || character == ' ' || character == '\t'
            }) {
            line_start
        } else {
            self.open.start
        };
        start..self.close.end
    }
}

#[derive(Debug)]
struct Replacement {
    range: Range<usize>,
    text: String,
}

pub(super) fn resolve_include_variable_iftags(
    source: &mut String,
    variables: &VariableMap<'_>,
    page_info: &PageInfo<'_>,
) {
    resolve_include_variable_iftags_with_tags(source, variables, &page_info.tags);
}

/// Resolve dynamic `iftags` names in a source rendered without an include
/// callsite. Such names have no value to bind and therefore use Wikidot's
/// absent-value behavior.
pub(super) fn resolve_unbound_include_variable_iftags(source: &mut String) {
    resolve_include_variable_iftags_with_tags(source, &VariableMap::new(), &[]);
}

fn resolve_include_variable_iftags_with_tags(
    source: &mut String,
    variables: &VariableMap<'_>,
    tags: &[std::borrow::Cow<'_, str>],
) {
    if !source
        .as_bytes()
        .windows(b"[[ift{$".len())
        .any(|window| window.eq_ignore_ascii_case(b"[[ift{$"))
    {
        return;
    }

    let literal_regions = LiteralRegionIndex::new_wikidot_syntax(source);
    let Some(roots) = parse_dynamic_roots(source, &literal_regions) else {
        return;
    };
    let mut replacements = Vec::new();
    for root in roots {
        collect_replacements(source, &root, variables, tags, &mut replacements);
    }
    if replacements.is_empty() {
        return;
    }

    replacements.sort_unstable_by_key(|replacement| replacement.range.start);
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for replacement in replacements {
        if replacement.range.start < cursor {
            continue;
        }
        output.push_str(&source[cursor..replacement.range.start]);
        output.push_str(&replacement.text);
        cursor = replacement.range.end;
    }
    output.push_str(&source[cursor..]);
    *source = output;
}

fn parse_dynamic_roots(
    source: &str,
    literal_regions: &LiteralRegionIndex,
) -> Option<Vec<ClosedGate>> {
    let mut stack = Vec::<OpenGate>::new();
    let mut roots = Vec::new();
    let mut saw_dynamic = false;

    for captures in TOKEN_REGEX.captures_iter(source) {
        let token = captures.get(0).expect("conditional token");
        if literal_regions.contains(token.start()) {
            continue;
        }

        if let Some(name) = captures.name("open_name") {
            saw_dynamic = true;
            stack.push(OpenGate {
                kind: GateKind::Dynamic {
                    name: name.as_str().to_owned(),
                    spec: captures
                        .name("spec")
                        .map_or("", |value| value.as_str())
                        .to_owned(),
                },
                open: token.start()..token.end(),
                children: Vec::new(),
            });
        } else if captures.name("ordinary_open").is_some() {
            stack.push(OpenGate {
                kind: GateKind::Ordinary {
                    spec: captures
                        .name("ordinary_spec")
                        .map_or("", |value| value.as_str())
                        .to_owned(),
                },
                open: token.start()..token.end(),
                children: Vec::new(),
            });
        } else if let Some(name) = captures.name("close_name") {
            let open = stack.pop()?;
            if !matches!(&open.kind, GateKind::Dynamic { name: open_name, .. } if open_name == name.as_str())
            {
                return None;
            }
            attach_closed(open, token.start()..token.end(), &mut stack, &mut roots);
        } else if captures.name("ordinary_close").is_some() && !stack.is_empty() {
            let open = stack.pop()?;
            if !matches!(open.kind, GateKind::Ordinary { .. }) {
                return None;
            }
            attach_closed(open, token.start()..token.end(), &mut stack, &mut roots);
        }
    }

    if saw_dynamic && stack.is_empty() {
        Some(roots)
    } else {
        None
    }
}

fn attach_closed(
    open: OpenGate,
    close: Range<usize>,
    stack: &mut [OpenGate],
    roots: &mut Vec<ClosedGate>,
) {
    let closed = ClosedGate {
        kind: open.kind,
        open: open.open,
        close,
        children: open.children,
    };
    if let Some(parent) = stack.last_mut() {
        parent.children.push(closed);
    } else if matches!(closed.kind, GateKind::Dynamic { .. }) {
        roots.push(closed);
    }
}

fn collect_replacements(
    source: &str,
    gate: &ClosedGate,
    variables: &VariableMap<'_>,
    tags: &[std::borrow::Cow<'_, str>],
    replacements: &mut Vec<Replacement>,
) {
    let GateKind::Dynamic { name, spec } = &gate.kind else {
        return;
    };
    let value = variables.get(name.as_str()).map(|value| value.trim());
    match value {
        Some("a") => {
            if wikidot_tag_conditions_match(spec, tags) {
                let body = active_body(source, gate);
                replacements.push(Replacement {
                    range: gate.replacement_range(source),
                    text: body,
                });
            } else {
                replacements.push(Replacement {
                    range: gate.replacement_range(source),
                    text: String::new(),
                });
            }
        }
        None | Some("") => {
            replacements.push(Replacement {
                range: gate.replacement_range(source),
                text: if has_only_empty_nested_gate(source, gate) {
                    String::new()
                } else {
                    quoted_body(source, gate.open.end, gate.close.start)
                },
            });
        }
        Some(_) => {}
    }
}

fn active_body(source: &str, gate: &ClosedGate) -> String {
    if has_only_empty_nested_gate(source, gate) {
        let child = &gate.children[0];
        return quoted_body(source, child.open.end, child.close.start);
    }
    quoted_body(source, gate.open.end, gate.close.start)
}

fn has_only_empty_nested_gate(source: &str, gate: &ClosedGate) -> bool {
    gate.children.len() == 1
        && matches!(&gate.children[0].kind, GateKind::Ordinary { spec } if spec.trim().is_empty())
        && wrapper_is_only_child(source, gate, &gate.children[0])
}

fn quoted_body(source: &str, start: usize, end: usize) -> String {
    let body = &source[start..end];
    let trimmed_end = body.trim_end_matches([' ', '\t']).len();
    let line_start = body[..trimmed_end].rfind('\n').map_or(0, |index| index + 1);
    if body[line_start..trimmed_end]
        .chars()
        .all(|character| character == '>')
    {
        let mut output = body[..line_start].to_owned();
        output.push_str(&body[trimmed_end..]);
        output
    } else {
        body.to_owned()
    }
}

fn wrapper_is_only_child(source: &str, parent: &ClosedGate, child: &ClosedGate) -> bool {
    structural_padding(&source[parent.open.end..child.open.start])
        && structural_padding(&source[child.close.end..parent.close.start])
}

fn structural_padding(value: &str) -> bool {
    value
        .chars()
        .all(|character| character.is_whitespace() || character == '>')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    fn resolve(source: &str, variables: &[(&str, &str)], tags: &[&str]) -> String {
        let variables = variables
            .iter()
            .map(|&(name, value)| (Cow::Borrowed(name), Cow::Borrowed(value)))
            .collect();
        let page_info = PageInfo {
            page: Cow::Borrowed("page"),
            category: None,
            site: Cow::Borrowed("site"),
            title: Cow::Borrowed("Page"),
            alt_title: None,
            score: ftml::data::ScoreValue::Integer(0),
            tags: tags.iter().map(|&tag| Cow::Borrowed(tag)).collect(),
            language: Cow::Borrowed("en"),
        };
        let mut source = source.to_owned();
        resolve_include_variable_iftags(&mut source, &variables, &page_info);
        source
    }

    #[test]
    fn active_a_uses_page_tag_conditions() {
        let source = "[[ift{$mode}gs +theme]]yes[[/ift{$mode}gs]]";
        assert_eq!(resolve(source, &[("mode", "a")], &["theme"]), "yes");
        assert_eq!(resolve(source, &[("mode", "a")], &[]), "");
    }

    #[test]
    fn candidate_check_preserves_case_insensitive_dynamic_syntax() {
        let source = "[[IFT{$mode}GS +theme]]yes[[/IFT{$mode}GS]]";
        assert_eq!(resolve(source, &[("mode", "a")], &["theme"]), "yes");
    }

    #[test]
    fn absent_value_unwraps_the_dynamic_boundary() {
        let source = "before[[ift{$mode}gs +theme]]yes[[/ift{$mode}gs]]after";
        assert_eq!(resolve(source, &[], &[]), "beforeyesafter");
    }

    #[test]
    fn active_a_unwraps_a_structural_empty_nested_gate() {
        let source = ">[[ift{$mode}gs -override]]\n>[[iftags]]\ncss\n>[[/iftags]]\n>[[/ift{$mode}gs]]";
        assert_eq!(resolve(source, &[("mode", "a")], &[]), "\ncss\n");
    }

    #[test]
    fn absent_value_drops_a_structural_empty_nested_gate() {
        let source = ">[[ift{$mode}gs -override]]\n>[[iftags]]\ncss\n>[[/iftags]]\n>[[/ift{$mode}gs]]";
        assert_eq!(resolve(source, &[], &[]), "");
    }

    #[test]
    fn resolves_multiple_independent_dynamic_boundaries() {
        let source =
            "[[ift{$one}gs +x]]x[[/ift{$one}gs]][[ift{$two}gs -y]]z[[/ift{$two}gs]]";
        assert_eq!(resolve(source, &[("one", "a"), ("two", "a")], &["x"]), "xz");
    }

    #[test]
    fn preserves_invalid_value() {
        let source = "[[ift{$mode}gs +theme]]yes[[/ift{$mode}gs]]";
        assert_eq!(resolve(source, &[("mode", "invalid")], &["theme"]), source);
    }

    #[test]
    fn does_not_resolve_dynamic_syntax_inside_an_authored_gate() {
        let source =
            "[[iftags +outer]][[ift{$mode}gs +theme]]yes[[/ift{$mode}gs]][[/iftags]]";
        assert_eq!(resolve(source, &[("mode", "a")], &["theme"]), source);
    }

    #[test]
    fn ignores_tokens_in_literal_regions() {
        let source = "[[code]]\n[[ift{$mode}gs +theme]]yes[[/ift{$mode}gs]]\n[[/code]]";
        assert_eq!(resolve(source, &[("mode", "a")], &["theme"]), source);
    }

    #[test]
    fn preserves_unbalanced_and_mismatched_boundaries() {
        let unbalanced = "[[ift{$mode}gs +theme]]yes";
        let mismatched = "[[ift{$one}gs +theme]]yes[[/ift{$two}gs]]";
        assert_eq!(
            resolve(unbalanced, &[("mode", "a")], &["theme"]),
            unbalanced
        );
        assert_eq!(resolve(mismatched, &[("one", "a")], &["theme"]), mismatched);
    }
}
