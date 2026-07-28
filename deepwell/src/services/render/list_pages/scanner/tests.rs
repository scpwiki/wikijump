use super::*;
use crate::services::render::module_arguments::module_arguments_are_complete;

#[path = "tests/stress.rs"]
mod stress;

#[test]
fn runtime_head_recognizer_matches_the_execution_regex() {
    for (head, expected) in [
        ("", true),
        (" name=\"x\"", true),
        ("name='x'other=bare", true),
        ("name=\"unterminated", true),
        ("name=\"unterminated next=bare", true),
        ("name=\"x\"wrapper=\"y\"", true),
        ("name=\"x\"wrapper=", false),
        ("1=bare", false),
        ("[!--=bare", false),
        ("name=a]b", false),
        ("limit\n=\"1\"", true),
        ("name=\"secret@site.example\" wrapper=no\"", true),
        ("name = \"secret\"wrapper=\"no\"", true),
        ("name = \"secret\\\" wrapper=no\"", true),
        (r#"order="name"" limit="1""#, true),
        (r#"tags="+scp rating="<0" separate="no""#, true),
        ("name=\u{00a0}bare", true),
        ("name=\"x\" \u{000b}", true),
        ("name=\"x\" @", false),
        ("ſ=\"x\"", false),
        ("K=\"x\"", false),
    ] {
        assert_eq!(
            runtime_regex_recognizes_entire_head(head),
            expected,
            "{head:?}",
        );
        assert_eq!(
            module_arguments_are_complete(head),
            expected,
            "execution regex: {head:?}",
        );
    }
}

#[test]
fn runtime_head_validation_accepts_static_wildcard_selectors() {
    for head in [r#" category="*" "#, r#" category="*" tags="codex" "#] {
        assert_eq!(
            validate_module_head(head, true),
            ModuleHeadValidation::RuntimeSafe,
            "{head:?}",
        );
        assert!(list_pages_runtime_head_is_safe(head), "{head:?}");
    }
}

#[test]
fn linear_list_pages_scanner_preserves_nested_modules_and_source_order() {
    let source = concat!(
        "before\n",
        "[[module Listpages name=\"first\"]]A",
        "[[module ListUsers users=\".\"]]nested[[/module]]",
        "B[[/module]]\n",
        "between\n",
        "[[module LISTPAGES name=\"second\"]]C[[/module]]\n",
        "after",
    );
    let modules = find_list_pages_module_matches(source);
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].head, "name=\"first\"");
    assert!(modules[0].body.contains("[[module ListUsers"));
    assert!(modules[0].body.ends_with('B'));
    assert_eq!(modules[1].head, "name=\"second\"");
    assert_eq!(modules[1].body, "C");
    assert!(modules[0].end < modules[1].start);
    assert_eq!(
        modules[0].original,
        "[[module Listpages name=\"first\"]]A[[module ListUsers users=\".\"]]nested[[/module]]B[[/module]]",
    );
}

#[test]
fn list_pages_scanner_requires_module_and_subname_delimiters() {
    let source = concat!(
        "[[moduleListPages name=\"joined\"]]ignored[[/module]]\n",
        "[[module ListPagesExtra name=\"suffix\"]]ignored[[/module]]\n",
        "[[module ListPages.other name=\"suffix\"]]ignored[[/module]]\n",
        "[[module654 ListPages name=\"legacy\"]]ignored[[/module654]]\n",
        "[[module\tLISTPAGES\tname=\"live\"]]kept[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
    assert_eq!(modules[0].body, "kept");
    assert!(modules[0].runtime_safe);

    let missing_subname = concat!(
        "[[module ListPages name=\"outer\"]]A",
        "[[module]]missing[[/module]]",
        "after[[/module]]",
    );
    let modules = find_list_pages_module_matches(missing_subname);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].body, "A[[module]]missing");

    let legacy_nested = concat!(
        "[[module ListPages name=\"outer\"]]A",
        "[[module654 ListUsers]]B[[/module654]]C",
        "[[module654 ListUsers]]D[[/module]]E",
        "[[/module]]",
    );
    let modules = find_list_pages_module_matches(legacy_nested);
    assert_eq!(modules.len(), 1);
    assert_eq!(
        modules[0].body,
        "A[[module654 ListUsers]]B[[/module654]]C[[module654 ListUsers]]D[[/module]]E",
    );
}

#[test]
fn list_pages_scanner_accepts_pinned_horizontal_space_delimiters() {
    let source = concat!(
        "[[ \tmodule\tListPages name=\"spaced\"]]A[[/ \tmodule \t]]\n",
        "[[module ListPages name=\"outer\"]]B",
        "[[ \tmodule654\tListUsers]]nested[[/ \tmodule654\t]]C",
        "[[/module]]\n",
        "[[ module654 ListPages name=\"legacy\"]]ignored[[/ module654 ]]\n",
        "[[module ListPages name=\"last\"]]D[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 3);
    assert_eq!(modules[0].head, "name=\"spaced\"");
    assert_eq!(modules[0].body, "A");
    assert_eq!(
        modules[0].original,
        "[[ \tmodule\tListPages name=\"spaced\"]]A[[/ \tmodule \t]]",
    );
    assert_eq!(modules[1].head, "name=\"outer\"");
    assert_eq!(
        modules[1].body,
        "B[[ \tmodule654\tListUsers]]nested[[/ \tmodule654\t]]C",
    );
    assert_eq!(modules[2].head, "name=\"last\"");
    assert_eq!(modules[2].body, "D");
}

#[test]
fn list_pages_scanner_uses_pinned_module_name_delimiters() {
    for invalid_open in [
        "[[\nmodule ListPages name=\"lf-before-module\"]]body[[/module]]",
        "[[module \nListPages name=\"space-before-lf\"]]body[[/module]]",
        "[[module\u{000b}ListPages name=\"vt\"]]body[[/module]]",
        "[[module\u{000c}ListPages name=\"ff\"]]body[[/module]]",
    ] {
        assert!(
            find_list_pages_module_matches(invalid_open).is_empty(),
            "invalid opener was recognized: {invalid_open:?}",
        );
    }

    for invalid_close in ["[[/\nmodule]]", "[[/module \n]]"] {
        let source =
            format!("[[module ListPages name=\"live\"]]A{invalid_close}B[[/module]]",);
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "invalid closer {invalid_close:?}");
        assert_eq!(
            modules[0].body,
            format!("A{invalid_close}B"),
            "invalid closer {invalid_close:?}",
        );
    }

    let scored_root = "[[module ListPages name=\"live\"]]A[[/module_]]OUT[[/module]]";
    let modules = find_list_pages_module_matches(scored_root);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].body, "A");

    let scored_nested = concat!(
        "[[module ListPages name=\"outer\"]]A",
        "[[module654 ListUsers]]B[[/module654_]]C[[/module]]",
    );
    let modules = find_list_pages_module_matches(scored_nested);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].body, "A[[module654 ListUsers]]B[[/module654_]]C");

    for optional_left_block_close in ["[[/[[module]]", "[[/ [[ module]]"] {
        let source = format!(
            "[[module ListPages name=\"live\"]]A{optional_left_block_close}OUT[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "{optional_left_block_close:?}");
        assert_eq!(modules[0].body, "A", "{optional_left_block_close:?}");
    }

    let nested_optional_left_block = concat!(
        "[[module ListPages name=\"outer\"]]A",
        "[[module654 ListUsers]]B[[/ [[ module654_]]C[[/module]]",
    );
    let modules = find_list_pages_module_matches(nested_optional_left_block);
    assert_eq!(modules.len(), 1);
    assert_eq!(
        modules[0].body,
        "A[[module654 ListUsers]]B[[/ [[ module654_]]C"
    );

    for trimmed_close in ["[[/module\u{000b}]]", "[[/module\u{000c}]]"] {
        let source =
            format!("[[module ListPages name=\"live\"]]A{trimmed_close}B[[/module]]",);
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "trimmed closer {trimmed_close:?}");
        assert_eq!(modules[0].body, "A", "trimmed closer {trimmed_close:?}");
    }

    for (label, separator) in [
        ("LF", "\n"),
        ("CRLF", "\r\n"),
        ("CR", "\r"),
        ("paragraph LF", "\n\n"),
        ("paragraph CRLF", "\r\n\r\n"),
    ] {
        let source = format!(
            "[[module{separator} \tListPages name=\"{label}\"]]body[[/module{separator} \t]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{label}");
        assert_eq!(modules[0].head, format!("name=\"{label}\""), "{label}");
        assert_eq!(modules[0].body, "body", "{label}");
    }
}

