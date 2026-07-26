//! Wikidot `iftags` selection against runtime page tags.
//!
//! Real Wikidot evaluates root-level gates once. If an active gate contains
//! another `iftags` pair, the nested pair remains literal instead of becoming
//! another condition. Render preparation has several passes, so nested tokens
//! are registered temporarily to keep later passes from evaluating them.

use super::compat::text_fragments::CompatTextFragments;
use super::literal_regions::LiteralRegionIndex;
use regex::Regex;
use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

static IFTAGS_TOKEN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)(?P<open>\[\[iftags(?P<spec>[^\]\r\n]*)\]\])|(?P<close>\[\[/iftags\]\])"#,
    )
    .unwrap()
});

#[derive(Debug)]
struct OpenGate {
    start: usize,
    end: usize,
    spec: String,
    nested_tokens: Vec<Range<usize>>,
    last_nested_closer: Option<Range<usize>>,
}

#[derive(Debug)]
struct Replacement {
    range: Range<usize>,
    text: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UnmatchedBoundaryMode {
    Defer,
    Preserve,
}

pub(super) fn resolve_outermost_wikidot_iftags(
    wikitext: &mut String,
    tags: &[Cow<'_, str>],
    preserved: &mut CompatTextFragments,
) {
    resolve_outermost_wikidot_iftags_with_mode(
        wikitext,
        tags,
        preserved,
        UnmatchedBoundaryMode::Preserve,
    );
}

pub(super) fn resolve_outermost_wikidot_iftags_before_include_expansion(
    wikitext: &mut String,
    tags: &[Cow<'_, str>],
    preserved: &mut CompatTextFragments,
) {
    // An included target can open a gate that a later caller token closes (and
    // vice versa). Resolve only self-contained roots here; the caller-level
    // pass finalizes recovery or literal protection after textual expansion.
    resolve_outermost_wikidot_iftags_with_mode(
        wikitext,
        tags,
        preserved,
        UnmatchedBoundaryMode::Defer,
    );
}

fn resolve_outermost_wikidot_iftags_with_mode(
    wikitext: &mut String,
    tags: &[Cow<'_, str>],
    preserved: &mut CompatTextFragments,
    unmatched_mode: UnmatchedBoundaryMode,
) {
    let literal_regions = LiteralRegionIndex::new_wikidot_conditional_syntax(wikitext);
    let mut stack = Vec::<OpenGate>::new();
    let mut replacements = Vec::<Replacement>::new();

    for captures in IFTAGS_TOKEN_REGEX.captures_iter(wikitext) {
        let token = captures.get(0).expect("iftags token");
        let literal = literal_regions.containing_range(token.start());
        let closes_inactive_root = captures.name("close").is_some()
            && stack
                .first()
                .is_some_and(|outer| !wikidot_tag_conditions_match(&outer.spec, tags))
            && literal.is_some_and(|range| {
                inactive_gate_closes_inside_literal(wikitext, range, token.end())
            });
        if literal.is_some() && !closes_inactive_root {
            continue;
        }

        if captures.name("open").is_some() {
            if let Some(outer) = stack.first_mut() {
                outer.nested_tokens.push(token.start()..token.end());
            }
            stack.push(OpenGate {
                start: token.start(),
                end: token.end(),
                spec: captures
                    .name("spec")
                    .map_or("", |value| value.as_str())
                    .to_owned(),
                nested_tokens: Vec::new(),
                last_nested_closer: None,
            });
            continue;
        }

        if stack.len() > 1 {
            let range = token.start()..token.end();
            let outer = stack.first_mut().expect("nested iftags has outer gate");
            outer.nested_tokens.push(range.clone());
            outer.last_nested_closer = Some(range);
            stack.pop();
            continue;
        }

        let Some(outer) = stack.pop() else {
            if unmatched_mode == UnmatchedBoundaryMode::Preserve {
                replacements.push(Replacement {
                    range: token.start()..token.end(),
                    text: preserved.push_escaped_html_text(token.as_str()),
                });
            }
            continue;
        };
        let text = if wikidot_tag_conditions_match(&outer.spec, tags) {
            preserve_nested_tokens(
                wikitext,
                outer.end..token.start(),
                &outer.nested_tokens,
                preserved,
            )
        } else {
            String::new()
        };
        replacements.push(Replacement {
            range: outer.start..token.end(),
            text,
        });
    }

    if !stack.is_empty() && unmatched_mode == UnmatchedBoundaryMode::Preserve {
        let root = stack.remove(0);
        if let Some(recovery_closer) = root.last_nested_closer {
            let nested_tokens = root
                .nested_tokens
                .iter()
                .filter(|token| token.start < recovery_closer.start)
                .cloned()
                .collect::<Vec<_>>();
            let text = if wikidot_tag_conditions_match(&root.spec, tags) {
                preserve_nested_tokens(
                    wikitext,
                    root.end..recovery_closer.start,
                    &nested_tokens,
                    preserved,
                )
            } else {
                String::new()
            };
            replacements.push(Replacement {
                range: root.start..recovery_closer.end,
                text,
            });
        }
    }

    if replacements.is_empty() {
        return;
    }
    replacements.sort_unstable_by_key(|replacement| replacement.range.start);
    let mut output = String::with_capacity(wikitext.len());
    let mut cursor = 0;
    for replacement in replacements {
        debug_assert!(cursor <= replacement.range.start);
        output.push_str(&wikitext[cursor..replacement.range.start]);
        output.push_str(&replacement.text);
        cursor = replacement.range.end;
    }
    output.push_str(&wikitext[cursor..]);
    *wikitext = output;
}

fn inactive_gate_closes_inside_literal(
    source: &str,
    literal: &Range<usize>,
    closer_end: usize,
) -> bool {
    let literal_source = source[literal.clone()].trim_start_matches([' ', '\t']);
    let head = literal_source.to_ascii_lowercase();
    if head.starts_with("[[raw") {
        return true;
    }
    if head.starts_with("[!--") {
        return true;
    }
    for (open, close) in [
        ("[[code", "[[/code]]"),
        ("[[html", "[[/html]]"),
        ("[[module", "[[/module]]"),
    ] {
        if head.starts_with(open) {
            return !source[closer_end..literal.end]
                .to_ascii_lowercase()
                .contains(close);
        }
    }
    false
}

fn preserve_nested_tokens(
    source: &str,
    body: Range<usize>,
    tokens: &[Range<usize>],
    preserved: &mut CompatTextFragments,
) -> String {
    let mut output = String::with_capacity(body.len());
    let mut cursor = body.start;
    for token in tokens {
        debug_assert!(body.start <= token.start && token.end <= body.end);
        output.push_str(&source[cursor..token.start]);
        output.push_str(&preserved.push_escaped_html_text(&source[token.clone()]));
        cursor = token.end;
    }
    output.push_str(&source[cursor..body.end]);
    output
}

pub(super) fn wikidot_tag_conditions_match(spec: &str, tags: &[Cow<'_, str>]) -> bool {
    if spec.trim().is_empty() {
        return false;
    }

    let mut required = true;
    let mut prohibited = true;
    let mut present = false;
    let mut had_present = false;

    for raw_condition in spec.split_whitespace() {
        let (operator, tag) = raw_condition.split_at(usize::from(
            raw_condition.starts_with('+') || raw_condition.starts_with('-'),
        ));
        if tag.is_empty() {
            continue;
        }

        let has_tag = tags.iter().any(|value| value.as_ref() == tag);
        match operator {
            "+" => required &= has_tag,
            "-" => prohibited &= !has_tag,
            _ => {
                had_present = true;
                present |= has_tag;
            }
        }
    }

    if !had_present {
        present = true;
    }

    required && prohibited && present
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn resolve(source: &str, tags: &[&str], passes: usize) -> String {
        let mut source = source.to_owned();
        let tags = tags
            .iter()
            .map(|tag| Cow::Borrowed(*tag))
            .collect::<Vec<_>>();
        let mut preserved = CompatTextFragments::new(&source);
        for _ in 0..passes {
            resolve_outermost_wikidot_iftags(&mut source, &tags, &mut preserved);
        }
        preserved.restore(&source)
    }

    #[test]
    fn resolves_root_siblings_and_preserves_following_text() {
        let source = concat!(
            "before\n",
            "[[iftags +alpha]]yes[[/iftags]]\n",
            "[[iftags -alpha]]no[[/iftags]]\n",
            "after\n",
        );
        assert_eq!(resolve(source, &["alpha"], 1), "before\nyes\n\nafter\n");
    }

    #[test]
    fn active_outer_preserves_nested_tokens_across_repeated_passes() {
        let source = concat!(
            "[[iftags +alpha]]\n",
            "outer-before\n",
            "[[iftags +beta]]\n",
            "inner\n",
            "[[/iftags]]\n",
            "outer-after\n",
            "[[/iftags]]\n",
            "root-after\n",
        );
        assert_eq!(
            resolve(source, &["alpha"], 3),
            concat!(
                "\nouter-before\n",
                "[[iftags +beta]]\n",
                "inner\n",
                "[[/iftags]]\n",
                "outer-after\n\n",
                "root-after\n",
            ),
        );
    }

    #[test]
    fn inactive_outer_removes_nested_body_without_consuming_root_text() {
        let source = concat!(
            "[[iftags +alpha]]\n",
            "outer\n",
            "[[iftags +beta]]inner[[/iftags]]\n",
            "[[/iftags]]\n",
            "root-after\n",
        );
        assert_eq!(resolve(source, &["beta"], 3), "\nroot-after\n");
    }

    #[test]
    fn partially_balanced_outer_recovers_the_final_closer() {
        let source = "[[iftags +alpha]]outer [[iftags +beta]]inner[[/iftags]]";
        assert_eq!(
            resolve(source, &["alpha", "beta"], 3),
            "outer [[iftags +beta]]inner",
        );
        assert_eq!(resolve(source, &["beta"], 3), "");
    }

    #[test]
    fn partially_balanced_outer_consumes_only_the_final_available_closer() {
        let source = concat!(
            "[[iftags +alpha]]outer\n",
            "[[iftags +beta]]inner[[/iftags]]\n",
            "between\n",
            "[[iftags -beta]]following[[/iftags]]\n",
            "root-after\n",
        );
        assert_eq!(
            resolve(source, &["alpha", "beta"], 3),
            concat!(
                "outer\n",
                "[[iftags +beta]]inner[[/iftags]]\n",
                "between\n",
                "[[iftags -beta]]following\n",
                "root-after\n",
            ),
        );
        assert_eq!(resolve(source, &["beta"], 3), "\nroot-after\n");
    }

    #[test]
    fn include_prepass_defers_openers_and_closer_until_the_sources_are_joined() {
        let tags = [Cow::Borrowed("alpha")];
        let mut preserved = CompatTextFragments::new("");
        let mut target = concat!(
            "target-before\n",
            "[[iftags +alpha]]outer\n",
            "[[iftags +beta]]inner\n",
            "target-end\n",
        )
        .to_owned();
        let mut caller_suffix = "caller-between\n[[/iftags]]\ncaller-after\n".to_owned();

        resolve_outermost_wikidot_iftags_before_include_expansion(
            &mut target,
            &tags,
            &mut preserved,
        );
        resolve_outermost_wikidot_iftags_before_include_expansion(
            &mut caller_suffix,
            &tags,
            &mut preserved,
        );
        assert!(target.contains("[[iftags +alpha]]"));
        assert!(caller_suffix.contains("[[/iftags]]"));

        let mut expanded = format!("{target}{caller_suffix}");
        resolve_outermost_wikidot_iftags(&mut expanded, &tags, &mut preserved);
        let expanded = preserved.restore(&expanded);
        assert_eq!(
            expanded,
            concat!(
                "target-before\n",
                "outer\n",
                "[[iftags +beta]]inner\n",
                "target-end\n",
                "caller-between\n\n",
                "caller-after\n",
            ),
        );
    }

    #[test]
    fn include_prepass_defers_eof_recovery_when_a_caller_closer_can_balance_the_root() {
        let tags = [Cow::Borrowed("alpha")];
        let mut preserved = CompatTextFragments::new("");
        let mut target =
            "[[iftags +alpha]]outer [[iftags +beta]]inner[[/iftags]]".to_owned();

        resolve_outermost_wikidot_iftags_before_include_expansion(
            &mut target,
            &tags,
            &mut preserved,
        );
        assert_eq!(
            target,
            "[[iftags +alpha]]outer [[iftags +beta]]inner[[/iftags]]",
        );

        target.push_str(" caller[[/iftags]] after");
        resolve_outermost_wikidot_iftags(&mut target, &tags, &mut preserved);
        let target = preserved.restore(&target);
        assert_eq!(
            target,
            "outer [[iftags +beta]]inner[[/iftags]] caller after",
        );
    }

    #[test]
    fn include_prepass_still_resolves_self_contained_balanced_gates() {
        let tags = [Cow::Borrowed("alpha")];
        let mut preserved = CompatTextFragments::new("");
        let mut source = concat!(
            "[[iftags -alpha]][[include hidden]][[/iftags]]\n",
            "[[include visible]]\n",
        )
        .to_owned();

        resolve_outermost_wikidot_iftags_before_include_expansion(
            &mut source,
            &tags,
            &mut preserved,
        );
        assert_eq!(source, "\n[[include visible]]\n");
    }

    #[test]
    fn unclosed_openers_and_extra_closers_remain_literal_across_passes() {
        let source = concat!(
            "[[/iftags]]\n",
            "[[iftags +alpha]]selected[[/iftags]]\n",
            "[[/iftags]]\n",
            "[[iftags -alpha]]unclosed\n",
            "[[iftags +beta]]repeated\n",
        );
        assert_eq!(
            resolve(source, &["alpha"], 3),
            concat!(
                "[[/iftags]]\n",
                "selected\n",
                "[[/iftags]]\n",
                "[[iftags -alpha]]unclosed\n",
                "[[iftags +beta]]repeated\n",
            ),
        );
    }

    #[test]
    fn final_pass_leaves_unclosed_openers_for_ftml_literal_recovery() {
        let mut source = concat!(
            "[[iftags +test]]\n",
            "[[div_ class=\"authorlink-wrapper\"]]\n",
            "Calibold",
        )
        .to_owned();
        let original = source.clone();
        let tags = [Cow::Borrowed("test")];
        let mut preserved = CompatTextFragments::new(&source);

        resolve_outermost_wikidot_iftags(&mut source, &tags, &mut preserved);

        assert_eq!(source, original);
    }

    #[test]
    fn literal_region_tokens_do_not_change_pairing() {
        let source = concat!(
            "[[code]]\n[[iftags +alpha]]literal[[/iftags]]\n[[/code]]\n",
            "[[iftags +alpha]]active[[/iftags]]\n",
        );
        assert_eq!(
            resolve(source, &["alpha"], 1),
            concat!(
                "[[code]]\n[[iftags +alpha]]literal[[/iftags]]\n[[/code]]\n",
                "active\n",
            ),
        );
    }

    #[test]
    fn unmatched_inline_raw_does_not_hide_later_inactive_closer() {
        let source = concat!(
            "before\n",
            "[[iftags +component]]\n",
            "documentation\n",
            "* Escaping with @@\n",
            "[[/iftags]]\n",
            "after\n",
        );
        assert_eq!(resolve(source, &[], 1), "before\n\nafter\n");
    }

    #[test]
    fn unmatched_inline_raw_does_not_hide_later_active_closer() {
        let source = concat!(
            "[[iftags +component]]\n",
            "documentation\n",
            "* Escaping with @@\n",
            "[[/iftags]]\n",
        );
        assert_eq!(
            resolve(source, &["component"], 1),
            "\ndocumentation\n* Escaping with @@\n\n",
        );
    }

    #[test]
    fn unmatched_inline_raw_does_not_hide_same_line_inactive_closer() {
        let source =
            "before [[iftags +component]]documentation @@ prose [[/iftags]] after";
        assert_eq!(resolve(source, &[], 1), "before  after");
    }

    #[test]
    fn unmatched_inline_raw_does_not_hide_same_line_active_closer() {
        let source = "[[iftags +component]]documentation @@ prose [[/iftags]]";
        assert_eq!(
            resolve(source, &["component"], 1),
            "documentation @@ prose ",
        );
    }

    #[test]
    fn balanced_inline_raw_closer_does_not_close_active_gate() {
        let source = concat!(
            "[[iftags +component]]\n",
            "@@[[/iftags]]@@\n",
            "selected\n",
            "[[/iftags]]\n",
        );
        assert_eq!(
            resolve(source, &["component"], 1),
            "\n@@[[/iftags]]@@\nselected\n\n",
        );
    }

    #[test]
    fn block_literal_closer_does_not_close_active_gate() {
        let source = concat!(
            "[[iftags +component]]\n",
            "[[code]]\n[[/iftags]]\n[[/code]]\n",
            "selected\n",
            "[[/iftags]]\n",
        );
        assert_eq!(
            resolve(source, &["component"], 1),
            "\n[[code]]\n[[/iftags]]\n[[/code]]\nselected\n\n",
        );
    }

    #[test]
    fn inactive_gate_closes_before_unclosed_literal_blocks() {
        for (source, expected) in [
            (
                concat!(
                    "[[iftags +missing]]\n",
                    "[[code @=\"bad\"]]\n",
                    "[[/iftags]]\n",
                    "[[html]]\n",
                    "<b>malformed head</b>\n",
                    "[[/html]]",
                ),
                "\n[[html]]\n<b>malformed head</b>\n[[/html]]",
            ),
            (
                "[[iftags +missing]]\n[[raw]]\n[[/iftags]]\nunclosed raw",
                "\nunclosed raw",
            ),
            (
                concat!(
                    "[[iftags +missing]]\n",
                    "[[module Rate]]\n",
                    "[[html]]\n",
                    "<b>guarded</b>\n",
                    "[[/html]]\n",
                    "[[/iftags]]\n",
                    "visible",
                ),
                "\nvisible",
            ),
            (
                concat!(
                    "[[iftags +missing]]\n",
                    "[[module CSS]]\n",
                    "[[/iftags]]\n",
                    ".unclosed { color: red; }",
                ),
                "\n.unclosed { color: red; }",
            ),
            (
                concat!(
                    "[[iftags +missing]]\n",
                    "[[module ListPages]]\n",
                    "[[/iftags]]\n",
                    "unclosed module",
                ),
                "\nunclosed module",
            ),
        ] {
            assert_eq!(resolve(source, &[], 1), expected);
        }
    }

    #[test]
    fn inactive_gate_keeps_balanced_module_bodies_opaque() {
        for module in ["ListPages", "CSS"] {
            let source = format!(
                "[[iftags +missing]]\n[[module {module}]]\n[[/iftags]] raw-module\n[[/module]]\n[[html]]\n<b>guarded</b>\n[[/html]]\n[[/iftags]]\nvisible",
            );
            assert_eq!(resolve(&source, &[], 1), "\nvisible", "{module}");
        }
    }

    #[test]
    fn inactive_gate_closes_at_comment_and_raw_cross_boundaries() {
        let comment = concat!(
            "[[iftags +missing]]\n",
            "[!-- [[/iftags]] raw-comment --]\n",
            "[[html]]\n",
            "<b>guarded</b>\n",
            "[[/html]]\n",
            "[[/iftags]]\n",
            "visible",
        );
        assert_eq!(
            resolve(comment, &[], 1),
            concat!(
                " raw-comment --]\n",
                "[[html]]\n",
                "<b>guarded</b>\n",
                "[[/html]]\n",
                "[[/iftags]]\n",
                "visible",
            ),
        );

        let raw = concat!(
            "[[iftags +missing]]\n",
            "[[raw]]\n",
            "[[/iftags]]\n",
            "[[html]]raw-raw[[/html]]\n",
            "[[/raw]]\n",
            "[[html]]\n",
            "<b>guarded</b>\n",
            "[[/html]]\n",
            "[[/iftags]]\n",
            "visible",
        );
        assert_eq!(
            resolve(raw, &[], 1),
            concat!(
                "\n[[html]]raw-raw[[/html]]\n",
                "[[/raw]]\n",
                "[[html]]\n",
                "<b>guarded</b>\n",
                "[[/html]]\n",
                "[[/iftags]]\n",
                "visible",
            ),
        );
    }

    #[test]
    fn inactive_partially_nested_gate_leaves_following_html() {
        let source = concat!(
            "[[iftags +missing]]\n",
            "[[iftags +missing]]\n",
            "inner unclosed\n",
            "[[/iftags]]\n",
            "[[html]]\n",
            "<b>unclosed nested</b>\n",
            "[[/html]]",
        );
        assert_eq!(
            resolve(source, &[], 1),
            "\n[[html]]\n<b>unclosed nested</b>\n[[/html]]",
        );
    }

    #[test]
    fn special_inline_raw_run_does_not_hide_real_closer() {
        let source = "[[iftags +component]]@@@@@@[[/iftags]]@@";
        assert_eq!(resolve(source, &[], 1), "@@");
        assert_eq!(resolve(source, &["component"], 1), "@@@@@@@@");
    }

    #[test]
    fn url_owned_raw_delimiter_does_not_hide_real_closer() {
        let source = "[[iftags +component]]https://e.test/a@@b[[/iftags]]@@tail";
        assert_eq!(resolve(source, &[], 1), "@@tail");
        assert_eq!(
            resolve(source, &["component"], 1),
            "https://e.test/a@@b@@tail",
        );
    }

    #[test]
    fn raw_delimiter_in_tag_head_does_not_hide_real_closer() {
        let source = "[[iftags +x]][[div data=\"@@\"]]body[[/iftags]]@@tail";
        assert_eq!(resolve(source, &[], 1), "@@tail");
        assert_eq!(resolve(source, &["x"], 1), "[[div data=\"@@\"]]body@@tail",);
    }

    #[test]
    fn inactive_gate_closes_inside_comment_while_active_gate_preserves_it() {
        let source = "[[iftags +x]][!-- https://e.test/a--] [[/iftags]] --]";
        assert_eq!(resolve(source, &[], 1), " --]");
        assert_eq!(resolve(source, &["x"], 1), source);
    }

    #[test]
    fn malformed_block_closer_does_not_expose_conditional_closer() {
        let source = concat!(
            "[[iftags +x]][[code]]\n",
            "[[/code]]]\n[[/iftags]]\n",
            "[[/code]]\n[[/iftags]]",
        );
        assert_eq!(resolve(source, &[], 1), "");
        assert_eq!(
            resolve(source, &["x"], 1),
            "[[code]]\n[[/code]]]\n[[/iftags]]\n[[/code]]\n",
        );
    }

    #[test]
    fn preserved_nested_spec_is_html_escaped_for_late_restoration() {
        let source = "[[iftags +alpha]][[iftags \"<unsafe>&]]body[[/iftags]][[/iftags]]";
        assert_eq!(
            resolve(source, &["alpha"], 1),
            "[[iftags &quot;&lt;unsafe&gt;&amp;]]body[[/iftags]]",
        );
    }

    #[test]
    fn repeated_siblings_and_unclosed_openers_stay_within_linear_budget() {
        let mut source = String::new();
        for index in 0..10_000 {
            source.push_str(&format!("[[iftags +alpha]]selected-{index}[[/iftags]]\n"));
        }
        for _ in 0..10_000 {
            source.push_str("[[iftags +alpha]]");
        }
        let tags = [Cow::Borrowed("alpha")];
        let mut preserved = CompatTextFragments::new(&source);
        let started = Instant::now();
        resolve_outermost_wikidot_iftags(&mut source, &tags, &mut preserved);
        assert!(started.elapsed() < Duration::from_secs(2));
        source = preserved.restore(&source);
        assert_eq!(source.matches("selected-").count(), 10_000);
        assert_eq!(source.matches("[[iftags +alpha]]").count(), 10_000);

        let mut source = String::from("[[iftags +alpha]]");
        for _ in 0..10_000 {
            source.push_str("[[iftags +beta]]");
        }
        for _ in 0..9_999 {
            source.push_str("[[/iftags]]");
        }
        let mut preserved = CompatTextFragments::new(&source);
        let started = Instant::now();
        resolve_outermost_wikidot_iftags(&mut source, &tags, &mut preserved);
        assert!(started.elapsed() < Duration::from_secs(2));
        source = preserved.restore(&source);
        assert_eq!(source.matches("[[iftags +beta]]").count(), 10_000);
        assert_eq!(source.matches("[[/iftags]]").count(), 9_998);
    }
}
