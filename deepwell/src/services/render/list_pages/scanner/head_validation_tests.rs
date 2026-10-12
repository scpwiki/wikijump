use super::*;

#[test]
fn quote_balance_counts_only_unescaped_quotes() {
    for balanced in [r#"abc"#, r#""a""#, r#""a\\""#, r#"'a'"#, r#""""#] {
        let (head, quote) = if balanced.contains('\'') {
            (balanced, b'\'')
        } else {
            (balanced, b'"')
        };
        assert!(
            list_pages_head_quotes_are_balanced(head, quote),
            "{balanced:?}"
        );
    }

    // An odd quote, or a quote escaped by an odd run of backslashes, is
    // unbalanced.
    assert!(!list_pages_head_quotes_are_balanced(r#""a"#, b'"'));
    assert!(!list_pages_head_quotes_are_balanced(r#""a\""#, b'"'));
    assert!(!list_pages_head_quotes_are_balanced(r#"'a"#, b'\''));
    assert!(list_pages_head_quotes_are_balanced(r#""a""#, b'"'));
    assert!(!list_pages_head_double_quotes_are_balanced(r#""a"#));
    assert!(list_pages_head_double_quotes_are_balanced(r#""a""#));
}

#[test]
fn nested_module_tokens_are_detected_through_whitespace_and_closing_markers() {
    for module in [
        "[[module ListPages]]",
        "[[ module ListPages]]",
        "[[/module]]",
        "[[module_ ListPages]]",
        "[[MoDuLe ListPages]]",
    ] {
        assert!(
            list_pages_head_contains_nested_module_token(module),
            "{module}"
        );
    }
    for not_module in ["[[a]]", "[[moduleX]]", "no brackets", ""] {
        assert!(
            !list_pages_head_contains_nested_module_token(not_module),
            "{not_module}"
        );
    }
}

#[test]
fn bracket_tokens_require_a_closed_pair_and_reject_nested_modules() {
    for supported in ["", "[[a]]", "[[a]] [[b]]", "plain"] {
        assert!(
            list_pages_head_bracket_tokens_are_supported(supported),
            "{supported}"
        );
    }
    for unsupported in [
        "[a]",
        "[a]]",
        "x [ y",
        "a ]] b",
        "[[a]",
        "[[module x]]",
        "[[/module]]",
    ] {
        assert!(
            !list_pages_head_bracket_tokens_are_supported(unsupported),
            "{unsupported}"
        );
    }
}

#[test]
fn definite_invalid_heads_do_not_execute() {
    for invalid in [
        "a\u{000b}b",
        "a\u{000c}b",
        "a\u{00a0}b",
        "[[!-- comment --]]",
        "x --] y",
        "[[a]",
        "]",
        "tags='unbalanced",
    ] {
        assert!(
            !list_pages_definite_invalid_head_can_execute(invalid),
            "{invalid:?}"
        );
    }

    // A plain head and an argument head with no nested module execute.
    assert!(list_pages_definite_invalid_head_can_execute("ListPages"));
    assert!(list_pages_definite_invalid_head_can_execute(
        r#"ListPages tags="a""#
    ));
    // A nested module token inside the arguments does not.
    assert!(!list_pages_definite_invalid_head_can_execute(
        r#"ListPages tags="[[module x]]""#
    ));
}

#[test]
fn backslash_escaping_depends_on_parity() {
    for (source, start, expected) in [
        ("[[a]]", 0, false),
        (r"\[[a]]", 1, true),
        (r"\\[[a]]", 2, false),
        (r"x\[[a]]", 2, true),
        (r"x\\[[a]]", 3, false),
        (r"x\\\[[a]]", 4, true),
    ] {
        assert_eq!(
            source_block_open_is_backslash_escaped(source, start),
            expected,
            "{source:?} at {start}",
        );
    }
}

#[test]
fn url_value_quote_recognition_requires_the_at_url_prefix() {
    let starts = |value: &str, quote: usize| {
        list_pages_url_value_quote_starts_at(value.as_bytes(), quote)
    };
    assert!(starts(r#""@URL""#, 0));
    assert!(starts(r#""@url|""#, 0));
    assert!(!starts(r#""@URL ""#, 0));
    assert!(!starts(r#""@URL""#, 5));
    assert!(!starts(r#""other""#, 0));

    let ends = |value: &str, quote: usize, lower_bound: usize| {
        list_pages_url_value_quote_ends_at(value.as_bytes(), quote, lower_bound)
    };
    assert!(ends(r#""@URL""#, 5, 0));
    assert!(!ends(r#""other""#, 5, 0));
    assert!(!ends(r#""@URL"x""#, 7, 0));
    assert!(!ends("abc", 3, 0));
    // A non-zero lower bound must still resolve the opening quote.
    assert!(ends(r#"xx"@URL""#, 7, 2));
}

#[test]
fn empty_list_pages_assignment_leaves_later_arguments_and_body_available() {
    // Frozen Wikidot oracle: listpages-head-boundary-live.jsonl,
    // case listpages-head-boundary-empty-prepend-assignment.
    let source = concat!(
        "[[module ListPages category=\"*\" limit=\"1\" order=\"name\" prependLine=]]\n",
        "ROW|%%fullname%%\n",
        "[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1, "{modules:#?}");
    assert!(list_pages_runtime_head_can_execute(modules[0].head));
    assert!(!modules[0].preserve_original, "{modules:#?}");
    assert_eq!(modules[0].body, "\nROW|%%fullname%%\n");

    let arguments = wikidot_list_pages_arguments(modules[0].head);
    assert_eq!(
        arguments
            .iter()
            .map(|argument| (argument.key, argument.op, argument.value))
            .collect::<Vec<_>>(),
        vec![
            ("category", "=", "*"),
            ("limit", "=", "1"),
            ("order", "=", "name")
        ],
    );
}