#[test]
fn count_pages_preflight_accepts_regex_valid_mixed_whitespace() {
    for separator in [" \n", "\t\r\n \t", "\n "] {
        let source =
            format!("[[module{separator}CountPages tags=\"+active\"]]A[[/module]]");
        assert!(
            has_count_pages_module_opening_candidate(&source),
            "CountPages preflight should accept regex-valid separator {separator:?}",
        );
    }
}

#[test]
fn list_pages_preflight_keeps_pinned_module_name_delimiters() {
    assert!(!has_list_pages_module_opening_candidate(
        "[[module \nListPages name=\"space-before-lf\"]]body[[/module]]",
    ));
}

#[test]
fn multiline_module_names_participate_in_active_nesting() {
    for (label, separator) in [
        ("LF", "\n"),
        ("CRLF", "\r\n"),
        ("CR", "\r"),
        ("paragraph", "\n\n"),
    ] {
        let nested = format!("[[module{separator} Foo]]B[[/module{separator} ]]C");
        let source = format!("[[module ListPages name=\"outer\"]]A{nested}[[/module]]",);
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{label}");
        assert_eq!(modules[0].body, format!("A{nested}"), "{label}");
    }
}

#[test]
fn list_pages_scanner_uses_projected_structural_opens_only_for_active_nesting() {
    let source = concat!(
        "[[module ListPages name=\"outer\"]]before",
        "[[module\0Foo]]generic[[/module]]",
        "[[module ListPages name=\"inner\"]]inner[[/module]]",
        "after[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"outer\"");
    assert_eq!(
        modules[0].body,
        concat!(
            "before[[module\0Foo]]generic[[/module]]",
            "[[module ListPages name=\"inner\"]]inner[[/module]]after",
        ),
    );

    let projected_roots = concat!(
        "[[module\0ListPages name=\"nul\"]]ignored[[/module]]\n",
        "[[module\\\n ListPages name=\"continued\"]]ignored[[/module]]\n",
        "[[module ListPages name=\"live\"]]kept[[/module]]",
    );
    let modules = find_list_pages_module_matches(projected_roots);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
}

#[test]
fn list_pages_scanner_projects_continued_structural_open_delimiters() {
    for (label, line_end) in [("LF", "\n"), ("CRLF", "\r\n"), ("CR", "\r")] {
        let nested_open = format!("[[module\\{line_end} Foo]]");
        let source = format!(
            "[[module ListPages name=\"outer\"]]A{nested_open}B[[/module]]\
             [[module ListPages name=\"inner\"]]C[[/module]]D[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{label}");
        assert_eq!(modules[0].head, "name=\"outer\"", "{label}");
        assert!(modules[0].body.contains(&nested_open), "{label}");
        assert!(modules[0].body.ends_with('D'), "{label}");
    }

    let cascading = concat!(
        "[[module ListPages name=\"outer\"]]A",
        "[[module\\\\\n\n Foo]]B[[/module]]",
        "[[module ListPages name=\"inner\"]]C[[/module]]D",
        "[[/module]]",
    );
    let modules = find_list_pages_module_matches(cascading);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"outer\"");
    assert!(modules[0].body.ends_with('D'));
}

#[test]
fn list_pages_scanner_does_not_invent_a_missing_projected_module_delimiter() {
    for (label, line_end) in [("LF", "\n"), ("CRLF", "\r\n"), ("CR", "\r")] {
        let source = format!(
            "[[module ListPages name=\"outer\"]]A\
             [[module\\{line_end}Foo]]B[[/module]]\
             [[module ListPages name=\"inner\"]]C[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 2, "{label}");
        assert_eq!(modules[0].head, "name=\"outer\"", "{label}");
        assert_eq!(modules[1].head, "name=\"inner\"", "{label}");
    }
}

#[test]
fn list_pages_scanner_projects_structural_module_closers() {
    for (label, close) in [
        ("NUL", "[[/module\0]]"),
        ("LF", "[[/module\\\n ]]"),
        ("CRLF", "[[/module\\\r\n ]]"),
        ("CR", "[[/module\\\r ]]"),
    ] {
        let source = format!(
            "[[module ListPages name=\"outer\"]]body{close}\
             [[module ListPages name=\"live\"]]kept[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 2, "{label}");
        assert_eq!(modules[0].head, "name=\"outer\"", "{label}");
        assert_eq!(modules[0].body, "body", "{label}");
        assert_eq!(
            modules[0].original,
            format!("[[module ListPages name=\"outer\"]]body{close}"),
            "{label}"
        );
        assert_eq!(modules[1].head, "name=\"live\"", "{label}");
    }
}

#[test]
fn list_pages_scanner_respects_left_bracket_token_precedence() {
    for run in [4usize, 8] {
        let source = format!(
            "{}module ListPages name=\"forged\"]]body[[/module]]",
            "[".repeat(run),
        );
        assert!(
            find_list_pages_module_matches(&source).is_empty(),
            "run {run}"
        );
    }

    let nine = format!(
        "{}module ListPages name=\"real\"]]body[[/module]]",
        "[".repeat(9),
    );
    let modules = find_list_pages_module_matches(&nine);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].start, 7);
    assert_eq!(modules[0].body, "body");

    let forged_close = concat!(
        "[[module ListPages name=\"outer\"]]A",
        "[[[[/module]]B[[/module]]\n",
        "[[module ListPages name=\"inner\"]]C[[/module]]",
    );
    let modules = find_list_pages_module_matches(forged_close);
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].body, "A[[[[/module]]B");
    assert_eq!(modules[1].head, "name=\"inner\"");
}

