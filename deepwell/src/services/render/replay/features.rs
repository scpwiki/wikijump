/*
 * services/render/replay/features.rs
 *
 * Deterministic, allocation-bounded syntax features and parser error anchors.
 */

use super::model::{
    ErrorSite, FailureSignature, ReplayStage, SyntaxFeatures, sha256_hex,
};
use ftml::parsing::ParseError;
use std::collections::BTreeMap;

const MARKERS: [(&str, &str); 19] = [
    ("block_open", "[["),
    ("block_close", "]]"),
    ("align_right_open", "[[>]]"),
    ("align_right_close", "[[/>]]"),
    ("bold", "**"),
    ("italics", "//"),
    ("underline", "__"),
    ("superscript", "^^"),
    ("subscript", ",,"),
    ("color", "##"),
    ("size", "[[size"),
    ("collapsible", "[[collapsible"),
    ("tab", "[[tab"),
    ("div", "[[div"),
    ("span", "[[span"),
    ("include", "[[include"),
    ("module", "[[module"),
    ("table", "||"),
    ("raw", "@@"),
];

pub(crate) fn syntax_features(wikitext: &str) -> SyntaxFeatures {
    let mut features = SyntaxFeatures {
        bytes: wikitext.len(),
        lines: wikitext
            .lines()
            .count()
            .max(usize::from(!wikitext.is_empty())),
        ..SyntaxFeatures::default()
    };
    let mut shapes = BTreeMap::<String, usize>::new();

    for line in wikitext.lines() {
        features.max_line_bytes = features.max_line_bytes.max(line.len());
        let trimmed = line.trim_start();
        let quote_depth = trimmed.bytes().take_while(|byte| *byte == b'>').count();
        features.max_quote_depth = features.max_quote_depth.max(quote_depth);

        bump_if(&mut features.marker_counts, "quoted_line", quote_depth > 0);
        bump_if(
            &mut features.marker_counts,
            "ordered_list_line",
            line_prefix_after_quotes(trimmed).starts_with('#'),
        );
        bump_if(
            &mut features.marker_counts,
            "bullet_list_line",
            line_prefix_after_quotes(trimmed).starts_with('*'),
        );

        let shape = line_shape(line);
        if !shape.is_empty() {
            *shapes.entry(shape).or_default() += 1;
        }
    }

    for (name, marker) in MARKERS {
        let count = wikitext.matches(marker).count();
        if count > 0 {
            features.marker_counts.insert(name.to_owned(), count);
        }
    }

    for (name, open, close) in [
        ("block", "[[", "]]"),
        ("bold", "**", "**"),
        ("underline", "__", "__"),
        ("superscript", "^^", "^^"),
        ("subscript", ",,", ",,"),
        ("raw", "@@", "@@"),
    ] {
        let open_count = wikitext.matches(open).count();
        let close_count = wikitext.matches(close).count();
        let unbalanced = if open == close {
            open_count % 2 != 0
        } else {
            open_count != close_count
        };
        if unbalanced {
            features.unbalanced_mask.push(name.to_owned());
        }
    }

    if let Some((shape, count)) = shapes
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .max_by(|(left_shape, left_count), (right_shape, right_count)| {
            left_count
                .cmp(right_count)
                .then_with(|| right_shape.cmp(left_shape))
        })
    {
        features.dominant_line_shape = Some(sha256_hex(shape.as_bytes()));
        features.dominant_line_repetitions = count;
    }

    features
}

fn bump_if(counts: &mut BTreeMap<String, usize>, key: &str, condition: bool) {
    if condition {
        *counts.entry(key.to_owned()).or_default() += 1;
    }
}

fn line_prefix_after_quotes(mut line: &str) -> &str {
    while let Some(rest) = line.strip_prefix('>') {
        line = rest.trim_start_matches([' ', '\t']);
    }
    line
}

