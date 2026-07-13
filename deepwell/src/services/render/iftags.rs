//! Wikidot `iftags` selection against runtime page tags.
//!
//! Real Wikidot evaluates root-level gates once. If an active gate contains
//! another `iftags` pair, the nested pair remains literal instead of becoming
//! another condition. Render preparation has several passes, so nested tokens
//! are registered temporarily to keep later passes from evaluating them.

use super::compat_text_fragments::CompatTextFragments;
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
}

#[derive(Debug)]
struct Replacement {
    range: Range<usize>,
    text: String,
}

pub(super) fn resolve_outermost_wikidot_iftags(
    wikitext: &mut String,
    tags: &[Cow<'_, str>],
    preserved: &mut CompatTextFragments,
) {
    let literal_regions = LiteralRegionIndex::new_wikidot_syntax(wikitext);
    let mut stack = Vec::<OpenGate>::new();
    let mut replacements = Vec::<Replacement>::new();

    for captures in IFTAGS_TOKEN_REGEX.captures_iter(wikitext) {
        let token = captures.get(0).expect("iftags token");
        if literal_regions.contains(token.start()) {
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
            });
            continue;
        }

        if stack.len() > 1 {
            stack
                .first_mut()
                .expect("nested iftags has outer gate")
                .nested_tokens
                .push(token.start()..token.end());
            stack.pop();
            continue;
        }

        let Some(outer) = stack.pop() else {
            replacements.push(Replacement {
                range: token.start()..token.end(),
                text: preserved.push(&escape_html(token.as_str())),
            });
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

    for unclosed in stack {
        replacements.push(Replacement {
            range: unclosed.start..unclosed.end,
            text: preserved.push(&escape_html(&wikitext[unclosed.start..unclosed.end])),
        });
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
        output.push_str(&preserved.push(&escape_html(&source[token.clone()])));
        cursor = token.end;
    }
    output.push_str(&source[cursor..body.end]);
    output
}

fn escape_html(value: &str) -> String {
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
    fn unbalanced_outer_remains_unchanged() {
        let source = "[[iftags +alpha]]outer [[iftags +beta]]inner[[/iftags]]";
        assert_eq!(resolve(source, &["alpha", "beta"], 1), source);
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
    }
}