#[test]
fn projected_scanning_is_applied_exactly_once() {
    for source in [
        "\0> [[module ListPages name=\"nul\"]]A[[/module]]",
        "\\\n > [[module ListPages name=\"continued\"]]B[[/module]]",
    ] {
        let modules = find_list_pages_module_matches(source);
        assert_eq!(modules.len(), 1, "{source:?}");
    }
}

#[test]
fn projected_structural_end_excludes_deleted_eof_suffix() {
    let source = "[[module ListPages name=\"live\"]]body[[/module]]\\\n";
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1);
    assert_eq!(
        modules[0].original,
        "[[module ListPages name=\"live\"]]body[[/module]]"
    );
    assert_eq!(modules[0].end, source.len() - 2);
}

#[test]
fn unresolved_parser_functions_always_fail_closed() {
    let ambiguous = concat!(
        "prefix\\\n",
        "[[#if true]][[module ListPages name=\"hidden\"]]A[[/module]][[/if]]",
    );
    assert!(find_list_pages_module_matches(ambiguous).is_empty());

    for unresolved in ["[[#if true]]text[[/if]]", "[[[#if true]]text[[/if]]"] {
        let source =
            format!("{unresolved}[[module ListPages name=\"hidden\"]]B[[/module]]",);
        assert!(find_list_pages_module_matches(&source).is_empty());
    }

    let rolled_back_generic = concat!(
        "[[span title='[[#if true]]owned[[/if]]']]",
        "[[module ListPages name=\"hidden\"]]B[[/module]]",
    );
    assert!(find_list_pages_module_matches(rolled_back_generic).is_empty());
}

#[test]
fn owned_unresolved_and_whole_head_markers_do_not_suppress_live_modules() {
    for source in [
        concat!(
            "@@[[#if true]]@@ ",
            "[[module ListPages name=\"live\"]]Y[[/module]]",
        ),
        concat!(
            "[[span title=\"[[#if true]]owned[[/if]]\"]] ",
            "[[module ListPages name=\"live\"]]Y[[/module]]",
        ),
        concat!(
            "[https://e.test/ [[user foo [[module ListPages name=\"fake\"]]",
            "X[[/module]] ]] ] ",
            "[[module ListPages name=\"live\"]]Y[[/module]]",
        ),
        concat!(
            "[[# tabanchor]]\n",
            "[[module ListPages name=\"live\"]]Y[[/module]]",
        ),
    ] {
        let modules = find_list_pages_module_matches(source);
        assert_eq!(modules.len(), 1, "{source:?}");
        assert_eq!(modules[0].head, "name=\"live\"", "{source:?}");
    }
}

#[test]
fn original_css_ownership_filters_projected_structural_events() {
    let source = concat!(
        "[[module ListPages name=\"outer\"]]A\n",
        ">\0[[module CSS]]",
        "[[module ListPages name=\"hidden\"]]B[[/module]]",
        "[[/module]]C",
        "[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"outer\"");
    assert!(modules[0].body.ends_with('C'));
}

#[test]
fn continued_quote_close_can_still_be_a_direct_list_pages_candidate() {
    for line_end in ["\n", "\r\n", "\r"] {
        let source =
            format!("[[module ListPages name=\"live\"\\{line_end}]]body[[/module]]",);
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "{line_end:?}");
        assert_eq!(modules[0].body, "body", "{line_end:?}");
    }
}

#[test]
fn malformed_nested_module_head_does_not_swallow_the_outer_close() {
    let source = concat!(
        "[[module ListPages name=\"outer\"]]A",
        "[[module Foo [[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"outer\"");
    assert_eq!(modules[0].body, "A[[module Foo ");
}