/// Preserve syntax atoms and collapse page-specific prose/attribute values.
pub(crate) fn line_shape(line: &str) -> String {
    let mut shape = String::new();
    let mut text_run = false;
    let mut quote = None;

    for character in line.trim().chars().take(4096) {
        if let Some(delimiter) = quote {
            if character == delimiter {
                quote = None;
                shape.push('Q');
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            quote = Some(character);
            text_run = false;
            continue;
        }
        if character.is_alphanumeric() || matches!(character, '-' | '_') {
            if !text_run {
                shape.push('A');
                text_run = true;
            }
            continue;
        }
        text_run = false;
        if !character.is_whitespace() {
            shape.push(character);
        }
    }
    if quote.is_some() {
        shape.push('Q');
    }
    shape
}

pub(crate) fn error_sites(wikitext: &str, errors: &[ParseError]) -> Vec<ErrorSite> {
    let mut line_starts = vec![0];
    line_starts.extend(
        wikitext
            .match_indices('\n')
            .map(|(index, _)| index.saturating_add(1)),
    );

    errors
        .iter()
        .take(256)
        .map(|error| {
            let span = error.span();
            let line_index = line_starts
                .partition_point(|start| *start <= span.start)
                .saturating_sub(1);
            let line_start = line_starts[line_index];
            let line_end = wikitext[line_start..]
                .find('\n')
                .map_or(wikitext.len(), |offset| line_start + offset);
            let line = &wikitext[line_start..line_end];
            let context_start = line_index.saturating_sub(1);
            let context_byte_start = line_starts[context_start];
            let context_byte_end = line_starts
                .get(line_index + 2)
                .copied()
                .unwrap_or(wikitext.len());

            ErrorSite {
                rule: error.rule().to_owned(),
                kind: serde_json::to_value(error.kind())
                    .ok()
                    .and_then(|value| value.as_str().map(ToOwned::to_owned))
                    .unwrap_or_else(|| format!("{:?}", error.kind())),
                token: error.token().name().to_owned(),
                start: span.start,
                end: span.end,
                line: line_index + 1,
                column: wikitext[line_start..span.start.min(wikitext.len())]
                    .chars()
                    .count()
                    + 1,
                line_shape: line_shape(line),
                context_hash: sha256_hex(
                    &wikitext.as_bytes()[context_byte_start..context_byte_end],
                ),
            }
        })
        .collect()
}

pub(crate) fn parser_error_signature(sites: &[ErrorSite]) -> FailureSignature {
    let primary = sites.first();
    let key = primary.map_or_else(
        || "parser-error-without-site".to_owned(),
        |site| {
            sha256_hex(
                format!(
                    "{}|{}|{}|{}",
                    site.rule, site.kind, site.token, site.line_shape
                )
                .as_bytes(),
            )
        },
    );
    FailureSignature {
        class: "parser_errors".to_owned(),
        stage: ReplayStage::Parse,
        key,
    }
}

pub(crate) fn timeout_signature(
    stage: ReplayStage,
    _features: &SyntaxFeatures,
) -> FailureSignature {
    FailureSignature {
        class: "timeout".to_owned(),
        stage,
        key: format!("{:?}", stage),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn features_detect_repeated_quoted_alignment_and_ordered_size_lines() {
        let input = concat!(
            "> [[>]]\n> aligned\n> [[/>]]\n",
            "# [[size 0%]]__ [[/size]]\n",
            "# [[size 0%]]__ [[/size]]\n",
        );
        let features = syntax_features(input);

        assert_eq!(features.marker_counts["quoted_line"], 3);
        assert_eq!(features.marker_counts["ordered_list_line"], 2);
        assert_eq!(features.marker_counts["size"], 2);
        assert_eq!(features.dominant_line_repetitions, 2);
        assert!(features.dominant_line_shape.is_some());
    }

    #[test]
    fn line_shape_ignores_page_specific_text_and_quoted_values() {
        assert_eq!(
            line_shape(r#"# [[size 0%]]Alpha [[/size]]"#),
            line_shape(r#"# [[size 99%]]Beta [[/size]]"#),
        );
    }

    #[test]
    fn parser_error_signature_keeps_the_syntax_shape_for_ddmin() {
        let mut first = ErrorSite {
            rule: "paragraph".to_owned(),
            kind: "unexpected_token".to_owned(),
            token: "block-end".to_owned(),
            start: 0,
            end: 1,
            line: 1,
            column: 1,
            line_shape: "[[A]]".to_owned(),
            context_hash: "page-specific".to_owned(),
        };
        let first_signature = parser_error_signature(&[first.clone()]);

        first.context_hash = "changed-by-minimization".to_owned();
        assert_eq!(first_signature, parser_error_signature(&[first.clone()]));

        first.line_shape = "[[/A]]".to_owned();
        assert_ne!(first_signature, parser_error_signature(&[first]));
    }
}