#[test]
fn list_pages_scanner_ignores_right_block_tokens_inside_double_quoted_arguments() {
    let source = concat!(
        "[[module ListPages name=\"first\" prependLine=\"literal ]] value\"]]",
        "A[[/module]]\n",
        "[[module ListPages name=\"doubled\" prependLine=\"\"literal ]] value\"\"]]",
        "B[[/module]]\n",
        "[[module ListPages name=\"embedded\" prependLine=\"the \"literal ]] value\" suffix\"]]",
        "C[[/module]]\n",
        "[[module ListPages name=\"last\"]]D[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 4);
    assert_eq!(
        modules[0].head,
        "name=\"first\" prependLine=\"literal ]] value\"",
    );
    assert_eq!(modules[0].body, "A");
    assert_eq!(
        modules[1].head,
        "name=\"doubled\" prependLine=\"\"literal ]] value\"\"",
    );
    assert_eq!(modules[1].body, "B");
    assert_eq!(
        modules[2].head,
        "name=\"embedded\" prependLine=\"the \"literal ]] value\" suffix\"",
    );
    assert_eq!(modules[2].body, "C");
    assert_eq!(modules[3].head, "name=\"last\"");
    assert_eq!(modules[3].body, "D");
}

#[test]
fn list_pages_scanner_ignores_right_block_tokens_inside_single_quoted_arguments() {
    let source = concat!(
        "[[module ListPages name='first' prependLine='literal ]] value']]",
        "A[[/module]]\n",
        r#"[[module ListPages name='escaped' prependLine='literal \' ]] value']]"#,
        "B[[/module]]\n",
        "[[module ListPages name='last']]C[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 3);
    assert_eq!(
        modules[0].head,
        "name='first' prependLine='literal ]] value'",
    );
    assert_eq!(modules[0].body, "A");
    assert_eq!(
        modules[1].head,
        r#"name='escaped' prependLine='literal \' ]] value'"#,
    );
    assert_eq!(modules[1].body, "B");
    assert_eq!(modules[2].head, "name='last'");
    assert_eq!(modules[2].body, "C");
}

#[test]
fn list_pages_scanner_rejects_right_link_runs_as_module_head_terminators() {
    let source = concat!(
        "[[module ListPages name=\"malformed\"]]]ignored[[/module]]\n",
        "[[module ListPages name=\"live\"]]kept[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
    assert_eq!(modules[0].body, "kept");
}

#[test]
fn list_pages_scanner_rejects_right_link_runs_as_module_close_terminators() {
    let source = concat!(
        "[[module ListPages name=\"outer\"]]",
        "before[[/module]]]after",
        "[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"outer\"");
    assert_eq!(modules[0].body, "before[[/module]]]after");
}

#[test]
fn list_pages_scanner_fails_closed_when_a_quoted_head_reaches_a_physical_line_end() {
    for (label, line_end) in [("LF", "\n"), ("CRLF", "\r\n"), ("CR", "\r")] {
        let source = format!(
            "[[module ListPages name=\"first\"]]A[[/module]]{line_end}\
             [[module ListPages name=\"unterminated{line_end}\
             [[module ListPages name=\"second\"]]B[[/module]]",
        );
        assert!(
            find_list_pages_module_matches(&source).is_empty(),
            "{label}"
        );
    }
}

#[test]
fn list_pages_scanner_fails_closed_when_a_single_quoted_head_reaches_a_physical_line_end()
{
    for (label, line_end) in [("LF", "\n"), ("CRLF", "\r\n"), ("CR", "\r")] {
        let source = format!(
            "[[module ListPages name='first']]A[[/module]]{line_end}\
             [[module ListPages name='unterminated{line_end}\
             [[module ListPages name='second']]B[[/module]]",
        );
        assert!(
            find_list_pages_module_matches(&source).is_empty(),
            "{label}"
        );
    }
}

#[test]
fn list_pages_scanner_does_not_treat_quoted_attribute_literals_as_syntax() {
    for (attribute, expected_head) in [
        ("prependLine=\"@@\"", "name=\"first\" prependLine=\"@@\""),
        (
            r#"prependLine="[!--""#,
            r#"name="first" prependLine="[!--""#,
        ),
    ] {
        let source = format!(
            "[[module ListPages name=\"first\" {attribute}]]A[[/module]]\n\
             [[module ListPages name=\"second\"]]B[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 2, "attribute {attribute:?}");
        assert_eq!(modules[0].head, expected_head, "attribute {attribute:?}");
        assert_eq!(modules[0].body, "A", "attribute {attribute:?}");
        assert_eq!(
            modules[1].head, "name=\"second\"",
            "attribute {attribute:?}"
        );
        assert_eq!(modules[1].body, "B", "attribute {attribute:?}");
    }

    for (label, line_end) in [("LF", "\n"), ("CRLF", "\r\n"), ("CR", "\r")] {
        let malformed_generic_head = format!(
            "[[span title='@@ [!--{line_end}\
             [[module ListPages name=\"live\"]]body[[/module]]",
        );
        let modules = find_list_pages_module_matches(&malformed_generic_head);
        assert_eq!(modules.len(), 1, "{label}");
        assert_eq!(modules[0].head, "name=\"live\"", "{label}");

        let malformed_unquoted_head = format!(
            "[[span class=unterminated title=\"still-malformed\"{line_end}\
             [[module ListPages name=\"live\"]]body[[/module]]",
        );
        let modules = find_list_pages_module_matches(&malformed_unquoted_head);
        assert_eq!(modules.len(), 1, "unquoted {label}");
        assert_eq!(modules[0].head, "name=\"live\"", "unquoted {label}");
    }
}

#[test]
fn list_pages_scanner_jumps_to_the_end_of_an_owning_literal_region() {
    for literal in ["[!--\n[[broken\n--]", "@@[[broken@@", "@<[[broken>@"] {
        let source =
            format!("{literal}\n[[module ListPages name=\"live\"]]body[[/module]]",);
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{literal:?}");
        assert_eq!(modules[0].head, "name=\"live\"", "{literal:?}");
        assert_eq!(modules[0].body, "body", "{literal:?}");
    }
}

#[test]
fn list_pages_scanner_ignores_modules_inside_pinned_literal_forms() {
    let cases = [
        (
            "inline raw",
            "@@[[module ListPages name=\"fake\"]]body[[/module]]@@",
        ),
        (
            "angle raw",
            "@<[[module ListPages name=\"fake\"]]body[[/module]]>@",
        ),
        (
            "inline math",
            "[[$ [[module ListPages name=\"fake\"]]body[[/module]] $]]",
        ),
        (
            "math block",
            "[[math]]\n[[module ListPages name=\"fake\"]]body[[/module]]\n[[/math]]",
        ),
        (
            "multiline math head",
            "[[math\n label=\"display\"]]\n[[module ListPages name=\"fake\"]]body[[/module]]\n[[/math]]",
        ),
        (
            "empty embed block",
            "[[embed]]\n[[module ListPages name=\"fake\"]]body[[/module]]\n[[/embed]]",
        ),
        (
            "CSS module",
            "[[module CSS]]\n[[module ListPages name=\"fake\"]]body\n[[/module]]",
        ),
    ];

    for (label, literal) in cases {
        let source =
            format!("{literal}\n[[module ListPages name=\"live\"]]kept[[/module]]",);
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{label}");
        assert_eq!(modules[0].head, "name=\"live\"", "{label}");
        assert_eq!(modules[0].body, "kept", "{label}");
    }

    for (label, opener) in [
        ("unclosed inline raw", "@@"),
        ("unclosed angle raw", "@<"),
        ("unclosed inline math", "[[$"),
    ] {
        let source = format!(
            "{opener}[[module ListPages name=\"fake\"]]\n\
             [[module ListPages name=\"live\"]]kept[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{label}");
        assert_eq!(modules[0].head, "name=\"live\"", "{label}");
    }

    let continued_raw = concat!(
        "@@prefix\\\n",
        "[[module ListPages name=\"fake\"]]body[[/module]]@@\n",
        "[[module ListPages name=\"live\"]]kept[[/module]]",
    );
    let modules = find_list_pages_module_matches(continued_raw);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
}

#[test]
fn list_pages_scanner_skips_module_text_inside_generic_tag_heads() {
    let source = concat!(
        "[[span title=\"[[module ListPages name='fake']]fake[[/module]]\"]]",
        "outside",
        "[[module ListPages name=\"outer\"]]",
        "A[[span title=\"[[module ListUsers]]nested[[/module]]\"]]B",
        r#"C[[span title="escaped \" [[module ListUsers]]nested[[/module]] value"]]D"#,
        "[[/module]]",
        "[[module ListPages name=\"live\"]]C[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].head, "name=\"outer\"");
    assert_eq!(
        modules[0].body,
        concat!(
            "A[[span title=\"[[module ListUsers]]nested[[/module]]\"]]B",
            r#"C[[span title="escaped \" [[module ListUsers]]nested[[/module]] value"]]D"#,
        ),
    );
    assert_eq!(modules[1].head, "name=\"live\"");
    assert_eq!(modules[1].body, "C");
}

#[test]
fn list_pages_scanner_skips_module_text_inside_multiline_generic_tag_heads() {
    for (label, line_end) in [("LF", "\n"), ("CRLF", "\r\n"), ("CR", "\r")] {
        let source = format!(
            "[[div{line_end}\
             class=\"[[module ListPages name='fake']]fake[[/module]]\"{line_end}\
             data-kind=\"compatibility\"]]{line_end}\
             [[module ListPages name=\"live\"]]kept[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{label}");
        assert_eq!(modules[0].head, "name=\"live\"", "{label}");
        assert_eq!(modules[0].body, "kept", "{label}");
    }
}

#[test]
fn list_pages_scanner_preserves_continued_generic_tag_head_quotes() {
    for (label, line_end) in [("LF", "\n"), ("CRLF", "\r\n"), ("CR", "\r")] {
        let source = format!(
            "[[span title=\"before\\{line_end}\
             [[module ListPages name='hidden']]ignored[[/module]]\"]]\
             [[module ListPages name=\"live\"]]kept[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{label}");
        assert_eq!(modules[0].head, "name=\"live\"", "{label}");
        assert_eq!(modules[0].body, "kept", "{label}");
    }

    let cascading = concat!(
        "[[span title=\"before\\\\\n\n",
        "[[module ListPages name='hidden']]ignored[[/module]]\"]]",
        "[[module ListPages name=\"live\"]]kept[[/module]]",
    );
    let modules = find_list_pages_module_matches(cascading);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
}

#[test]
fn list_pages_scanner_preserves_multiline_image_link_and_quoted_alt_head() {
    for image in ["image", "=image", "<image", ">image", "f<image", "f>image"] {
        for (label, line_end) in [("LF", "\n"), ("CRLF", "\r\n"), ("CR", "\r")] {
            let source = format!(
                "[[{image} https://e.test/x.png link= \t #{line_end}\
                 alt=\"[[module ListPages name='hidden']]ignored[[/module]]\"]]{line_end}\
                 [[module ListPages name=\"live\"]]kept[[/module]]",
            );
            let modules = find_list_pages_module_matches(&source);

            assert_eq!(modules.len(), 1, "{image} {label}");
            assert_eq!(modules[0].head, "name=\"live\"", "{image} {label}");
            assert_eq!(modules[0].body, "kept", "{image} {label}");
        }
    }

    let nested_syntax_shaped_value = concat!(
        "[[image x.png link=#fragment[[module ListPages name='hidden']]ignored[[/module]] ",
        "alt=\"safe\"]]",
        "[[module ListPages name=\"live\"]]kept[[/module]]",
    );
    let modules = find_list_pages_module_matches(nested_syntax_shaped_value);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
}

#[test]
fn positional_heads_keep_competing_tokens_and_quotes_in_the_source_value() {
    for source_value in ["foo]]]\n", "foo$]]\n", "foo--]]\n", "foo\"bar\n"] {
        let source = format!(
            "[[image {source_value}alt=\"[[module ListPages name='fake']]ignored[[/module]]\"\n]]\n\
             [[module ListPages name=\"real\"]]kept[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{source_value:?}");
        assert_eq!(modules[0].head, "name=\"real\"", "{source_value:?}");
        assert_eq!(modules[0].body, "kept", "{source_value:?}");
    }

    for star in ["*radio", "* radio"] {
        let source = format!(
            "[[{star} choice=one\n title=\"[[module ListPages name='fake']]ignored[[/module]]\"\n]]\n\
             [[module ListPages name=\"real\"]]kept[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{star}");
        assert_eq!(modules[0].head, "name=\"real\"", "{star}");
    }

    for separator in ["\u{000b}", "\u{000c}"] {
        let source = format!(
            "[[radio foo{separator}[[module ListPages = \"x\"]]ignored[[/module]] ]]\n\
             [[module ListPages name=\"real\"]]kept[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 1, "{separator:?}");
        assert_eq!(modules[0].head, "name=\"real\"", "{separator:?}");
    }
}

#[test]
fn complete_whole_heads_own_nested_blocks_across_competing_right_tokens() {
    for competing in ["$]]", "--]]"] {
        let source = format!(
            "[[user foo{competing} [[module ListPages name=\"hidden\"]]x[[/module]] ]]\n\
             [[module ListPages name=\"also-hidden\"]]y[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "{competing:?}");
        assert_eq!(modules[0].head, "name=\"also-hidden\"", "{competing:?}");
    }

    let uppercase_url = concat!(
        "[[user HTTP://e.test/a$]][[module ListPages name=\"hidden\"]]x[[/module]] ]]",
        "[[module ListPages name=\"also-hidden\"]]y[[/module]]",
    );
    let modules = find_list_pages_module_matches(uppercase_url);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"also-hidden\"");

    let lowercase_url = concat!(
        "[[user http://e.test/a$]]",
        "[[module ListPages name=\"live\"]]x[[/module]]",
    );
    let modules = find_list_pages_module_matches(lowercase_url);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");

    let url_owned_comment_prefix = concat!(
        "[[target https://e.test/a--]]] [[module ListPages]]fake[[/module]] ]]\n",
        "[[module ListPages]]real[[/module]]",
    );
    let modules = find_list_pages_module_matches(url_owned_comment_prefix);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "");
}

#[test]
fn malformed_generic_heads_roll_back_to_the_first_nested_block() {
    let whole_head_newline = concat!(
        "[[module ListPages name=\"first\"]]A[[/module]]",
        "[[user foo [[module\n ListPages name=\"hidden\"]]H[[/module]]",
    );
    let modules = find_list_pages_module_matches(whole_head_newline);
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].head, "name=\"first\"");
    assert_eq!(modules[1].head, "name=\"hidden\"");

    let invalid_argument_tail = concat!(
        "[[module ListPages name=\"first\"]]A[[/module]]",
        "[[span title=\"x\" bad other=\"[[module ListPages]]H[[/module]]\"]]",
    );
    let modules = find_list_pages_module_matches(invalid_argument_tail);
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].head, "name=\"first\"");
    assert_eq!(modules[1].head, "");
    assert_eq!(modules[1].body, "H");

    let nonmodule_nested_block = concat!(
        "[[span title=\"[[span]]\n",
        "[[module ListPages name=\"live\"]]Y[[/module]]",
    );
    let modules = find_list_pages_module_matches(nonmodule_nested_block);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");

    let single_quoted_generic = concat!(
        "[[span title='",
        "[[module ListPages name=\"inner\"]]Y[[/module]]",
        "']]",
    );
    let modules = find_list_pages_module_matches(single_quoted_generic);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"inner\"");

    let malformed_generic_module = concat!(
        "[[module Foo title=\"[[span]]\n",
        "[[module ListPages name=\"live\"]]Y[[/module]]",
    );
    let modules = find_list_pages_module_matches(malformed_generic_module);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
}

#[test]
fn rolled_back_competing_parser_function_runs_fail_closed() {
    let live = "[[module ListPages name=\"live\"]]Y[[/module]]";
    for malformed in [
        format!("[[span title='[[[#if true]]owned[[/if]]']] {live}",),
        format!("[[module Foo title='[[[#if true]]owned[[/if]]']] {live}",),
        format!("[[user foo [[[#if true\n{live}"),
    ] {
        assert!(
            find_list_pages_module_matches(&malformed).is_empty(),
            "{malformed:?}",
        );
    }

    let complete_owner = format!("[[span title=\"[[[#if true]]owned[[/if]]\"]] {live}",);
    let modules = find_list_pages_module_matches(&complete_owner);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
}

#[test]
fn email_owned_quotes_do_not_change_tag_or_module_head_state() {
    let generic = concat!(
        "[[span title=\"foo@bar.example\" x=y ",
        "[[module ListPages name='fake']]ignored[[/module]] end\" safe=\"ok\"]]",
        "[[module ListPages name=\"real\"]]kept[[/module]]",
    );
    let modules = find_list_pages_module_matches(generic);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"real\"");

    let module = concat!(
        "[[module ListPages title=\"foo@bar.example\" x=y ",
        "[[module ListPages]]fake[[/module]] end\" safe=\"ok\"]]",
        "OUTER[[/module]]",
    );
    let modules = find_list_pages_module_matches(module);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].body, "OUTER");

    let unclosed = "[[module ListPages title=\"foo@bar.example\" ]]BODY[[/module]]";
    assert!(find_list_pages_module_matches(unclosed).is_empty());

    let quote_end_lookahead = concat!(
        "[[span a=\"prefix\" foo=bar@x.y ",
        "[[module ListPages]]fake[[/module]] final\"]]",
        "[[module ListPages name=\"real\"]]kept[[/module]]",
    );
    let modules = find_list_pages_module_matches(quote_end_lookahead);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"real\"");

    let url_owned_backslash = concat!(
        "[[span a=\"https://e.test/x\\\" b=\" foo=bar ",
        "[[module ListPages]]fake[[/module]] final\"]]",
        "[[module ListPages name=\"real\"]]kept[[/module]]",
    );
    let modules = find_list_pages_module_matches(url_owned_backslash);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"real\"");

    let continuation_owned_lookahead = concat!(
        "[[span a=\"prefix\"\\\n",
        "foo=bar@x.y [[module ListPages]]fake[[/module]] final\"]]",
        "[[module ListPages name=\"real\"]]kept[[/module]]",
    );
    let modules = find_list_pages_module_matches(continuation_owned_lookahead);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"real\"");

    let owned_token_resets_line_continuation = concat!(
        "[[span title=\"\\foo@bar.example\n",
        "[[module ListPages name='live']]X[[/module]]\n\"]]",
    );
    assert!(
        find_list_pages_module_matches(owned_token_resets_line_continuation).is_empty()
    );
}

#[test]
fn owned_url_bytes_preserve_positional_tabs_and_line_continuations() {
    for outer in [
        "[[image https://e.test/a\t[[module ListPages = \"x\"]]X[[/module]] ]]",
        "[[image x link=https://e.test/a\t[[module ListPages = \"x\"]]X[[/module]] ]]",
    ] {
        let source =
            format!("{outer}\n[[module ListPages name=\"real\"]]kept[[/module]]",);
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "{outer:?}");
        assert_eq!(modules[0].head, "name=\"real\"", "{outer:?}");
    }

    let continued = concat!(
        "[[module ListPages title=\"https://e.test/a\\\n",
        "\" name=\"x\"]]BODY[[/module]]",
    );
    let modules = find_list_pages_module_matches(continued);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].body, "BODY");
}

#[test]
fn module_head_validation_preserves_nesting_without_executing_malformed_heads() {
    let source = concat!(
        "[[module ListPages limit=\"1\"]]A",
        "[[module Foo bare]]B[[/module]]C[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].body, "A[[module Foo bare]]B");

    for malformed in [
        "[[module ListPages limit\n=\"1\"]]x[[/module]]",
        "[[module ListPages limit=\"1\"\u{000b}]]x[[/module]]",
        "[[module ListPages limit=\u{000c}1]]x[[/module]]",
        "[[module ListPages \u{00a0}limit=\"1\"]]x[[/module]]",
        "[[module ListPages name=\"secret@site.example\" wrapper=no\"]]x[[/module]]",
        "[[module ListPages name=a]b]]x[[/module]]",
        "[[module ListPages ---]=\"x\"]]x[[/module]]",
        "[[module ListPages 1=bare]]x[[/module]]",
        "[[module ListPages [!--=bare]]x[[/module]]",
    ] {
        assert!(
            find_list_pages_module_matches(malformed).is_empty(),
            "{malformed:?}",
        );
    }

    for compatible in [
        "[[module ListPages limit=1]]x[[/module]]",
        "[[module ListPages name='x']]x[[/module]]",
        "[[module ListPages _field!=value]]x[[/module]]",
    ] {
        let modules = find_list_pages_module_matches(compatible);
        assert_eq!(modules.len(), 1, "{compatible:?}");
        assert!(modules[0].runtime_safe, "{compatible:?}");
    }

    for structurally_valid_but_unsafe in [
        "[[module ListPages name\0=\0\"x\"]]x[[/module]]",
        "[[module ListPages limit\\\n=\"1\"]]x[[/module]]",
        "[[module ListPages limit\\\n\u{00a0}=\"1\"]]x[[/module]]",
        "[[module ListPages limit\\\r\n\u{2007}=\"1\"]]x[[/module]]",
        "[[module ListPages limit=\\\n\"1\"]]x[[/module]]",
        "[[module ListPages\n\u{00a0}\u{2007}limit=\"1\"]]x[[/module]]",
        "[[module ListPages name = \"secret@site.example\" wrapper=no\"]]x[[/module]]",
        "[[module ListPages name = \"secret\"wrapper=\"no\"]]x[[/module]]",
        "[[module ListPages name = \"secret\\\" wrapper=no\"]]x[[/module]]",
        "[[module ListPages name=\"foo\tbar\"]]x[[/module]]",
        "[[module ListPages name=\"foo...\"]]x[[/module]]",
        "[[module ListPages name=\"`x'\"]]x[[/module]]",
        "[[module ListPages name=\"foo\\nbar\"]]x[[/module]]",
        "[[module ListPages [!--=\"x\"]]x[[/module]]",
        "[[module ListPages --]=\"x\"]]x[[/module]]",
        "[[module ListPages [!----]=\"x\"]]x[[/module]]",
        "[[module ListPages 1=\"x\"]]x[[/module]]",
    ] {
        let modules = find_list_pages_module_matches(structurally_valid_but_unsafe);
        assert_eq!(modules.len(), 1, "{structurally_valid_but_unsafe:?}");
        assert!(
            !modules[0].runtime_safe,
            "{structurally_valid_but_unsafe:?}"
        );
    }

    let positional_subname = concat!(
        "[[module Foo]]][[module ListPages name='hidden']]X[[/module]] ]]",
        "[[module ListPages name=\"live\"]]Y[[/module]]",
    );
    assert!(find_list_pages_module_matches(positional_subname).is_empty());

    let definite_invalid_marker = concat!(
        "[[module ListPages name=\"complete\"]]A[[/module]]",
        "[[module Foo [!-?=x]]ignored[[/module]]",
        "[[module ListPages name=\"later\"]]B[[/module]]",
    );
    let modules = find_list_pages_module_matches(definite_invalid_marker);
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].head, "name=\"complete\"");
    assert_eq!(modules[1].head, "name=\"later\"");

    for raw_runtime_invalid_head in ["1=bare", "name=a]b"] {
        let source = format!(
            "[[module ListPages name=\"outer\"]]A[[module ListPages {raw_runtime_invalid_head}]]B[[/module]]C[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "{raw_runtime_invalid_head:?}");
        assert_eq!(
            modules[0].body,
            format!("A[[module ListPages {raw_runtime_invalid_head}]]B"),
            "{raw_runtime_invalid_head:?}",
        );
    }

    let competing_left_run = concat!(
        "[[module ListPages name=\"outer\"]]A",
        "[[module ListPages name=foo[[[[bar]]B[[/module]]C[[/module]]",
    );
    assert!(find_list_pages_module_matches(competing_left_run).is_empty());

    let self_masked_malformed_head = concat!(
        "[[module ListPages name=foo",
        "[[module ListPages name=\"secret\"]]H[[/module]]",
    );
    assert!(find_list_pages_module_matches(self_masked_malformed_head).is_empty());

    let invalid_list_pages_head = concat!(
        "[[module ListPages name=\"complete\"]]A[[/module]]",
        "[[module ListPages limit\n=\"1\"]]ignored",
        "[[module ListPages name=\"inner\"]]B[[/module]]",
        "[[/module]]",
    );
    assert!(find_list_pages_module_matches(invalid_list_pages_head).is_empty());

    for runtime_ambiguous_head in [
        concat!(
            "[[module ListPages name=\"x\ny\"]]",
            "[[module ListPages name=secret]]H[[/module]][[/module]]",
        ),
        concat!(
            "[[module ListPages name=foo[[x]]",
            "[[module ListPages name=secret]]H[[/module]][[/module]]",
        ),
        concat!(
            "[[module ListPages name=\"outer\"]]A",
            "[[module ListPages name=foo[[[[bar]]B[[/module]]C[[/module]]",
        ),
        concat!(
            "[[module ListPages name=\"outer\"]]A",
            "[[module ListPages name=\"x\ny\"]]B[[/module]]C[[/module]]",
        ),
        concat!(
            "[[module ListPages name=\"outer\"]]A",
            "[[module ListPages name=foo[[x=y]]B[[/module]]C[[/module]]",
        ),
        concat!(
            "[[module ListPages name=\"outer\"]]A",
            "[[module ListPages name=\"unterminated\n]]B",
            "[[/module]]C[[/module]]",
        ),
        concat!(
            "[[module ListPages name=\"outer\"]]A",
            "[[module ListPages name=foo[[x=\"unterminated]]B",
            "[[/module]]C[[/module]]",
        ),
    ] {
        assert!(
            find_list_pages_module_matches(runtime_ambiguous_head).is_empty(),
            "{runtime_ambiguous_head:?}",
        );
    }

    let invalid_head_with_suppressed_nested_open = concat!(
        "[[module ListPages limit=\"1\"]]A",
        "[[module Foo x='[[module ListPages name=\"inner\"]]']]B",
        "[[/module]]C[[/module]]",
    );
    let modules =
        find_list_pages_module_matches(invalid_head_with_suppressed_nested_open);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "limit=\"1\"");
}

#[test]
fn full_source_typography_changes_make_direct_heads_runtime_unsafe() {
    for source in [
        "`prefix [[module ListPages name=\"foo'bar\"]]B[[/module]]",
        "[[module ListPages name=\"foo`bar\"]]B' suffix[[/module]]",
    ] {
        let modules = find_list_pages_module_matches(source);
        assert_eq!(modules.len(), 1, "{source:?}");
        assert!(!modules[0].runtime_safe, "{source:?}");
    }
}

#[test]
fn long_unicode_trim_runs_keep_positional_and_whole_head_ownership() {
    let trim = "\u{000b}".repeat(32);
    let positional = format!(
        "[[radio{trim} foo[[module ListPages]]fake[[/module]] ]]\n\
         [[module ListPages name=\"live\"]]Y[[/module]]",
    );
    let modules = find_list_pages_module_matches(&positional);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");

    let whole_head = format!(
        "[[user{trim} foo [[module ListPages]]fake[[/module]] ]]\n\
         [[module ListPages name=\"also-hidden\"]]Y[[/module]]",
    );
    let modules = find_list_pages_module_matches(&whole_head);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"also-hidden\"");
}

#[test]
fn module_names_and_subnames_use_unicode_trim() {
    for separator in ["\u{000b}", "\u{000c}"] {
        let source = format!(
            "[[module ListPages{separator} name=\"not-list-pages\"]]ignored[[/module]]\n\
             [[module ListPages name=\"real\"]]kept[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);

        assert_eq!(modules.len(), 2, "{separator:?}");
        assert_eq!(modules[0].head, "name=\"not-list-pages\"", "{separator:?}");
        assert_eq!(modules[1].head, "name=\"real\"", "{separator:?}");
    }

    let unicode = concat!(
        "[[module\u{00a0} ListPages\u{2007} name=\"unicode\"]]A[[/module\u{00a0} ]]\n",
        "[[module ListPages name=\"ascii\"]]B[[/module]]",
    );
    let modules = find_list_pages_module_matches(unicode);
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].head, "name=\"unicode\"");
    assert_eq!(modules[1].head, "name=\"ascii\"");
}

#[test]
fn module_heads_do_not_close_on_math_or_comment_right_tokens() {
    for competing in ["$]]", "--]]"] {
        let source = format!(
            "[[module ListPages {competing}A[[/module]]\n\
             [[module ListPages name=\"live\"]]B[[/module]]",
        );
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "{competing:?}");
        assert_eq!(modules[0].head, "name=\"live\"", "{competing:?}");
    }
}

#[test]
fn list_pages_scanner_rejects_right_link_runs_as_generic_tag_terminators() {
    let same_line = concat!(
        "[[span title=\"malformed\"]]]",
        "[[module ListPages name=\"live\"]]kept[[/module]]",
    );
    assert!(find_list_pages_module_matches(same_line).is_empty());

    let next_line = concat!(
        "[[span title=\"malformed\"]]]\n",
        "[[module ListPages name=\"live\"]]kept[[/module]]",
    );
    let modules = find_list_pages_module_matches(next_line);

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
    assert_eq!(modules[0].body, "kept");
}

#[test]
fn list_pages_scanner_ignores_module_events_on_quoted_lines() {
    let source = concat!(
        "> [[module ListPages name=\"spaced\"]]ignored[[/module]]\n",
        ">> [[module ListPages name=\"nested\"]]ignored[[/module]]\n",
        "> > [[module ListPages name=\"split\"]]ignored[[/module]]\n",
        ">[[module ListPages name=\"tight\"]]ignored[[/module]]\n",
        "[[module ListPages name=\"outer\"]]A\n",
        "> [[module ListUsers]]\n",
        "> [[/module]]\n",
        "  >[[module ListUsers]]\n",
        "  > [[/module]]\n",
        "  >[[/module]]\n",
        "B[[/module]]\n",
        "[[module ListPages name=\"live\"]]C[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].head, "name=\"outer\"");
    assert_eq!(
        modules[0].body,
        concat!(
            "A\n> [[module ListUsers]]\n> [[/module]]\n",
            "  >[[module ListUsers]]\n  > [[/module]]\n  >[[/module]]\nB",
        ),
    );
    assert_eq!(modules[1].head, "name=\"live\"");
    assert_eq!(modules[1].body, "C");
}

#[test]
fn list_pages_scanner_keeps_completed_matches_before_an_unclosed_body() {
    let source = concat!(
        "[[module ListPages name=\"complete\"]]A[[/module]]\n",
        "[[module ListPages name=\"unclosed\"]]B",
    );
    let modules = find_list_pages_module_matches(source);
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].head, "name=\"complete\"");
    assert_eq!(modules[1].head, "name=\"unclosed\"");
    assert_eq!(modules[1].body, "");
    assert_eq!(
        modules[1].original,
        "[[module ListPages name=\"unclosed\"]]"
    );

    let unclosed_outer = concat!(
        "[[module ListPages name=\"unclosed\"]]A",
        "[[module ListPages name=\"nested\"]]B[[/module]]",
    );
    let modules = find_list_pages_module_matches(unclosed_outer);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"unclosed\"");
    assert_eq!(modules[0].body, "");
}

#[test]
fn list_pages_scanner_keeps_adjacent_custom_listing_modules() {
    let source = r#"[[div class="top-border"]]
[[/div]]
[[div style="text-align: center;"]]
+ SCPs
[[module Listpages created_by="=" category="-fragment" tag="+scp -co-authored" limit="1" order="random"]]
([[[*%%link%%|Random]]])
[[/module]]
-----
[[/div]]

[[module Listpages created_by="=" order="" category="-fragment" tag="+scp -co-authored" perPage="250"]]
[[div class="content-box no"]]
++ **%%title_linked%%**
[[div class="content-section"]]
------
**Rating:** +%%rating%%
**Comments:** %%comments%%
**+/- :** +[[#expr ((%%rating%%+%%rating_votes%%)/2)]]/-[[#expr ((%%rating_votes%%-%%rating%%)/2)]]
[[/div]]
[[/module]]"#;

    let second_start = source.rfind("[[module Listpages").unwrap();
    assert_eq!(
        find_list_pages_module_matches(&source[..second_start]).len(),
        1
    );
    assert_eq!(
        find_list_pages_module_matches(&source[second_start..]).len(),
        1
    );
    let modules = find_list_pages_module_matches(source);
    assert_eq!(modules.len(), 2);
}

#[test]
fn linear_list_pages_scanner_ignores_literal_module_tokens() {
    let source = concat!(
        "@@[[module ListPages name=\"inline-literal\"]]@@\n",
        "stray literal example body[[/module]]\n",
        "[[code]]\n",
        "[[module ListPages name=\"block-literal\"]]ignored[[/module]]\n",
        "@@\n",
        "[!--\n",
        "[[/code]]\n",
        "[[module ListPages name=\"live\"]]",
        "before @@[[module ListUsers]][[/module]]@@ after",
        "[[/module]]",
    );
    let modules = find_list_pages_module_matches(source);

    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].head, "name=\"live\"");
    assert_eq!(
        modules[0].body,
        "before @@[[module ListUsers]][[/module]]@@ after",
    );

    for body in [
        "before @@[[/module]]@@ after",
        "before @@[[module ListUsers]]@@ after",
    ] {
        let source = format!("[[module ListPages name=\"live\"]]{body}[[/module]]",);
        let modules = find_list_pages_module_matches(&source);
        assert_eq!(modules.len(), 1, "literal token changed nesting: {body}");
        assert_eq!(modules[0].body, body);
    }
}
